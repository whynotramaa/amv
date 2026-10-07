use anyhow::{bail, Context, Result};
use rusqlite::{params, Connection, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[path = "usage.rs"]
mod usage;
pub use usage::*;

#[path = "budgets.rs"]
mod budgets;
pub use budgets::*;

const SCHEMA_VERSION: i64 = 11;
const MAX_CONTEXT_REQUEST: usize = 2 * 1024 * 1024;
const MAX_CONTEXT_METADATA: usize = 1024 * 1024;
const MAX_DOCUMENT_TEXT: usize = 256 * 1024;
const MAX_DOCUMENT_CHUNK: usize = 4096;
const MAX_MEMORY_BODY: usize = 16 * 1024;
const MAX_MEMORY_PAGE: usize = 50;
const MAX_MEMORY_PAGE_BYTES: usize = 64 * 1024;

const MAX_MEETING_TITLE: usize = 200;
const MAX_TRANSCRIPT_TEXT: usize = 64 * 1024;
const MAX_TRANSCRIPT_PAGE: u32 = 500;
const MAX_SAVED_MEETINGS: usize = 100;
const MAX_TRANSCRIPT_SEARCH_TERMS: usize = 16;
const MAX_TRANSCRIPT_SEARCH_QUERY: usize = 512;
const MAX_TIME_MS: i64 = 100_000_000_000_000;
const MAX_MESSAGE_HISTORY: usize = 32;
const MAX_REQUEST_TEXT: usize = 64 * 1024;
const MAX_ANSWER: usize = 256 * 1024;
const MAX_HISTORY_BYTES: usize = 64 * 1024;
const MAX_REQUEST_ID_TEXT: usize = 200;
const MAX_REQUEST_ERROR: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub category: String,
    pub project: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPage {
    pub entries: Vec<MemoryEntry>,
    pub has_more: bool,
    pub next: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInput {
    pub title: String,
    pub body: String,
    pub category: String,
    pub project: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
pub struct DocumentInput {
    pub title: String,
    pub source_path: String,
    pub stored_path: Option<String>,
    pub content_hash: String,
    pub modified_at: i64,
    pub indexing_version: u32,
    pub text: String,
    pub project: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: i64,
    pub title: String,
    pub source_path: String,
    pub source_policy: String,
    pub content_hash: String,
    pub modified_at: i64,
    pub indexed_at: i64,
    pub indexing_version: u32,
    pub project: Option<String>,
    pub enabled: bool,
    pub chunk_count: usize,
    pub text_bytes: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPage {
    pub documents: Vec<Document>,
    pub has_more: bool,
    pub next: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct DocumentSource {
    pub source_path: String,
    pub stored_path: Option<String>,
    pub project: Option<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    pub id: i64,
    pub document_id: i64,
    pub title: String,
    pub source_path: String,
    pub content_hash: String,
    pub indexing_version: u32,
    pub chunk_index: usize,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeetingStatus {
    Starting,
    Active,
    Paused,
    Stopping,
    Completed,
    Interrupted,
}

impl MeetingStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Stopping => "stopping",
            Self::Completed => "completed",
            Self::Interrupted => "interrupted",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "starting" => Ok(Self::Starting),
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "stopping" => Ok(Self::Stopping),
            "completed" => Ok(Self::Completed),
            "interrupted" => Ok(Self::Interrupted),
            _ => bail!("invalid meeting status {value}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscriptSource {
    System,
    Microphone,
}

impl TranscriptSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Microphone => "microphone",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "system" => Ok(Self::System),
            "microphone" => Ok(Self::Microphone),
            _ => bail!("invalid transcript source {value}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meeting {
    pub id: i64,
    pub title: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub status: MeetingStatus,
    pub settings_snapshot: Settings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptSegment {
    pub id: i64,
    pub meeting_id: i64,
    pub event_id: u64,
    pub source: TranscriptSource,
    pub start_ms: i64,
    pub end_ms: Option<i64>,
    pub revision: u32,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TranscriptCursor {
    pub start_ms: i64,
    pub row_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptPage {
    pub segments: Vec<TranscriptSegment>,
    pub next: Option<TranscriptCursor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
    Transcript,
    Question,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    InFlight,
    Completed,
    Partial,
}

impl RequestStatus {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "inflight" => Ok(Self::InFlight),
            "completed" => Ok(Self::Completed),
            "partial" => Ok(Self::Partial),
            _ => bail!("invalid request status {value}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestUsage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub total_tokens: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSnapshot {
    pub credential_id: Option<String>,
    pub account_id: Option<String>,
    pub id: i64,
    pub request_id: i64,
    pub provider: String,
    pub model: String,
    pub created_at: i64,
    pub request: serde_json::Value,
    pub metadata: serde_json::Value,
    pub upstream_omitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingRequest {
    pub id: i64,
    pub meeting_id: i64,
    pub kind: RequestKind,
    pub status: RequestStatus,
    pub user_text: String,
    pub cursor_before: Option<i64>,
    pub cursor_after: Option<i64>,
    pub segment_ids: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageHistoryEntry {
    pub request_id: i64,
    pub meeting_id: i64,
    pub role: String,
    pub text: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub usage: RequestUsage,
}

#[derive(Clone, Debug)]
pub struct MessageHistory {
    pub entries: Vec<MessageHistoryEntry>,
    pub omitted_requests: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountMetadata {
    pub account_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub client_id: String,
    #[serde(default)]
    pub selected_model: Option<String>,
}

impl AccountMetadata {
    pub fn validate(&self) -> Result<()> {
        if self.account_id.len() != 43
            || !self
                .account_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.client_id.is_empty()
            || self.client_id.len() > 512
            || self.client_id == "dynamic_agent_client"
            || self.client_id.chars().any(char::is_control)
        {
            bail!("Invalid account metadata");
        }
        if self.selected_model.as_ref().is_some_and(|model| {
            model.is_empty() || model.len() > 200 || model.chars().any(char::is_control)
        }) {
            bail!("Invalid selected model");
        }
        Ok(())
    }
}

pub fn validate_account_id(account_id: &str) -> Result<()> {
    if account_id.len() != 43
        || !account_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        bail!("Invalid account id")
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub send_mode: String,
    pub response_mode: String,
    pub custom_instruction: String,
    pub overlay_shortcut: String,
    pub send_shortcut: String,
    #[serde(default = "default_new_chat_shortcut")]
    pub new_chat_shortcut: String,
    pub launch_on_login: bool,
    #[serde(default = "default_true")]
    pub include_microphone: bool,
    #[serde(default = "default_auto_send_delay")]
    pub auto_send_delay_ms: u32,
}

fn default_true() -> bool {
    true
}

fn default_new_chat_shortcut() -> String {
    "Ctrl+Alt+N".into()
}

fn default_auto_send_delay() -> u32 {
    2000
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            send_mode: "on_hotkey".into(),
            response_mode: "suggested_answers".into(),
            custom_instruction: String::new(),
            overlay_shortcut: "Ctrl+Space".into(),
            send_shortcut: "Ctrl+Shift+Enter".into(),
            new_chat_shortcut: default_new_chat_shortcut(),
            launch_on_login: false,
            include_microphone: true,
            auto_send_delay_ms: default_auto_send_delay(),
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.send_mode.as_str(), "on_hotkey" | "automatic") {
            bail!("Send mode must be on_hotkey or automatic");
        }
        if !(500..=30_000).contains(&self.auto_send_delay_ms) {
            bail!("Auto-send delay must be between 0.5 and 30 seconds");
        }
        if !matches!(
            self.response_mode.as_str(),
            "suggested_answers" | "summary" | "custom"
        ) {
            bail!("Response mode must be suggested_answers, summary, or custom");
        }
        let instruction_len = self.custom_instruction.chars().count();
        if instruction_len > 4000 || self.custom_instruction.len() > 8192 {
            bail!("Custom instruction must be at most 4,000 characters and 8,192 UTF-8 bytes");
        }
        if self.response_mode == "custom" && self.custom_instruction.trim().is_empty() {
            bail!("Custom instruction is required for custom response mode");
        }
        if self.overlay_shortcut.trim().is_empty() || self.overlay_shortcut.chars().count() > 100 {
            bail!("Overlay shortcut must be between 1 and 100 characters");
        }
        if self.send_shortcut.trim().is_empty() || self.send_shortcut.chars().count() > 100 {
            bail!("Send shortcut must be between 1 and 100 characters");
        }
        if self
            .overlay_shortcut
            .trim()
            .eq_ignore_ascii_case(self.send_shortcut.trim())
        {
            bail!("Overlay and send shortcuts must be different");
        }
        if self.new_chat_shortcut.trim().is_empty() || self.new_chat_shortcut.chars().count() > 100
        {
            bail!("New chat shortcut must be between 1 and 100 characters");
        }
        Ok(())
    }
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let mut conn = Connection::open(path).context("open database")?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .context("enable foreign keys")?;
        conn.busy_timeout(Duration::from_secs(5))
            .context("set database busy timeout")?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("enable WAL journal")?;

        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .context("read schema version")?;
        if version > SCHEMA_VERSION {
            bail!("database schema version {version} is newer than supported version {SCHEMA_VERSION}");
        }
        if version < SCHEMA_VERSION {
            let tx = conn.transaction().context("start migration")?;
            if version < 1 {
                tx.execute_batch(include_str!("../migrations/001_initial.sql"))
                    .context("apply initial migration")?;
            }
            if version < 2 {
                tx.execute_batch(include_str!("../migrations/002_provider_settings.sql"))
                    .context("apply provider migration")?;
            }
            if version < 3 {
                tx.execute_batch(include_str!("../migrations/003_accounts.sql"))
                    .context("apply account migration")?;
            }
            if version < 4 {
                tx.execute_batch(include_str!(
                    "../migrations/004_transcript_event_identity.sql"
                ))
                .context("apply transcript identity migration")?;
            }
            if version < 5 {
                tx.execute_batch(include_str!("../migrations/005_message_delivery.sql"))
                    .context("apply message delivery migration")?;
            }
            if version < 6 {
                tx.execute_batch(include_str!("../migrations/006_transcript_slices.sql"))
                    .context("migrate transcript slices")?;
            }
            if version < 7 {
                tx.execute_batch(include_str!("../migrations/007_manual_memory.sql"))
                    .context("migrate manual memory")?;
            }
            if version < 8 {
                tx.execute_batch(include_str!("../migrations/008_documents.sql"))
                    .context("apply document migration")?;
            }
            if version < 9 {
                tx.execute_batch(include_str!("../migrations/009_request_context.sql"))
                    .context("apply request context migration")?;
            }
            if version < 10 {
                tx.execute_batch(include_str!("../migrations/010_usage.sql"))
                    .context("apply usage migration")?;
            }
            if version < 11 {
                tx.execute_batch(include_str!("../migrations/011_api_budgets.sql"))
                    .context("apply API budget migration")?;
            }
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)
                .context("set schema version")?;
            tx.commit().context("commit migration")?;
        }
        Ok(Self { conn })
    }

    /// Called once at startup, never by readers opening another connection.
    pub fn recover_interrupted_requests(&self) -> Result<usize> {
        Ok(self.conn.execute(
            "UPDATE message_requests
             SET status = CASE WHEN assistant_text IS NULL OR assistant_text = ''
                               THEN 'pending' ELSE 'partial' END,
                 completed_at = CASE WHEN assistant_text IS NULL OR assistant_text = ''
                                     THEN NULL ELSE unixepoch() END
             WHERE status = 'inflight'",
            [],
        )?)
    }

    pub fn settings(&self) -> Result<Settings> {
        let value = self
            .conn
            .query_row("SELECT value FROM settings WHERE id = 1", [], |row| {
                row.get::<_, String>(0)
            });
        match value {
            Ok(value) => {
                let settings: Settings =
                    serde_json::from_str(&value).context("stored settings are corrupt")?;
                settings.validate().context("stored settings are invalid")?;
                Ok(settings)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Settings::default()),
            Err(error) => Err(error).context("read settings"),
        }
    }

    pub fn save_settings(&mut self, settings: &Settings) -> Result<()> {
        settings.validate()?;
        let value = serde_json::to_string(settings).context("encode settings")?;
        let tx = self.conn.transaction().context("start settings save")?;
        tx.execute(
            "INSERT INTO settings (id, value) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET value = excluded.value",
            params![value],
        )?;
        tx.commit().context("commit settings save")?;
        Ok(())
    }

    pub fn providers(&self) -> Result<Vec<crate::providers::ProviderConfig>> {
        let value = self.conn.query_row(
            "SELECT value FROM provider_settings WHERE id = 1",
            [],
            |row| row.get::<_, String>(0),
        );
        match value {
            Ok(value) => {
                let configs: Vec<crate::providers::ProviderConfig> =
                    serde_json::from_str(&value).context("stored provider settings are corrupt")?;
                crate::providers::validate_all(&configs)?;
                Ok(configs)
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(crate::providers::defaults()),
            Err(error) => Err(error).context("read provider settings"),
        }
    }

    pub fn save_providers(&mut self, configs: &[crate::providers::ProviderConfig]) -> Result<()> {
        crate::providers::validate_all(configs)?;
        let value = serde_json::to_string(configs)?;
        self.conn.execute("INSERT INTO provider_settings (id, value) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET value = excluded.value", params![value])?;
        Ok(())
    }

    pub fn oauth_host_id(&self) -> Result<String> {
        self.conn.execute(
            "INSERT OR IGNORE INTO oauth_host (id, host_id) VALUES (1, ?1)",
            [uuid::Uuid::new_v4().urn().to_string()],
        )?;
        self.conn
            .query_row("SELECT host_id FROM oauth_host WHERE id = 1", [], |r| {
                r.get(0)
            })
            .context("read OAuth host identifier")
    }

    pub fn accounts(&self) -> Result<Vec<AccountMetadata>> {
        let mut statement = self
            .conn
            .prepare("SELECT metadata FROM chatgpt_accounts ORDER BY rowid")?;
        let rows = statement.query_map([], |r| r.get::<_, String>(0))?;
        rows.map(|row| {
            let account: AccountMetadata = serde_json::from_str(&row?)?;
            account.validate()?;
            Ok(account)
        })
        .collect()
    }

    pub fn save_account(&self, account: &AccountMetadata) -> Result<()> {
        account.validate()?;
        self.conn.execute("INSERT INTO chatgpt_accounts (id, metadata) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET metadata = excluded.metadata",
            params![account.account_id, serde_json::to_string(account)?])?;
        Ok(())
    }

    pub fn set_selected_model(&self, account_id: &str, model: &str) -> Result<()> {
        validate_account_id(account_id)?;
        if model.is_empty() || model.len() > 200 || model.chars().any(char::is_control) {
            bail!("Invalid selected model");
        }
        let mut account = self
            .accounts()?
            .into_iter()
            .find(|account| account.account_id == account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;
        account.selected_model = Some(model.to_owned());
        self.save_account(&account)
    }

    pub fn deselect_selected_model(&self, account_id: &str, model: &str) -> Result<()> {
        validate_account_id(account_id)?;
        let mut account = self
            .accounts()?
            .into_iter()
            .find(|account| account.account_id == account_id)
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;
        if account.selected_model.as_deref() == Some(model) {
            account.selected_model = None;
            self.save_account(&account)?;
        }
        Ok(())
    }

    pub fn active_account(&self) -> Result<Option<String>> {
        use rusqlite::OptionalExtension;
        let account = self
            .conn
            .query_row(
                "SELECT account_id FROM active_chatgpt_account WHERE id = 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        if let Some(account) = &account {
            validate_account_id(account)?;
        }
        Ok(account)
    }

    pub fn select_account(&self, account_id: &str) -> Result<()> {
        validate_account_id(account_id)?;
        self.conn.execute("INSERT INTO active_chatgpt_account (id, account_id) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET account_id = excluded.account_id", [account_id])?;
        Ok(())
    }

    pub fn clear_active_if_matching(&self, account_id: &str) -> Result<()> {
        validate_account_id(account_id)?;
        self.conn.execute(
            "DELETE FROM active_chatgpt_account WHERE id = 1 AND account_id = ?1",
            [account_id],
        )?;
        Ok(())
    }

    pub fn remove_account(&self, account_id: &str) -> Result<()> {
        validate_account_id(account_id)?;
        self.conn
            .execute("DELETE FROM chatgpt_accounts WHERE id = ?1", [account_id])?;
        Ok(())
    }

    pub fn create_meeting(
        &mut self,
        title: &str,
        started_at: i64,
        settings: &Settings,
    ) -> Result<Meeting> {
        validate_time(started_at)?;
        let title = validate_title(title)?;
        settings.validate()?;
        let snapshot = serde_json::to_string(settings).context("encode meeting settings")?;
        let tx = self.conn.transaction().context("start meeting creation")?;
        tx.execute(
            "INSERT INTO meetings (started_at, title, status, settings_snapshot)
             VALUES (?1, ?2, 'starting', ?3)",
            params![started_at, title, snapshot],
        )?;
        let meeting = Meeting {
            id: tx.last_insert_rowid(),
            title,
            started_at,
            ended_at: None,
            status: MeetingStatus::Starting,
            settings_snapshot: settings.clone(),
        };
        tx.commit().context("commit meeting creation")?;
        Ok(meeting)
    }

    pub fn set_meeting_response(&mut self, id: i64, settings: &Settings) -> Result<()> {
        validate_id(id, "meeting")?;
        settings.validate()?;
        let mut snapshot = read_meeting(&self.conn, id)?.settings_snapshot;
        snapshot.response_mode = settings.response_mode.clone();
        snapshot.custom_instruction = settings.custom_instruction.clone();
        self.conn.execute(
            "UPDATE meetings SET settings_snapshot = ?1 WHERE id = ?2",
            params![serde_json::to_string(&snapshot)?, id],
        )?;
        Ok(())
    }

    pub fn transition_meeting(&mut self, id: i64, next: MeetingStatus, at: i64) -> Result<Meeting> {
        if id <= 0 {
            bail!("meeting id must be positive");
        }
        validate_time(at)?;
        let tx = self
            .conn
            .transaction()
            .context("start meeting transition")?;
        let current = read_meeting(&tx, id)?;
        if !valid_transition(current.status, next) {
            bail!(
                "invalid meeting transition {:?} -> {:?}",
                current.status,
                next
            );
        }
        if at < current.started_at {
            bail!("meeting transition time precedes meeting start");
        }
        let ended_at =
            matches!(next, MeetingStatus::Completed | MeetingStatus::Interrupted).then_some(at);
        tx.execute(
            "UPDATE meetings SET status = ?1, ended_at = ?2 WHERE id = ?3",
            params![next.as_str(), ended_at, id],
        )?;
        let meeting = Meeting {
            status: next,
            ended_at,
            ..current
        };
        tx.commit().context("commit meeting transition")?;
        Ok(meeting)
    }

    pub fn recover_stale_meetings(&mut self, at: i64) -> Result<usize> {
        validate_time(at)?;
        Ok(self.conn.execute(
            "UPDATE meetings SET status = 'interrupted', ended_at = ?1
             WHERE status IN ('starting', 'active', 'paused', 'stopping')
               AND ended_at IS NULL AND started_at <= ?1",
            [at],
        )?)
    }

    pub fn meeting(&self, id: i64) -> Result<Meeting> {
        validate_id(id, "meeting")?;
        read_meeting(&self.conn, id)
    }

    pub fn list_saved_meetings(
        &self,
        before_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<Meeting>> {
        if let Some(before_id) = before_id {
            validate_id(before_id, "meeting cursor")?;
        }
        if !(1..=MAX_SAVED_MEETINGS).contains(&limit) {
            bail!("saved meeting limit must be between 1 and {MAX_SAVED_MEETINGS}");
        }
        let mut statement = self.conn.prepare(
            "SELECT id, title, started_at, ended_at, status, settings_snapshot
             FROM meetings
             WHERE status IN ('completed', 'interrupted')
               AND (?1 IS NULL OR id < ?1)
             ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![before_id, limit as i64], meeting_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_finalized_transcript(
        &mut self,
        meeting_id: i64,
        event_id: u64,
        source: TranscriptSource,
        start_ms: i64,
        end_ms: Option<i64>,
        revision: u32,
        text: &str,
    ) -> Result<TranscriptSegment> {
        validate_id(meeting_id, "meeting")?;
        if event_id == 0 || event_id > i64::MAX as u64 || revision == 0 {
            bail!("event id and revision must be positive");
        }
        validate_time(start_ms)?;
        if let Some(end_ms) = end_ms {
            validate_time(end_ms)?;
            if end_ms < start_ms {
                bail!("transcript end precedes start");
            }
        }
        let text = validate_text(text)?;
        let tx = self.conn.transaction().context("start transcript insert")?;
        if tx
            .query_row("SELECT 1 FROM meetings WHERE id = ?1", [meeting_id], |_| {
                Ok(())
            })
            .is_err()
        {
            bail!("meeting does not exist");
        }
        tx.execute(
            "INSERT INTO transcript_segments
                (meeting_id, event_id, start_ms, end_ms, source, is_final, revision, text)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)
             ON CONFLICT(meeting_id, source, event_id) DO UPDATE SET
                start_ms = excluded.start_ms, end_ms = excluded.end_ms,
                revision = excluded.revision, text = excluded.text
             WHERE excluded.revision > transcript_segments.revision",
            params![
                meeting_id,
                event_id as i64,
                start_ms,
                end_ms,
                source.as_str(),
                revision,
                text
            ],
        )?;
        let result = tx.query_row(
            "SELECT id, meeting_id, event_id, source, start_ms, end_ms, revision, text
             FROM transcript_segments WHERE meeting_id = ?1 AND source = ?2 AND event_id = ?3",
            params![meeting_id, source.as_str(), event_id as i64],
            transcript_from_row,
        )?;
        tx.commit().context("commit transcript insert")?;
        Ok(result)
    }

    pub fn read_transcript_page(
        &self,
        meeting_id: i64,
        cursor: Option<TranscriptCursor>,
        limit: u32,
    ) -> Result<TranscriptPage> {
        validate_id(meeting_id, "meeting")?;
        if let Some(cursor) = cursor {
            validate_time(cursor.start_ms)?;
            validate_id(cursor.row_id, "transcript cursor")?;
        }
        if limit == 0 || limit > MAX_TRANSCRIPT_PAGE {
            bail!("transcript page limit must be between 1 and {MAX_TRANSCRIPT_PAGE}");
        }
        let mut statement = self.conn.prepare(
            "SELECT id, meeting_id, event_id, source, start_ms, end_ms, revision, text
             FROM transcript_segments
             WHERE meeting_id = ?1 AND is_final = 1
               AND (?2 IS NULL OR start_ms > ?2 OR (start_ms = ?2 AND id > ?3))
             ORDER BY start_ms, id LIMIT ?4",
        )?;
        let rows = statement.query_map(
            params![
                meeting_id,
                cursor.map(|value| value.start_ms),
                cursor.map(|value| value.row_id),
                limit,
            ],
            transcript_from_row,
        )?;
        let segments: Vec<_> = rows.collect::<rusqlite::Result<_>>()?;
        let next = segments
            .last()
            .map(|segment: &TranscriptSegment| TranscriptCursor {
                start_ms: segment.start_ms,
                row_id: segment.id,
            });
        Ok(TranscriptPage { segments, next })
    }

    /// Newest finalized context, bounded before copying text out of SQLite.
    pub fn recent_transcript(
        &self,
        meeting_id: i64,
        since_ms: i64,
        include_microphone: bool,
    ) -> Result<(Vec<TranscriptSegment>, bool)> {
        validate_id(meeting_id, "meeting")?;
        validate_time(since_ms)?;
        let mut statement = self.conn.prepare(
            "SELECT id, meeting_id, event_id, source, start_ms, end_ms, revision, text,
                    length(CAST(text AS BLOB)) FROM transcript_segments
             WHERE meeting_id = ?1 AND is_final = 1 AND start_ms >= ?2 AND (?3 OR source = 'system')
             ORDER BY start_ms DESC, id DESC LIMIT 101",
        )?;
        let mut query = statement.query(params![meeting_id, since_ms, include_microphone])?;
        let mut rows = Vec::new();
        let mut bytes = 0usize;
        let mut omitted = false;
        while let Some(row) = query.next()? {
            let size = usize::try_from(row.get::<_, i64>(8)?)?;
            if rows.len() == 100 || bytes.saturating_add(size) > 64 * 1024 {
                omitted = true;
                break;
            }
            bytes += size;
            rows.push(transcript_from_row(row)?);
        }
        rows.reverse();
        Ok((rows, omitted))
    }

    pub fn finalized_system_after(
        &self,
        meeting_id: i64,
        after_id: Option<i64>,
        limit: u32,
    ) -> Result<TranscriptPage> {
        validate_id(meeting_id, "meeting")?;
        if let Some(after_id) = after_id {
            validate_id(after_id, "transcript cursor")?;
        }
        if limit == 0 || limit > MAX_TRANSCRIPT_PAGE {
            bail!("transcript page limit must be between 1 and {MAX_TRANSCRIPT_PAGE}");
        }
        let mut statement = self.conn.prepare(
            "SELECT id, meeting_id, event_id, source, start_ms, end_ms, revision, text
             FROM transcript_segments
             WHERE meeting_id = ?1 AND source = 'system' AND is_final = 1
               AND (?2 IS NULL OR id > ?2)
             ORDER BY id LIMIT ?3",
        )?;
        let rows =
            statement.query_map(params![meeting_id, after_id, limit], transcript_from_row)?;
        let segments: Vec<_> = rows.collect::<rusqlite::Result<_>>()?;
        let next = segments
            .last()
            .map(|segment: &TranscriptSegment| TranscriptCursor {
                start_ms: segment.start_ms,
                row_id: segment.id,
            });
        Ok(TranscriptPage { segments, next })
    }

    pub fn latest_finalized_id(&self, meeting_id: i64) -> Result<Option<i64>> {
        validate_id(meeting_id, "meeting")?;
        Ok(self.conn.query_row(
            "SELECT MAX(id) FROM transcript_segments WHERE meeting_id = ?1 AND is_final = 1",
            [meeting_id],
            |row| row.get(0),
        )?)
    }

    pub fn prepare_transcript_request(
        &mut self,
        meeting_id: i64,
        max_rows: u32,
        max_bytes: usize,
    ) -> Result<Option<PendingRequest>> {
        self.prepare_transcript_through(meeting_id, max_rows, max_bytes, None)
    }

    pub fn prepare_transcript_through(
        &mut self,
        meeting_id: i64,
        max_rows: u32,
        max_bytes: usize,
        through_id: Option<i64>,
    ) -> Result<Option<PendingRequest>> {
        if let Some(id) = through_id {
            validate_id(id, "transcript upper bound")?;
        }
        validate_id(meeting_id, "meeting")?;
        if max_rows == 0 || max_rows > 100 || max_bytes == 0 || max_bytes > MAX_REQUEST_TEXT {
            bail!("transcript request bounds are max 100 rows and 64 KiB");
        }
        let max_rows = i64::from(max_rows);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("start transcript request")?;
        read_meeting(&tx, meeting_id)?;
        if let Some(existing) = active_request(&tx, meeting_id)? {
            if existing.kind != RequestKind::Transcript {
                bail!("meeting already has a pending request");
            }
            return Ok(Some(read_pending_request(&tx, existing.id)?));
        }
        use rusqlite::OptionalExtension;
        let delivered: Option<(i64, i64, usize)> = tx
            .query_row(
                "SELECT id, cursor_after, cursor_after_offset FROM message_requests
             WHERE meeting_id = ?1 AND kind = 'transcript' AND status IN ('completed', 'partial')
             ORDER BY cursor_after DESC, id DESC LIMIT 1",
                [meeting_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get::<_, i64>(2)? as usize)),
            )
            .optional()?;
        let cursor_before = delivered.map(|(_, id, _)| id);
        let offset = delivered.map_or(0, |(_, _, offset)| offset);
        let tail: Option<(u32, String)> = if offset > 0 {
            Some(tx.query_row("SELECT revision, full_text FROM message_request_segments WHERE request_id = ?1 AND segment_id = ?2", params![delivered.unwrap().0, cursor_before], |row| Ok((row.get(0)?, row.get(1)?)))?)
        } else {
            None
        };
        let mut statement = tx.prepare(
            "SELECT id, revision, text FROM transcript_segments
             WHERE meeting_id = ?1 AND is_final = 1
               AND (?2 IS NULL OR id > ?2 OR (id = ?2 AND ?5 > 0))
               AND (?3 IS NULL OR id <= ?3)
             ORDER BY id LIMIT ?4",
        )?;
        let candidates = statement.query_map(
            params![
                meeting_id,
                cursor_before,
                through_id,
                max_rows,
                offset as i64
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )?;
        let mut frozen = Vec::new();
        let mut bytes: usize = 0;
        let mut cursor_after_offset = 0;
        for candidate in candidates {
            let (id, mut revision, mut text) = candidate?;
            let start = if Some(id) == cursor_before { offset } else { 0 };
            if start > 0 {
                let (frozen_revision, full_text) =
                    tail.as_ref().context("missing frozen transcript tail")?;
                revision = *frozen_revision;
                text = full_text.clone();
            }
            if start >= text.len() || !text.is_char_boundary(start) {
                bail!("invalid transcript slice offset");
            }
            let separator = usize::from(!frozen.is_empty());
            if bytes
                .saturating_add(separator)
                .saturating_add(text.len() - start)
                > max_bytes
                && !frozen.is_empty()
            {
                break;
            }
            let mut end = text
                .len()
                .min(start.saturating_add(max_bytes - bytes - separator));
            while end > start && !text.is_char_boundary(end) {
                end -= 1;
            }
            if end == start {
                bail!("transcript byte bound cannot hold one character");
            }
            let part = text[start..end].to_owned();
            bytes += separator + part.len();
            cursor_after_offset = if end < text.len() { end } else { 0 };
            let full_text = (cursor_after_offset > 0).then_some(text);
            frozen.push((id, revision, part, full_text));
            if cursor_after_offset > 0 {
                break;
            }
        }
        drop(statement);
        if frozen.is_empty() {
            tx.commit().context("commit empty transcript request")?;
            return Ok(None);
        }
        let user_text = frozen
            .iter()
            .map(|(_, _, text, _)| text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        tx.execute(
            "INSERT INTO message_requests
                (meeting_id, kind, status, user_text, cursor_before, cursor_after, cursor_after_offset)
             VALUES (?1, 'transcript', 'pending', ?2, ?3, ?4, ?5)",
            params![meeting_id, user_text, cursor_before, frozen.last().map(|row| row.0), cursor_after_offset as i64],
        )?;
        let request_id = tx.last_insert_rowid();
        for (segment_id, revision, text, full_text) in frozen {
            tx.execute(
                "INSERT INTO message_request_segments (request_id, segment_id, revision, text, full_text)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![request_id, segment_id, revision, text, full_text],
            )?;
        }
        let result = read_pending_request(&tx, request_id)?;
        tx.commit().context("commit transcript request")?;
        Ok(Some(result))
    }

    pub fn prepare_question_request(
        &mut self,
        meeting_id: i64,
        text: &str,
    ) -> Result<PendingRequest> {
        validate_id(meeting_id, "meeting")?;
        let text = validate_request_text(text, "question")?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("start question request")?;
        read_meeting(&tx, meeting_id)?;
        if active_request(&tx, meeting_id)?.is_some() {
            bail!("meeting already has a pending request");
        }
        tx.execute(
            "INSERT INTO message_requests (meeting_id, kind, status, user_text)
             VALUES (?1, 'question', 'pending', ?2)",
            params![meeting_id, text],
        )?;
        let result = read_pending_request(&tx, tx.last_insert_rowid())?;
        tx.commit().context("commit question request")?;
        Ok(result)
    }

    pub fn pending_request(&self, request_id: i64) -> Result<Option<PendingRequest>> {
        validate_id(request_id, "request")?;
        use rusqlite::OptionalExtension;
        let exists = self.conn.query_row(
            "SELECT id FROM message_requests WHERE id = ?1 AND status IN ('pending', 'inflight')",
            [request_id],
            |row| row.get(0),
        ).optional()?;
        exists
            .map(|id| read_pending_request(&self.conn, id))
            .transpose()
    }

    pub fn retry_request(&mut self, request_id: i64) -> Result<PendingRequest> {
        let request = self
            .pending_request(request_id)?
            .ok_or_else(|| anyhow::anyhow!("request is not pending"))?;
        if request.status != RequestStatus::Pending {
            bail!("request is already in flight");
        }
        self.ensure_pending_has_no_output(request_id)?;
        Ok(request)
    }

    pub fn begin_request(&mut self, request_id: i64) -> Result<PendingRequest> {
        validate_id(request_id, "request")?;
        let tx = self.conn.transaction().context("begin request")?;
        let changed = tx.execute(
            "UPDATE message_requests SET status = 'inflight'
             WHERE id = ?1 AND status = 'pending'
               AND (assistant_text IS NULL OR assistant_text = '')",
            [request_id],
        )?;
        if changed != 1 {
            bail!("request is not pending");
        }
        let result = read_pending_request(&tx, request_id)?;
        tx.commit().context("commit request begin")?;
        Ok(result)
    }

    /// Append the semantic inputs prepared for one provider attempt, before networking.
    #[allow(clippy::too_many_arguments)]
    pub fn save_request_context(
        &self,
        request_id: i64,
        provider: &str,
        account_id: Option<&str>,
        model: &str,
        request_json: &str,
        metadata_json: &str,
        upstream_omitted: bool,
    ) -> Result<ContextSnapshot> {
        self.save_attributed_request_context(
            request_id,
            provider,
            account_id,
            None,
            model,
            request_json,
            metadata_json,
            upstream_omitted,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_attributed_request_context(
        &self,
        request_id: i64,
        provider: &str,
        account_id: Option<&str>,
        credential_id: Option<&str>,
        model: &str,
        request_json: &str,
        metadata_json: &str,
        upstream_omitted: bool,
    ) -> Result<ContextSnapshot> {
        if let Some(id) = credential_id {
            validate_account_id(id)?;
            if provider == "chatgpt" {
                bail!("API key attribution is only supported for API providers");
            }
        }
        validate_id(request_id, "request")?;
        if !matches!(provider, "chatgpt" | "gemini" | "deepseek") {
            bail!("unsupported context provider");
        }
        validate_request_id_text(model, "model")?;
        if let Some(id) = account_id {
            validate_account_id(id)?;
            if provider != "chatgpt" {
                bail!("account attribution is only supported for ChatGPT");
            }
        }
        let request = bounded_context_json(request_json, MAX_CONTEXT_REQUEST)?;
        let metadata = bounded_context_json(metadata_json, MAX_CONTEXT_METADATA)?;
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let inserted = tx.execute(
            "INSERT INTO request_context (request_id, provider, model, request_json, metadata_json, upstream_omitted, account_id, credential_id)
             SELECT id, ?2, ?3, ?4, ?5, ?6, ?7, ?8 FROM message_requests WHERE id = ?1 AND status = 'inflight'",
            params![request_id, provider, model, request_json, metadata_json, upstream_omitted, account_id, credential_id],
        )?;
        if inserted != 1 {
            bail!("request is not in flight");
        }
        let id = tx.last_insert_rowid();
        let created_at = tx.query_row(
            "SELECT created_at FROM request_context WHERE id=?1",
            [id],
            |row| row.get(0),
        )?;
        tx.commit()?;
        Ok(ContextSnapshot {
            credential_id: credential_id.map(str::to_owned),
            account_id: account_id.map(str::to_owned),
            id,
            request_id,
            provider: provider.into(),
            model: model.into(),
            created_at,
            request,
            metadata,
            upstream_omitted,
        })
    }

    /// Read one attempt at a time; retries and fallback retain earlier prepared contexts.
    pub fn request_context(
        &self,
        request_id: i64,
        before_id: Option<i64>,
    ) -> Result<Option<ContextSnapshot>> {
        validate_id(request_id, "request")?;
        if let Some(id) = before_id {
            validate_id(id, "context cursor")?;
        }
        let mut statement = self.conn.prepare(
            "SELECT id, request_id, provider, model, created_at, request_json, metadata_json, upstream_omitted, account_id, credential_id
             FROM request_context WHERE request_id = ?1 AND (?2 IS NULL OR id < ?2)
             ORDER BY id DESC LIMIT 1",
        )?;
        let mut rows = statement.query(params![request_id, before_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        bounded_row_fields(
            row,
            &[
                (2, 8),
                (3, MAX_REQUEST_ID_TEXT),
                (5, MAX_CONTEXT_REQUEST),
                (6, MAX_CONTEXT_METADATA),
                (8, 43),
                (9, 43),
            ],
        )?;
        let provider: String = row.get(2)?;
        if !matches!(provider.as_str(), "chatgpt" | "gemini" | "deepseek") {
            bail!("invalid stored context provider");
        }
        let model: String = row.get(3)?;
        validate_request_id_text(&model, "model")?;
        let account_id: Option<String> = row.get(8)?;
        if let Some(id) = &account_id {
            validate_account_id(id)?;
        }
        let credential_id: Option<String> = row.get(9)?;
        if let Some(id) = &credential_id {
            validate_account_id(id)?;
            if provider == "chatgpt" {
                bail!("invalid API key attribution");
            }
        }
        Ok(Some(ContextSnapshot {
            credential_id,
            account_id,
            id: row.get(0)?,
            request_id: row.get(1)?,
            provider,
            model,
            created_at: row.get(4)?,
            request: bounded_context_json(row.get_ref(5)?.as_str()?, MAX_CONTEXT_REQUEST)?,
            metadata: bounded_context_json(row.get_ref(6)?.as_str()?, MAX_CONTEXT_METADATA)?,
            upstream_omitted: row.get(7)?,
        }))
    }

    pub fn complete_request(
        &mut self,
        request_id: i64,
        assistant_text: &str,
        provider: &str,
        model: &str,
        usage: Option<RequestUsage>,
    ) -> Result<()> {
        validate_id(request_id, "request")?;
        validate_answer_text(assistant_text, "assistant response")?;
        validate_request_id_text(provider, "provider")?;
        validate_request_id_text(model, "model")?;
        if let Some(usage) = &usage {
            for value in [usage.input_tokens, usage.output_tokens, usage.total_tokens]
                .into_iter()
                .flatten()
            {
                if value < 0 {
                    bail!("usage values must be non-negative");
                }
            }
        }
        let tx = self.conn.transaction().context("complete request")?;
        let (status, persisted, persisted_provider, persisted_model): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = tx.query_row(
            "SELECT status, assistant_text, provider, model FROM message_requests WHERE id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        if status != "inflight" {
            bail!("request is not in flight");
        }
        if !assistant_text.starts_with(persisted.as_deref().unwrap_or("")) {
            bail!("assistant response must extend the persisted checkpoint");
        }
        if persisted_provider
            .as_deref()
            .is_some_and(|value| value != provider)
            || persisted_model
                .as_deref()
                .is_some_and(|value| value != model)
        {
            bail!("provider and model must match the persisted checkpoint");
        }
        let changed = tx.execute(
            "UPDATE message_requests SET status = 'completed', assistant_text = ?1,
                provider = ?2, model = ?3, error_summary = NULL,
                input_tokens = ?4, output_tokens = ?5, total_tokens = ?6,
                completed_at = unixepoch()
             WHERE id = ?7 AND status = 'inflight'",
            params![
                assistant_text,
                provider,
                model,
                usage.as_ref().and_then(|u| u.input_tokens),
                usage.as_ref().and_then(|u| u.output_tokens),
                usage.as_ref().and_then(|u| u.total_tokens),
                request_id
            ],
        )?;
        if changed != 1 {
            bail!("request is not pending");
        }
        tx.commit().context("commit request completion")?;
        Ok(())
    }

    pub fn checkpoint_request_output(
        &mut self,
        request_id: i64,
        assistant_text: &str,
        provider: &str,
        model: &str,
    ) -> Result<()> {
        validate_id(request_id, "request")?;
        validate_answer_text(assistant_text, "assistant response")?;
        validate_request_id_text(provider, "provider")?;
        validate_request_id_text(model, "model")?;
        let tx = self.conn.transaction().context("checkpoint request")?;
        let (status, existing, existing_provider, existing_model): (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = tx.query_row(
            "SELECT status, assistant_text, provider, model FROM message_requests WHERE id = ?1",
            [request_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        if status != "inflight" {
            bail!("request is not in flight");
        }
        let existing = existing.as_deref().unwrap_or("");
        if !assistant_text.starts_with(existing) {
            bail!("checkpoint must extend the existing output");
        }
        if existing_provider
            .as_deref()
            .is_some_and(|value| value != provider)
            || existing_model
                .as_deref()
                .is_some_and(|value| value != model)
        {
            bail!("provider and model must match the existing checkpoint");
        }
        tx.execute(
            "UPDATE message_requests SET assistant_text = ?1, provider = ?2, model = ?3
             WHERE id = ?4 AND status = 'inflight'",
            params![assistant_text, provider, model, request_id],
        )?;
        tx.commit().context("commit request checkpoint")?;
        Ok(())
    }

    pub fn fail_request(
        &mut self,
        request_id: i64,
        error_summary: &str,
        partial_text: Option<&str>,
    ) -> Result<()> {
        validate_id(request_id, "request")?;
        if error_summary.len() > MAX_REQUEST_ERROR || error_summary.chars().any(char::is_control) {
            bail!(
                "error summary must be at most {MAX_REQUEST_ERROR} bytes and contain no controls"
            );
        }
        if let Some(partial) = partial_text {
            validate_answer_text(partial, "partial response")?;
        }
        let tx = self.conn.transaction().context("fail request")?;
        let row = tx.query_row(
            "SELECT status, assistant_text FROM message_requests WHERE id = ?1",
            [request_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
        )?;
        if !matches!(row.0.as_str(), "pending" | "inflight") {
            bail!("request is not pending");
        }
        let existing = row.1.as_deref().unwrap_or("");
        let partial = partial_text.or(row.1.as_deref());
        if let Some(partial) = partial {
            if !partial.starts_with(existing) {
                bail!("partial response must extend the persisted checkpoint");
            }
        }
        let status = if partial.is_some_and(|text| !text.is_empty()) {
            "partial"
        } else {
            "pending"
        };
        tx.execute(
            "UPDATE message_requests SET status = ?1, assistant_text = ?2,
                error_summary = ?3,
                completed_at = CASE WHEN ?1 = 'partial' THEN unixepoch() ELSE NULL END
             WHERE id = ?4",
            params![status, partial, error_summary, request_id],
        )?;
        tx.commit().context("commit request failure")?;
        Ok(())
    }

    pub fn cancel_request(&mut self, request_id: i64) -> Result<()> {
        validate_id(request_id, "request")?;
        let tx = self.conn.transaction().context("cancel request")?;
        let output: Option<String> = tx.query_row(
            "SELECT assistant_text FROM message_requests
             WHERE id = ?1 AND status IN ('pending', 'inflight')",
            [request_id],
            |row| row.get(0),
        )?;
        let status = output
            .as_deref()
            .filter(|text| !text.is_empty())
            .map(|_| "partial")
            .unwrap_or("pending");
        let changed = tx.execute(
            "UPDATE message_requests SET status = ?1, error_summary = 'cancelled',
                completed_at = CASE WHEN ?1 = 'partial' THEN unixepoch() ELSE NULL END
             WHERE id = ?2 AND status IN ('pending', 'inflight')",
            params![status, request_id],
        )?;
        if changed != 1 {
            bail!("request is not pending");
        }
        tx.commit().context("commit request cancellation")?;
        Ok(())
    }

    pub fn meeting_pending_request(&self, meeting_id: i64) -> Result<Option<PendingRequest>> {
        validate_id(meeting_id, "meeting")?;
        read_meeting(&self.conn, meeting_id)?;
        active_request(&self.conn, meeting_id)?
            .map(|request| read_pending_request(&self.conn, request.id))
            .transpose()
    }

    fn ensure_pending_has_no_output(&self, request_id: i64) -> Result<()> {
        let output: Option<String> = self.conn.query_row(
            "SELECT assistant_text FROM message_requests WHERE id = ?1 AND status = 'pending'",
            [request_id],
            |row| row.get(0),
        )?;
        if output.is_some_and(|text| !text.is_empty()) {
            bail!("request has persisted output");
        }
        Ok(())
    }

    pub fn completed_history(&self, meeting_id: i64) -> Result<MessageHistory> {
        validate_id(meeting_id, "meeting")?;
        read_meeting(&self.conn, meeting_id)?;
        let mut statement = self.conn.prepare(
            "SELECT id, user_text, assistant_text, provider, model,
                    input_tokens, output_tokens, total_tokens,
                    length(CAST(user_text AS BLOB)) + length(CAST(assistant_text AS BLOB))
             FROM message_requests
             WHERE meeting_id = ?1 AND status IN ('completed', 'partial')
             ORDER BY id DESC LIMIT ?2",
        )?;
        let history_limit = bounded_i64(MAX_MESSAGE_HISTORY / 2, "history limit")?;
        let mut rows = statement.query(params![meeting_id, history_limit])?;
        let mut requests = Vec::new();
        let mut selected_bytes = 0usize;
        while let Some(row) = rows.next()? {
            let size = row
                .get::<_, Option<i64>>(8)?
                .ok_or_else(|| anyhow::anyhow!("delivered request has no assistant text"))?;
            let size = usize::try_from(size)?;
            if size > MAX_HISTORY_BYTES.saturating_sub(selected_bytes) {
                continue;
            }
            selected_bytes += size;
            requests.push((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                RequestUsage {
                    input_tokens: row.get(5)?,
                    output_tokens: row.get(6)?,
                    total_tokens: row.get(7)?,
                },
            ));
        }
        let mut result = Vec::new();
        let mut bytes: usize = 0;
        let total: i64 = self.conn.query_row("SELECT COUNT(*) FROM message_requests WHERE meeting_id = ?1 AND status IN ('completed', 'partial')", [meeting_id], |row| row.get(0))?;
        for (id, user, assistant, provider, model, usage) in requests {
            let assistant = assistant
                .ok_or_else(|| anyhow::anyhow!("delivered request has no assistant text"))?;
            let added = user.len() + assistant.len();
            if bytes.saturating_add(added) > MAX_HISTORY_BYTES {
                continue;
            }
            bytes += added;
            result.push(MessageHistoryEntry {
                request_id: id,
                meeting_id,
                role: "user".into(),
                text: user,
                provider: None,
                model: None,
                usage: usage.clone(),
            });
            result.push(MessageHistoryEntry {
                request_id: id,
                meeting_id,
                role: "assistant".into(),
                text: assistant,
                provider,
                model,
                usage,
            });
        }
        // Sort whole pairs, retaining user before assistant within each request.
        result.sort_by_key(|entry| entry.request_id);
        let included = result.len() / 2;
        Ok(MessageHistory {
            entries: result,
            omitted_requests: usize::try_from(total)?.saturating_sub(included),
        })
    }

    pub fn upsert_document(&self, input: &DocumentInput, now: i64) -> Result<Document> {
        validate_document_input(input)?;
        validate_time(now)?;
        let title = input.title.trim();
        let project = input
            .project
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty());
        // ponytail: fixed UTF-8 slices; paragraph-aware splitting only if retrieval quality needs it.
        let mut chunks = Vec::new();
        let mut remaining = input.text.as_str();
        while !remaining.is_empty() {
            let mut end = remaining.len().min(MAX_DOCUMENT_CHUNK);
            while !remaining.is_char_boundary(end) {
                end -= 1;
            }
            chunks.push(&remaining[..end]);
            remaining = &remaining[end..];
        }
        if chunks.len() > 128 {
            bail!("document has too many chunks");
        }
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        {
            let mut statement =
                tx.prepare("SELECT stored_path FROM documents WHERE source_path=?1")?;
            let mut rows = statement.query([&input.source_path])?;
            if let Some(row) = rows.next()? {
                bounded_row_fields(row, &[(0, 8192)])?;
                let old_path: Option<String> = row.get(0)?;
                if old_path != input.stored_path {
                    if let Some(path) = old_path {
                        tx.execute(
                            "INSERT OR IGNORE INTO document_copy_cleanup(path) VALUES(?1)",
                            [path],
                        )?;
                    }
                }
            }
        }
        tx.execute("INSERT INTO documents(title,source_path,stored_path,content_hash,modified_at,indexed_at,indexing_version,project,enabled,chunk_count,text_bytes) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(source_path) DO UPDATE SET title=excluded.title,stored_path=excluded.stored_path,content_hash=excluded.content_hash,modified_at=excluded.modified_at,indexed_at=excluded.indexed_at,indexing_version=excluded.indexing_version,project=excluded.project,chunk_count=excluded.chunk_count,text_bytes=excluded.text_bytes",
            params![title,input.source_path,input.stored_path,input.content_hash,input.modified_at,now,input.indexing_version,project,input.enabled,chunks.len() as i64,input.text.len() as i64])?;
        let id: i64 = tx.query_row(
            "SELECT id FROM documents WHERE source_path=?1",
            [&input.source_path],
            |r| r.get(0),
        )?;
        tx.execute("DELETE FROM document_chunks WHERE document_id=?1", [id])?;
        {
            let mut insert = tx.prepare("INSERT INTO document_chunks(document_id,chunk_index,title,text,project) VALUES(?1,?2,?3,?4,?5)")?;
            for (index, text) in chunks.iter().enumerate() {
                insert.execute(params![id, index as i64, title, text, project])?;
            }
        }
        if let Some(path) = &input.stored_path {
            tx.execute("DELETE FROM document_copy_cleanup WHERE path=?1", [path])?;
        }
        tx.commit()?;
        self.document(id)
    }

    pub fn document(&self, id: i64) -> Result<Document> {
        validate_id(id, "document")?;
        let sql = format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE id=?1");
        let mut statement = self.conn.prepare(&sql)?;
        let mut rows = statement.query([id])?;
        document_from_row(rows.next()?.context("document not found")?)
    }

    pub fn list_documents(&self, before_id: Option<i64>, limit: usize) -> Result<DocumentPage> {
        validate_memory_limit(limit)?;
        if let Some(id) = before_id {
            validate_id(id, "document cursor")?;
        }
        let sql = format!("SELECT {DOCUMENT_COLUMNS} FROM documents WHERE (?1 IS NULL OR id<?1) ORDER BY id DESC LIMIT ?2");
        let mut statement = self.conn.prepare(&sql)?;
        let mut rows = statement.query(params![before_id, (limit + 1) as i64])?;
        let mut documents = Vec::new();
        while let Some(row) = rows.next()? {
            documents.push(document_from_row(row)?);
        }
        let has_more = documents.len() > limit;
        documents.truncate(limit);
        let next = if has_more {
            documents.last().map(|d| d.id)
        } else {
            None
        };
        Ok(DocumentPage {
            documents,
            has_more,
            next,
        })
    }

    pub fn document_source(&self, id: i64) -> Result<DocumentSource> {
        validate_id(id, "document")?;
        let mut statement = self
            .conn
            .prepare("SELECT source_path,stored_path,project,enabled FROM documents WHERE id=?1")?;
        let mut rows = statement.query([id])?;
        let row = rows.next()?.context("document not found")?;
        bounded_row_fields(row, &[(0, 8192), (1, 8192), (2, 400)])?;
        Ok(DocumentSource {
            source_path: row.get(0)?,
            stored_path: row.get(1)?,
            project: row.get(2)?,
            enabled: row.get(3)?,
        })
    }

    pub fn set_document_enabled(&self, id: i64, enabled: bool) -> Result<Document> {
        validate_id(id, "document")?;
        if self.conn.execute(
            "UPDATE documents SET enabled=?1 WHERE id=?2",
            params![enabled, id],
        )? != 1
        {
            bail!("document not found");
        }
        self.document(id)
    }

    pub fn delete_document(&self, id: i64) -> Result<Option<String>> {
        let tx = rusqlite::Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let source = self.document_source(id)?;
        if let Some(path) = &source.stored_path {
            tx.execute(
                "INSERT OR IGNORE INTO document_copy_cleanup(path) VALUES(?1)",
                [path],
            )?;
        }
        if tx.execute("DELETE FROM documents WHERE id=?1", [id])? != 1 {
            bail!("document not found");
        }
        tx.commit()?;
        Ok(source.stored_path)
    }

    pub fn pending_document_copies(&self, limit: usize) -> Result<Vec<String>> {
        if !(1..=100).contains(&limit) {
            bail!("document cleanup limit must be between 1 and 100");
        }
        let mut statement = self
            .conn
            .prepare("SELECT path FROM document_copy_cleanup ORDER BY path LIMIT ?1")?;
        let mut rows = statement.query([limit as i64])?;
        let mut paths = Vec::new();
        while let Some(row) = rows.next()? {
            bounded_row_fields(row, &[(0, 8192)])?;
            paths.push(row.get(0)?);
        }
        Ok(paths)
    }

    pub fn complete_document_copy_cleanup(&self, path: &str) -> Result<()> {
        if path.is_empty() || path.len() > 8192 || path.contains('\0') {
            bail!("invalid document cleanup path");
        }
        self.conn
            .execute("DELETE FROM document_copy_cleanup WHERE path=?1", [path])?;
        Ok(())
    }

    pub fn search_documents(&self, query: &str, limit: usize) -> Result<Vec<DocumentChunk>> {
        if !(1..=50).contains(&limit) {
            bail!("document search limit must be between 1 and 50");
        }
        self.search_document_rows(query, limit, false)
    }

    pub fn queue_document_copy_cleanup(&self, path: &str) -> Result<()> {
        if path.is_empty() || path.len() > 8192 || path.contains('\0') {
            bail!("invalid copied-source path");
        }
        self.conn.execute(
            "INSERT OR IGNORE INTO document_copy_cleanup(path) VALUES (?1)",
            [path],
        )?;
        Ok(())
    }

    pub fn retrieve_documents(&self, query: &str, limit: usize) -> Result<Vec<DocumentChunk>> {
        if !(1..=8).contains(&limit) {
            bail!("document retrieval limit must be between 1 and 8");
        }
        self.search_document_rows(query, limit, true)
    }

    fn search_document_rows(
        &self,
        query: &str,
        limit: usize,
        enabled_only: bool,
    ) -> Result<Vec<DocumentChunk>> {
        let query = literal_search_query_with(query, enabled_only)?;
        let mut statement = self.conn.prepare("SELECT c.id,c.document_id,d.title,d.source_path,d.content_hash,d.indexing_version,c.chunk_index,c.text FROM document_chunks AS c JOIN documents AS d ON d.id=c.document_id JOIN document_chunks_fts ON document_chunks_fts.rowid=c.id WHERE document_chunks_fts MATCH ?1 AND (?3=0 OR d.enabled=1) ORDER BY bm25(document_chunks_fts,3.0,1.0,2.0),c.id DESC LIMIT ?2")?;
        let mut rows = statement.query(params![query, limit as i64, enabled_only])?;
        let mut result = Vec::new();
        let mut bytes = 0;
        while let Some(row) = rows.next()? {
            bounded_row_fields(
                row,
                &[(2, 800), (3, 8192), (4, 64), (7, MAX_DOCUMENT_CHUNK)],
            )?;
            let size = row.get_ref(7)?.as_str()?.len();
            if bytes + size > 16 * 1024 {
                break;
            }
            result.push(DocumentChunk {
                id: row.get(0)?,
                document_id: row.get(1)?,
                title: row.get(2)?,
                source_path: row.get(3)?,
                content_hash: row.get(4)?,
                indexing_version: row.get(5)?,
                chunk_index: usize::try_from(row.get::<_, i64>(6)?)?,
                text: row.get(7)?,
            });
            bytes += size;
        }
        Ok(result)
    }

    pub fn save_memory(
        &self,
        id: Option<i64>,
        input: &MemoryInput,
        now: i64,
    ) -> Result<MemoryEntry> {
        validate_time(now)?;
        let title = input.title.trim();
        let body = input.body.trim();
        let project = input
            .project
            .as_deref()
            .map(str::trim)
            .filter(|p| !p.is_empty());
        if title.is_empty()
            || title.len() > 800
            || title.chars().count() > 200
            || title.chars().any(char::is_control)
        {
            bail!("memory title must be 1 to 200 characters / 800 bytes without controls");
        }
        if body.is_empty() || body.len() > MAX_MEMORY_BODY {
            bail!("memory body must be 1 to {MAX_MEMORY_BODY} bytes");
        }
        if project.is_some_and(|p| {
            p.len() > 400 || p.chars().count() > 100 || p.chars().any(char::is_control)
        }) {
            bail!("memory project must be at most 100 characters / 400 bytes without controls");
        }
        if !matches!(
            input.category.as_str(),
            "project"
                | "person"
                | "organization"
                | "preference"
                | "note"
                | "decision"
                | "experience"
                | "education"
                | "term"
        ) {
            bail!("invalid memory category");
        }
        let id = match id {
            Some(id) => {
                validate_id(id, "memory")?;
                if self.conn.execute(
                    "UPDATE memories SET title=?1, body=?2, category=?3, project=?4, enabled=?5, updated_at=?6 WHERE id=?7 AND source='manual' AND created_at<=?6 AND updated_at<=?6",
                    params![title, body, input.category, project, input.enabled, now, id],
                )? != 1 {
                    bail!("manual memory missing or update time predates the saved memory");
                }
                id
            }
            None => {
                self.conn.execute(
                    "INSERT INTO memories(title,body,category,project,enabled,created_at,updated_at,source) VALUES (?1,?2,?3,?4,?5,?6,?6,'manual')",
                    params![title, body, input.category, project, input.enabled, now],
                )?;
                self.conn.last_insert_rowid()
            }
        };
        self.get_memory(id)
    }

    pub fn get_memory(&self, id: i64) -> Result<MemoryEntry> {
        validate_id(id, "memory")?;
        let mut statement = self.conn.prepare("SELECT id,title,body,category,project,enabled,created_at,updated_at,source FROM memories WHERE id=?1")?;
        let mut rows = statement.query([id])?;
        memory_from_row(rows.next()?.context("memory not found")?)
    }

    pub fn delete_memory(&self, id: i64) -> Result<()> {
        validate_id(id, "memory")?;
        if self
            .conn
            .execute("DELETE FROM memories WHERE id=?1", [id])?
            != 1
        {
            bail!("memory not found");
        }
        Ok(())
    }

    pub fn list_memories(&self, before_id: Option<i64>, limit: usize) -> Result<Vec<MemoryEntry>> {
        Ok(self.list_memory_page(before_id, limit)?.entries)
    }

    pub fn list_memory_page(&self, before_id: Option<i64>, limit: usize) -> Result<MemoryPage> {
        validate_memory_limit(limit)?;
        if let Some(id) = before_id {
            validate_id(id, "memory cursor")?;
        }
        let mut statement = self.conn.prepare("SELECT id,title,body,category,project,enabled,created_at,updated_at,source FROM memories WHERE (?1 IS NULL OR id<?1) ORDER BY id DESC LIMIT ?2")?;
        let mut rows = statement.query(params![before_id, (limit + 1) as i64])?;
        let mut entries = Vec::new();
        let mut bytes = 0;
        let mut has_more = false;
        while let Some(row) = rows.next()? {
            let body_bytes = row.get_ref(2)?.as_str()?.len();
            if body_bytes > MAX_MEMORY_BODY {
                bail!("stored memory body exceeds its bound");
            }
            if entries.len() == limit || bytes + body_bytes > MAX_MEMORY_PAGE_BYTES {
                has_more = true;
                break;
            }
            entries.push(memory_from_row(row)?);
            bytes += body_bytes;
        }
        let next = if has_more {
            entries.last().map(|entry| entry.id)
        } else {
            None
        };
        Ok(MemoryPage {
            entries,
            has_more,
            next,
        })
    }

    pub fn search_memories(
        &self,
        query: &str,
        limit: usize,
        enabled_only: bool,
    ) -> Result<Vec<MemoryEntry>> {
        self.search_memory_rows(query, limit, enabled_only, false, MAX_MEMORY_PAGE_BYTES)
    }

    pub fn retrieve_memories(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>> {
        if !(1..=8).contains(&limit) {
            bail!("memory retrieval limit must be between 1 and 8");
        }
        self.search_memory_rows(query, limit, true, true, MAX_MEMORY_BODY)
    }

    fn search_memory_rows(
        &self,
        query: &str,
        limit: usize,
        enabled_only: bool,
        any_term: bool,
        max_bytes: usize,
    ) -> Result<Vec<MemoryEntry>> {
        validate_memory_limit(limit)?;
        let query = literal_search_query_with(query, any_term)?;
        let mut statement = self.conn.prepare("SELECT m.id,m.title,m.body,m.category,m.project,m.enabled,m.created_at,m.updated_at,m.source FROM memories AS m JOIN memories_fts ON memories_fts.rowid=m.id WHERE memories_fts MATCH ?1 AND (?2=0 OR m.enabled=1) ORDER BY bm25(memories_fts, 3.0, 1.0, 2.0), m.id DESC LIMIT ?3")?;
        let rows = statement.query(params![query, enabled_only, limit as i64])?;
        collect_memories(rows, max_bytes)
    }

    pub fn search_transcripts(
        &self,
        meeting_id: i64,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TranscriptSegment>> {
        validate_id(meeting_id, "meeting")?;
        read_meeting(&self.conn, meeting_id)?;
        if !(1..=MAX_SAVED_MEETINGS).contains(&limit) {
            bail!("transcript search limit must be between 1 and {MAX_SAVED_MEETINGS}");
        }
        let fts_query = literal_search_query(query)?;
        let mut statement = self.conn.prepare(
            "SELECT t.id, t.meeting_id, t.event_id, t.source, t.start_ms,
                    t.end_ms, t.revision, t.text
             FROM transcript_segments AS t
             JOIN transcript_segments_fts AS f
               ON f.rowid = t.id
             WHERE t.meeting_id = ?1 AND t.is_final = 1
               AND f.transcript_segments_fts MATCH ?2
             ORDER BY t.start_ms, t.id LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![meeting_id, fts_query, limit as i64],
            transcript_from_row,
        )?;
        let mut total_text = 0;
        let mut results = Vec::new();
        for row in rows {
            let segment = row?;
            if total_text + segment.text.len() > MAX_TRANSCRIPT_TEXT {
                break;
            }
            total_text += segment.text.len();
            results.push(segment);
        }
        Ok(results)
    }
}

fn literal_search_query(query: &str) -> Result<String> {
    literal_search_query_with(query, false)
}

fn literal_search_query_with(query: &str, any_term: bool) -> Result<String> {
    if query.trim().is_empty() || query.len() > MAX_TRANSCRIPT_SEARCH_QUERY {
        bail!("search query must be 1 to {MAX_TRANSCRIPT_SEARCH_QUERY} bytes");
    }
    let terms: Vec<_> = query
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect();
    if terms.is_empty() || terms.len() > MAX_TRANSCRIPT_SEARCH_TERMS {
        bail!("search query must contain 1 to {MAX_TRANSCRIPT_SEARCH_TERMS} terms");
    }
    Ok(terms.join(if any_term { " OR " } else { " AND " }))
}

const DOCUMENT_COLUMNS: &str = "id,title,source_path,stored_path,content_hash,modified_at,indexed_at,indexing_version,project,enabled,chunk_count,text_bytes";

fn bounded_row_fields(row: &rusqlite::Row<'_>, fields: &[(usize, usize)]) -> Result<()> {
    for &(index, maximum) in fields {
        let value = row.get_ref(index)?;
        if !matches!(value, rusqlite::types::ValueRef::Null) && value.as_str()?.len() > maximum {
            bail!("stored field exceeds its bound");
        }
    }
    Ok(())
}

fn document_from_row(row: &rusqlite::Row<'_>) -> Result<Document> {
    bounded_row_fields(row, &[(1, 800), (2, 8192), (3, 8192), (4, 64), (8, 400)])?;
    let source_policy = if matches!(row.get_ref(3)?, rusqlite::types::ValueRef::Null) {
        "reference"
    } else {
        "copy"
    };
    Ok(Document {
        id: row.get(0)?,
        title: row.get(1)?,
        source_path: row.get(2)?,
        source_policy: source_policy.into(),
        content_hash: row.get(4)?,
        modified_at: row.get(5)?,
        indexed_at: row.get(6)?,
        indexing_version: row.get(7)?,
        project: row.get(8)?,
        enabled: row.get(9)?,
        chunk_count: usize::try_from(row.get::<_, i64>(10)?)?,
        text_bytes: usize::try_from(row.get::<_, i64>(11)?)?,
    })
}

fn validate_document_input(input: &DocumentInput) -> Result<()> {
    validate_time(input.modified_at)?;
    let title = input.title.trim();
    if title.is_empty()
        || title.len() > 800
        || title.chars().count() > 200
        || title.chars().any(char::is_control)
    {
        bail!("document title must be 1 to 200 characters / 800 bytes without controls");
    }
    for path in [
        Some(input.source_path.as_str()),
        input.stored_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if path.is_empty() || path.len() > 8192 || path.contains('\0') {
            bail!("document path must be 1 to 8192 bytes without NUL");
        }
    }
    if input.content_hash.len() != 64 || !input.content_hash.bytes().all(|b| b.is_ascii_hexdigit())
    {
        bail!("document hash must be 64 hexadecimal characters");
    }
    if input.indexing_version == 0 {
        bail!("document indexing version must be positive");
    }
    if input.text.trim().is_empty() || input.text.len() > MAX_DOCUMENT_TEXT {
        bail!("document text must be 1 to {MAX_DOCUMENT_TEXT} bytes");
    }
    if input
        .project
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .is_some_and(|p| {
            p.len() > 400 || p.chars().count() > 100 || p.chars().any(char::is_control)
        })
    {
        bail!("document project must be at most 100 characters / 400 bytes without controls");
    }
    Ok(())
}

fn validate_memory_limit(limit: usize) -> Result<()> {
    if !(1..=MAX_MEMORY_PAGE).contains(&limit) {
        bail!("memory limit must be between 1 and {MAX_MEMORY_PAGE}");
    }
    Ok(())
}

fn memory_from_row(row: &rusqlite::Row<'_>) -> Result<MemoryEntry> {
    // Inspect SQLite's borrowed values before allocating untrusted persisted text.
    for (index, maximum) in [(1, 800), (2, MAX_MEMORY_BODY), (3, 20), (4, 400), (8, 20)] {
        let value = row.get_ref(index)?;
        if !matches!(value, rusqlite::types::ValueRef::Null) && value.as_str()?.len() > maximum {
            bail!("stored memory field exceeds its bound");
        }
    }
    Ok(MemoryEntry {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        category: row.get(3)?,
        project: row.get(4)?,
        enabled: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        source: row.get(8)?,
    })
}

fn collect_memories(mut rows: rusqlite::Rows<'_>, max_bytes: usize) -> Result<Vec<MemoryEntry>> {
    let mut results = Vec::new();
    let mut bytes = 0;
    while let Some(row) = rows.next()? {
        let body_bytes = row.get_ref(2)?.as_str()?.len();
        if body_bytes > MAX_MEMORY_BODY {
            bail!("stored memory body exceeds its bound");
        }
        if bytes + body_bytes > max_bytes {
            break;
        }
        results.push(memory_from_row(row)?);
        bytes += body_bytes;
    }
    Ok(results)
}

fn validate_id(id: i64, kind: &str) -> Result<()> {
    if id <= 0 {
        bail!("{kind} id must be positive");
    }
    Ok(())
}

fn bounded_i64(value: usize, kind: &str) -> Result<i64> {
    i64::try_from(value).with_context(|| format!("{kind} does not fit SQLite integer"))
}

fn validate_time(time: i64) -> Result<()> {
    if !(0..=MAX_TIME_MS).contains(&time) {
        bail!("time must be between 0 and {MAX_TIME_MS} milliseconds");
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<String> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > MAX_MEETING_TITLE {
        bail!("meeting title must be between 1 and {MAX_MEETING_TITLE} characters");
    }
    Ok(title.to_owned())
}

fn validate_text(text: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() || text.len() > MAX_TRANSCRIPT_TEXT {
        bail!("transcript text must be between 1 and {MAX_TRANSCRIPT_TEXT} bytes");
    }
    Ok(text.to_owned())
}

fn validate_request_text(text: &str, kind: &str) -> Result<String> {
    let text = text.trim();
    if text.is_empty() || text.len() > MAX_REQUEST_TEXT {
        bail!("{kind} must be between 1 and {MAX_REQUEST_TEXT} bytes");
    }
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
    {
        bail!("{kind} contains control characters");
    }
    Ok(text.to_owned())
}

fn validate_answer_text(text: &str, kind: &str) -> Result<()> {
    if text.len() > MAX_ANSWER {
        bail!("{kind} must be at most {MAX_ANSWER} bytes");
    }
    Ok(())
}

fn bounded_context_json(text: &str, maximum: usize) -> Result<serde_json::Value> {
    if text.len() > maximum {
        bail!("prepared context JSON exceeds its byte limit");
    }
    serde_json::from_str(text).context("invalid prepared context JSON")
}

fn validate_request_id_text(text: &str, kind: &str) -> Result<()> {
    if text.is_empty() || text.len() > MAX_REQUEST_ID_TEXT || text.chars().any(char::is_control) {
        bail!("{kind} must be between 1 and {MAX_REQUEST_ID_TEXT} bytes and contain no controls");
    }
    Ok(())
}

struct ActiveRequest {
    id: i64,
    kind: RequestKind,
}

fn request_kind(value: &str) -> rusqlite::Result<RequestKind> {
    match value {
        "transcript" => Ok(RequestKind::Transcript),
        "question" => Ok(RequestKind::Question),
        _ => Err(rusqlite::Error::InvalidColumnType(
            0,
            "kind".into(),
            rusqlite::types::Type::Text,
        )),
    }
}

fn active_request(conn: &Connection, meeting_id: i64) -> Result<Option<ActiveRequest>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT id, kind FROM message_requests
         WHERE meeting_id = ?1 AND status IN ('pending', 'inflight')",
        [meeting_id],
        |row| {
            Ok(ActiveRequest {
                id: row.get(0)?,
                kind: request_kind(&row.get::<_, String>(1)?)?,
            })
        },
    )
    .optional()
    .context("read active request")
}

fn read_pending_request(conn: &Connection, request_id: i64) -> Result<PendingRequest> {
    let (meeting_id, kind, status, user_text, cursor_before, cursor_after) = conn
        .query_row(
            "SELECT meeting_id, kind, status, user_text, cursor_before, cursor_after
         FROM message_requests WHERE id = ?1 AND status IN ('pending', 'inflight')",
            [request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    request_kind(&row.get::<_, String>(1)?)?,
                    RequestStatus::parse(&row.get::<_, String>(2)?)
                        .map_err(|_| rusqlite::Error::InvalidQuery)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .context("read pending request")?;
    let mut statement = conn.prepare(
        "SELECT segment_id FROM message_request_segments WHERE request_id = ?1 ORDER BY segment_id",
    )?;
    let segment_ids = statement
        .query_map([request_id], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i64>>>()?;
    Ok(PendingRequest {
        id: request_id,
        meeting_id,
        kind,
        status,
        user_text,
        cursor_before,
        cursor_after,
        segment_ids,
    })
}

fn valid_transition(current: MeetingStatus, next: MeetingStatus) -> bool {
    matches!(
        (current, next),
        (
            MeetingStatus::Starting,
            MeetingStatus::Active | MeetingStatus::Interrupted
        ) | (
            MeetingStatus::Active,
            MeetingStatus::Paused | MeetingStatus::Stopping | MeetingStatus::Interrupted
        ) | (
            MeetingStatus::Paused,
            MeetingStatus::Active | MeetingStatus::Stopping | MeetingStatus::Interrupted
        ) | (
            MeetingStatus::Stopping,
            MeetingStatus::Completed | MeetingStatus::Interrupted
        )
    )
}

fn read_meeting(conn: &Connection, id: i64) -> Result<Meeting> {
    conn.query_row(
        "SELECT id, title, started_at, ended_at, status, settings_snapshot FROM meetings WHERE id = ?1",
        [id],
        meeting_from_row,
    ).context("read meeting")
}

fn meeting_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Meeting> {
    let snapshot: String = row.get(5)?;
    let settings = serde_json::from_str(&snapshot).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Meeting {
        id: row.get(0)?,
        title: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        status: MeetingStatus::parse(&row.get::<_, String>(4)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    error.to_string(),
                )),
            )
        })?,
        settings_snapshot: settings,
    })
}

fn transcript_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TranscriptSegment> {
    Ok(TranscriptSegment {
        id: row.get(0)?,
        meeting_id: row.get(1)?,
        event_id: row.get::<_, i64>(2)? as u64,
        source: TranscriptSource::parse(&row.get::<_, String>(3)?).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    error.to_string(),
                )),
            )
        })?,
        start_ms: row.get(4)?,
        end_ms: row.get(5)?,
        revision: row.get::<_, i64>(6)? as u32,
        text: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, time::SystemTime};

    fn test_path(prefix: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{prefix}-{}.sqlite", uuid::Uuid::new_v4()))
    }

    #[test]
    fn context_attempts_survive_retry_and_reopen_and_cascade_with_meeting() -> Result<()> {
        let path = test_path("harness-context");
        let result = (|| -> Result<()> {
            // Schema 8 upgrade preserves requests and every earlier setting.
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("context", 0, &Settings::default())?;
            let pending = store.prepare_question_request(meeting.id, "new question")?;
            store.conn.execute_batch(
                "DROP TRIGGER api_usage_legacy; DROP TABLE api_legacy_usage; DROP TABLE api_budget_receipts; DROP TABLE api_key_caps; DROP TABLE api_prices; DROP TABLE usage_events; DROP TABLE request_context; PRAGMA user_version=8;",
            )?;
            drop(store);
            let mut store = Store::open(&path)?;
            assert!(store
                .save_request_context(pending.id, "chatgpt", None, "model", "{}", "{}", false)
                .is_err());
            store.begin_request(pending.id)?;
            let request = r#"{"developer":"quoted evidence","user":"new question"}"#;
            let metadata = r#"{"sources":["memory 1"],"omittedHistory":2}"#;
            let first = store.save_request_context(
                pending.id, "chatgpt", None, "first", request, metadata, true,
            )?;
            let fallback = store.save_request_context(
                pending.id, "gemini", None, "fallback", request, "{}", false,
            )?;
            assert!(fallback.id > first.id);
            store.fail_request(pending.id, "retryable", None)?;
            assert!(store
                .save_request_context(pending.id, "deepseek", None, "model", "{}", "{}", false)
                .is_err());
            store.begin_request(pending.id)?;
            let retry = store.save_request_context(
                pending.id, "deepseek", None, "retry", request, "{}", false,
            )?;
            store.complete_request(pending.id, "answer", "deepseek", "retry", None)?;
            assert!(store
                .save_request_context(pending.id, "deepseek", None, "model", "{}", "{}", false)
                .is_err());
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(
                store.request_context(pending.id, None)?.unwrap().id,
                retry.id
            );
            assert_eq!(
                store
                    .request_context(pending.id, Some(retry.id))?
                    .unwrap()
                    .id,
                fallback.id
            );
            let oldest = store
                .request_context(pending.id, Some(fallback.id))?
                .unwrap();
            assert_eq!(oldest.id, first.id);
            assert_eq!(
                oldest.request,
                serde_json::from_str::<serde_json::Value>(request)?
            );
            assert_eq!(
                oldest.metadata,
                serde_json::from_str::<serde_json::Value>(metadata)?
            );
            assert!(oldest.upstream_omitted);
            assert!(store.request_context(pending.id, Some(first.id))?.is_none());
            assert!(store.request_context(pending.id, Some(0)).is_err());
            store
                .conn
                .execute("DELETE FROM meetings WHERE id=?1", [meeting.id])?;
            assert!(store.request_context(pending.id, None)?.is_none());
            Ok(())
        })();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }

    #[test]
    fn context_admission_and_corrupt_row_reads_are_bounded() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("context bounds", 0, &Settings::default())?;
        let pending = store.prepare_question_request(meeting.id, "question")?;
        store.begin_request(pending.id)?;
        for (provider, model, request, metadata) in [
            ("unknown", "model", "{}".to_owned(), "{}".to_owned()),
            ("chatgpt", "bad\nmodel", "{}".into(), "{}".into()),
            ("chatgpt", "model", "not json".into(), "{}".into()),
            ("chatgpt", "model", "{}".into(), "not json".into()),
            (
                "chatgpt",
                "model",
                " ".repeat(MAX_CONTEXT_REQUEST + 1),
                "{}".into(),
            ),
            (
                "chatgpt",
                "model",
                "{}".into(),
                " ".repeat(MAX_CONTEXT_METADATA + 1),
            ),
        ] {
            assert!(store
                .save_request_context(pending.id, provider, None, model, &request, &metadata, false)
                .is_err());
        }
        assert!(store
            .save_request_context(0, "chatgpt", None, "model", "{}", "{}", false)
            .is_err());
        assert!(store.request_context(pending.id, None)?.is_none());
        let snapshot =
            store.save_request_context(pending.id, "chatgpt", None, "model", "{}", "{}", false)?;
        // Tampered SQLite values are checked while borrowed, before JSON/string allocation.
        store.conn.execute(
            "UPDATE request_context SET request_json=?1 WHERE id=?2",
            params![" ".repeat(MAX_CONTEXT_REQUEST + 1), snapshot.id],
        )?;
        assert!(store.request_context(pending.id, None).is_err());
        store.conn.execute(
            "UPDATE request_context SET request_json='broken' WHERE id=?1",
            [snapshot.id],
        )?;
        assert!(store.request_context(pending.id, None).is_err());
        Ok(())
    }

    #[test]
    fn documents_upgrade_reindex_sharing_utf8_and_deletion() -> Result<()> {
        let path = test_path("harness-documents");
        let result = (|| -> Result<()> {
            let old = Store::open(&path)?;
            let settings = old.settings()?;
            old.conn.execute_batch("DROP TRIGGER api_usage_legacy; DROP TABLE api_legacy_usage; DROP TABLE api_budget_receipts; DROP TABLE api_key_caps; DROP TABLE api_prices; DROP TABLE usage_events; DROP TABLE request_context; DROP TRIGGER document_chunks_ai; DROP TRIGGER document_chunks_ad; DROP TRIGGER document_chunks_au; DROP TABLE document_chunks_fts; DROP TABLE document_chunks; DROP TABLE documents; DROP TABLE document_copy_cleanup; PRAGMA user_version=7;")?;
            drop(old);
            let store = Store::open(&path)?;
            assert_eq!(store.settings()?, settings);
            assert_eq!(
                store
                    .conn
                    .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))?,
                SCHEMA_VERSION
            );
            let mut input = DocumentInput {
                title: " Budget OR review ".into(),
                source_path: "/private/budget.md".into(),
                stored_path: None,
                content_hash: "a".repeat(64),
                modified_at: 100,
                indexing_version: 1,
                text: "alpha beta résumé 界".into(),
                project: Some(" Atlas ".into()),
                enabled: false,
            };
            let original = store.upsert_document(&input, 200)?;
            assert_eq!(original.source_policy, "reference");
            assert_eq!(original.title, "Budget OR review");
            assert_eq!(original.project.as_deref(), Some("Atlas"));
            assert!(store.retrieve_documents("alpha", 8)?.is_empty());
            store.set_document_enabled(original.id, true)?;
            assert_eq!(store.retrieve_documents("alpha nonexistent", 8)?.len(), 1);
            assert_eq!(store.retrieve_documents("\"alpha\"", 8)?.len(), 1);
            // OR is searched as a literal token, not injected FTS syntax.
            assert!(store.retrieve_documents("missing NOT", 8)?.is_empty());
            assert_eq!(store.retrieve_documents("OR", 8)?.len(), 1);
            input.enabled = true;
            input.title = "Unicode chunks".into();
            input.text = "replacement 界".repeat(16_000);
            input.content_hash = "b".repeat(64);
            input.stored_path = Some("/private/app/docs/budget.md".into());
            let revised = store.upsert_document(&input, 300)?;
            assert_eq!(revised.id, original.id);
            assert_eq!(revised.source_policy, "copy");
            assert_eq!(revised.text_bytes, input.text.len());
            assert!(revised.chunk_count > 1 && revised.chunk_count <= 128);
            assert!(store.retrieve_documents("alpha", 8)?.is_empty());
            let hits = store.retrieve_documents("replacement", 8)?;
            assert!(!hits.is_empty());
            assert!(hits.iter().map(|c| c.text.len()).sum::<usize>() <= 16 * 1024);
            assert!(hits
                .iter()
                .all(|c| c.content_hash == input.content_hash && c.text.len() <= 4096));
            let rebuilt = store
                .conn
                .prepare(
                    "SELECT text FROM document_chunks WHERE document_id=?1 ORDER BY chunk_index",
                )?
                .query_map([original.id], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
                .concat();
            assert_eq!(rebuilt, input.text);
            assert_eq!(
                store.document_source(original.id)?.stored_path,
                input.stored_path
            );
            // A replacement copy keeps its old cleanup locator durably too.
            input.stored_path = Some("/private/app/docs/replacement.md".into());
            let revised = store.upsert_document(&input, 350)?;
            assert_eq!(
                store.pending_document_copies(100)?,
                vec!["/private/app/docs/budget.md".to_owned()]
            );
            store.complete_document_copy_cleanup("/private/app/docs/budget.md")?;
            assert!(!serde_json::to_string(&revised)?.contains("storedPath"));
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(store.document(original.id)?, revised);
            store.set_document_enabled(original.id, false)?;
            assert!(store.retrieve_documents("replacement", 8)?.is_empty());
            input.source_path = "/private/second.md".into();
            input.text = "second content".into();
            let second = store.upsert_document(&input, 400)?;
            let page = store.list_documents(None, 1)?;
            assert_eq!(page.documents[0].id, second.id);
            assert!(page.has_more);
            assert_eq!(
                store.list_documents(page.next, 1)?.documents[0].id,
                original.id
            );
            assert_eq!(
                store.delete_document(original.id)?,
                Some("/private/app/docs/replacement.md".into())
            );
            assert_eq!(
                store.pending_document_copies(100)?,
                vec!["/private/app/docs/replacement.md".to_owned()]
            );
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(store.pending_document_copies(100)?.len(), 1);
            store.complete_document_copy_cleanup("/private/app/docs/replacement.md")?;
            assert!(store.pending_document_copies(100)?.is_empty());
            assert!(store.pending_document_copies(101).is_err());
            assert!(store.document(original.id).is_err());
            assert!(store.delete_document(original.id).is_err());
            assert_eq!(
                store.conn.query_row(
                    "SELECT COUNT(*) FROM document_chunks WHERE document_id=?1",
                    [original.id],
                    |r| r.get::<_, i64>(0)
                )?,
                0
            );
            assert_eq!(store.conn.query_row("SELECT COUNT(*) FROM document_chunks_fts WHERE document_chunks_fts MATCH 'replacement'",[],|r|r.get::<_,i64>(0))?,0);
            assert!(store.list_documents(None, 51).is_err());
            assert!(store.list_documents(Some(0), 1).is_err());
            assert!(store.retrieve_documents("x", 9).is_err());
            assert!(store.retrieve_documents(" ", 1).is_err());
            input.text = "界".repeat(MAX_DOCUMENT_TEXT / 3 + 1);
            assert!(store.upsert_document(&input, 500).is_err());
            input.text = "ok".into();
            input.title = "a".repeat(201);
            assert!(store.upsert_document(&input, 500).is_err());
            input.title = "ok".into();
            input.content_hash = "z".repeat(64);
            assert!(store.upsert_document(&input, 500).is_err());
            input.content_hash = "a".repeat(64);
            input.project = Some("a".repeat(101));
            assert!(store.upsert_document(&input, 500).is_err());
            input.project = None;
            input.source_path = "bad\0path".into();
            assert!(store.upsert_document(&input, 500).is_err());
            Ok(())
        })();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }

    #[test]
    fn manual_memory_upgrade_persistence_literal_search_and_bounds() -> Result<()> {
        let path =
            std::env::temp_dir().join(format!("harness-memory-{}.sqlite", uuid::Uuid::new_v4()));
        let result = (|| -> Result<()> {
            // Exercise the actual previous-schema upgrade without changing other data.
            let old = Store::open(&path)?;
            old.conn.execute_batch("DROP TRIGGER api_usage_legacy; DROP TABLE api_legacy_usage; DROP TABLE api_budget_receipts; DROP TABLE api_key_caps; DROP TABLE api_prices; DROP TABLE usage_events; DROP TABLE request_context; DROP TRIGGER document_chunks_ai; DROP TRIGGER document_chunks_ad; DROP TRIGGER document_chunks_au; DROP TABLE document_chunks_fts; DROP TABLE document_chunks; DROP TABLE documents; DROP TABLE document_copy_cleanup; DROP TRIGGER memories_ai; DROP TRIGGER memories_ad; DROP TRIGGER memories_au; DROP TABLE memories_fts; DROP TABLE memories; PRAGMA user_version=6;")?;
            let settings = old.settings()?;
            drop(old);
            let store = Store::open(&path)?;
            assert_eq!(store.settings()?, settings);
            assert_eq!(
                store
                    .conn
                    .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))?,
                SCHEMA_VERSION
            );
            let mut input = MemoryInput {
                title: "  Budget OR review  ".into(),
                body: "alpha beta résumé 界".into(),
                category: "decision".into(),
                project: Some("  Atlas  ".into()),
                enabled: false,
            };
            let first = store.save_memory(None, &input, 100)?;
            assert_eq!(first.title, "Budget OR review");
            assert_eq!(first.project.as_deref(), Some("Atlas"));
            assert_eq!(first.source, "manual");
            assert!(!first.enabled);
            assert_eq!(
                store.search_memories("alpha OR", 5, false)?,
                vec![first.clone()]
            );
            assert!(store
                .search_memories("alpha OR nonexistent", 5, false)?
                .is_empty());
            assert_eq!(store.search_memories("\"alpha\"", 5, false)?.len(), 1);
            assert_eq!(store.search_memories("résumé 界", 5, false)?.len(), 1);
            assert!(store.search_memories("alpha", 5, true)?.is_empty());
            assert!(store.retrieve_memories("alpha missing", 5)?.is_empty());
            input.enabled = true;
            input.title = "Revised review".into();
            input.body = "gamma 界".into();
            input.project = Some("   ".into());
            let edited = store.save_memory(Some(first.id), &input, 200)?;
            assert_eq!(edited.created_at, 100);
            assert_eq!(edited.updated_at, 200);
            assert_eq!(edited.project, None);
            assert!(store.search_memories("alpha", 5, false)?.is_empty());
            assert_eq!(
                store.retrieve_memories("gamma missing", 5)?,
                vec![edited.clone()]
            );
            assert!(store.search_memories("gamma missing", 5, false)?.is_empty());
            assert!(store.save_memory(Some(first.id), &input, 150).is_err());
            assert!(store.save_memory(Some(999999), &input, 300).is_err());
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(store.get_memory(first.id)?, edited);
            store.delete_memory(first.id)?;
            assert!(store.get_memory(first.id).is_err());
            assert!(store.delete_memory(first.id).is_err());
            assert!(store.search_memories("gamma", 5, false)?.is_empty());
            input.body = "界".repeat(MAX_MEMORY_BODY / 3);
            let mut ids = Vec::new();
            for index in 0..6 {
                input.title = format!("Unicode {index}");
                ids.push(store.save_memory(None, &input, 300)?.id);
            }
            let page = store.list_memory_page(None, 50)?;
            assert_eq!(page.entries.len(), 4);
            assert!(page.has_more);
            assert_eq!(page.next, Some(ids[2]));
            let rest = store.list_memory_page(page.next, 50)?;
            assert_eq!(rest.entries.len(), 2);
            assert!(!rest.has_more);
            assert_eq!(rest.next, None);
            assert_eq!(store.list_memories(None, 50)?.len(), 4);
            assert_eq!(store.search_memories("Unicode", 50, false)?.len(), 4);
            let retrieved = store.retrieve_memories("Unicode", 8)?;
            assert_eq!(retrieved.len(), 1);
            assert!(retrieved.iter().map(|e| e.body.len()).sum::<usize>() <= MAX_MEMORY_BODY);
            assert!(store.list_memories(None, 0).is_err());
            assert!(store.list_memories(Some(0), 5).is_err());
            assert!(store.search_memories("x", 51, false).is_err());
            assert!(store.search_memories(" ", 5, false).is_err());
            assert!(store.search_memories(&"x".repeat(513), 5, false).is_err());
            assert!(store.search_memories(&"x ".repeat(17), 5, false).is_err());
            assert!(store.retrieve_memories("x", 9).is_err());
            input.body.push('界');
            assert!(store.save_memory(None, &input, 300).is_err());
            input.body = "ok".into();
            input.title = "a".repeat(201);
            assert!(store.save_memory(None, &input, 300).is_err());
            input.title = "ok".into();
            input.project = Some("a".repeat(101));
            assert!(store.save_memory(None, &input, 300).is_err());
            input.project = None;
            input.category = "arbitrary".into();
            assert!(store.save_memory(None, &input, 300).is_err());
            input.category = "note".into();
            assert!(store.save_memory(None, &input, -1).is_err());
            // Even an externally damaged database cannot force unbounded text allocation.
            store
                .conn
                .execute_batch("PRAGMA ignore_check_constraints=ON;")?;
            store.conn.execute(
                "UPDATE memories SET body=?1 WHERE id=?2",
                params!["z".repeat(MAX_MEMORY_BODY + 1), ids[5]],
            )?;
            assert!(store.get_memory(ids[5]).is_err());
            assert!(store.list_memories(None, 5).is_err());
            Ok(())
        })();
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("sqlite-wal"));
        let _ = fs::remove_file(path.with_extension("sqlite-shm"));
        result
    }

    #[test]
    fn custom_instruction_unicode_byte_bound_matches_the_compiler() {
        let settings = Settings {
            response_mode: "custom".into(),
            custom_instruction: "😀".repeat(3000),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
        assert!(Settings {
            custom_instruction: "😀".repeat(2000),
            ..settings
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn large_transcript_slices_preserve_utf8_and_frozen_revision() -> Result<()> {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("large final", 0, &Settings::default())?;
        let original = "é😀".repeat(10_000);
        let row = store.insert_finalized_transcript(
            meeting.id,
            1,
            TranscriptSource::System,
            0,
            None,
            1,
            &original,
        )?;
        let mut text = String::new();
        while let Some(request) =
            store.prepare_transcript_through(meeting.id, 100, 4096, Some(row.id))?
        {
            assert!(request.user_text.len() <= 4096);
            assert_eq!(request.segment_ids, vec![row.id]);
            assert_eq!(
                store
                    .meeting_pending_request(meeting.id)?
                    .unwrap()
                    .user_text,
                request.user_text
            );
            store.begin_request(request.id)?;
            if text.is_empty() {
                store.fail_request(request.id, "offline", None)?;
                assert_eq!(
                    store
                        .prepare_transcript_through(meeting.id, 100, 4096, Some(row.id))?
                        .unwrap()
                        .id,
                    request.id
                );
                store.begin_request(request.id)?;
            }
            text.push_str(&request.user_text);
            store.complete_request(request.id, "answer", "test", "model", None)?;
            if text.len() == request.user_text.len() {
                store.insert_finalized_transcript(
                    meeting.id,
                    1,
                    TranscriptSource::System,
                    0,
                    None,
                    2,
                    "later revision",
                )?;
            }
        }
        assert_eq!(text, original);
        assert!(store
            .prepare_transcript_request(meeting.id, 100, 4096)?
            .is_none());
        Ok(())
    }

    #[test]
    fn bounded_manual_batches_stop_at_the_press_and_recent_context_uses_relative_time() -> Result<()>
    {
        let mut store = Store::open(":memory:")?;
        let meeting = store.create_meeting("batches", 1_000_000, &Settings::default())?;
        for event in 1..=105 {
            store.insert_finalized_transcript(
                meeting.id,
                event,
                TranscriptSource::System,
                event as i64,
                None,
                1,
                "old",
            )?;
        }
        let recent = store.insert_finalized_transcript(
            meeting.id,
            200,
            TranscriptSource::System,
            100_000,
            None,
            1,
            "recent",
        )?;
        let (rows, omitted) = store.recent_transcript(meeting.id, 90_000, true)?;
        assert!(!omitted);
        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            vec![recent.id]
        );
        let (_rows, omitted) = store.recent_transcript(meeting.id, 0, true)?;
        assert!(omitted);
        let through = store.latest_finalized_id(meeting.id)?.unwrap();
        let late = store.insert_finalized_transcript(
            meeting.id,
            201,
            TranscriptSource::System,
            0,
            None,
            1,
            "after the press",
        )?;
        let mut delivered = Vec::new();
        while let Some(request) =
            store.prepare_transcript_through(meeting.id, 50, 4096, Some(through))?
        {
            assert!(request.segment_ids.iter().all(|id| *id <= through));
            delivered.extend(request.segment_ids.clone());
            store.begin_request(request.id)?;
            store.complete_request(request.id, "answer", "test", "model", None)?;
        }
        assert_eq!(delivered.len(), 106);
        assert!(!delivered.contains(&late.id));
        let next = store
            .prepare_transcript_request(meeting.id, 100, 4096)?
            .unwrap();
        assert_eq!(next.segment_ids, vec![late.id]);
        Ok(())
    }

    #[test]
    fn message_delivery_reuses_pending_rows_and_advances_only_on_delivery() -> Result<()> {
        let path = test_path("harness-delivery");
        let result = (|| {
            let settings = Settings::default();
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("call", 0, &settings)?;
            let first = store.insert_finalized_transcript(
                meeting.id,
                1,
                TranscriptSource::System,
                100,
                None,
                1,
                "é",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::Microphone,
                101,
                None,
                1,
                "mic",
            )?;
            assert!(store
                .prepare_transcript_request(meeting.id, 100, 0)
                .is_err());
            let pending = store
                .prepare_transcript_request(meeting.id, 100, 64 * 1024)?
                .unwrap();
            assert_eq!(pending.user_text, "é");
            assert_eq!(pending.segment_ids, vec![first.id]);
            let request_id = pending.id;
            store.insert_finalized_transcript(
                meeting.id,
                1,
                TranscriptSource::System,
                100,
                None,
                2,
                "revised é",
            )?;
            drop(store);

            let mut store = Store::open(&path)?;
            let reopened = store.pending_request(request_id)?.unwrap();
            assert_eq!(reopened.id, request_id);
            assert_eq!(reopened.user_text, "é");
            store.begin_request(request_id)?;
            let reader = Store::open(&path)?;
            assert_eq!(
                reader.pending_request(request_id)?.unwrap().status,
                RequestStatus::InFlight
            );
            assert!(store.retry_request(request_id).is_err());
            assert!(store.begin_request(request_id).is_err());
            assert_eq!(reader.recover_interrupted_requests()?, 1);
            assert_eq!(
                store.pending_request(request_id)?.unwrap().status,
                RequestStatus::Pending
            );
            store.fail_request(request_id, "offline", None)?;
            assert_eq!(store.pending_request(request_id)?.unwrap().id, request_id);
            store.begin_request(request_id)?;
            store.complete_request(request_id, "answer", "provider", "model", None)?;
            assert!(store.pending_request(request_id)?.is_none());

            store.insert_finalized_transcript(
                meeting.id,
                3,
                TranscriptSource::System,
                1,
                None,
                1,
                "late system",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                4,
                TranscriptSource::Microphone,
                2,
                None,
                1,
                "late mic",
            )?;
            let next = store
                .prepare_transcript_request(meeting.id, 100, 64 * 1024)?
                .unwrap();
            assert_eq!(next.user_text, "late system");
            store.begin_request(next.id)?;
            store.complete_request(
                next.id,
                "second",
                "provider",
                "model",
                Some(RequestUsage {
                    input_tokens: Some(1),
                    output_tokens: Some(2),
                    total_tokens: Some(3),
                }),
            )?;
            let history = store.completed_history(meeting.id)?;
            assert_eq!(
                history
                    .entries
                    .iter()
                    .map(|row| row.role.as_str())
                    .collect::<Vec<_>>(),
                ["user", "assistant", "user", "assistant"]
            );
            assert!(store
                .prepare_question_request(meeting.id, &"x".repeat(MAX_REQUEST_TEXT + 1))
                .is_err());
            Ok::<_, anyhow::Error>(())
        })();
        let _ = std::fs::remove_file(&path);
        result
    }

    #[test]
    fn bounded_history_prefers_newest_and_partial_output_is_not_retried() -> Result<()> {
        let path = test_path("harness-message-history");
        let result = (|| {
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("history", 0, &Settings::default())?;
            for id in 0..20 {
                let request =
                    store.prepare_question_request(meeting.id, &format!("question {id}"))?;
                store.begin_request(request.id)?;
                store.complete_request(request.id, "answer", "gemini", "model", None)?;
            }
            let history = store.completed_history(meeting.id)?;
            assert_eq!(history.entries.len(), 32);
            assert_eq!(history.entries[0].text, "question 4");
            assert_eq!(history.entries[30].text, "question 19");
            assert_eq!(history.omitted_requests, 4);
            let partial = store.prepare_question_request(meeting.id, "partial question")?;
            store.begin_request(partial.id)?;
            store.fail_request(partial.id, "interrupted", Some("partial answer"))?;
            assert!(store.retry_request(partial.id).is_err());
            assert!(store.pending_request(partial.id)?.is_none());
            Ok::<_, anyhow::Error>(())
        })();
        let _ = fs::remove_file(path);
        result
    }

    #[test]
    fn streamed_output_recovery_preserves_exact_text_and_never_resends() -> Result<()> {
        let path = test_path("harness-output-checkpoint");
        let result = (|| {
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("stream", 0, &Settings::default())?;
            store.insert_finalized_transcript(
                meeting.id,
                1,
                TranscriptSource::System,
                100,
                None,
                1,
                "delivered speech",
            )?;
            let request = store
                .prepare_transcript_request(meeting.id, 100, MAX_REQUEST_TEXT)?
                .unwrap();
            store.begin_request(request.id)?;
            store.checkpoint_request_output(
                request.id, " 
", "chatgpt", "model",
            )?;
            assert!(store
                .checkpoint_request_output(request.id, "changed", "chatgpt", "model")
                .is_err());
            assert!(store
                .complete_request(request.id, "wrong prefix", "chatgpt", "model", None)
                .is_err());
            drop(store);
            let mut store = Store::open(&path)?;
            assert_eq!(
                store.pending_request(request.id)?.unwrap().status,
                RequestStatus::InFlight
            );
            assert_eq!(store.recover_interrupted_requests()?, 1);
            assert!(store.pending_request(request.id)?.is_none());
            assert!(store.retry_request(request.id).is_err());
            assert!(store
                .prepare_transcript_request(meeting.id, 100, MAX_REQUEST_TEXT)?
                .is_none());
            let history = store.completed_history(meeting.id)?;
            assert_eq!(
                history.entries[1].text,
                " 
"
            );
            assert_eq!(history.entries[1].provider.as_deref(), Some("chatgpt"));
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::System,
                1,
                None,
                1,
                "new late speech",
            )?;
            let next = store
                .prepare_transcript_request(meeting.id, 100, MAX_REQUEST_TEXT)?
                .unwrap();
            assert_eq!(next.user_text, "new late speech");
            store.begin_request(next.id)?;
            let large = "🙂".repeat(MAX_ANSWER / 4);
            store.checkpoint_request_output(next.id, &large, "gemini", "model")?;
            assert!(store
                .checkpoint_request_output(next.id, &(large.clone() + "x"), "gemini", "model")
                .is_err());
            store.fail_request(next.id, "interrupted", None)?;
            assert!(store.retry_request(next.id).is_err());
            assert_eq!(store.completed_history(meeting.id)?.omitted_requests, 1);
            let question = store.prepare_question_request(meeting.id, "no output")?;
            store.begin_request(question.id)?;
            let reader = Store::open(&path)?;
            assert_eq!(
                reader.pending_request(question.id)?.unwrap().status,
                RequestStatus::InFlight
            );
            assert_eq!(reader.recover_interrupted_requests()?, 1);
            assert_eq!(store.retry_request(question.id)?.id, question.id);
            store.begin_request(question.id)?;
            store.checkpoint_request_output(question.id, "	", "deepseek", "model")?;
            store.cancel_request(question.id)?;
            assert!(store.pending_request(question.id)?.is_none());
            Ok::<_, anyhow::Error>(())
        })();
        let _ = fs::remove_file(path);
        result
    }

    #[test]
    fn meeting_lifecycle_reopens_and_recovers_stale_rows() -> Result<()> {
        let path = test_path("harness-meeting");
        let result = (|| {
            let settings = Settings::default();
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("  Sales call  ", 1_000, &settings)?;
            assert_eq!(meeting.status, MeetingStatus::Starting);
            assert!(store
                .transition_meeting(meeting.id, MeetingStatus::Completed, 1_001)
                .is_err());
            store.transition_meeting(meeting.id, MeetingStatus::Active, 1_001)?;
            store.transition_meeting(meeting.id, MeetingStatus::Paused, 1_002)?;
            store.transition_meeting(meeting.id, MeetingStatus::Active, 1_003)?;
            store.transition_meeting(meeting.id, MeetingStatus::Stopping, 1_004)?;
            let completed =
                store.transition_meeting(meeting.id, MeetingStatus::Completed, 1_005)?;
            assert_eq!(completed.ended_at, Some(1_005));
            let stale = store.create_meeting("stale", 2_000, &settings)?;
            drop(store);
            let mut reopened = Store::open(&path)?;
            assert_eq!(
                reopened.meeting(meeting.id)?.status,
                MeetingStatus::Completed
            );
            assert_eq!(reopened.recover_stale_meetings(2_500)?, 1);
            assert_eq!(
                reopened.meeting(stale.id)?.status,
                MeetingStatus::Interrupted
            );
            assert_eq!(reopened.meeting(stale.id)?.ended_at, Some(2_500));
            Ok::<_, anyhow::Error>(())
        })();
        fs::remove_file(&path)?;
        result
    }

    #[test]
    fn transcript_revisions_are_idempotent_and_system_cursor_excludes_microphone() -> Result<()> {
        let path = test_path("harness-transcript");
        let result = (|| {
            let settings = Settings::default();
            let mut store = Store::open(&path)?;
            let meeting = store.create_meeting("call", 0, &settings)?;
            assert!(store
                .insert_finalized_transcript(
                    meeting.id,
                    1,
                    TranscriptSource::System,
                    0,
                    None,
                    1,
                    &"é".repeat(MAX_TRANSCRIPT_TEXT / 2 + 1),
                )
                .is_err());
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::System,
                20,
                None,
                1,
                "later",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::System,
                20,
                None,
                1,
                "duplicate",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::System,
                20,
                None,
                2,
                "revised",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                2,
                TranscriptSource::Microphone,
                15,
                None,
                1,
                "private",
            )?;
            let first = store.insert_finalized_transcript(
                meeting.id,
                1,
                TranscriptSource::System,
                10,
                None,
                1,
                "first",
            )?;
            store.insert_finalized_transcript(
                meeting.id,
                3,
                TranscriptSource::System,
                5,
                None,
                1,
                "late",
            )?;
            drop(store);
            let mut reopened = Store::open(&path)?;
            let page = reopened.read_transcript_page(meeting.id, None, 10)?;
            assert_eq!(page.segments.len(), 4);
            assert!(page
                .segments
                .iter()
                .any(|segment| segment.text == "private"));
            let system = reopened.finalized_system_after(meeting.id, None, 10)?;
            assert_eq!(
                system
                    .segments
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<Vec<_>>(),
                ["revised", "first", "late"]
            );
            let after_first = reopened.finalized_system_after(meeting.id, Some(first.id), 10)?;
            assert_eq!(after_first.segments.len(), 1);
            assert_eq!(after_first.segments[0].text, "late");

            let ordered = reopened.create_meeting("same timestamp", 0, &settings)?;
            let same_system = reopened.insert_finalized_transcript(
                ordered.id,
                7,
                TranscriptSource::System,
                30,
                None,
                1,
                "system",
            )?;
            reopened.insert_finalized_transcript(
                ordered.id,
                7,
                TranscriptSource::Microphone,
                30,
                None,
                1,
                "microphone",
            )?;
            let first_page = reopened.read_transcript_page(ordered.id, None, 1)?;
            let second_page = reopened.read_transcript_page(ordered.id, first_page.next, 1)?;
            assert_eq!(first_page.segments[0].id, same_system.id);
            assert_eq!(second_page.segments[0].source, TranscriptSource::Microphone);
            Ok::<_, anyhow::Error>(())
        })();
        fs::remove_file(&path)?;
        result
    }

    #[test]
    fn saved_history_and_literal_search_survive_reopen() -> Result<()> {
        let path = test_path("harness-history");
        let result = (|| {
            let settings = Settings::default();
            let mut store = Store::open(&path)?;
            let completed = store.create_meeting("completed", 10, &settings)?;
            store.transition_meeting(completed.id, MeetingStatus::Active, 11)?;
            store.transition_meeting(completed.id, MeetingStatus::Stopping, 12)?;
            store.transition_meeting(completed.id, MeetingStatus::Completed, 13)?;
            let interrupted = store.create_meeting("interrupted", 20, &settings)?;
            store.transition_meeting(interrupted.id, MeetingStatus::Interrupted, 21)?;
            let active = store.create_meeting("active", 30, &settings)?;

            store.insert_finalized_transcript(
                completed.id,
                1,
                TranscriptSource::System,
                40,
                None,
                1,
                "alpha OR beta",
            )?;
            store.insert_finalized_transcript(
                completed.id,
                2,
                TranscriptSource::Microphone,
                50,
                None,
                1,
                "alpha private",
            )?;
            store.insert_finalized_transcript(
                completed.id,
                3,
                TranscriptSource::System,
                60,
                None,
                1,
                "remove me",
            )?;
            store.insert_finalized_transcript(
                active.id,
                4,
                TranscriptSource::System,
                70,
                None,
                1,
                "alpha active",
            )?;
            let revised = store.insert_finalized_transcript(
                completed.id,
                1,
                TranscriptSource::System,
                40,
                None,
                2,
                "gamma revised",
            )?;
            store.conn.execute(
                "DELETE FROM transcript_segments WHERE id = ?1",
                [store.conn.query_row(
                    "SELECT id FROM transcript_segments WHERE event_id = 3",
                    [],
                    |row| row.get::<_, i64>(0),
                )?],
            )?;
            store.conn.execute(
                "INSERT INTO transcript_segments (meeting_id, event_id, start_ms, source, is_final, revision, text) VALUES (?1, 99, 80, 'system', 0, 1, 'unfinished draft')",
                [completed.id],
            )?;
            assert!(store
                .read_transcript_page(completed.id, None, 100)?
                .segments
                .iter()
                .all(|row| row.text != "unfinished draft"));
            drop(store);

            let reopened = Store::open(&path)?;
            let saved = reopened.list_saved_meetings(None, 100)?;
            assert_eq!(
                saved.iter().map(|m| m.id).collect::<Vec<_>>(),
                vec![interrupted.id, completed.id]
            );
            assert_eq!(
                reopened.list_saved_meetings(Some(interrupted.id), 1)?[0].id,
                completed.id
            );
            assert!(reopened.list_saved_meetings(None, 0).is_err());
            assert!(reopened.list_saved_meetings(Some(0), 1).is_err());

            let results = reopened.search_transcripts(completed.id, "alpha OR gamma", 10)?;
            assert!(results.is_empty());
            let results = reopened.search_transcripts(completed.id, "gamma", 10)?;
            assert_eq!(results, vec![revised]);
            assert_eq!(results[0].source, TranscriptSource::System);
            assert!(reopened
                .search_transcripts(completed.id, "remove", 10)?
                .is_empty());
            assert!(
                reopened
                    .search_transcripts(completed.id, "alpha", 10)?
                    .len()
                    == 1
            );
            assert!(reopened.search_transcripts(active.id, "alpha", 10)?.len() == 1);
            assert!(reopened.search_transcripts(completed.id, "   ", 1).is_err());
            assert!(reopened
                .search_transcripts(completed.id, "x ".repeat(257).as_str(), 1)
                .is_err());
            assert!(reopened.search_transcripts(completed.id, "x", 0).is_err());
            assert!(reopened.search_transcripts(999_999, "x", 1).is_err());
            Ok::<_, anyhow::Error>(())
        })();
        fs::remove_file(&path)?;
        result
    }

    #[test]
    fn validates_modes_lengths_and_shortcuts() {
        let mut settings = Settings {
            response_mode: "custom".into(),
            ..Settings::default()
        };
        assert!(settings.validate().is_err());
        settings.custom_instruction = "   ".into();
        assert!(settings.validate().is_err());
        settings.custom_instruction = "Do this".into();
        assert!(settings.validate().is_ok());
        settings.send_mode = "never".into();
        assert!(settings.validate().is_err());
        settings.send_mode = "on_hotkey".into();
        settings.send_shortcut = " ctrl+space ".into();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn upgrades_existing_settings_and_keeps_account_selection_explicit() -> Result<()> {
        let path =
            std::env::temp_dir().join(format!("harness-upgrade-{}.sqlite", uuid::Uuid::new_v4()));
        let result = (|| -> Result<()> {
            let conn = Connection::open(&path)?;
            conn.execute_batch(include_str!("../migrations/001_initial.sql"))?;
            let settings = Settings {
                send_mode: "automatic".into(),
                ..Settings::default()
            };
            conn.execute(
                "INSERT INTO settings (id, value) VALUES (1, ?1)",
                [serde_json::to_string(&settings)?],
            )?;
            conn.pragma_update(None, "user_version", 1)?;
            drop(conn);
            let store = Store::open(&path)?;
            assert_eq!(store.settings()?, settings);
            assert_eq!(store.providers()?, crate::providers::defaults());
            let host = store.oauth_host_id()?;
            assert_eq!(host, store.oauth_host_id()?);
            let account = AccountMetadata {
                account_id: "a".repeat(43),
                display_name: Some("Sales".into()),
                email: None,
                client_id: "issued-client".into(),
                selected_model: None,
            };
            store.save_account(&account)?;
            store.set_selected_model(&account.account_id, "gpt-test")?;
            assert_eq!(
                store.accounts()?[0].selected_model.as_deref(),
                Some("gpt-test")
            );
            store.deselect_selected_model(&account.account_id, "other")?;
            assert_eq!(
                store.accounts()?[0].selected_model.as_deref(),
                Some("gpt-test")
            );
            store.deselect_selected_model(&account.account_id, "gpt-test")?;
            assert_eq!(store.accounts()?[0].selected_model, None);
            assert_eq!(store.active_account()?, None);
            assert!(store.select_account("missing").is_err());
            store.select_account(&account.account_id)?;
            assert_eq!(store.accounts()?, vec![account.clone()]);
            drop(store);
            let store = Store::open(&path)?;
            assert_eq!(store.oauth_host_id()?, host);
            assert_eq!(store.active_account()?, Some(account.account_id.clone()));
            store.remove_account(&account.account_id)?;
            assert_eq!(store.active_account()?, None);
            Ok(())
        })();
        fs::remove_file(&path)?;
        result
    }

    #[test]
    fn persists_settings_and_migrates_file() {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("amv-store-{suffix}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("store.sqlite");
        let result = (|| -> Result<()> {
            let mut store = Store::open(&path)?;
            let version: i64 = store
                .conn
                .pragma_query_value(None, "user_version", |row| row.get(0))?;
            assert_eq!(version, SCHEMA_VERSION);
            let settings = Settings {
                send_mode: "automatic".into(),
                ..Settings::default()
            };
            store.save_settings(&settings)?;
            let mut providers = crate::providers::defaults();
            providers[0].base_url = "https://example.com/api/".into();
            store.save_providers(&providers)?;
            drop(store);
            assert_eq!(Store::open(&path)?.settings()?, settings);
            assert_eq!(Store::open(&path)?.providers()?, providers);

            let store = Store::open(&path)?;
            store.conn.execute(
                "INSERT INTO meetings (started_at) VALUES (1767225600000)",
                [],
            )?;
            store.conn.execute(
                "INSERT INTO transcript_segments (meeting_id, start_ms, source, text) VALUES (1, 0, 'system', 'budget review')",
                [],
            )?;
            let found: String = store.conn.query_row(
                "SELECT text FROM transcript_segments_fts WHERE transcript_segments_fts MATCH 'budget'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(found, "budget review");
            store.conn.execute(
                "UPDATE transcript_segments SET text = 'revised decision' WHERE id = 1",
                [],
            )?;
            let old_count: i64 = store.conn.query_row("SELECT count(*) FROM transcript_segments_fts WHERE transcript_segments_fts MATCH 'budget'", [], |r| r.get(0))?;
            assert_eq!(old_count, 0);
            store.conn.execute("INSERT INTO transcript_segments (meeting_id, start_ms, source, text, is_final) VALUES (1, 1000, 'microphone', 'unfinished', 0)", [])?;
            let partial_count: i64 = store.conn.query_row("SELECT count(*) FROM transcript_segments_fts WHERE transcript_segments_fts MATCH 'unfinished'", [], |r| r.get(0))?;
            assert_eq!(partial_count, 0);
            store
                .conn
                .execute("DELETE FROM meetings WHERE id = 1", [])?;
            let remaining: i64 = store.conn.query_row("SELECT count(*) FROM transcript_segments_fts WHERE transcript_segments_fts MATCH 'decision'", [], |r| r.get(0))?;
            assert_eq!(remaining, 0);
            store
                .conn
                .execute("UPDATE settings SET value = 'broken'", [])?;
            assert!(store.settings().is_err());
            store.conn.pragma_update(None, "user_version", 99)?;
            drop(store);
            assert!(Store::open(&path).is_err());
            Ok(())
        })();
        fs::remove_dir_all(&dir).unwrap();
        result.unwrap();
    }
}
