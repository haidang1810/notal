// Gemini LLM provider — native multimodal + JSON Schema structured output

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};
use std::time::Duration;

use crate::llm::provider::LLMProvider;
use crate::llm::types::{
    CompletionRequest, CompletionResponse, LLMError, StructuredRequest, TokenUsage,
};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client for Gemini");

        Self {
            client,
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    fn generate_url(&self) -> String {
        format!(
            "{}/models/{}:generateContent?key={}",
            BASE_URL, self.model, self.api_key
        )
    }

    /// Build a contents array from prompt text and optional images.
    fn build_contents(system_prompt: Option<&str>, user_message: &str, images: &[crate::llm::types::ImageData]) -> Value {
        let mut parts: Vec<Value> = Vec::new();

        if let Some(sys) = system_prompt {
            parts.push(json!({ "text": format!("{sys}\n\n{user_message}") }));
        } else {
            parts.push(json!({ "text": user_message }));
        }

        for img in images {
            let b64 = B64.encode(&img.bytes);
            parts.push(json!({
                "inlineData": {
                    "mimeType": img.mime_type,
                    "data": b64,
                }
            }));
        }

        json!([{ "role": "user", "parts": parts }])
    }

    /// Map HTTP status + reqwest errors to LLMError.
    fn map_http_err(status: reqwest::StatusCode, body: &Value) -> LLMError {
        let msg = body["error"]["message"]
            .as_str()
            .unwrap_or("unknown error")
            .to_string();

        match status.as_u16() {
            401 | 403 => LLMError::InvalidConfig(format!("Gemini auth error: {msg}")),
            429 => LLMError::RateLimited(format!("Gemini rate limit: {msg}")),
            _ => LLMError::InternalError(format!("Gemini {status}: {msg}")),
        }
    }

    fn map_reqwest_err(err: reqwest::Error, ctx: &str) -> LLMError {
        if err.is_timeout() {
            LLMError::Timeout(format!("Gemini timeout: {ctx}"))
        } else if err.is_connect() {
            LLMError::Unavailable(format!("Gemini connect failed: {ctx}"))
        } else {
            LLMError::InternalError(format!("Gemini request error ({ctx}): {err}"))
        }
    }

    /// Extract text from Gemini response JSON.
    fn extract_text(json: &Value) -> Result<String, LLMError> {
        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| LLMError::ParseError("missing candidates[0].content.parts[0].text".into()))
    }
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    async fn generate_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let contents = Self::build_contents(
            request.system_prompt.as_deref(),
            &request.user_message,
            &request.images,
        );

        let mut body = json!({
            "contents": contents,
            "generationConfig": {
                "temperature": request.temperature,
            }
        });

        if let Some(max_t) = request.max_tokens {
            body["generationConfig"]["maxOutputTokens"] = json!(max_t);
        }

        let resp = self
            .client
            .post(self.generate_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_reqwest_err(e, "generateContent"))?;

        let status = resp.status();
        let json: Value = resp
            .json()
            .await
            .map_err(|e| LLMError::ParseError(format!("Gemini response parse: {e}")))?;

        if !status.is_success() {
            return Err(Self::map_http_err(status, &json));
        }

        let text = Self::extract_text(&json)?;

        let usage = json["usageMetadata"].as_object().map(|u| TokenUsage {
            prompt_tokens: u["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["totalTokenCount"].as_u64().unwrap_or(0) as u32,
        });

        Ok(CompletionResponse { text, usage })
    }

    async fn generate_structured(
        &self,
        request: StructuredRequest,
    ) -> Result<Value, LLMError> {
        let contents = Self::build_contents(
            request.system_prompt.as_deref(),
            &request.user_message,
            &request.images,
        );

        let body = json!({
            "contents": contents,
            "generationConfig": {
                "temperature": request.temperature,
                "responseMimeType": "application/json",
                "responseSchema": request.response_schema,
            }
        });

        let resp = self
            .client
            .post(self.generate_url())
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_reqwest_err(e, "generateContent/structured"))?;

        let status = resp.status();
        let json: Value = resp
            .json()
            .await
            .map_err(|e| LLMError::ParseError(format!("Gemini structured parse: {e}")))?;

        if !status.is_success() {
            return Err(Self::map_http_err(status, &json));
        }

        let raw = Self::extract_text(&json)?;

        serde_json::from_str(&raw)
            .map_err(|e| LLMError::ParseError(format!("structured JSON parse: {e}")))
    }

    async fn is_available(&self) -> bool {
        // Lightweight ping — short timeout, minimal tokens
        let body = json!({
            "contents": [{ "role": "user", "parts": [{ "text": "ping" }] }],
            "generationConfig": { "maxOutputTokens": 1 }
        });

        let probe = self
            .client
            .post(self.generate_url())
            .timeout(Duration::from_secs(5))
            .json(&body)
            .send()
            .await;

        matches!(probe, Ok(r) if r.status().is_success())
    }

    fn provider_name(&self) -> &str {
        "gemini"
    }
}
