// LLM module — re-exports and provider factory

pub mod fallback;
pub mod gemini;
pub mod ollama;
pub mod provider;
pub mod types;

pub use fallback::FallbackManager;
pub use provider::LLMProvider;
pub use types::*;

use crate::models::settings::{AppSettings, LlmProviderType};

/// Build a `FallbackManager` from application settings.
///
/// | Setting  | Primary | Fallback                       |
/// |----------|---------|-------------------------------|
/// | Ollama   | Ollama  | Gemini (if api_key set)        |
/// | Gemini   | Gemini  | Ollama (always attempted)      |
/// | Auto     | Ollama  | Gemini (if api_key set)        |
pub fn create_provider(settings: &AppSettings) -> Result<FallbackManager, LLMError> {
    let ollama_primary = || -> Box<dyn LLMProvider> {
        Box::new(ollama::OllamaProvider::new(
            &settings.ollama_endpoint,
            "qwen2.5:latest",
        ))
    };

    let gemini_model = if settings.gemini_model.is_empty() {
        "gemini-2.0-flash-lite".to_string()
    } else {
        settings.gemini_model.clone()
    };

    let gemini_primary = || -> Result<Box<dyn LLMProvider>, LLMError> {
        if settings.gemini_api_key.is_empty() {
            return Err(LLMError::InvalidConfig(
                "Gemini selected but gemini_api_key is not set".into(),
            ));
        }
        Ok(Box::new(gemini::GeminiProvider::new(
            &settings.gemini_api_key,
            &gemini_model,
        )))
    };

    let gemini_fallback = || -> Option<Box<dyn LLMProvider>> {
        if settings.gemini_api_key.is_empty() {
            return None;
        }
        Some(Box::new(gemini::GeminiProvider::new(
            &settings.gemini_api_key,
            &gemini_model,
        )))
    };

    let manager = match &settings.llm_provider {
        LlmProviderType::Ollama => {
            FallbackManager::new(ollama_primary(), gemini_fallback())
        }
        LlmProviderType::Gemini => {
            let primary = gemini_primary()?;
            let fallback: Option<Box<dyn LLMProvider>> =
                Some(Box::new(ollama::OllamaProvider::new(
                    &settings.ollama_endpoint,
                    "qwen2.5:latest",
                )));
            FallbackManager::new(primary, fallback)
        }
        LlmProviderType::Auto => {
            // Default: prefer local Ollama, fall back to Gemini if key available
            FallbackManager::new(ollama_primary(), gemini_fallback())
        }
    };

    Ok(manager)
}
