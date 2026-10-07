use std::{path::PathBuf, time::Duration};

use anyhow::Result;
use futures_util::future::{select, Either};
use serde::Serialize;
use tokio::{sync::oneshot, time::Instant};
use url::Url;
use zeroize::Zeroizing;

use crate::{
    context::{self, CompileInput, ModelBudget},
    inference::{self, FailureKind, StreamUsage, WireApi},
    store::{PendingRequest, RequestKind, RequestUsage, Store},
};

const MAX_USER_TEXT: usize = 64 * 1024;
const MAX_ANSWER: usize = 256 * 1024;
const MAX_TARGETS: usize = 3;
const EMIT_BYTES: usize = 4 * 1024;
const EMIT_INTERVAL: Duration = Duration::from_millis(100);

/// A selected provider target. The bearer is deliberately not serializable or debuggable.
pub struct Target {
    pub provider: String,
    pub model: String,
    pub endpoint: Url,
    pub api: WireApi,
    pub bearer: Zeroizing<String>,
    pub allow_fallback: bool,
}

pub struct DispatchInput {
    pub targets: Vec<Target>,
    pub client: reqwest::Client,
    pub dbpath: PathBuf,
    pub pending_id: i64,
    pub compile: CompileInput,
    pub budget: ModelBudget,
    pub history_omitted: bool,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DispatchStatus {
    Preparing,
    Streaming,
    Completed,
    Partial,
    Error,
    Cancelled,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DispatchState {
    pub request_id: i64,
    pub attempt_id: u64,
    pub meeting_id: i64,
    pub status: DispatchStatus,
    pub answer: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub context_omitted: bool,
    pub error: Option<String>,
    pub usage: Option<RequestUsage>,
    pub user_text: Option<String>,
}

#[derive(Debug)]
pub struct DispatchResult {
    pub request_id: i64,
    pub state: DispatchState,
}

/// Runs exactly one already-reserved request. The caller owns queueing and credentials.
pub async fn run<F>(
    input: DispatchInput,
    cancel: oneshot::Receiver<()>,
    mut on_state: F,
) -> Result<DispatchResult>
where
    F: FnMut(DispatchState),
{
    validate_targets(&input.targets)?;
    let mut store = Store::open(&input.dbpath)?;
    let pending = store.begin_request(input.pending_id)?;
    let user_text = bounded_user_text(&pending.user_text)?;
    let mut state = DispatchState {
        request_id: pending.id,
        attempt_id: 0,
        meeting_id: pending.meeting_id,
        status: DispatchStatus::Preparing,
        answer: String::new(),
        provider: None,
        model: None,
        context_omitted: input.history_omitted,
        error: None,
        usage: None,
        user_text: Some(user_text),
    };
    on_state(state.clone());

    let mut cancel = Some(Box::pin(cancel));
    let mut last_emit = None;
    let mut emitted_bytes = 0usize;

    for (target_index, target) in input.targets.iter().enumerate() {
        if !matches!(
            cancel
                .as_mut()
                .expect("live cancellation receiver")
                .as_mut()
                .get_mut()
                .try_recv(),
            Err(oneshot::error::TryRecvError::Empty)
        ) {
            store.cancel_request(pending.id)?;
            state.status = DispatchStatus::Cancelled;
            on_state(state.clone());
            return Ok(DispatchResult {
                request_id: pending.id,
                state,
            });
        }
        let compile_input = compile_input(&input.compile, &pending, &target.model)?;
        let compiled = match context::compile(compile_input, input.budget) {
            Ok(value) => value,
            Err(error) => {
                return finish_error(
                    &mut store,
                    &pending,
                    state,
                    format!("context compilation failed: {error:?}"),
                    &mut on_state,
                );
            }
        };
        let snapshot = serde_json::to_string(&compiled.request)
            .and_then(|request| {
                serde_json::to_string(&compiled.metadata).map(|metadata| (request, metadata))
            })
            .map_err(anyhow::Error::from)
            .and_then(|(request, metadata)| {
                store.save_request_context(
                    pending.id,
                    &target.provider,
                    &target.model,
                    &request,
                    &metadata,
                    input.history_omitted,
                )
            });
        if snapshot.is_err() {
            return finish_error(
                &mut store,
                &pending,
                state,
                "Could not save request context; nothing was sent for this attempt".into(),
                &mut on_state,
            );
        }
        drop(snapshot);
        state.context_omitted = input.history_omitted || !compiled.metadata.omissions.is_empty();
        state.provider = Some(target.provider.clone());
        state.model = Some(target.model.clone());
        state.status = DispatchStatus::Streaming;
        state.error = None;
        on_state(state.clone());

        let mut answer = state.answer.clone();
        let mut checkpoint_error = false;
        let stream_result = {
            let stream = inference::stream(
                &input.client,
                target.endpoint.clone(),
                target.bearer.as_str(),
                target.api,
                &compiled.request,
                |delta| {
                    if answer.len().saturating_add(delta.len()) > MAX_ANSWER {
                        checkpoint_error = true;
                        return false;
                    }
                    answer.push_str(delta);
                    let now = Instant::now();
                    let ready = last_emit.is_none()
                        || answer.len().saturating_sub(emitted_bytes) >= EMIT_BYTES
                        || last_emit.is_some_and(|at| now.duration_since(at) >= EMIT_INTERVAL);
                    if ready {
                        if store
                            .checkpoint_request_output(
                                pending.id,
                                &answer,
                                &target.provider,
                                &target.model,
                            )
                            .is_err()
                        {
                            checkpoint_error = true;
                            return false;
                        }
                        state.answer = answer.clone();
                        on_state(state.clone());
                        emitted_bytes = answer.len();
                        last_emit = Some(now);
                    }
                    true
                },
            );
            match select(
                Box::pin(stream),
                cancel.take().expect("live cancellation receiver"),
            )
            .await
            {
                Either::Left((result, receiver)) => {
                    cancel = Some(receiver);
                    Some(result)
                }
                Either::Right((_, _stream)) => None,
            }
        };

        if checkpoint_error {
            return finish_error(
                &mut store,
                &pending,
                state,
                "output checkpoint failed".into(),
                &mut on_state,
            );
        }

        if answer.len() > state.answer.len() {
            if store
                .checkpoint_request_output(pending.id, &answer, &target.provider, &target.model)
                .is_err()
            {
                return finish_error(
                    &mut store,
                    &pending,
                    state,
                    "output checkpoint failed".into(),
                    &mut on_state,
                );
            }
            state.answer = answer.clone();
        }

        if stream_result.is_none() {
            store.cancel_request(pending.id)?;
            state.answer = answer;
            state.status = if state.answer.is_empty() {
                DispatchStatus::Cancelled
            } else {
                DispatchStatus::Partial
            };
            state.error = None;
            on_state(state.clone());
            return Ok(DispatchResult {
                request_id: pending.id,
                state,
            });
        }

        match stream_result.expect("stream result is present") {
            Ok(usage) => {
                state.answer = answer;
                state.usage = match to_request_usage(&usage) {
                    Ok(usage) => Some(usage),
                    Err(_) => {
                        return finish_error(
                            &mut store,
                            &pending,
                            state,
                            "provider usage exceeds supported bounds".into(),
                            &mut on_state,
                        )
                    }
                };
                store.complete_request(
                    pending.id,
                    &state.answer,
                    &target.provider,
                    &target.model,
                    state.usage.clone(),
                )?;
                state.status = DispatchStatus::Completed;
                on_state(state.clone());
                return Ok(DispatchResult {
                    request_id: pending.id,
                    state,
                });
            }
            Err(failure) => {
                state.answer = answer;
                if state.answer.is_empty()
                    && target_index + 1 < input.targets.len()
                    && should_fallback(&failure, input.targets[target_index + 1].allow_fallback)
                {
                    continue;
                }
                let status = if state.answer.is_empty() {
                    DispatchStatus::Error
                } else {
                    DispatchStatus::Partial
                };
                let message = failure.to_string();
                store.fail_request(pending.id, &message, None)?;
                state.status = status;
                state.error = Some(message);
                on_state(state.clone());
                return Ok(DispatchResult {
                    request_id: pending.id,
                    state,
                });
            }
        }
    }

    finish_error(
        &mut store,
        &pending,
        state,
        "no inference target".into(),
        &mut on_state,
    )
}

fn compile_input(
    base: &CompileInput,
    pending: &PendingRequest,
    model: &str,
) -> Result<CompileInput> {
    let mut input = base.clone();
    input.model = model.to_owned();
    input.user_message = match pending.kind {
        RequestKind::Transcript => {
            transcript_user_message(&pending.user_text, &pending.segment_ids)?
        }
        RequestKind::Question => pending.user_text.clone(),
    };
    Ok(input)
}

fn transcript_user_message(text: &str, segment_ids: &[i64]) -> Result<String> {
    Ok(serde_json::json!({
        "source": "system",
        "segment_ids": segment_ids,
        "text": text,
    })
    .to_string())
}

fn should_fallback(failure: &inference::StreamFailure, allowed: bool) -> bool {
    // Only explicit provider-limit rotation is safe; auth, server, offline, and partial
    // failures stay attached to this request for retry.
    allowed
        && inference::can_fallback(failure, true)
        && !failure.any_output
        && failure.kind == FailureKind::Limited
}

fn validate_targets(targets: &[Target]) -> Result<()> {
    if targets.is_empty() || targets.len() > MAX_TARGETS {
        anyhow::bail!("invalid target count")
    }
    for target in targets {
        if target.provider.is_empty()
            || target.provider.len() > 200
            || target.provider.chars().any(char::is_control)
            || target.model.is_empty()
            || target.model.len() > 200
            || target.model.chars().any(char::is_control)
        {
            anyhow::bail!("invalid inference target")
        }
    }
    Ok(())
}

fn bounded_user_text(text: &str) -> Result<String> {
    if text.len() > MAX_USER_TEXT {
        anyhow::bail!("request text exceeds 64 KiB")
    }
    Ok(text.to_owned())
}

fn to_request_usage(usage: &StreamUsage) -> Result<RequestUsage> {
    Ok(RequestUsage {
        input_tokens: usage.input_tokens.map(i64::try_from).transpose()?,
        output_tokens: usage.output_tokens.map(i64::try_from).transpose()?,
        total_tokens: None,
    })
}

fn finish_error<F>(
    store: &mut Store,
    pending: &PendingRequest,
    mut state: DispatchState,
    error: String,
    on_state: &mut F,
) -> Result<DispatchResult>
where
    F: FnMut(DispatchState),
{
    store.fail_request(pending.id, &error, None)?;
    state.status = if state.answer.is_empty() {
        DispatchStatus::Error
    } else {
        DispatchStatus::Partial
    };
    state.error = Some(error);
    on_state(state.clone());
    Ok(DispatchResult {
        request_id: pending.id,
        state,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_message_contains_only_frozen_text_and_ids() {
        let value = transcript_user_message("hello", &[4, 9]).unwrap();
        assert_eq!(
            value,
            r#"{"segment_ids":[4,9],"source":"system","text":"hello"}"#
        );
    }

    #[test]
    fn question_is_not_rewritten() {
        let pending = PendingRequest {
            id: 1,
            meeting_id: 2,
            kind: RequestKind::Question,
            status: crate::store::RequestStatus::Pending,
            user_text: "custom question".into(),
            cursor_before: None,
            cursor_after: None,
            segment_ids: vec![8],
        };
        let base = CompileInput {
            model: "old".into(),
            user_message: "parent value".into(),
            custom_instruction: Some("keep me".into()),
            recent_transcript: vec![],
            history: vec![],
            evidence: vec![],
            microphone: vec![],
            include_microphone: false,
        };
        let input = compile_input(&base, &pending, "new").unwrap();
        assert_eq!(input.user_message, "custom question");
        assert_eq!(input.custom_instruction.as_deref(), Some("keep me"));
    }

    #[test]
    fn cancellation_and_local_validation_preserve_retry_identity() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let path =
            std::env::temp_dir().join(format!("harness-dispatch-{}.sqlite", uuid::Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let meeting = store
            .create_meeting("fixture", 0, &crate::store::Settings::default())
            .unwrap();
        let pending = store
            .prepare_question_request(meeting.id, "question")
            .unwrap();
        let make_input = || DispatchInput {
            targets: vec![Target {
                provider: "deepseek".into(),
                model: "model".into(),
                endpoint: Url::parse("http://invalid.example/v1").unwrap(),
                api: WireApi::ChatCompletions,
                bearer: Zeroizing::new("test".into()),
                allow_fallback: false,
            }],
            client: reqwest::Client::new(),
            dbpath: path.clone(),
            pending_id: pending.id,
            compile: CompileInput {
                model: "model".into(),
                user_message: String::new(),
                custom_instruction: None,
                recent_transcript: vec![],
                history: vec![],
                evidence: vec![],
                microphone: vec![],
                include_microphone: false,
            },
            budget: ModelBudget {
                context_tokens: 4096,
                response_reserve_tokens: 512,
            },
            history_omitted: false,
        };
        let (sender, receiver) = oneshot::channel();
        sender.send(()).unwrap();
        let cancelled = runtime
            .block_on(run(make_input(), receiver, |_| {}))
            .unwrap();
        assert_eq!(cancelled.state.status, DispatchStatus::Cancelled);
        assert!(store.request_context(pending.id, None).unwrap().is_none());
        assert_eq!(
            store.pending_request(pending.id).unwrap().unwrap().status,
            crate::store::RequestStatus::Pending
        );
        let (_sender, receiver) = oneshot::channel();
        let mut events = Vec::new();
        let failed = runtime
            .block_on(run(make_input(), receiver, |state| {
                events.push(state.status)
            }))
            .unwrap();
        assert_eq!(failed.state.status, DispatchStatus::Error);
        let snapshot = store.request_context(pending.id, None).unwrap().unwrap();
        assert_eq!(snapshot.provider, "deepseek");
        assert_eq!(
            snapshot.request["messages"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()["content"],
            "question"
        );
        assert!(!serde_json::to_string(&snapshot).unwrap().contains("bearer"));
        assert_eq!(events.last(), Some(&DispatchStatus::Error));
        assert_eq!(
            store.pending_request(pending.id).unwrap().unwrap().id,
            pending.id
        );
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fallback_requires_explicit_limit_and_no_output() {
        let failure = inference::StreamFailure {
            kind: FailureKind::Limited,
            status: Some(429),
            request_id: None,
            body_shape: None,
            code: None,
            param: None,
            any_output: false,
        };
        assert!(should_fallback(&failure, true));
        assert!(!should_fallback(&failure, false));
    }
}
