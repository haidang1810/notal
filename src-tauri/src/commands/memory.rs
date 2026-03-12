// Memory stats Tauri command — counts per layer, unenriched count

use tauri::State;
use crate::db::{DbState, MemoryStats, connection as db};

/// Return aggregate memory statistics for the dashboard.
#[tauri::command]
pub async fn get_memory_stats(
    db: State<'_, DbState>,
) -> Result<MemoryStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::get_memory_stats(&conn)
        .map_err(|e| format!("get_memory_stats failed: {e}"))
}
