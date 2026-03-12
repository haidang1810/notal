// LLMProvider trait — async, object-safe via async_trait

use async_trait::async_trait;
use crate::llm::types::{CompletionRequest, CompletionResponse, LLMError, StructuredRequest};

/// Core abstraction for all LLM backends.
/// Implement this for Ollama, Gemini, and any future provider.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a free-form text completion (with optional images).
    async fn generate_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError>;

    /// Generate a response that conforms to the given JSON Schema.
    async fn generate_structured(
        &self,
        request: StructuredRequest,
    ) -> Result<serde_json::Value, LLMError>;

    /// Lightweight connectivity probe — returns true if the provider can accept requests.
    async fn is_available(&self) -> bool;

    /// Human-readable provider identifier (e.g. "ollama", "gemini").
    fn provider_name(&self) -> &str;
}
