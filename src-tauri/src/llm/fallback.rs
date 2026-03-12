// FallbackManager — tries primary provider, switches to fallback on failure with cooldown

use std::sync::Mutex as StdMutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde_json::Value;

use crate::llm::provider::LLMProvider;
use crate::llm::types::{CompletionRequest, CompletionResponse, LLMError, StructuredRequest};

/// Wraps a primary and optional fallback provider.
/// On Unavailable or Timeout from primary, switches to fallback for `primary_cooldown`.
/// Uses interior mutability for cooldown state so `&self` trait methods can update it.
pub struct FallbackManager {
    primary: Box<dyn LLMProvider>,
    fallback: Option<Box<dyn LLMProvider>>,
    /// How long to stay on fallback after primary failure (default 5 min).
    primary_cooldown: Duration,
    /// Interior-mutable cooldown state — updated from `&self` methods.
    cooldown_until: StdMutex<Option<Instant>>,
}

impl FallbackManager {
    pub fn new(
        primary: Box<dyn LLMProvider>,
        fallback: Option<Box<dyn LLMProvider>>,
    ) -> Self {
        Self {
            primary,
            fallback,
            primary_cooldown: Duration::from_secs(300),
            cooldown_until: StdMutex::new(None),
        }
    }

    /// Override the default 5-minute cooldown duration.
    pub fn with_cooldown(mut self, duration: Duration) -> Self {
        self.primary_cooldown = duration;
        self
    }

    /// Replace the primary provider at runtime (e.g. after settings change).
    pub fn switch_primary(&mut self, new_primary: Box<dyn LLMProvider>) {
        self.primary = new_primary;
        *self.cooldown_until.lock().unwrap() = None;
    }

    /// Returns true when the primary is currently in a cooldown window.
    fn primary_in_cooldown(&self) -> bool {
        self.cooldown_until
            .lock()
            .unwrap()
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    /// Record a primary failure — sets the cooldown window.
    fn mark_primary_failed(&self) {
        *self.cooldown_until.lock().unwrap() = Some(Instant::now() + self.primary_cooldown);
        log::warn!(
            "[FallbackManager] Primary '{}' failed — cooldown for {}s",
            self.primary.provider_name(),
            self.primary_cooldown.as_secs()
        );
    }

    /// True if the error warrants a fallback attempt.
    fn is_fallback_worthy(err: &LLMError) -> bool {
        matches!(err, LLMError::Unavailable(_) | LLMError::Timeout(_))
    }

    /// Try fallback provider or return error.
    async fn try_fallback_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LLMError> {
        if let Some(fb) = &self.fallback {
            log::info!("[FallbackManager] Using fallback '{}'", fb.provider_name());
            fb.generate_completion(request).await
        } else {
            Err(LLMError::Unavailable("primary unavailable, no fallback configured".into()))
        }
    }

    async fn try_fallback_structured(&self, request: StructuredRequest) -> Result<Value, LLMError> {
        if let Some(fb) = &self.fallback {
            log::info!("[FallbackManager] Using fallback '{}'", fb.provider_name());
            fb.generate_structured(request).await
        } else {
            Err(LLMError::Unavailable("primary unavailable, no fallback configured".into()))
        }
    }
}

#[async_trait]
impl LLMProvider for FallbackManager {
    async fn generate_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        if !self.primary_in_cooldown() {
            match self.primary.generate_completion(request.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(ref e) if Self::is_fallback_worthy(e) => {
                    self.mark_primary_failed();
                }
                Err(e) => return Err(e),
            }
        }
        self.try_fallback_completion(request).await
    }

    async fn generate_structured(
        &self,
        request: StructuredRequest,
    ) -> Result<Value, LLMError> {
        if !self.primary_in_cooldown() {
            match self.primary.generate_structured(request.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(ref e) if Self::is_fallback_worthy(e) => {
                    self.mark_primary_failed();
                }
                Err(e) => return Err(e),
            }
        }
        self.try_fallback_structured(request).await
    }

    async fn is_available(&self) -> bool {
        if self.primary.is_available().await {
            return true;
        }
        if let Some(fb) = &self.fallback {
            return fb.is_available().await;
        }
        false
    }

    fn provider_name(&self) -> &str {
        if self.primary_in_cooldown() {
            self.fallback
                .as_ref()
                .map(|fb| fb.provider_name())
                .unwrap_or("none")
        } else {
            self.primary.provider_name()
        }
    }
}
