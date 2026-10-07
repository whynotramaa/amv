CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    value TEXT NOT NULL
);

CREATE TABLE meetings (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    title TEXT NOT NULL DEFAULT 'Untitled meeting',
    status TEXT NOT NULL DEFAULT 'starting' CHECK (status IN ('starting', 'active', 'paused', 'stopping', 'completed', 'interrupted')),
    settings_snapshot TEXT NOT NULL DEFAULT '{}',
    ended_at INTEGER,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE transcript_segments (
    id INTEGER PRIMARY KEY,
    meeting_id INTEGER NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER,
    source TEXT NOT NULL CHECK (source IN ('system', 'microphone')),
    is_final INTEGER NOT NULL DEFAULT 1 CHECK (is_final IN (0, 1)),
    revision INTEGER NOT NULL DEFAULT 0,
    text TEXT NOT NULL
);

CREATE VIRTUAL TABLE transcript_segments_fts USING fts5(
    text,
    source,
    content = 'transcript_segments',
    content_rowid = 'id'
);

CREATE TRIGGER transcript_segments_ai AFTER INSERT ON transcript_segments WHEN new.is_final = 1 BEGIN
    INSERT INTO transcript_segments_fts(rowid, text, source)
    VALUES (new.id, new.text, new.source);
END;

CREATE TRIGGER transcript_segments_ad AFTER DELETE ON transcript_segments WHEN old.is_final = 1 BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text, source)
    VALUES ('delete', old.id, old.text, old.source);
END;

CREATE TRIGGER transcript_segments_au_delete AFTER UPDATE ON transcript_segments WHEN old.is_final = 1 BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text, source)
    VALUES ('delete', old.id, old.text, old.source);
END;

CREATE TRIGGER transcript_segments_au_insert AFTER UPDATE ON transcript_segments WHEN new.is_final = 1 BEGIN
    INSERT INTO transcript_segments_fts(rowid, text, source)
    VALUES (new.id, new.text, new.source);
END;
