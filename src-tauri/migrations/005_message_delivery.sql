CREATE TABLE message_requests (
    id INTEGER PRIMARY KEY,
    meeting_id INTEGER NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('transcript', 'question')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'inflight', 'completed', 'partial')),
    user_text TEXT NOT NULL,
    assistant_text TEXT,
    provider TEXT,
    model TEXT,
    error_summary TEXT,
    cursor_before INTEGER,
    cursor_after INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    completed_at INTEGER
);

CREATE UNIQUE INDEX message_requests_one_active_per_meeting
    ON message_requests (meeting_id)
    WHERE status IN ('pending', 'inflight');

CREATE INDEX message_requests_history
    ON message_requests (meeting_id, id)
    WHERE status IN ('completed', 'partial');

CREATE TABLE message_request_segments (
    request_id INTEGER NOT NULL REFERENCES message_requests(id) ON DELETE CASCADE,
    segment_id INTEGER NOT NULL REFERENCES transcript_segments(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL,
    text TEXT NOT NULL,
    PRIMARY KEY (request_id, segment_id)
);

CREATE INDEX message_request_segments_request_order
    ON message_request_segments (request_id, segment_id);
