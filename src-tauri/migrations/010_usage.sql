ALTER TABLE request_context ADD COLUMN account_id TEXT;

CREATE TABLE usage_events (
    id INTEGER PRIMARY KEY,
    context_id INTEGER NOT NULL UNIQUE REFERENCES request_context(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('question', 'transcript')),
    status TEXT NOT NULL DEFAULT 'in_flight' CHECK (status IN ('in_flight', 'ok', 'limited', 'error', 'fell_back', 'partial', 'cancelled', 'interrupted')),
    input_tokens INTEGER CHECK (input_tokens BETWEEN 0 AND 1000000000000),
    output_tokens INTEGER CHECK (output_tokens BETWEEN 0 AND 1000000000000),
    cached_tokens INTEGER CHECK (cached_tokens BETWEEN 0 AND 1000000000000),
    first_token_ms INTEGER CHECK (first_token_ms BETWEEN 0 AND 100000000000000),
    total_ms INTEGER CHECK (total_ms BETWEEN 0 AND 100000000000000),
    error_code TEXT CHECK (length(error_code) <= 100),
    limit_hit INTEGER NOT NULL DEFAULT 0 CHECK (limit_hit IN (0, 1)),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    CHECK (cached_tokens IS NULL OR input_tokens IS NULL OR cached_tokens <= input_tokens),
    CHECK (first_token_ms IS NULL OR total_ms IS NULL OR first_token_ms <= total_ms)
);
CREATE INDEX usage_events_created ON usage_events(created_at);
