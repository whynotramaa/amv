use std::fmt;

use futures_util::StreamExt;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::time::{timeout, Duration};
use url::Url;

const MAX_EVENT: usize = 64 * 1024;
const MAX_ANSWER: usize = 256 * 1024;
const MAX_HTTP_ERROR: usize = 64 * 1024;
const MAX_INPUT: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireApi {
    Chatgpt,
    ChatCompletions,
}

#[derive(Clone, Debug, Serialize)]
pub struct TextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StreamRequest {
    pub model: String,
    pub messages: Vec<TextMessage>,
    pub instructions: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamEvent {
    Delta(String),
    Usage(StreamUsage),
    Terminal(TerminalEvent),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StreamUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cached_tokens: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalEvent {
    Completed,
    Failed,
    Incomplete,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    Http,
    Auth,
    Limited,
    Server,
    Timeout,
    Incomplete,
    Transport,
    InvalidRequest,
}

#[derive(Clone, PartialEq, Eq)]
pub struct StreamFailure {
    pub kind: FailureKind,
    pub status: Option<u16>,
    pub request_id: Option<String>,
    pub body_shape: Option<String>,
    pub code: Option<String>,
    pub param: Option<String>,
    pub any_output: bool,
}

impl fmt::Debug for StreamFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamFailure")
            .field("kind", &self.kind)
            .field("status", &self.status)
            .field("request_id", &self.request_id)
            .field("body_shape", &self.body_shape)
            .field("code", &self.code)
            .field("param", &self.param)
            .field("any_output", &self.any_output)
            .finish()
    }
}

impl fmt::Display for StreamFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Inference {:?}", self.kind)?;
        if let Some(status) = self.status {
            write!(f, "; HTTP {status}")?;
        }
        if let Some(code) = &self.code {
            write!(f, "; code {code}")?;
        }
        if let Some(param) = &self.param {
            write!(f, "; parameter {param}")?;
        }
        if let Some(id) = &self.request_id {
            write!(f, "; request {id}")?;
        }
        Ok(())
    }
}

impl std::error::Error for StreamFailure {}

pub async fn stream<F>(
    client: &reqwest::Client,
    endpoint: Url,
    bearer: &str,
    api: WireApi,
    request: &StreamRequest,
    mut on_delta: F,
) -> Result<StreamUsage, StreamFailure>
where
    F: FnMut(&str) -> bool,
{
    validate_endpoint(&endpoint)?;
    if api == WireApi::Chatgpt && endpoint.as_str() != "https://api.openai.com/v1/responses" {
        return Err(local_failure(FailureKind::InvalidRequest));
    }
    if bearer.is_empty() || bearer.len() > 16 * 1024 {
        return Err(local_failure(FailureKind::InvalidRequest));
    }
    validate_request(request)?;

    let body = match api {
        WireApi::Chatgpt => responses_body(request),
        WireApi::ChatCompletions => compat_body(request),
    };
    let body = serialize_body(&body)?;
    let header = zeroize::Zeroizing::new(format!("Bearer {bearer}"));
    let mut authorization = reqwest::header::HeaderValue::from_str(&header)
        .map_err(|_| local_failure(FailureKind::InvalidRequest))?;
    authorization.set_sensitive(true);
    let post = client
        .post(endpoint)
        .header(AUTHORIZATION, authorization)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "text/event-stream")
        .timeout(Duration::from_secs(600));
    let response = timeout(Duration::from_secs(30), post.body(body).send())
        .await
        .map_err(|_| local_failure(FailureKind::Timeout))?
        .map_err(|error| transport_failure(error.is_timeout()))?;

    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            value.starts_with("req_")
                && value.len() <= 128
                && value[4..].bytes().all(|b| b.is_ascii_hexdigit())
        })
        .map(str::to_owned);
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let (body_shape, code, param) = timeout(Duration::from_secs(10), read_http_error(response))
            .await
            .unwrap_or_default();
        return Err(StreamFailure {
            kind: http_kind(status),
            status: Some(status),
            request_id,
            body_shape,
            code,
            param,
            any_output: false,
        });
    }

    let mut bytes = response.bytes_stream();
    let mut parser = SseParser::default();
    let mut usage = StreamUsage::default();
    let mut any_output = false;
    let mut visible_output = false;
    let mut terminal = false;
    let mut compat_finish = false;
    let mut answer_bytes = 0usize;

    loop {
        let read = timeout(Duration::from_secs(60), bytes.next())
            .await
            .map_err(|_| StreamFailure {
                kind: FailureKind::Timeout,
                status: None,
                request_id: request_id.clone(),
                body_shape: None,
                code: None,
                param: None,
                any_output,
            })?;
        let Some(chunk) = read else { break };
        let chunk = chunk.map_err(|_| StreamFailure {
            kind: FailureKind::Transport,
            status: None,
            request_id: request_id.clone(),
            body_shape: None,
            code: None,
            param: None,
            any_output,
        })?;

        let events = parser.feed(&chunk).map_err(|_| StreamFailure {
            kind: FailureKind::Transport,
            status: None,
            request_id: request_id.clone(),
            body_shape: None,
            code: None,
            param: None,
            any_output,
        })?;
        for event in events {
            let parsed = parse_event(api, &event).map_err(|failure| StreamFailure {
                any_output,
                request_id: request_id.clone(),
                ..failure
            })?;
            for item in parsed {
                match item {
                    StreamEvent::Delta(delta) if !delta.is_empty() => {
                        if !visible_output && delta.len() > MAX_ANSWER {
                            return Err(local_failure_with_output(
                                FailureKind::Transport,
                                any_output,
                            ));
                        }
                        if delta.len() + answer_bytes > MAX_ANSWER {
                            return Err(local_failure_with_output(FailureKind::Transport, true));
                        }
                        any_output = true;
                        visible_output = true;
                        if !on_delta(&delta) {
                            return Err(local_failure_with_output(
                                FailureKind::InvalidRequest,
                                true,
                            ));
                        }
                        answer_bytes += delta.len();
                    }
                    StreamEvent::Delta(_) => {}
                    StreamEvent::Usage(value) => merge_usage(&mut usage, value),
                    StreamEvent::Terminal(kind) => {
                        if kind == TerminalEvent::Done {
                            if api != WireApi::ChatCompletions || !compat_finish {
                                return Err(local_failure_with_output(
                                    FailureKind::Incomplete,
                                    any_output,
                                ));
                            }
                            return Ok(usage);
                        }
                        if api == WireApi::ChatCompletions && kind == TerminalEvent::Completed {
                            compat_finish = true;
                        }
                        if api == WireApi::Chatgpt && kind == TerminalEvent::Completed {
                            return Ok(usage);
                        }
                        terminal = true;
                        if matches!(kind, TerminalEvent::Failed) {
                            return Err(local_failure_with_output(FailureKind::Server, any_output));
                        }
                        if matches!(kind, TerminalEvent::Incomplete) {
                            return Err(local_failure_with_output(
                                FailureKind::Incomplete,
                                any_output,
                            ));
                        }
                    }
                }
            }
        }
    }

    if api == WireApi::ChatCompletions {
        return Err(local_failure_with_output(
            FailureKind::Transport,
            any_output,
        ));
    }
    if api == WireApi::Chatgpt && !terminal {
        return Err(local_failure_with_output(
            FailureKind::Transport,
            any_output,
        ));
    }
    Ok(usage)
}

pub fn can_fallback(failure: &StreamFailure, allowed_by_user: bool) -> bool {
    allowed_by_user
        && !failure.any_output
        && match failure.kind {
            FailureKind::Limited => {
                failure.status == Some(429)
                    || failure.code.as_deref() == Some("subscription_sharing_usage_limit_exceeded")
            }
            FailureKind::Auth => matches!(failure.status, Some(401 | 403)),
            FailureKind::Server => matches!(failure.status, Some(500..=599)),
            FailureKind::Timeout => true,
            _ => false,
        }
}

fn responses_body(request: &StreamRequest) -> Value {
    let input: Vec<_> = request.messages.iter().map(|message| {
        let kind = if message.role == "assistant" { "output_text" } else { "input_text" };
        json!({"type": "message", "role": message.role, "content": [{"type": kind, "text": message.content}]})
    }).collect();
    json!({"model": request.model, "instructions": request.instructions, "input": input, "store": false, "stream": true})
}

fn compat_body(request: &StreamRequest) -> Value {
    let mut messages =
        Vec::with_capacity(request.messages.len() + usize::from(request.instructions.is_some()));
    if let Some(instructions) = &request.instructions {
        messages.push(json!({ "role": "system", "content": instructions }));
    }
    messages.extend(request.messages.iter().map(|message| {
        json!({
            // Compatibility providers may only support system/user/assistant.
            // Quoted context stays below the trusted product instruction.
            "role": if message.role == "developer" { "user" } else { &message.role },
            "content": message.content,
        })
    }));
    json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    })
}

fn parse_event(api: WireApi, event: &SseEvent) -> Result<Vec<StreamEvent>, StreamFailure> {
    if event.data == "[DONE]" {
        return Ok(vec![StreamEvent::Terminal(TerminalEvent::Done)]);
    }
    let value: Value =
        serde_json::from_str(&event.data).map_err(|_| local_failure(FailureKind::Transport))?;
    if event.event.as_deref() == Some("error") || value.get("error").is_some() {
        return Err(structured_failure(&value, FailureKind::Server));
    }
    match api {
        WireApi::Chatgpt => parse_responses_event(event.event.as_deref(), &value),
        WireApi::ChatCompletions => parse_compat_event(&value),
    }
}

fn parse_responses_event(
    event: Option<&str>,
    value: &Value,
) -> Result<Vec<StreamEvent>, StreamFailure> {
    Ok(
        match event.or_else(|| value.get("type").and_then(Value::as_str)) {
            Some("response.output_text.delta") => value
                .get("delta")
                .and_then(Value::as_str)
                .map(|delta| vec![StreamEvent::Delta(delta.to_owned())])
                .unwrap_or_default(),
            Some("response.completed") => vec![
                usage_event(value),
                StreamEvent::Terminal(TerminalEvent::Completed),
            ],
            Some("response.failed") => return Err(structured_failure(value, FailureKind::Server)),
            Some("response.incomplete") => {
                return Err(structured_failure(value, FailureKind::Incomplete))
            }
            _ => Vec::new(),
        },
    )
}

fn parse_compat_event(value: &Value) -> Result<Vec<StreamEvent>, StreamFailure> {
    let mut events = Vec::new();
    if let Some(content) = value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
    {
        events.push(StreamEvent::Delta(content.to_owned()));
    }
    if let Some(reason) = value
        .pointer("/choices/0/finish_reason")
        .and_then(Value::as_str)
    {
        if !reason.is_empty() {
            events.push(StreamEvent::Usage(usage_from(value.get("usage"))));
            events.push(StreamEvent::Terminal(if reason == "stop" {
                TerminalEvent::Completed
            } else {
                TerminalEvent::Incomplete
            }));
        }
    }
    if value.get("usage").is_some() && events.is_empty() {
        events.push(StreamEvent::Usage(usage_from(value.get("usage"))));
    }
    Ok(events)
}

fn usage_event(value: &Value) -> StreamEvent {
    StreamEvent::Usage(usage_from(
        value
            .get("usage")
            .or_else(|| value.pointer("/response/usage")),
    ))
}

fn usage_from(value: Option<&Value>) -> StreamUsage {
    let Some(value) = value else {
        return StreamUsage::default();
    };
    StreamUsage {
        input_tokens: value
            .get("input_tokens")
            .or_else(|| value.get("prompt_tokens"))
            .and_then(Value::as_u64),
        output_tokens: value
            .get("output_tokens")
            .or_else(|| value.get("completion_tokens"))
            .and_then(Value::as_u64),
        cached_tokens: value
            .pointer("/input_tokens_details/cached_tokens")
            .or_else(|| value.pointer("/prompt_tokens_details/cached_tokens"))
            .and_then(Value::as_u64),
    }
}

fn structured_failure(value: &Value, fallback: FailureKind) -> StreamFailure {
    let error = value
        .get("error")
        .or_else(|| value.pointer("/response/error"))
        .unwrap_or(value);
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .and_then(safe_code);
    let kind = match code.as_deref() {
        Some("subscription_sharing_usage_limit_exceeded") => FailureKind::Limited,
        Some("subscription_sharing_invalid_user")
        | Some("chatpass_v2_scope_not_authorized")
        | Some("chatpass_v2_invalid_authorization_context") => FailureKind::Auth,
        _ => fallback,
    };
    StreamFailure {
        kind,
        status: None,
        request_id: None,
        body_shape: None,
        code,
        param: error
            .get("param")
            .and_then(Value::as_str)
            .and_then(safe_param),
        any_output: false,
    }
}

// Provider error bodies can reflect input; only known diagnostic fields may escape.
fn safe_code(value: &str) -> Option<String> {
    match value {
        "subscription_sharing_usage_limit_exceeded"
        | "subscription_sharing_invalid_user"
        | "subscription_sharing_user_not_eligible"
        | "subscription_sharing_usage_unavailable"
        | "subscription_sharing_unsupported_capability"
        | "subscription_sharing_route_not_supported"
        | "subscription_sharing_user_unavailable"
        | "chatpass_v2_scope_not_authorized"
        | "chatpass_v2_invalid_authorization_context"
        | "invalid_api_key"
        | "model_not_found"
        | "rate_limit_exceeded"
        | "insufficient_quota"
        | "invalid_request_error"
        | "server_error"
        | "context_length_exceeded" => Some(value.into()),
        _ => None,
    }
}
fn safe_param(value: &str) -> Option<String> {
    match value {
        "model" | "input" | "messages" | "instructions" | "stream" | "store" => Some(value.into()),
        _ => None,
    }
}

fn validate_endpoint(endpoint: &Url) -> Result<(), StreamFailure> {
    if endpoint.scheme() != "https"
        || endpoint.host_str().is_none()
        || !endpoint.username().is_empty()
        || endpoint.password().is_some()
        || endpoint.query().is_some()
        || endpoint.fragment().is_some()
    {
        return Err(local_failure(FailureKind::InvalidRequest));
    }
    Ok(())
}

fn serialize_body(body: &Value) -> Result<Vec<u8>, StreamFailure> {
    struct CappedBody(Vec<u8>);
    impl std::io::Write for CappedBody {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_INPUT.saturating_sub(self.0.len()) {
                return Err(std::io::Error::other("request body exceeds limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = CappedBody(Vec::new());
    serde_json::to_writer(&mut output, body)
        .map_err(|_| local_failure(FailureKind::InvalidRequest))?;
    Ok(output.0)
}

fn validate_request(request: &StreamRequest) -> Result<(), StreamFailure> {
    if request.model.trim().is_empty()
        || request.model.len() > 200
        || request.model.chars().any(char::is_control)
        || request.messages.is_empty()
        || request.messages.len() > 256
    {
        return Err(local_failure(FailureKind::InvalidRequest));
    }
    let mut bytes = request.instructions.as_ref().map_or(0, String::len);
    for message in &request.messages {
        if !matches!(
            message.role.as_str(),
            "system" | "developer" | "user" | "assistant"
        ) || message.content.is_empty()
            || message.content.len() > MAX_ANSWER
        {
            return Err(local_failure(FailureKind::InvalidRequest));
        }
        bytes += message.content.len();
    }
    if bytes > MAX_INPUT {
        return Err(local_failure(FailureKind::InvalidRequest));
    }
    Ok(())
}

fn http_kind(status: u16) -> FailureKind {
    match status {
        401 | 403 => FailureKind::Auth,
        408 => FailureKind::Timeout,
        429 => FailureKind::Limited,
        500..=599 => FailureKind::Server,
        400..=499 => FailureKind::InvalidRequest,
        _ => FailureKind::Http,
    }
}

fn transport_failure(timeout: bool) -> StreamFailure {
    local_failure(if timeout {
        FailureKind::Timeout
    } else {
        FailureKind::Transport
    })
}

fn local_failure(kind: FailureKind) -> StreamFailure {
    local_failure_with_output(kind, false)
}

fn local_failure_with_output(kind: FailureKind, any_output: bool) -> StreamFailure {
    StreamFailure {
        kind,
        status: None,
        request_id: None,
        body_shape: None,
        code: None,
        param: None,
        any_output,
    }
}

async fn read_http_error(
    response: reqwest::Response,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(Ok(chunk)) = stream.next().await {
        let remaining = MAX_HTTP_ERROR.saturating_sub(body.len());
        body.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        if body.len() == MAX_HTTP_ERROR {
            break;
        }
    }
    let Ok(value) = serde_json::from_slice::<Value>(&body) else {
        return (None, None, None);
    };
    let shape = Some(
        match &value {
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::String(_) => "string",
            Value::Number(_) => "number",
            Value::Bool(_) => "boolean",
            Value::Null => "null",
        }
        .to_owned(),
    );
    let error = value.get("error").unwrap_or(&value);
    (
        shape,
        error
            .get("code")
            .and_then(Value::as_str)
            .and_then(safe_code),
        error
            .get("param")
            .and_then(Value::as_str)
            .and_then(safe_param),
    )
}

fn merge_usage(target: &mut StreamUsage, value: StreamUsage) {
    target.input_tokens = value.input_tokens.or(target.input_tokens);
    target.output_tokens = value.output_tokens.or(target.output_tokens);
    target.cached_tokens = value.cached_tokens.or(target.cached_tokens);
}

#[derive(Debug)]
struct SseEvent {
    event: Option<String>,
    data: String,
}

#[derive(Default)]
struct SseParser {
    buffer: Vec<u8>,
    event: Option<String>,
    data: Vec<String>,
    size: usize,
}

impl SseParser {
    fn feed(&mut self, bytes: &[u8]) -> Result<Vec<SseEvent>, ()> {
        self.buffer.extend_from_slice(bytes);
        let mut events = Vec::new();
        while let Some(end) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let mut line = self.buffer.drain(..=end).collect::<Vec<_>>();
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            self.line(&line, &mut events)?;
        }
        if self.buffer.len() > MAX_EVENT {
            return Err(());
        }
        Ok(events)
    }

    fn line(&mut self, line: &[u8], events: &mut Vec<SseEvent>) -> Result<(), ()> {
        if line.is_empty() {
            if !self.data.is_empty() {
                events.push(SseEvent {
                    event: self.event.clone(),
                    data: self.data.join("\n"),
                });
            }
            self.event = None;
            self.data.clear();
            self.size = 0;
            return Ok(());
        }
        if line[0] == b':' {
            return Ok(());
        }
        let (field, value) = match line.iter().position(|byte| *byte == b':') {
            Some(index) => (&line[..index], &line[index + 1..]),
            None => (line, &[][..]),
        };
        let value = value.strip_prefix(b" ").unwrap_or(value);
        self.size = self.size.saturating_add(line.len());
        if self.size > MAX_EVENT {
            return Err(());
        }
        match field {
            b"event" => self.event = Some(String::from_utf8(value.to_vec()).map_err(|_| ())?),
            b"data" => self
                .data
                .push(String::from_utf8(value.to_vec()).map_err(|_| ())?),
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_keep_safe_receipts_without_provider_messages() {
        let mut failure = structured_failure(
            &json!({"error":{"code":"subscription_sharing_user_not_eligible", "param":"model", "message":"private transcript"}}),
            FailureKind::Auth,
        );
        failure.status = Some(403);
        failure.request_id = Some("req_public_receipt".into());
        let diagnostic = failure.to_string();
        assert!(diagnostic.contains("HTTP 403"));
        assert!(diagnostic.contains("subscription_sharing_user_not_eligible"));
        assert!(diagnostic.contains("req_public_receipt"));
        assert!(!diagnostic.contains("private transcript"));
        assert!(safe_code("arbitrary private text").is_none());
        assert!(!can_fallback(&failure, false));
    }

    #[test]
    fn compatibility_context_is_data_under_the_trusted_instruction() {
        let request = StreamRequest {
            model: "model".into(),
            instructions: Some("trusted rules".into()),
            messages: vec![
                TextMessage {
                    role: "developer".into(),
                    content: "quoted untrusted context".into(),
                },
                TextMessage {
                    role: "user".into(),
                    content: "current question".into(),
                },
            ],
        };
        let public = responses_body(&request);
        assert_eq!(public["instructions"], "trusted rules");
        assert_eq!(public["input"][0]["role"], "developer");
        assert_eq!(
            public["input"][0]["content"][0]["text"],
            "quoted untrusted context"
        );
        assert_eq!(public["store"], false);
        assert_eq!(public["stream"], true);
        let value = compat_body(&request);
        assert_eq!(value["messages"][0]["role"], "system");
        assert_eq!(value["messages"][1]["role"], "user");
        assert_eq!(value["messages"][1]["content"], "quoted untrusted context");
        assert_eq!(value["messages"][2]["content"], "current question");
    }

    #[test]
    fn bounded_requests_and_nested_limits_require_consent_without_output() {
        let mut request = StreamRequest {
            model: "discovered-model".into(),
            messages: vec![TextMessage {
                role: "user".into(),
                content: "new speech".into(),
            }],
            instructions: None,
        };
        assert!(validate_request(&request).is_ok());
        for api in [WireApi::Chatgpt, WireApi::ChatCompletions] {
            let mut escaped = request.clone();
            escaped.messages = (0..4)
                .map(|_| TextMessage {
                    role: "user".into(),
                    content: "\0".repeat(MAX_ANSWER),
                })
                .collect();
            assert!(validate_request(&escaped).is_ok());
            let body = match api {
                WireApi::Chatgpt => responses_body(&escaped),
                WireApi::ChatCompletions => compat_body(&escaped),
            };
            assert!(serialize_body(&body).is_err());
        }

        request.messages[0].role = "tool".into();
        assert!(validate_request(&request).is_err());
        request.messages[0].role = "user".into();
        request.instructions = Some("x".repeat(MAX_INPUT));
        assert!(validate_request(&request).is_err());
        let mut failure = structured_failure(
            &json!({"response":{"error":{"code":"subscription_sharing_usage_limit_exceeded"}}}),
            FailureKind::Server,
        );
        assert!(can_fallback(&failure, true));
        assert!(!can_fallback(&failure, false));
        failure.any_output = true;
        assert!(!can_fallback(&failure, true));
    }

    #[test]
    fn parser_handles_chunked_crlf_json_terminal_and_eof() {
        let mut parser = SseParser::default();
        let mut events = Vec::new();
        for chunk in [
            b": comment\r\neven:ignored\r\neve".as_slice(),
            b"nt: response.output_text.delta\r\ndata: {\"delta\":\"hel".as_slice(),
            b"lo\"}\r\ndata: {\"delta\":\" world\"}\r\n\r\n".as_slice(),
            b"event: response.failed\r\ndata: {\"error\":{\"code\":\"server_error\"}}\r\n\r\n"
                .as_slice(),
        ] {
            events.extend(parser.feed(chunk).unwrap());
        }
        assert_eq!(
            events[0].event.as_deref(),
            Some("response.output_text.delta")
        );
        assert_eq!(
            events[0].data,
            "{\"delta\":\"hello\"}\n{\"delta\":\" world\"}"
        );
        assert_eq!(events[1].event.as_deref(), Some("response.failed"));
        assert!(parser
            .feed(b"data: {\"delta\":\"orphan\"}")
            .unwrap()
            .is_empty());
        assert_eq!(
            parse_event(WireApi::Chatgpt, &events[1])
                .unwrap_err()
                .code
                .as_deref(),
            Some("server_error")
        );
    }
}
