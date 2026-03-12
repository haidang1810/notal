// Layer promotion and demotion logic for multi-layer memory

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;

/// Thresholds that determine when notes move between memory layers.
pub struct PromotionThresholds {
    /// Working -> Episodic: access_count reaches this value
    pub working_to_episodic_access: i64,
    /// Working -> Episodic: importance reaches this value
    pub working_to_episodic_importance: f64,
    /// Episodic -> Semantic: access_count_since_promotion reaches this value
    pub episodic_to_semantic_access: i64,
    /// Episodic -> Semantic: importance reaches this value
    pub episodic_to_semantic_importance: f64,
    /// Episodic -> Working (demotion): score drops below this
    pub episodic_demotion_score: f64,
    /// Semantic -> Episodic (demotion): score drops below this
    pub semantic_demotion_score: f64,
}

impl Default for PromotionThresholds {
    fn default() -> Self {
        Self {
            working_to_episodic_access: 3,
            working_to_episodic_importance: 0.7,
            episodic_to_semantic_access: 5,
            episodic_to_semantic_importance: 0.9,
            episodic_demotion_score: 0.2,
            semantic_demotion_score: 0.1,
        }
    }
}

/// Summary of promotion/demotion activity in a single consolidation cycle.
#[derive(Debug, Clone, Serialize)]
pub struct PromotionReport {
    pub promoted_count: u64,
    pub demoted_count: u64,
}

/// Row data fetched for promotion evaluation.
struct NoteRow {
    id: i64,
    layer: String,
    importance: f64,
    current_score: f64,
    access_count: i64,
    access_count_since_promotion: i64,
}

/// Fetch all active, non-pinned notes with fields needed for layer evaluation.
fn fetch_active_notes(conn: &Connection) -> rusqlite::Result<Vec<NoteRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, layer, importance, current_score,
                access_count, access_count_since_promotion
         FROM notes
         WHERE archived = 0 AND pinned = 0",
    )?;

    let rows = stmt
        .query_map([], |row| {
            Ok(NoteRow {
                id: row.get(0)?,
                layer: row.get(1)?,
                importance: row.get(2)?,
                current_score: row.get(3)?,
                access_count: row.get(4)?,
                access_count_since_promotion: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Apply layer change: update layer, adjust score with bonus, reset promotion counter.
fn apply_layer_change(
    conn: &Connection,
    id: i64,
    new_layer: &str,
    old_layer: &str,
    old_score: f64,
    score_delta: f64,
    reason: &str,
    now: &str,
) -> rusqlite::Result<()> {
    let new_score = (old_score + score_delta).clamp(0.0, 1.0);

    conn.execute(
        "UPDATE notes
         SET layer = ?1,
             current_score = ?2,
             access_count_since_promotion = 0,
             layer_promoted_at = ?3,
             last_updated_at = ?3
         WHERE id = ?4",
        params![new_layer, new_score, now, id],
    )?;

    conn.execute(
        "INSERT INTO decay_history
             (note_id, old_score, new_score, layer, reason, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, old_score, new_score, old_layer, reason, now],
    )?;

    Ok(())
}

/// Evaluate and apply all promotions and demotions for active notes.
///
/// Promotion rules:
///   Working  -> Episodic : access_count >= 3 OR importance >= 0.7  (+0.10 bonus)
///   Episodic -> Semantic  : access_count_since_promotion >= 5 OR importance >= 0.9  (+0.15 bonus)
///
/// Demotion rules:
///   Episodic -> Working   : current_score < 0.2
///   Semantic -> Episodic  : current_score < 0.1
///
/// All transitions are logged to decay_history with the given reason string.
pub fn check_promotions(
    conn: &Connection,
    thresholds: &PromotionThresholds,
) -> rusqlite::Result<PromotionReport> {
    let notes = fetch_active_notes(conn)?;
    let now = Utc::now().to_rfc3339();
    let mut promoted: u64 = 0;
    let mut demoted: u64 = 0;

    for note in notes {
        match note.layer.as_str() {
            "working" => {
                let should_promote =
                    note.access_count >= thresholds.working_to_episodic_access
                        || note.importance >= thresholds.working_to_episodic_importance;

                if should_promote {
                    apply_layer_change(
                        conn,
                        note.id,
                        "episodic",
                        &note.layer,
                        note.current_score,
                        0.10,
                        "promotion",
                        &now,
                    )?;
                    promoted += 1;
                }
            }
            "episodic" => {
                // Check demotion first
                if note.current_score < thresholds.episodic_demotion_score {
                    apply_layer_change(
                        conn,
                        note.id,
                        "working",
                        &note.layer,
                        note.current_score,
                        0.0,
                        "demotion",
                        &now,
                    )?;
                    demoted += 1;
                } else {
                    // Check promotion to semantic
                    let should_promote =
                        note.access_count_since_promotion
                            >= thresholds.episodic_to_semantic_access
                            || note.importance >= thresholds.episodic_to_semantic_importance;

                    if should_promote {
                        apply_layer_change(
                            conn,
                            note.id,
                            "semantic",
                            &note.layer,
                            note.current_score,
                            0.15,
                            "promotion",
                            &now,
                        )?;
                        promoted += 1;
                    }
                }
            }
            "semantic" => {
                if note.current_score < thresholds.semantic_demotion_score {
                    apply_layer_change(
                        conn,
                        note.id,
                        "episodic",
                        &note.layer,
                        note.current_score,
                        0.0,
                        "demotion",
                        &now,
                    )?;
                    demoted += 1;
                }
            }
            _ => {} // Unknown layer — skip
        }
    }

    Ok(PromotionReport { promoted_count: promoted, demoted_count: demoted })
}
