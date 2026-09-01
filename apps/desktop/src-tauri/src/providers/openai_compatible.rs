use super::{
    ChatChunk, ChatMessage, ChatRequest, ChatResponse, ChunkStream, ConnectionTestResult,
    LLMProviderAdapter, ModelInfo, ProviderError,
};
use async_trait::async_trait;
use futures::stream::{self, Stream, StreamExt, TryStreamExt};
use reqwest::Client;
use std::time::Instant;
use tokio::time::{timeout, Duration};

pub struct OpenAICompatibleAdapter {
    provider_id: String,
    base_url: String,
    api_key: Option<String>,
    client: Client,
    timeout_secs: u64,
}

impl OpenAICompatibleAdapter {
    pub fn new(provider_id: String, base_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            provider_id,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client,
            timeout_secs: 120,
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    fn build_request(&self, request: &ChatRequest) -> reqwest::Request {
        let mut builder = self
            .client
            .post(format!("{}/chat/completions", self.base_url));

        if let Some(ref key) = self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", key));
        }

        builder
            .json(request)
            .build()
            .expect("Failed to build request")
    }
}

#[async_trait]
impl LLMProviderAdapter for OpenAICompatibleAdapter {
    fn provider_type(&self) -> &str {
        "openai_compatible"
    }

    fn provider_id(&self) -> &str {
        &self.provider_id
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError> {
        let start = Instant::now();
        let req = self.build_request(&request);

        let response = timeout(
            Duration::from_secs(self.timeout_secs),
            self.client.execute(req),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(ProviderError::from)?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 401 {
                return Err(ProviderError::AuthError("Invalid API key".to_string()));
            } else if status.as_u16() == 429 {
                return Err(ProviderError::RateLimitExceeded);
            }

            let error_body = response.text().await.unwrap_or_default();
            return Err(ProviderError::InvalidResponse(format!(
                "HTTP {}: {}",
                status, error_body
            )));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        Ok(chat_response)
    }

    async fn stream(&self, request: ChatRequest) -> Result<ChunkStream, ProviderError> {
        let mut stream_request = request;
        stream_request.stream = Some(true);

        let req = self.build_request(&stream_request);
        let response = timeout(
            Duration::from_secs(self.timeout_secs),
            self.client.execute(req),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(ProviderError::from)?;

        if !response.status().is_success() {
            return Err(ProviderError::InvalidResponse(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let byte_stream = response.bytes_stream().map_ok(|b| b.to_vec());
        let stream = parse_sse(byte_stream);

        Ok(Box::pin(stream))
    }

    async fn test_connection(&self) -> Result<ConnectionTestResult, ProviderError> {
        let start = Instant::now();

        let request = ChatRequest {
            model: "test".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "test".to_string(),
                name: None,
            }],
            temperature: Some(0.0),
            max_tokens: Some(5),
            top_p: None,
            stream: Some(false),
            tools: None,
        };

        let response = self.chat(request).await;
        let latency = start.elapsed().as_millis() as u64;

        match response {
            Ok(_) => Ok(ConnectionTestResult {
                success: true,
                latency_ms: latency,
                error: None,
            }),
            Err(e) => Ok(ConnectionTestResult {
                success: false,
                latency_ms: latency,
                error: Some(e.to_string()),
            }),
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError> {
        let url = format!("{}/models", self.base_url);

        let mut req = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = timeout(
            Duration::from_secs(10),
            self.client
                .execute(req.build().expect("Failed to build request")),
        )
        .await
        .map_err(|_| ProviderError::Timeout)?
        .map_err(ProviderError::from)?;

        if !response.status().is_success() {
            return Err(ProviderError::InvalidResponse(format!(
                "HTTP {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }

        #[derive(Deserialize)]
        struct ModelData {
            id: String,
            #[serde(default)]
            display_name: Option<String>,
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        Ok(models_response
            .data
            .into_iter()
            .map(|m| {
                let id = m.id;
                let display_name = m.display_name.unwrap_or_else(|| id.clone());
                ModelInfo {
                    id,
                    display_name,
                    context_window: None,
                    supports_streaming: true,
                }
            })
            .collect())
    }
}

/// Robust SSE parser that buffers streamed chunks and emits complete `data:` events.
/// Empty `data: [DONE]` terminates the stream.
fn parse_sse(
    byte_stream: impl Stream<Item = Result<Vec<u8>, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<ChatChunk, ProviderError>> + Send {
    let byte_stream = Box::pin(byte_stream);
    stream::unfold(
        (byte_stream, String::new()),
        |(mut byte_stream, mut buffer)| async move {
            loop {
                match byte_stream.next().await {
                    Some(Ok(bytes)) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process all complete SSE events (separated by blank lines)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event = buffer[..pos].to_string();
                            buffer.drain(..pos + 2);

                            match parse_sse_event(&event) {
                                Some(SseItem::Done) => return None,
                                Some(SseItem::Chunk(chunk)) => {
                                    return Some((Ok(chunk), (byte_stream, buffer)))
                                }
                                None => continue,
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(ProviderError::NetworkError(e.to_string())),
                            (byte_stream, buffer),
                        ))
                    }
                    None => return None,
                }
            }
        },
    )
}

enum SseItem {
    Chunk(ChatChunk),
    Done,
}

fn parse_sse_event(event: &str) -> Option<SseItem> {
    for line in event.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim();
            if data == "[DONE]" {
                return Some(SseItem::Done);
            }
            if !data.is_empty() {
                if let Ok(chunk) = serde_json::from_str::<ChatChunk>(data) {
                    return Some(SseItem::Chunk(chunk));
                }
            }
        }
    }
    None
}

use serde::Deserialize;
