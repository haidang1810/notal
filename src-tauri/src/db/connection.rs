use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params};
use serde::Serialize;
use chrono::Utc;
use crate::models::note::Note;
use crate::models::memory_layer::MemoryLayer;
use crate::db::migrations::run_migrations;

/// Thread-safe SQLite connection managed as Tauri state.
/// Uses Arc<Mutex<>> so it can be cloned for background workers.
#[derive(Clone)]
pub struct DbState {
    pub conn: Arc<Mutex<Connection>>,
}

/// Aggregate stats for the memory dashboard.
#[derive(Debug, Clone, Serialize)]
pub struct MemoryStats {
    pub total: i64,
    pub working: i64,
    pub episodic: i64,
    pub semantic: i64,
    pub unenriched: i64,
}

/// Opens (or creates) the SQLite database, enables WAL + foreign keys, runs migrations.
pub fn init_db(db_path: &std::path::Path) -> Result<DbState, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open DB: {e}"))?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("PRAGMA setup failed: {e}"))?;

    run_migrations(&conn)
        .map_err(|e| format!("Migration failed: {e}"))?;

    Ok(DbState { conn: Arc::new(Mutex::new(conn)) })
}

// ─── helpers ────────────────────────────────────────────────────────────────

/// Deserialises a raw DB row into a Note, parsing JSON columns.
fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let entities_json: String = row.get(14)?;
    let topics_json: String   = row.get(15)?;
    let connections_json: String = row.get(16)?;

    let entities: Vec<String> = serde_json::from_str(&entities_json).unwrap_or_default();
    let topics: Vec<String>   = serde_json::from_str(&topics_json).unwrap_or_default();
    let connections: Vec<i64> = serde_json::from_str(&connections_json).unwrap_or_default();

    Ok(Note {
        id:                          row.get(0)?,
        raw_text:                    row.get(1)?,
        summary:                     row.get(2)?,
        importance:                  row.get(3)?,
        current_score:               row.get(4)?,
        layer:                       row.get(5)?,
        pinned:                      row.get::<_, i32>(6)? != 0,
        archived:                    row.get::<_, i32>(7)? != 0,
        created_at:                  row.get(8)?,
        last_accessed_at:            row.get(9)?,
        last_updated_at:             row.get(10)?,
        layer_promoted_at:           row.get(11)?,
        access_count:                row.get(12)?,
        access_count_since_promotion: row.get(13)?,
        entities,
        topics,
        connections,
        source:                      row.get(17)?,
        enriched:                    row.get::<_, i32>(18)? != 0,
    })
}

const NOTE_SELECT: &str = "
    SELECT id, raw_text, summary, importance, current_score, layer,
           pinned, archived, created_at, last_accessed_at, last_updated_at,
           layer_promoted_at, access_count, access_count_since_promotion,
           entities, topics, connections, source, enriched
    FROM notes
";

// ─── CRUD ───────────────────────────────────────────────────────────────────

/// Inserts a new note and returns its rowid.
pub fn insert_note(conn: &Connection, raw_text: &str, source: &str) -> rusqlite::Result<i64> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO notes (raw_text, source, created_at, last_updated_at)
         VALUES (?1, ?2, ?3, ?3)",
        params![raw_text, source, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Fetches a single note by id.
pub fn get_note(conn: &Connection, id: i64) -> rusqlite::Result<Note> {
    conn.query_row(
        &format!("{NOTE_SELECT} WHERE id = ?1"),
        params![id],
        row_to_note,
    )
}

/// Fetches all notes, optionally including archived ones.
pub fn get_all_notes(conn: &Connection, include_archived: bool) -> rusqlite::Result<Vec<Note>> {
    let sql = if include_archived {
        format!("{NOTE_SELECT} ORDER BY created_at DESC")
    } else {
        format!("{NOTE_SELECT} WHERE archived = 0 ORDER BY created_at DESC")
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect()
}

/// Updates enrichment fields on an existing note (summary, entities, topics, importance).
pub fn update_note_enrichment(
    conn: &Connection,
    id: i64,
    summary: &str,
    entities: &[String],
    topics: &[String],
    importance: f64,
) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    let entities_json = serde_json::to_string(entities).unwrap_or_else(|_| "[]".into());
    let topics_json   = serde_json::to_string(topics).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "UPDATE notes SET summary=?1, entities=?2, topics=?3, importance=?4,
                          enriched=1, last_updated_at=?5
         WHERE id=?6",
        params![summary, entities_json, topics_json, importance, now, id],
    )?;
    Ok(())
}

/// Updates mutable user-facing fields (raw_text, pinned, archived).
pub fn update_note_fields(
    conn: &Connection,
    id: i64,
    raw_text: Option<&str>,
    pinned: Option<bool>,
    archived: Option<bool>,
) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    if let Some(text) = raw_text {
        conn.execute(
            "UPDATE notes SET raw_text=?1, last_updated_at=?2, enriched=0 WHERE id=?3",
            params![text, now, id],
        )?;
    }
    if let Some(p) = pinned {
        conn.execute(
            "UPDATE notes SET pinned=?1, last_updated_at=?2 WHERE id=?3",
            params![p as i32, now, id],
        )?;
    }
    if let Some(a) = archived {
        conn.execute(
            "UPDATE notes SET archived=?1, last_updated_at=?2 WHERE id=?3",
            params![a as i32, now, id],
        )?;
    }
    Ok(())
}

/// Increments access_count and last_accessed_at for a note.
pub fn increment_access_count(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE notes SET access_count = access_count + 1,
                          access_count_since_promotion = access_count_since_promotion + 1,
                          last_accessed_at = ?1
         WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

/// Soft-deletes (archives) a note — hard delete if permanent = true.
pub fn delete_note(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

/// Sanitize user input for FTS5 MATCH — escapes special FTS5 syntax characters.
/// Wraps each token in double quotes to prevent query injection.
fn sanitize_fts_query(raw: &str) -> String {
    let tokens: Vec<String> = raw.split_whitespace()
        .map(|token| {
            let clean: String = token.chars()
                .filter(|c| !matches!(c, '"' | '*' | '^' | '(' | ')' | '{' | '}' | ':'))
                .collect();
            if clean.is_empty() {
                String::new()
            } else {
                format!("\"{}\"", clean)
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    // Use OR logic so any matching word returns results (not all required)
    tokens.join(" OR ")
}

/// Full-text search using FTS5; returns up to `limit` matching notes.
/// User input is sanitized to prevent FTS5 query injection.
pub fn search_fts(conn: &Connection, query: &str, limit: usize) -> rusqlite::Result<Vec<Note>> {
    let sanitized = sanitize_fts_query(query);
    if sanitized.is_empty() {
        return Ok(vec![]);
    }
    let sql = format!(
        "{NOTE_SELECT} WHERE id IN (
             SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?1
         ) AND archived = 0
         ORDER BY current_score DESC LIMIT ?2"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![sanitized, limit as i64], row_to_note)?;
    rows.collect()
}

/// Returns all non-archived notes in a specific memory layer.
pub fn get_notes_by_layer(conn: &Connection, layer: &MemoryLayer) -> rusqlite::Result<Vec<Note>> {
    let sql = format!("{NOTE_SELECT} WHERE layer = ?1 AND archived = 0 ORDER BY current_score DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![layer.to_string()], row_to_note)?;
    rows.collect()
}

/// Returns notes that have not yet been enriched by the LLM pipeline.
pub fn get_unenriched_notes(conn: &Connection) -> rusqlite::Result<Vec<Note>> {
    let sql = format!("{NOTE_SELECT} WHERE enriched = 0 AND archived = 0 ORDER BY created_at ASC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect()
}

// ─── Settings helpers ────────────────────────────────────────────────────────

/// Reads a single settings value by key. Returns None if the key doesn't exist.
pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |r| r.get(0),
    )
    .ok()
}

/// Inserts or replaces a single settings key-value pair.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

/// Returns aggregate memory statistics for the dashboard.
pub fn get_memory_stats(conn: &Connection) -> rusqlite::Result<MemoryStats> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE archived = 0", [], |r| r.get(0))?;
    let working: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE layer='working'  AND archived=0", [], |r| r.get(0))?;
    let episodic: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE layer='episodic' AND archived=0", [], |r| r.get(0))?;
    let semantic: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE layer='semantic' AND archived=0", [], |r| r.get(0))?;
    let unenriched: i64 = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE enriched=0 AND archived=0", [], |r| r.get(0))?;

    Ok(MemoryStats { total, working, episodic, semantic, unenriched })
}
