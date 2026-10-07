ALTER TABLE message_requests ADD COLUMN cursor_after_offset INTEGER NOT NULL DEFAULT 0 CHECK(cursor_after_offset >= 0);
ALTER TABLE message_request_segments ADD COLUMN full_text TEXT;
