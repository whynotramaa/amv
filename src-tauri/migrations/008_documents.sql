CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 200 AND length(CAST(title AS BLOB))<=800),
    source_path TEXT NOT NULL UNIQUE CHECK(length(CAST(source_path AS BLOB)) BETWEEN 1 AND 8192),
    stored_path TEXT CHECK(stored_path IS NULL OR length(CAST(stored_path AS BLOB)) BETWEEN 1 AND 8192),
    content_hash TEXT NOT NULL CHECK(length(content_hash)=64),
    modified_at INTEGER NOT NULL CHECK(modified_at BETWEEN 0 AND 100000000000000),
    indexed_at INTEGER NOT NULL CHECK(indexed_at BETWEEN 0 AND 100000000000000),
    indexing_version INTEGER NOT NULL CHECK(indexing_version BETWEEN 1 AND 4294967295),
    project TEXT CHECK(project IS NULL OR (length(project) BETWEEN 1 AND 100 AND length(CAST(project AS BLOB))<=400)),
    enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
    chunk_count INTEGER NOT NULL CHECK(chunk_count BETWEEN 1 AND 128),
    text_bytes INTEGER NOT NULL CHECK(text_bytes BETWEEN 1 AND 262144)
);
CREATE TABLE document_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL CHECK(chunk_index BETWEEN 0 AND 127),
    title TEXT NOT NULL,
    text TEXT NOT NULL CHECK(length(CAST(text AS BLOB)) BETWEEN 1 AND 4096),
    project TEXT,
    UNIQUE(document_id,chunk_index)
);
CREATE VIRTUAL TABLE document_chunks_fts USING fts5(title,text,project,content='document_chunks',content_rowid='id');
CREATE TRIGGER document_chunks_ai AFTER INSERT ON document_chunks BEGIN
    INSERT INTO document_chunks_fts(rowid,title,text,project) VALUES(new.id,new.title,new.text,new.project);
END;
CREATE TRIGGER document_chunks_ad AFTER DELETE ON document_chunks BEGIN
    INSERT INTO document_chunks_fts(document_chunks_fts,rowid,title,text,project) VALUES('delete',old.id,old.title,old.text,old.project);
END;
CREATE TRIGGER document_chunks_au AFTER UPDATE ON document_chunks BEGIN
    INSERT INTO document_chunks_fts(document_chunks_fts,rowid,title,text,project) VALUES('delete',old.id,old.title,old.text,old.project);
    INSERT INTO document_chunks_fts(rowid,title,text,project) VALUES(new.id,new.title,new.text,new.project);
END;
CREATE TABLE document_copy_cleanup (
    path TEXT PRIMARY KEY CHECK(length(CAST(path AS BLOB)) BETWEEN 1 AND 8192)
);
