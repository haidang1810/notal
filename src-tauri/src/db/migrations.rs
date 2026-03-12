use rusqlite::Connection;
use crate::db::schema::create_tables;

const CURRENT_VERSION: u32 = 1;

/// Runs pending migrations using PRAGMA user_version as the schema version tracker.
/// Each migration is wrapped in a transaction for atomicity.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let version: u32 = conn.query_row(
        "PRAGMA user_version",
        [],
        |row| row.get(0),
    )?;

    if version < 1 {
        migrate_v0_to_v1(conn)?;
    }

    // Future migrations: if version < 2 { migrate_v1_to_v2(conn)?; }

    Ok(())
}

/// Migration 0 → 1: create initial schema and insert default settings row.
fn migrate_v0_to_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("BEGIN;")?;

    create_tables(conn)?;

    // Seed default settings (JSON-serialized AppSettings defaults)
    conn.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![
            "app_settings",
            r#"{"llm_provider":"auto","ollama_endpoint":"http://localhost:11434","gemini_api_key":"","decay_rate_working":0.1,"decay_rate_episodic":0.05,"decay_rate_semantic":0.01,"consolidation_interval_minutes":60,"inbox_folder_path":"","hotkey_capture":"CmdOrCtrl+Shift+N","hotkey_open":"CmdOrCtrl+Shift+R"}"#
        ],
    )?;

    // Bump schema version — must use execute_batch as PRAGMA cannot be parameterised
    conn.execute_batch(&format!("PRAGMA user_version = {};", CURRENT_VERSION))?;
    conn.execute_batch("COMMIT;")?;

    Ok(())
}
