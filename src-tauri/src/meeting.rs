use crate::speech::{SpeechConfig, SpeechEvent, SpeechSession};
use crate::store::{Meeting, MeetingStatus, Settings, Store, TranscriptSource};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

const MAX_ROWS: usize = 100;
const MAX_TEXT_BYTES: usize = 64 * 1024;
const MANIFEST: &str = include_str!("../models/manifest.json");

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptRow {
    pub id: i64,
    pub source: &'static str,
    pub start_ms: i64,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingState {
    pub status: String,
    pub meeting_id: Option<i64>,
    pub title: String,
    pub started_at: Option<i64>,
    pub transcript: Vec<TranscriptRow>,
    pub error: Option<String>,
}

impl Default for MeetingState {
    fn default() -> Self {
        Self {
            status: "idle".into(),
            meeting_id: None,
            title: String::new(),
            started_at: None,
            transcript: Vec::new(),
            error: None,
        }
    }
}

struct Shared {
    state: Mutex<MeetingState>,
}

pub struct MeetingController {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    stop: Option<SyncSender<()>>,
    worker_meeting: Option<(PathBuf, i64)>,
}

impl Default for MeetingController {
    fn default() -> Self {
        Self::new()
    }
}

impl MeetingController {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(MeetingState::default()),
            }),
            worker: None,
            stop: None,
            worker_meeting: None,
        }
    }

    pub fn state(&self) -> Result<MeetingState, String> {
        self.shared
            .state
            .lock()
            .map(|state| state.clone())
            .map_err(|_| "Meeting state is unavailable".into())
    }

    pub fn restore(
        &mut self,
        app: &AppHandle,
        meeting: Meeting,
        rows: Vec<TranscriptRow>,
    ) -> Result<MeetingState, String> {
        self.reap_finished()?;
        if self.worker.is_some()
            || matches!(
                self.state()?.status.as_str(),
                "starting" | "active" | "stopping"
            )
        {
            return Err(
                "Stop the current meeting before opening a saved meeting in the assistant".into(),
            );
        }
        let mut transcript = Vec::new();
        for row in rows {
            push_transcript_row(&mut transcript, row);
        }
        *self
            .shared
            .state
            .lock()
            .map_err(|_| "Meeting state is unavailable")? = MeetingState {
            status: "idle".into(),
            meeting_id: Some(meeting.id),
            title: meeting.title,
            started_at: Some(meeting.started_at),
            transcript,
            error: None,
        };
        emit_state(app, &self.shared);
        self.state()
    }

    pub fn reserve(&mut self, app: &AppHandle, title: String) -> Result<(), String> {
        self.reserve_state(title)?;
        emit_state(app, &self.shared);
        Ok(())
    }

    fn reserve_state(&mut self, title: String) -> Result<(), String> {
        self.reap_finished()?;
        if self.worker.is_some() {
            return Err("A meeting worker is still stopping".into());
        }
        let state = self.state()?;
        if matches!(state.status.as_str(), "starting" | "active" | "stopping") {
            return Err("A meeting is already running".into());
        }
        if let Ok(mut state) = self.shared.state.lock() {
            *state = MeetingState {
                status: "starting".into(),
                title,
                ..MeetingState::default()
            };
        }
        Ok(())
    }

    pub fn cancel_reservation(&mut self, app: &AppHandle) {
        if self.worker.is_none() {
            if let Ok(mut state) = self.shared.state.lock() {
                if state.status == "starting" && state.meeting_id.is_none() {
                    *state = MeetingState::default();
                }
            }
            emit_state(app, &self.shared);
        }
    }

    pub fn start(
        &mut self,
        app: AppHandle,
        db_path: PathBuf,
        meeting: Meeting,
        selected_mic_id: Option<String>,
        model_path: PathBuf,
    ) -> Result<(), String> {
        self.reap_finished()?;
        if self.worker.is_some() {
            return Err("A meeting worker is still stopping".into());
        }
        let current = self.state()?;
        if matches!(current.status.as_str(), "active" | "stopping")
            || (current.status == "starting" && current.meeting_id.is_some())
        {
            return Err("A meeting is already running".into());
        }
        {
            let mut state = self
                .shared
                .state
                .lock()
                .map_err(|_| "Meeting state is unavailable")?;
            *state = MeetingState {
                status: "starting".into(),
                meeting_id: Some(meeting.id),
                title: meeting.title.clone(),
                started_at: Some(meeting.started_at),
                transcript: Vec::new(),
                error: None,
            };
        }
        emit_state(&app, &self.shared);
        let (stop, stop_rx) = mpsc::sync_channel(1);
        let shared = Arc::clone(&self.shared);
        self.stop = Some(stop);
        let worker_app = app.clone();
        let worker_db_path = db_path.clone();
        let worker_meeting = meeting.clone();
        let worker = match thread::Builder::new()
            .name("meeting-worker".into())
            .spawn(move || {
                run_worker(
                    worker_app,
                    worker_db_path,
                    worker_meeting,
                    selected_mic_id,
                    model_path,
                    stop_rx,
                    shared,
                );
            }) {
            Ok(worker) => worker,
            Err(error) => {
                mark_interrupted(&db_path, meeting.id);
                fail(
                    &app,
                    &self.shared,
                    format!("Couldn't start meeting worker: {error}"),
                );
                return Err(error.to_string());
            }
        };
        self.worker_meeting = Some((db_path, meeting.id));
        self.worker = Some(worker);
        Ok(())
    }

    pub fn stop(&mut self, app: &AppHandle) -> Result<(), String> {
        let state = self.state()?;
        if state.status == "stopping" {
            return Err("Meeting is already stopping".into());
        }
        if !matches!(state.status.as_str(), "starting" | "active" | "error") {
            return Ok(());
        }
        if let Some(stop) = &self.stop {
            let _ = stop.try_send(());
        }
        if state.status != "error" {
            if let Ok(mut state) = self.shared.state.lock() {
                state.status = "stopping".into();
            }
        }
        emit_state(app, &self.shared);
        Ok(())
    }

    pub fn stop_and_wait(&mut self, app: &AppHandle) -> Result<(), String> {
        let _ = self.stop(app);
        self.join_worker()
    }

    fn join_worker(&mut self) -> Result<(), String> {
        let panicked = self
            .worker
            .take()
            .is_some_and(|worker| worker.join().is_err());
        self.stop = None;
        let meeting = self.worker_meeting.take();
        if panicked {
            if let Some((path, id)) = meeting {
                mark_interrupted(&path, id);
            }
            if let Ok(mut state) = self.shared.state.lock() {
                state.status = "error".into();
                state.error = Some("Meeting worker panicked".into());
            }
            return Err("Meeting worker panicked".into());
        }
        Ok(())
    }

    fn reap_finished(&mut self) -> Result<(), String> {
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            self.join_worker()?;
        }
        Ok(())
    }
}

fn now_ms() -> Result<i64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is invalid".into())
        .and_then(|duration| {
            i64::try_from(duration.as_millis()).map_err(|_| "System clock is out of range".into())
        })
}

fn emit_state(app: &AppHandle, shared: &Arc<Shared>) {
    if let Ok(state) = shared.state.lock().map(|state| state.clone()) {
        let _ = app.emit("meeting-state", state);
    }
}

fn fail(app: &AppHandle, shared: &Arc<Shared>, message: String) {
    if let Ok(mut state) = shared.state.lock() {
        state.status = "error".into();
        state.error = Some(message);
    }
    emit_state(app, shared);
}

fn run_worker(
    app: AppHandle,
    db_path: PathBuf,
    meeting: Meeting,
    selected_mic_id: Option<String>,
    model_path: PathBuf,
    stop_rx: mpsc::Receiver<()>,
    shared: Arc<Shared>,
) {
    let mut store = match Store::open(&db_path) {
        Ok(store) => store,
        Err(error) => {
            mark_interrupted(&db_path, meeting.id);
            fail(
                &app,
                &shared,
                format!("Couldn't open meeting store: {error}"),
            );
            return;
        }
    };
    if stop_rx.try_recv().is_ok() {
        let _ = store.transition_meeting(
            meeting.id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(meeting.started_at),
        );
        fail(&app, &shared, "Meeting stopped during startup".into());
        return;
    }
    if let Err(error) = verify_model(&model_path) {
        let _ = store.transition_meeting(
            meeting.id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(meeting.started_at),
        );
        fail(&app, &shared, error);
        return;
    }
    if stop_rx.try_recv().is_ok() {
        let _ = store.transition_meeting(
            meeting.id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(meeting.started_at),
        );
        fail(&app, &shared, "Meeting stopped during startup".into());
        return;
    }
    let config = SpeechConfig {
        model_path,
        selected_mic_id,
        ..SpeechConfig::default()
    };
    let session = match SpeechSession::start(config) {
        Ok(session) => session,
        Err(error) => {
            let _ = store.transition_meeting(
                meeting.id,
                MeetingStatus::Interrupted,
                now_ms().unwrap_or(meeting.started_at),
            );
            fail(&app, &shared, error);
            return;
        }
    };
    if stop_rx.try_recv().is_ok() {
        flush_session(&app, &shared, &mut store, meeting.id, session);
        let _ = store.transition_meeting(
            meeting.id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(meeting.started_at),
        );
        fail(&app, &shared, "Meeting stopped during startup".into());
        return;
    }
    if let Err(error) = store.transition_meeting(
        meeting.id,
        MeetingStatus::Active,
        now_ms().unwrap_or(meeting.started_at),
    ) {
        flush_session(&app, &shared, &mut store, meeting.id, session);
        let _ = store.transition_meeting(
            meeting.id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(meeting.started_at),
        );
        fail(&app, &shared, format!("Couldn't activate meeting: {error}"));
        return;
    }
    if let Ok(mut state) = shared.state.lock() {
        state.status = "active".into();
    }
    emit_state(&app, &shared);
    let mut stopping = false;
    let mut failed = false;
    loop {
        if stop_rx.try_recv().is_ok() {
            stopping = true;
            break;
        }
        match session.events().recv_timeout(Duration::from_millis(100)) {
            Ok(event) => {
                if process_event(&app, &shared, &mut store, meeting.id, event).is_err() {
                    failed = true;
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                fail(
                    &app,
                    &shared,
                    "Local speech workers stopped unexpectedly".into(),
                );
                failed = true;
                break;
            }
        }
    }
    if stopping
        && store
            .transition_meeting(
                meeting.id,
                MeetingStatus::Stopping,
                now_ms().unwrap_or(meeting.started_at),
            )
            .is_err()
    {
        failed = true;
        fail(&app, &shared, "Couldn't mark meeting as stopping".into());
    }
    failed |= flush_session(&app, &shared, &mut store, meeting.id, session);
    let at = now_ms().unwrap_or(meeting.started_at);
    let terminal = if stopping && !failed {
        MeetingStatus::Completed
    } else {
        MeetingStatus::Interrupted
    };
    let terminal_ok = store.transition_meeting(meeting.id, terminal, at).is_ok();
    if !terminal_ok {
        fail(
            &app,
            &shared,
            "Couldn't complete meeting persistence".into(),
        );
    }
    if stopping && !failed && terminal_ok {
        if let Ok(mut state) = shared.state.lock() {
            state.status = "idle".into();
            state.error = None;
        }
        emit_state(&app, &shared);
    }
}

// Drain every final even when an earlier event reports a failure.
fn flush_session(
    app: &AppHandle,
    shared: &Arc<Shared>,
    store: &mut Store,
    meeting_id: i64,
    session: SpeechSession,
) -> bool {
    let pending = match session.stop() {
        Ok(events) => events,
        Err(error) => {
            fail(app, shared, error);
            return true;
        }
    };
    let mut failed = false;
    for event in pending {
        failed |= process_event(app, shared, store, meeting_id, event).is_err();
    }
    failed
}

fn process_event(
    app: &AppHandle,
    shared: &Arc<Shared>,
    store: &mut Store,
    meeting_id: i64,
    event: SpeechEvent,
) -> Result<(), ()> {
    match event {
        SpeechEvent::Final {
            id,
            source,
            timestamp,
            revision,
            text,
        } => {
            let source_db = match source {
                crate::audio::AudioSource::System => TranscriptSource::System,
                crate::audio::AudioSource::Microphone => TranscriptSource::Microphone,
            };
            let row = store
                .insert_finalized_transcript(
                    meeting_id,
                    id,
                    source_db,
                    timestamp.as_millis() as i64,
                    None,
                    revision,
                    &text,
                )
                .map_err(|error| {
                    fail(app, shared, format!("Couldn't save transcript: {error}"));
                })?;
            let source = match source_db {
                TranscriptSource::System => "system",
                TranscriptSource::Microphone => "microphone",
            };
            if let Ok(mut state) = shared.state.lock() {
                push_transcript_row(
                    &mut state.transcript,
                    TranscriptRow {
                        id: row.id,
                        source,
                        start_ms: row.start_ms,
                        text: row.text,
                    },
                );
            }
            emit_state(app, shared);
            if source_db == TranscriptSource::System {
                crate::desktop::speech_finalized(app, meeting_id);
            }
        }
        SpeechEvent::Error { message, .. } => {
            fail(app, shared, message);
            return Err(());
        }
        SpeechEvent::Partial { .. } => {}
    }
    Ok(())
}

fn push_transcript_row(rows: &mut Vec<TranscriptRow>, row: TranscriptRow) {
    if let Some(existing) = rows.iter_mut().find(|existing| existing.id == row.id) {
        *existing = row;
    } else {
        rows.push(row);
    }
    rows.sort_by_key(|row| (row.start_ms, row.id));
    while rows.len() > MAX_ROWS
        || rows.iter().map(|row| row.text.len()).sum::<usize>() > MAX_TEXT_BYTES
    {
        rows.remove(0);
    }
}

#[derive(Deserialize)]
struct ModelManifest {
    filename: String,
    sha256: String,
    bytes: u64,
}

fn verify_model(path: &Path) -> Result<(), String> {
    let manifest: ModelManifest = serde_json::from_str(MANIFEST)
        .map_err(|_| "Bundled model manifest is invalid".to_string())?;
    if path.file_name().and_then(|name| name.to_str()) != Some(manifest.filename.as_str()) {
        return Err("Bundled model path is invalid".into());
    }
    let file = File::open(path).map_err(|_| "Bundled speech model is missing".to_string())?;
    if file
        .metadata()
        .map_err(|_| "Couldn't inspect bundled speech model".to_string())?
        .len()
        != manifest.bytes
    {
        return Err("Bundled speech model has an unexpected size".into());
    }
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|_| "Couldn't read bundled speech model".to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    if format!("{:x}", hasher.finalize()) != manifest.sha256 {
        return Err("Bundled speech model failed integrity verification".into());
    }
    Ok(())
}

pub fn create_meeting(
    store: &mut Store,
    title: &str,
    settings: &Settings,
) -> Result<Meeting, String> {
    store
        .create_meeting(title, now_ms()?, settings)
        .map_err(|error| error.to_string())
}

fn mark_interrupted(db_path: &Path, meeting_id: i64) {
    if let Ok(mut store) = Store::open(db_path) {
        let _ = store.transition_meeting(
            meeting_id,
            MeetingStatus::Interrupted,
            now_ms().unwrap_or(0),
        );
    }
}

pub fn model_path(resource_dir: &Path) -> PathBuf {
    let filename = serde_json::from_str::<ModelManifest>(MANIFEST)
        .map(|manifest| manifest.filename)
        .unwrap_or_else(|_| "ggml-base.en-q5_1.bin".into());
    resource_dir.join("models").join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_state_is_bounded_contract_shape() {
        let state = MeetingState::default();
        assert_eq!(state.status, "idle");
        assert!(state.meeting_id.is_none());
        assert!(state.transcript.is_empty());
    }

    #[test]
    fn reservation_blocks_overlap_before_database_creation() {
        let mut controller = MeetingController::new();
        controller.reserve_state("first".into()).unwrap();
        assert_eq!(controller.state().unwrap().status, "starting");
        assert!(controller.reserve_state("second".into()).is_err());
        assert_eq!(controller.state().unwrap().title, "first");
    }

    #[test]
    fn panicked_worker_can_be_replaced_after_reaping() {
        let path = std::env::temp_dir().join(format!("harness-panic-{}.db", uuid::Uuid::new_v4()));
        let mut store = Store::open(&path).unwrap();
        let meeting = create_meeting(&mut store, "panic", &Settings::default()).unwrap();
        store
            .transition_meeting(meeting.id, MeetingStatus::Active, meeting.started_at)
            .unwrap();
        let mut controller = MeetingController::new();
        controller.shared.state.lock().unwrap().status = "active".into();
        controller.worker_meeting = Some((path.clone(), meeting.id));
        controller.worker = Some(thread::spawn(|| panic!("test worker")));
        while !controller.worker.as_ref().unwrap().is_finished() {
            thread::yield_now();
        }
        assert!(controller.reserve_state("retry".into()).is_err());
        assert_eq!(controller.state().unwrap().status, "error");
        assert_eq!(
            store.meeting(meeting.id).unwrap().status,
            MeetingStatus::Interrupted
        );
        controller.reserve_state("retry".into()).unwrap();
        assert_eq!(controller.state().unwrap().title, "retry");
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn transcript_buffer_upserts_sorts_and_bounds_utf8_bytes() {
        let mut rows = Vec::new();
        push_transcript_row(
            &mut rows,
            TranscriptRow {
                id: 2,
                source: "system",
                start_ms: 200,
                text: "late".into(),
            },
        );
        push_transcript_row(
            &mut rows,
            TranscriptRow {
                id: 1,
                source: "microphone",
                start_ms: 100,
                text: "earlier".into(),
            },
        );
        push_transcript_row(
            &mut rows,
            TranscriptRow {
                id: 2,
                source: "system",
                start_ms: 200,
                text: "revised".into(),
            },
        );
        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(rows[1].text, "revised");

        push_transcript_row(
            &mut rows,
            TranscriptRow {
                id: 3,
                source: "system",
                start_ms: 300,
                text: "🙂".repeat(MAX_TEXT_BYTES / 4),
            },
        );
        assert!(rows.iter().map(|row| row.text.len()).sum::<usize>() <= MAX_TEXT_BYTES);
        assert_eq!(rows.len(), 1);
    }
}
