use rusqlite::Connection;

/// Creates all tables, FTS5 virtual table, triggers, and indexes.
/// Called once during the initial migration (version 0 → 1).
pub fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        -- Main notes table with multi-layer memory fields
        CREATE TABLE IF NOT EXISTS notes (
            id                           INTEGER PRIMARY KEY AUTOINCREMENT,
            raw_text                     TEXT NOT NULL,
            summary                      TEXT NOT NULL DEFAULT '',
            importance                   REAL DEFAULT 0.5,
            current_score                REAL DEFAULT 0.5,
            layer                        TEXT DEFAULT 'working'
                                             CHECK(layer IN ('working', 'episodic', 'semantic')),
            pinned                       INTEGER DEFAULT 0,
            archived                     INTEGER DEFAULT 0,
            created_at                   TEXT NOT NULL,
            last_accessed_at             TEXT,
            last_updated_at              TEXT,
            layer_promoted_at            TEXT,
            access_count                 INTEGER DEFAULT 0,
            access_count_since_promotion INTEGER DEFAULT 0,
            entities                     TEXT DEFAULT '[]',
            topics                       TEXT DEFAULT '[]',
            connections                  TEXT DEFAULT '[]',
            source                       TEXT DEFAULT '',
            enriched                     INTEGER DEFAULT 0
        );

        -- Per-note embedding blobs (~1 KB each, f32 array bytes)
        CREATE TABLE IF NOT EXISTS note_embeddings (
            note_id   INTEGER PRIMARY KEY,
            embedding BLOB NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        -- FTS5 content table — mirrors notes columns used for search
        CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
            raw_text, summary, entities, topics,
            content=notes, content_rowid=id
        );

        -- FTS5 sync trigger: INSERT
        CREATE TRIGGER IF NOT EXISTS notes_fts_insert
        AFTER INSERT ON notes BEGIN
            INSERT INTO notes_fts(rowid, raw_text, summary, entities, topics)
            VALUES (new.id, new.raw_text, new.summary, new.entities, new.topics);
        END;

        -- FTS5 sync trigger: DELETE (must remove old row from FTS index)
        CREATE TRIGGER IF NOT EXISTS notes_fts_delete
        AFTER DELETE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, raw_text, summary, entities, topics)
            VALUES ('delete', old.id, old.raw_text, old.summary, old.entities, old.topics);
        END;

        -- FTS5 sync trigger: UPDATE (delete old then insert new)
        CREATE TRIGGER IF NOT EXISTS notes_fts_update
        AFTER UPDATE ON notes BEGIN
            INSERT INTO notes_fts(notes_fts, rowid, raw_text, summary, entities, topics)
            VALUES ('delete', old.id, old.raw_text, old.summary, old.entities, old.topics);
            INSERT INTO notes_fts(rowid, raw_text, summary, entities, topics)
            VALUES (new.id, new.raw_text, new.summary, new.entities, new.topics);
        END;

        -- Audit trail for score decay events
        CREATE TABLE IF NOT EXISTS decay_history (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            note_id    INTEGER NOT NULL,
            old_score  REAL NOT NULL,
            new_score  REAL NOT NULL,
            layer      TEXT NOT NULL,
            reason     TEXT,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
        );

        -- LLM-generated consolidation insights across multiple notes
        CREATE TABLE IF NOT EXISTS consolidation_insights (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            note_ids     TEXT NOT NULL,
            insight_text TEXT NOT NULL,
            created_at   TEXT NOT NULL
        );

        -- File inbox watcher: tracks already-processed file paths
        CREATE TABLE IF NOT EXISTS processed_files (
            path         TEXT PRIMARY KEY,
            processed_at TEXT NOT NULL
        );

        -- Application settings as key-value pairs
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        -- Composite index: layer + score for memory layer queries (active notes)
        CREATE INDEX IF NOT EXISTS idx_notes_layer_score
            ON notes(layer, current_score DESC) WHERE archived = 0;

        -- Partial index: quick lookup of notes pending LLM enrichment
        CREATE INDEX IF NOT EXISTS idx_notes_enriched
            ON notes(enriched) WHERE enriched = 0;

        -- Composite index: layer + last_updated for decay scheduling
        CREATE INDEX IF NOT EXISTS idx_notes_layer_lastupdate
            ON notes(layer, last_updated_at) WHERE archived = 0;
    ")
}
