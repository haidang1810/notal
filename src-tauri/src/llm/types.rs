// LLM request/response types and error enum for Phase 3

use serde::{Deserialize, Serialize};

/// Raw image bytes + MIME type for multimodal requests
#[derive(Debug, Clone)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

/// Standard text (+ optional image) completion request
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub system_prompt: Option<String>,
    pub user_message: String,
    pub images: Vec<ImageData>,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

/// Standard completion response
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub text: String,
    pub usage: Option<TokenUsage>,
}

/// Token usage metadata (not all providers return this)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Request that expects a structured JSON response matching a schema
#[derive(Debug, Clone)]
pub struct StructuredRequest {
    pub system_prompt: Option<String>,
    pub user_message: String,
    pub images: Vec<ImageData>,
    /// JSON Schema the response must conform to
    pub response_schema: serde_json::Value,
    pub temperature: f32,
}

/// Errors that LLM providers can return
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Provider unavailable: {0}")]
    Unavailable(String),

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
