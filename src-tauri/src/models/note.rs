use serde::{Deserialize, Serialize};

/// Represents a single note in the Notal app with all multi-layer memory fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub raw_text: String,
    pub summary: String,
    pub importance: f64,
    pub current_score: f64,
    pub layer: String,
    pub pinned: bool,
    pub archived: bool,
    pub created_at: String,
    pub last_accessed_at: Option<String>,
    pub last_updated_at: Option<String>,
    pub layer_promoted_at: Option<String>,
    pub access_count: i64,
    pub access_count_since_promotion: i64,
    /// JSON-decoded list of named entities extracted by LLM
    pub entities: Vec<String>,
    /// JSON-decoded list of topics
    pub topics: Vec<String>,
    /// JSON-decoded list of related note IDs
    pub connections: Vec<i64>,
    pub source: String,
    pub enriched: bool,
}
