// Note CRUD Tauri commands — create, read, update, delete, file ingestion

use tauri::State;
use crate::db::{DbState, connection as db};
use crate::models::note::Note;
use crate::llm::provider::LLMProvider;
use crate::llm::types::{CompletionRequest, ImageData};
use crate::LlmState;

// ─── helpers ────────────────────────────────────────────────────────────────

/// Supported text-based file extensions for direct content ingestion.
const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "md", "json", "csv", "log", "xml", "yaml", "yml",
    "rs", "py", "js", "ts", "html", "css", "toml", "ini", "conf",
];

/// Chunk text into pieces of ~`max_len` chars with `overlap` char overlap at boundaries.
/// Splits at paragraph then sentence boundaries where possible.
/// Uses char indices to avoid panics on multi-byte UTF-8 content.
pub fn chunk_text(text: &str, max_len: usize, overlap: usize) -> Vec<String> {
    let char_count = text.chars().count();
    if char_count <= max_len {
        return vec![text.to_string()];
    }

    // Build char→byte-offset index for safe slicing
    let char_offsets: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    let byte_len = text.len();

    let mut chunks = Vec::new();
    let mut start_char = 0;
    while start_char < char_count {
        let end_char = (start_char + max_len).min(char_count);
        let start_byte = char_offsets[start_char];
        let end_byte = if end_char < char_count { char_offsets[end_char] } else { byte_len };
        let slice = &text[start_byte..end_byte];

        // Try to break at paragraph then sentence boundary
        let break_byte = slice.rfind("\n\n")
            .or_else(|| slice.rfind('\n'))
            .or_else(|| slice.rfind(". "))
            .map(|p| p + 1)
            .unwrap_or(slice.len());

        let chunk_end_byte = start_byte + break_byte;
        chunks.push(text[start_byte..chunk_end_byte].trim().to_string());

        // Determine char position of chunk_end for overlap calculation
        let chunk_end_char = char_offsets.partition_point(|&o| o < chunk_end_byte);
        start_char = if chunk_end_char > overlap { chunk_end_char - overlap } else { chunk_end_char };
        if start_char >= char_count { break; }
    }
    chunks.into_iter().filter(|c| !c.is_empty()).collect()
}

fn extension_of(path: &str) -> String {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn is_text_file(ext: &str) -> bool {
    TEXT_EXTENSIONS.contains(&ext)
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

// ─── commands ───────────────────────────────────────────────────────────────

/// Create a new note, store immediately (enrichment happens in background).
#[tauri::command]
pub async fn create_note(
    db: State<'_, DbState>,
    raw_text: String,
    source: Option<String>,
) -> Result<Note, String> {
    let source_str = source.unwrap_or_default();
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = db::insert_note(&conn, &raw_text, &source_str)
        .map_err(|e| format!("insert_note failed: {e}"))?;
    db::get_note(&conn, id)
        .map_err(|e| format!("get_note failed: {e}"))
}

/// Fetch notes, optionally filtered by memory layer and archive status.
#[tauri::command]
pub async fn get_notes(
    db: State<'_, DbState>,
    layer: Option<String>,
    include_archived: Option<bool>,
) -> Result<Vec<Note>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    if let Some(ref layer_str) = layer {
        let memory_layer = crate::models::memory_layer::MemoryLayer::from_str(layer_str);
        db::get_notes_by_layer(&conn, &memory_layer)
            .map_err(|e| format!("get_notes_by_layer failed: {e}"))
    } else {
        db::get_all_notes(&conn, include_archived.unwrap_or(false))
            .map_err(|e| format!("get_all_notes failed: {e}"))
    }
}

/// Fetch a single note by ID.
#[tauri::command]
pub async fn get_note_by_id(
    db: State<'_, DbState>,
    id: i64,
) -> Result<Note, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::get_note(&conn, id)
        .map_err(|e| format!("get_note failed: {e}"))
}

/// Update mutable note fields — only fields that are Some() are applied.
#[tauri::command]
pub async fn update_note(
    db: State<'_, DbState>,
    id: i64,
    raw_text: Option<String>,
    pinned: Option<bool>,
    archived: Option<bool>,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::update_note_fields(&conn, id, raw_text.as_deref(), pinned, archived)
        .map_err(|e| format!("update_note_fields failed: {e}"))
}

/// Hard-delete a note by ID.
#[tauri::command]
pub async fn delete_note(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    db::delete_note(&conn, id)
        .map_err(|e| format!("delete_note failed: {e}"))
}

/// Ingest a file path into one or more notes.
/// Text files: read content, chunk, create notes.
/// Media files (PDF, images, audio, video): read bytes, send to LLM multimodal
/// for thorough content analysis, save the LLM description as note raw_text.
#[tauri::command]
pub async fn ingest_file(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    file_path: String,
) -> Result<Vec<i64>, String> {
    let path = std::path::Path::new(&file_path);
    if !path.is_absolute() {
        return Err("file_path must be an absolute path".into());
    }

    let ext = extension_of(&file_path);
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    if is_text_file(&ext) {
        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Cannot read file: {e}"))?;
        let chunks = chunk_text(&content, 2000, 200);
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            let source = if chunks.len() > 1 {
                format!("{filename} (part {}/{})", i + 1, chunks.len())
            } else {
                filename.clone()
            };
            let id = db::insert_note(&conn, chunk, &source)
                .map_err(|e| format!("insert_note failed: {e}"))?;
            ids.push(id);
        }
        Ok(ids)
    } else {
        // Media file — read bytes, send to LLM multimodal for content analysis
        let file_bytes = std::fs::read(&file_path)
            .map_err(|e| format!("Cannot read file: {e}"))?;
        let mime = mime_for_ext(&ext).to_string();

        let prompt = format!(
            "Remember this file (source: {filename}, type: {mime}).\n\n\
             Thoroughly analyze the content of this file and provide:\n\
             1. A detailed description of ALL the content\n\
             2. Key information, facts, and data points\n\
             3. Any text, numbers, or structured data visible\n\
             Be comprehensive — this description will be the only record of this file's content."
        );

        let request = CompletionRequest {
            system_prompt: Some("You are a file analysis assistant. Extract and describe ALL content thoroughly.".into()),
            user_message: prompt,
            images: vec![ImageData { bytes: file_bytes, mime_type: mime }],
            temperature: 0.1,
            max_tokens: Some(4096),
        };

        let manager = llm.lock().await;
        let response = manager.generate_completion(request).await
            .map_err(|e| format!("LLM analysis failed: {e}"))?;
        drop(manager);

        let raw_text = response.text;
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let id = db::insert_note(&conn, &raw_text, &filename)
            .map_err(|e| format!("insert_note failed: {e}"))?;
        Ok(vec![id])
    }
}
