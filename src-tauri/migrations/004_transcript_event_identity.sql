ALTER TABLE transcript_segments ADD COLUMN event_id INTEGER;

UPDATE transcript_segments SET event_id = id WHERE event_id IS NULL;

CREATE UNIQUE INDEX transcript_segments_event_key
    ON transcript_segments (meeting_id, source, event_id);

CREATE INDEX transcript_segments_meeting_order
    ON transcript_segments (meeting_id, start_ms, id);
