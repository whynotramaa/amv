CREATE TABLE request_context (
    id INTEGER PRIMARY KEY,
    request_id INTEGER NOT NULL REFERENCES message_requests(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('chatgpt', 'gemini', 'deepseek')),
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    request_json TEXT NOT NULL,
    metadata_json TEXT NOT NULL,
    upstream_omitted INTEGER NOT NULL CHECK (upstream_omitted IN (0, 1))
);

CREATE INDEX request_context_attempts ON request_context (request_id, id DESC);
