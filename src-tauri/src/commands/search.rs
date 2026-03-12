// FTS5 keyword search + ask-AI RAG command

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::db::{DbState, connection as db};
use crate::models::note::Note;
use crate::LlmState;
use crate::llm::provider::LLMProvider;
use crate::llm::types::CompletionRequest;

/// A note with an attached relevance score from the search pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub note: Note,
    pub relevance_score: f64,
}

/// Response from the ask-AI RAG command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskResponse {
    pub answer: String,
    /// IDs of notes cited in the answer (parsed from [#ID] markers).
    pub citations: Vec<i64>,
}

// ─── helpers ────────────────────────────────────────────────────────────────

/// Parse citation IDs from LLM response text (pattern: [#123]).
/// Uses manual scanning to avoid the `regex` dependency.
fn parse_citations(text: &str) -> Vec<i64> {
    let mut ids = std::collections::HashSet::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i + 2 < len {
        // Look for '[' '#' then digits then ']'
        if bytes[i] == b'[' && bytes[i + 1] == b'#' {
            let start = i + 2;
            let mut j = start;
            while j < len && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j > start && j < len && bytes[j] == b']' {
                if let Ok(id) = text[start..j].parse::<i64>() {
                    ids.insert(id);
                }
            }
        }
        i += 1;
    }
    ids.into_iter().collect()
}

/// Truncate text to `max_chars`, appending "…" if trimmed.
fn truncate(text: &str, max_chars: usize) -> &str {
    if text.len() <= max_chars {
        text
    } else {
        // Find a char boundary at or before max_chars
        let mut end = max_chars;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    }
}

// ─── commands ───────────────────────────────────────────────────────────────

/// FTS5 keyword search — returns notes ranked by current_score with relevance scores.
#[tauri::command]
pub async fn search_notes(
    db: State<'_, DbState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(20);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let notes = db::search_fts(&conn, &query, limit)
        .map_err(|e| format!("search_fts failed: {e}"))?;

    // Assign relevance score: blend FTS rank (position) with current_score
    let total = notes.len() as f64;
    let results: Vec<SearchResult> = notes
        .into_iter()
        .enumerate()
        .map(|(i, note)| {
            // Reciprocal rank component (0..1)
            let rank_score = 1.0 / (i as f64 + 1.0);
            let relevance_score = (rank_score + note.current_score).min(1.0);
            SearchResult { note, relevance_score }
        })
        .collect();

    let _ = total;

    // Access boost: increment access_count + last_accessed_at for every returned note
    let result_ids: Vec<i64> = results.iter().map(|r| r.note.id).collect();
    for note_id in result_ids {
        db::increment_access_count(&conn, note_id).ok();
    }

    Ok(results)
}

/// RAG: search relevant notes, build context, ask LLM, return answer + citations.
#[tauri::command]
pub async fn ask_ai(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    question: String,
) -> Result<AskResponse, String> {
    // 1. Find relevant notes via FTS (top 10)
    let notes = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        db::search_fts(&conn, &question, 10)
            .map_err(|e| format!("search_fts failed: {e}"))?
    };

    if notes.is_empty() {
        return Ok(AskResponse {
            answer: "No relevant notes found for your question.".into(),
            citations: vec![],
        });
    }

    // 2. Build context block — always use raw_text (full content) for detailed answers
    //    Include summary as a header when available, but raw_text is the primary content.
    let today = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    let context = notes
        .iter()
        .map(|n| {
            let content = truncate(&n.raw_text, 3000);
            let date = &n.created_at;
            if n.summary.is_empty() {
                format!("[#{id}] (created: {date})\n{content}", id = n.id)
            } else {
                format!("[#{id}] (created: {date}, summary: {summary})\n{content}", id = n.id, summary = n.summary)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // 3. Call LLM
    let system_prompt = format!(
        "You are a personal knowledge assistant. Today is {today}. \
        Answer based ONLY on the provided notes. \
        IMPORTANT: Resolve relative dates in notes (like 'tomorrow', 'ngày mai') \
        relative to each note's creation date, NOT today. \
        Cite each fact with [#ID] where ID is the note number. \
        If the notes don't contain the answer, say so."
    );

    let user_message = format!(
        "Notes:\n{context}\n\nQuestion: {question}"
    );

    let request = CompletionRequest {
        system_prompt: Some(system_prompt),
        user_message,
        images: vec![],
        temperature: 0.3,
        max_tokens: Some(1024),
    };

    let manager = llm.lock().await;
    let response = manager
        .generate_completion(request)
        .await
        .map_err(|e| format!("LLM error: {e}"))?;

    let answer = response.text;

    // 4. Parse citation IDs, update access counts, and boost cited note scores
    let citations = parse_citations(&answer);
    if !citations.is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        for &cited_id in &citations {
            db::increment_access_count(&conn, cited_id).ok();
            // Boost current_score by +0.1 for cited notes (capped at 1.0)
            conn.execute(
                "UPDATE notes SET current_score = MIN(1.0, current_score + 0.1) WHERE id = ?1",
                rusqlite::params![cited_id],
            )
            .ok();
        }
    }

    Ok(AskResponse { answer, citations })
}
