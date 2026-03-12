// Periodic consolidation orchestrator: decay → promote → connect → insights → cleanup

use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::Serialize;
use tauri::AppHandle;
use tauri::Emitter;

use crate::db::DbState;
use crate::llm::provider::LLMProvider;
use crate::llm::types::CompletionRequest;
use crate::LlmState;

use super::decay::{apply_decay, DecayConfig};
use super::promotion::{check_promotions, PromotionReport, PromotionThresholds};

// ─── config ──────────────────────────────────────────────────────────────────

pub struct ConsolidationConfig {
    /// How often to run the background loop (minutes).
    pub interval_minutes: u64,
    /// Archive notes with current_score below this threshold.
    pub cleanup_min_score: f64,
    /// Only archive notes older than this many days.
    pub cleanup_min_age_days: i64,
    /// Maximum number of note clusters to generate insights for per run.
    pub max_insight_clusters: usize,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            interval_minutes: 30,
            cleanup_min_score: 0.05,
            cleanup_min_age_days: 30,
            max_insight_clusters: 5,
        }
    }
}

// ─── report ──────────────────────────────────────────────────────────────────

/// Full summary emitted to the frontend after each consolidation run.
#[derive(Debug, Clone, Serialize)]
pub struct ConsolidationReport {
    pub decay_count: u64,
    pub promoted_count: u64,
    pub demoted_count: u64,
    pub connections_found: usize,
    pub insights_generated: usize,
    pub archived_count: u64,
}

// ─── connection detection ─────────────────────────────────────────────────────

/// Fetch (id, entities_json) for all active enriched notes with non-empty entities.
fn fetch_note_entities(conn: &Connection) -> rusqlite::Result<Vec<(i64, Vec<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT id, entities FROM notes
         WHERE archived = 0 AND entities != '[]' AND enriched = 1",
    )?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let entities_json: String = row.get(1)?;
            Ok((id, entities_json))
        })?
        .filter_map(|r| r.ok())
        .map(|(id, json)| {
            let entities: Vec<String> =
                serde_json::from_str(&json).unwrap_or_default();
            (id, entities)
        })
        .collect();

    Ok(rows)
}

/// Pairwise entity overlap: returns (note_id_a, note_id_b, shared_entity) for
/// pairs sharing at least 2 entities. Updates both notes' connections JSON.
pub fn find_connections(conn: &Connection) -> rusqlite::Result<Vec<(i64, i64, String)>> {
    let notes = fetch_note_entities(conn)?;
    let mut new_connections: Vec<(i64, i64, String)> = Vec::new();

    // Build entity-lowercase lookup per note for O(n) inner comparison
    let lower_notes: Vec<(i64, Vec<String>)> = notes
        .iter()
        .map(|(id, ents)| (*id, ents.iter().map(|e| e.to_lowercase()).collect()))
        .collect();

    for i in 0..lower_notes.len() {
        for j in (i + 1)..lower_notes.len() {
            let (id_a, ents_a) = &lower_notes[i];
            let (id_b, ents_b) = &lower_notes[j];

            let shared: Vec<&String> = ents_a
                .iter()
                .filter(|e| ents_b.contains(e))
                .collect();

            if shared.len() >= 2 {
                new_connections.push((*id_a, *id_b, shared[0].clone()));
            }
        }
    }

    if new_connections.is_empty() {
        return Ok(new_connections);
    }

    // Merge discovered connections into notes.connections JSON
    // Build a map: note_id -> set of connected note ids
    let mut conn_map: HashMap<i64, Vec<i64>> = HashMap::new();
    for &(a, b, _) in &new_connections {
        conn_map.entry(a).or_default().push(b);
        conn_map.entry(b).or_default().push(a);
    }

    let now = Utc::now().to_rfc3339();
    for (note_id, new_ids) in &conn_map {
        // Fetch existing connections
        let existing_json: String = conn
            .query_row(
                "SELECT connections FROM notes WHERE id = ?1",
                params![note_id],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "[]".into());

        let mut existing: Vec<i64> =
            serde_json::from_str(&existing_json).unwrap_or_default();

        for &nid in new_ids {
            if !existing.contains(&nid) {
                existing.push(nid);
            }
        }

        let updated_json = serde_json::to_string(&existing).unwrap_or_else(|_| "[]".into());
        conn.execute(
            "UPDATE notes SET connections = ?1, last_updated_at = ?2 WHERE id = ?3",
            params![updated_json, now, note_id],
        )?;
    }

    Ok(new_connections)
}

// ─── insight generation ───────────────────────────────────────────────────────

/// Simple union-find to group connected note IDs into clusters.
fn build_clusters(connections: &[(i64, i64, String)]) -> Vec<Vec<i64>> {
    let mut parent: HashMap<i64, i64> = HashMap::new();

    fn find(parent: &mut HashMap<i64, i64>, x: i64) -> i64 {
        let p = *parent.entry(x).or_insert(x);
        if p == x {
            return x;
        }
        let root = find(parent, p);
        parent.insert(x, root);
        root
    }

    fn union(parent: &mut HashMap<i64, i64>, a: i64, b: i64) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent.insert(ra, rb);
        }
    }

    for &(a, b, _) in connections {
        union(&mut parent, a, b);
    }

    // Group by root — collect keys first to avoid simultaneous borrow of `parent`
    let mut groups: HashMap<i64, Vec<i64>> = HashMap::new();
    let ids: Vec<i64> = parent.keys().copied().collect();
    for id in ids {
        let root = find(&mut parent, id);
        groups.entry(root).or_default().push(id);
    }

    groups.into_values().filter(|g| g.len() >= 3).collect()
}

/// For clusters of 3+ notes, ask LLM to generate a cross-note insight.
/// Stores results in consolidation_insights. Returns insight strings.
pub async fn generate_insights(
    db: &DbState,
    llm: &LlmState,
    connections: &[(i64, i64, String)],
    max_clusters: usize,
) -> Result<Vec<String>, String> {
    let clusters = build_clusters(connections);
    let mut insights = Vec::new();

    for cluster in clusters.iter().take(max_clusters) {
        // Gather summaries from DB
        let summaries: Vec<String> = {
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            cluster
                .iter()
                .filter_map(|&id| {
                    conn.query_row(
                        "SELECT COALESCE(NULLIF(summary,''), raw_text) FROM notes WHERE id=?1",
                        params![id],
                        |r| r.get::<_, String>(0),
                    )
                    .ok()
                })
                .map(|s| {
                    // Truncate each summary to keep prompt manageable
                    s.chars().take(300).collect::<String>()
                })
                .collect()
        };

        if summaries.is_empty() {
            continue;
        }

        let prompt = format!(
            "These notes share common themes:\n{}\n\nIn one sentence, what key insight connects them?",
            summaries
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let request = CompletionRequest {
            system_prompt: Some(
                "You are a knowledge synthesiser. Be concise and insightful.".into(),
            ),
            user_message: prompt,
            images: vec![],
            temperature: 0.4,
            max_tokens: Some(200),
        };

        let manager = llm.lock().await;
        match manager.generate_completion(request).await {
            Ok(resp) => {
                let insight = resp.text.trim().to_string();
                drop(manager);

                // Persist insight
                let note_ids_json =
                    serde_json::to_string(cluster).unwrap_or_else(|_| "[]".into());
                let now = Utc::now().to_rfc3339();
                {
                    let conn = db.conn.lock().map_err(|e| e.to_string())?;
                    conn.execute(
                        "INSERT INTO consolidation_insights (note_ids, insight_text, created_at)
                         VALUES (?1, ?2, ?3)",
                        params![note_ids_json, insight, now],
                    )
                    .map_err(|e| e.to_string())?;
                }

                insights.push(insight);
            }
            Err(e) => {
                log::warn!("[consolidation] insight LLM call failed: {e}");
            }
        }
    }

    Ok(insights)
}

// ─── cleanup ──────────────────────────────────────────────────────────────────

/// Archive notes whose score is below threshold and are older than min_age_days.
/// Pinned and already-archived notes are excluded.
pub fn cleanup(conn: &Connection, config: &ConsolidationConfig) -> rusqlite::Result<u64> {
    let now = Utc::now().to_rfc3339();
    let rows_changed = conn.execute(
        "UPDATE notes
         SET archived = 1, last_updated_at = ?1
         WHERE current_score < ?2
           AND julianday('now') - julianday(created_at) > ?3
           AND pinned = 0
           AND archived = 0",
        params![now, config.cleanup_min_score, config.cleanup_min_age_days],
    )?;
    Ok(rows_changed as u64)
}

// ─── orchestration ────────────────────────────────────────────────────────────

/// Execute a full consolidation cycle:
/// decay → promotions → connections → insights (LLM, lock dropped) → cleanup.
pub async fn run_consolidation(
    db: &DbState,
    llm: &LlmState,
    decay_config: &DecayConfig,
    promo_thresholds: &PromotionThresholds,
    config: &ConsolidationConfig,
) -> Result<ConsolidationReport, String> {
    // Phase 1-3: CPU-only DB operations under lock
    let (decay_count, promo_report, connections) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let decay_count = apply_decay(&conn, decay_config).map_err(|e| e.to_string())?;

        let promo_report =
            check_promotions(&conn, promo_thresholds).map_err(|e| e.to_string())?;

        let connections = find_connections(&conn).map_err(|e| e.to_string())?;

        (decay_count, promo_report, connections)
    }; // Lock released before LLM call

    // Phase 4: LLM insight generation (no DB lock held)
    let insights =
        generate_insights(db, llm, &connections, config.max_insight_clusters).await?;

    // Phase 5: cleanup under lock
    let archived_count = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        cleanup(&conn, config).map_err(|e| e.to_string())?
    };

    let PromotionReport { promoted_count, demoted_count } = promo_report;

    Ok(ConsolidationReport {
        decay_count,
        promoted_count,
        demoted_count,
        connections_found: connections.len(),
        insights_generated: insights.len(),
        archived_count,
    })
}

/// Background worker: sleeps interval_minutes, then runs a full consolidation cycle.
/// Emits "consolidation_complete" event with the report on success.
pub async fn start_consolidation_worker(
    db: DbState,
    llm: LlmState,
    app: AppHandle,
    config: ConsolidationConfig,
) {
    loop {
        tokio::time::sleep(Duration::from_secs(config.interval_minutes * 60)).await;

        match run_consolidation(
            &db,
            &llm,
            &DecayConfig::default(),
            &PromotionThresholds::default(),
            &config,
        )
        .await
        {
            Ok(report) => {
                log::info!(
                    "[consolidation] decayed={} promoted={} demoted={} connections={} insights={} archived={}",
                    report.decay_count,
                    report.promoted_count,
                    report.demoted_count,
                    report.connections_found,
                    report.insights_generated,
                    report.archived_count,
                );
                app.emit("consolidation_complete", &report).ok();
            }
            Err(e) => {
                log::error!("[consolidation] cycle failed: {e}");
            }
        }
    }
}
