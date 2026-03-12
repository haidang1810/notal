// Exponential decay calculation for multi-layer memory scores

use chrono::Utc;
use rusqlite::{params, Connection};

/// Decay rates per memory layer (fraction lost per day).
pub struct DecayConfig {
    pub working_rate: f64,  // 0.15 default — high churn
    pub episodic_rate: f64, // 0.05 default — medium retention
    pub semantic_rate: f64, // 0.01 default — near-permanent
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            working_rate: 0.15,
            episodic_rate: 0.05,
            semantic_rate: 0.01,
        }
    }
}

/// Exponential decay: importance * e^(-rate * days_elapsed).
/// Returns score in [0.0, 1.0].
pub fn calculate_decay(importance: f64, rate: f64, days_elapsed: f64) -> f64 {
    (importance * (-rate * days_elapsed).exp()).clamp(0.0, 1.0)
}

/// Returns decay rate for the given layer string.
fn rate_for_layer(layer: &str, config: &DecayConfig) -> f64 {
    match layer {
        "episodic" => config.episodic_rate,
        "semantic" => config.semantic_rate,
        _ => config.working_rate, // "working" or unknown
    }
}

/// Parse an ISO-8601 timestamp string and return days elapsed since then.
/// Falls back to 0 on parse failure (no decay applied).
fn days_since(ts: &str) -> f64 {
    match chrono::DateTime::parse_from_rfc3339(ts) {
        Ok(dt) => {
            let now = Utc::now();
            let delta = now.signed_duration_since(dt.with_timezone(&Utc));
            // Convert to fractional days; negative if ts is in the future
            (delta.num_seconds() as f64 / 86_400.0).max(0.0)
        }
        Err(_) => 0.0,
    }
}

/// Apply exponential decay to all active, non-pinned notes.
///
/// For each note:
/// 1. Compute days elapsed since `created_at` (stable reference — NOT `last_updated_at`
///    which would reset the decay timer every consolidation cycle).
/// 2. Calculate new score via `calculate_decay`.
/// 3. UPDATE notes.current_score only (do NOT touch `last_updated_at`).
/// 4. INSERT a row into decay_history.
///
/// Returns the number of notes updated.
pub fn apply_decay(conn: &Connection, config: &DecayConfig) -> rusqlite::Result<u64> {
    // Use created_at as the stable decay reference to avoid feedback loops.
    // last_accessed_at boosts are handled separately (increment_access_count).
    let mut stmt = conn.prepare(
        "SELECT id, importance, current_score, layer, created_at
         FROM notes
         WHERE archived = 0 AND pinned = 0",
    )?;

    // Collect into Vec to avoid holding the borrow across write operations
    let rows: Vec<(i64, f64, f64, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let now = Utc::now().to_rfc3339();
    let mut updated: u64 = 0;

    for (id, importance, old_score, layer, ts) in rows {
        let days = days_since(&ts);
        let rate = rate_for_layer(&layer, config);
        let new_score = calculate_decay(importance, rate, days);

        // Skip trivial updates (< 0.001 change) to reduce write load
        if (new_score - old_score).abs() < 0.001 {
            continue;
        }

        // Only update current_score — do NOT reset last_updated_at
        conn.execute(
            "UPDATE notes SET current_score = ?1 WHERE id = ?2",
            params![new_score, id],
        )?;

        conn.execute(
            "INSERT INTO decay_history (note_id, old_score, new_score, layer, reason, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'decay', ?5)",
            params![id, old_score, new_score, layer, now],
        )?;

        updated += 1;
    }

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_formula_reduces_score_over_time() {
        let score = calculate_decay(1.0, 0.15, 1.0); // 1 day working
        assert!(score < 1.0 && score > 0.8, "score={score}");
    }

    #[test]
    fn decay_clamped_to_zero() {
        let score = calculate_decay(0.01, 0.15, 1000.0);
        // After extreme decay, score should be effectively zero (within floating-point precision)
        assert!(score < 1e-60, "score should be effectively zero after extreme decay, got {}", score);
    }

    #[test]
    fn zero_days_no_change() {
        let score = calculate_decay(0.5, 0.15, 0.0);
        assert!((score - 0.5).abs() < 1e-9);
    }
}
