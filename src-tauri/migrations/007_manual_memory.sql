CREATE TABLE memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 200 AND length(CAST(title AS BLOB)) <= 800),
    body TEXT NOT NULL CHECK(length(CAST(body AS BLOB)) BETWEEN 1 AND 16384),
    category TEXT NOT NULL CHECK(category IN ('project','person','organization','preference','note','decision','experience','education','term')),
    project TEXT CHECK(project IS NULL OR (length(project) BETWEEN 1 AND 100 AND length(CAST(project AS BLOB)) <= 400)),
    enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN (0,1)),
    created_at INTEGER NOT NULL CHECK(created_at BETWEEN 0 AND 100000000000000),
    updated_at INTEGER NOT NULL CHECK(updated_at BETWEEN created_at AND 100000000000000),
    source TEXT NOT NULL DEFAULT 'manual' CHECK(source = 'manual')
);
CREATE VIRTUAL TABLE memories_fts USING fts5(title, body, project, content='memories', content_rowid='id');
CREATE TRIGGER memories_ai AFTER INSERT ON memories BEGIN
    INSERT INTO memories_fts(rowid,title,body,project) VALUES(new.id,new.title,new.body,new.project);
END;
CREATE TRIGGER memories_ad AFTER DELETE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts,rowid,title,body,project) VALUES('delete',old.id,old.title,old.body,old.project);
END;
CREATE TRIGGER memories_au AFTER UPDATE ON memories BEGIN
    INSERT INTO memories_fts(memories_fts,rowid,title,body,project) VALUES('delete',old.id,old.title,old.body,old.project);
    INSERT INTO memories_fts(rowid,title,body,project) VALUES(new.id,new.title,new.body,new.project);
END;
