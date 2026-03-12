// Ollama LLM provider — OpenAI-compatible chat + native JSON generation

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde_json::{json, Value};
use std::time::Duration;

use crate::llm::provider::LLMProvider;
use crate::llm::types::{
    CompletionRequest, CompletionResponse, LLMError, StructuredRequest, TokenUsage,
};

pub struct OllamaProvider {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: &str, model: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("failed to build reqwest client for Ollama");

        Self {
            client,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    /// Map reqwest errors to LLMError variants.
    fn map_err(err: reqwest::Error, ctx: &str) -> LLMError {
        if err.is_timeout() {
            LLMError::Timeout(format!("Ollama timeout: {ctx}"))
        } else if err.is_connect() {
            LLMError::Unavailable(format!("Ollama connect failed: {ctx}"))
        } else {
            LLMError::InternalError(format!("Ollama request error ({ctx}): {err}"))
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn generate_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let mut messages: Vec<Value> = Vec::new();

        if let Some(sys) = &request.system_prompt {
            messages.push(json!({ "role": "system", "content": sys }));
        }

        // Build user content — text + optional image parts
        let user_content: Value = if request.images.is_empty() {
            json!(request.user_message)
        } else {
            let mut parts: Vec<Value> = vec![json!({ "type": "text", "text": request.user_message })];
            for img in &request.images {
                let b64 = B64.encode(&img.bytes);
                let data_url = format!("data:{};base64,{}", img.mime_type, b64);
                parts.push(json!({ "type": "image_url", "image_url": { "url": data_url } }));
            }
            json!(parts)
        };

        messages.push(json!({ "role": "user", "content": user_content }));

        let mut body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": request.temperature,
            "stream": false,
        });

        if let Some(max_t) = request.max_tokens {
            body["max_tokens"] = json!(max_t);
        }

        let resp = self
            .client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_err(e, "chat/completions"))?;

        let status = resp.status();
        let json: Value = resp
            .json()
            .await
            .map_err(|e| LLMError::ParseError(format!("Ollama response parse: {e}")))?;

        if !status.is_success() {
            let msg = json["error"]["message"]
                .as_str()
                .unwrap_or("unknown error")
                .to_string();
            return Err(LLMError::InternalError(format!("Ollama {status}: {msg}")));
        }

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LLMError::ParseError("missing choices[0].message.content".into()))?
            .to_string();

        let usage = json["usage"].as_object().map(|u| TokenUsage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(CompletionResponse { text, usage })
    }

    async fn generate_structured(
        &self,
        request: StructuredRequest,
    ) -> Result<Value, LLMError> {
        // Embed schema instructions in prompt so Ollama returns valid JSON
        let schema_str = serde_json::to_string_pretty(&request.response_schema)
            .map_err(|e| LLMError::InternalError(format!("schema serialise: {e}")))?;

        let mut full_prompt = format!(
            "{}\n\nRespond ONLY with valid JSON matching this schema:\n{}",
            request.user_message, schema_str
        );

        if let Some(sys) = &request.system_prompt {
            full_prompt = format!("{sys}\n\n{full_prompt}");
        }

        let body = json!({
            "model": self.model,
            "prompt": full_prompt,
            "format": "json",
            "stream": false,
            "options": { "temperature": request.temperature },
        });

        let resp = self
            .client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| Self::map_err(e, "api/generate"))?;

        let status = resp.status();
        let json: Value = resp
            .json()
            .await
            .map_err(|e| LLMError::ParseError(format!("Ollama structured parse: {e}")))?;

        if !status.is_success() {
            let msg = json["error"].as_str().unwrap_or("unknown").to_string();
            return Err(LLMError::InternalError(format!("Ollama {status}: {msg}")));
        }

        let raw = json["response"]
            .as_str()
            .ok_or_else(|| LLMError::ParseError("missing response field".into()))?;

        serde_json::from_str(raw)
            .map_err(|e| LLMError::ParseError(format!("structured JSON parse: {e}")))
    }

    async fn is_available(&self) -> bool {
        let probe = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .timeout(Duration::from_secs(3))
            .send()
            .await;

        matches!(probe, Ok(r) if r.status().is_success())
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }
}
