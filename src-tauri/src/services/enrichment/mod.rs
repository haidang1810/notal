// AI enrichment pipeline — background worker that enriches unenriched notes
// Split: rule-based boost helpers live in enrichment_rules.rs

mod rules;

use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde::Deserialize;

use crate::db::{DbState, connection as db};
use crate::llm::provider::LLMProvider;
use crate::llm::types::{LLMError, StructuredRequest};
use crate::models::note::Note;
use crate::LlmState;

/// Structured JSON the LLM must return for each note.
#[derive(Debug, Deserialize)]
struct EnrichmentOutput {
    summary: String,
    entities: Vec<String>,
    topics: Vec<String>,
    importance: f64,
}

/// JSON Schema sent to the LLM to constrain its response shape.
fn enrichment_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "summary":    { "type": "string" },
            "entities":   { "type": "array", "items": { "type": "string" } },
            "topics":     { "type": "array", "items": { "type": "string" } },
            "importance": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
        },
        "required": ["summary", "entities", "topics", "importance"]
    })
}

/// Enrich a single note: call LLM, apply rule boosts, update DB.
pub async fn enrich_note(
    db: &DbState,
    llm: &LlmState,
    note: &Note,
) -> Result<(), LLMError> {
    let prompt = format!(
        "Analyze this note and respond with JSON:\n\
         {{\n  \"summary\": \"1-2 sentence summary\",\n\
         \"entities\": [\"entity1\"],\n\
         \"topics\": [\"topic1\"],\n\
         \"importance\": 0.0\n}}\n\n\
         Note created at: {created}\n\
         Note content:\n{content}",
        created = note.created_at,
        content = note.raw_text
    );

    let request = StructuredRequest {
        system_prompt: Some(
            "You are a note analysis assistant. \
             IMPORTANT: Resolve ALL relative dates/times in the note \
             (like 'tomorrow', 'next week', 'today', 'hôm nay', 'ngày mai', 'tuần sau') \
             into absolute dates based on the note's creation date. \
             Include these resolved dates in the summary. \
             Return ONLY valid JSON matching the schema.".into(),
        ),
        user_message: prompt,
        images: vec![],
        response_schema: enrichment_schema(),
        temperature: 0.1,
    };

    let manager = llm.lock().await;
    let raw = manager.generate_structured(request).await?;
    drop(manager);

    // Parse LLM output — on failure use safe defaults
    let output: EnrichmentOutput = serde_json::from_value(raw).map_err(|e| {
        LLMError::ParseError(format!("enrichment JSON parse failed: {e}"))
    })?;

    // Apply rule-based importance boosts
    let boosted = rules::apply_importance_boosts(output.importance, &note.raw_text);

    // Persist to DB
    let conn = db.conn.lock().map_err(|e| {
        LLMError::InternalError(format!("DB lock failed: {e}"))
    })?;
    db::update_note_enrichment(
        &conn,
        note.id,
        &output.summary,
        &output.entities,
        &output.topics,
        boosted,
    )
    .map_err(|e| LLMError::InternalError(format!("DB update failed: {e}")))?;

    Ok(())
}

/// Background loop: poll for unenriched notes, enrich each, emit frontend events.
/// Emits "llm_status" = "online" | "offline" so the frontend can show a status indicator.
/// When LLM is unavailable, sleeps 30s before retrying — notes are saved normally in the meantime.
pub async fn start_enrichment_worker(db: DbState, llm: LlmState, app: AppHandle) {
    loop {
        // Check LLM availability before processing; emit status for frontend indicator.
        let is_online = {
            let manager = llm.lock().await;
            manager.is_available().await
        };

        if !is_online {
            log::info!("[enrichment] LLM unavailable — offline mode, retrying in 30s");
            app.emit("llm_status", "offline").ok();
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }

        app.emit("llm_status", "online").ok();

        let unenriched = {
            match db.conn.lock() {
                Ok(conn) => db::get_unenriched_notes(&conn).unwrap_or_default(),
                Err(e) => {
                    log::error!("[enrichment] DB lock failed: {e}");
                    vec![]
                }
            }
        };

        for note in unenriched {
            let note_id = note.id;
            match enrich_note(&db, &llm, &note).await {
                Ok(()) => {
                    log::info!("[enrichment] Enriched note #{note_id}");
                    app.emit("note_enriched", note_id).ok();
                }
                Err(e) => {
                    log::warn!("[enrichment] Failed for #{note_id}: {e}");
                }
            }
            // Small delay between notes to avoid overwhelming the LLM
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Poll every 5 seconds for new unenriched notes
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
