use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

pub mod openai_compatible;

pub use openai_compatible::OpenAICompatibleAdapter;

pub type ChunkStream = Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Timeout error")]
    Timeout,
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ProviderError::Timeout
        } else if err.is_connect() {
            ProviderError::NetworkError(err.to_string())
        } else {
            ProviderError::NetworkError(err.to_string())
        }
    }
}

// ==================== Chat Request/Response ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
}

// ==================== Stream Chunk ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub id: String,
    pub model: Option<String>,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

// ==================== Provider Adapter Trait ====================

#[async_trait::async_trait]
pub trait LLMProviderAdapter: Send + Sync {
    fn provider_type(&self) -> &str;
    fn provider_id(&self) -> &str;

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;

    async fn stream(&self, request: ChatRequest) -> Result<ChunkStream, ProviderError>;

    async fn test_connection(&self) -> Result<ConnectionTestResult, ProviderError>;

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: Option<u32>,
    pub supports_streaming: bool,
}

// ==================== Provider Registry ====================

use std::collections::HashMap;
use std::sync::RwLock;

pub struct ProviderRegistry {
    adapters: RwLock<HashMap<String, Arc<dyn LLMProviderAdapter>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, provider_id: String, adapter: Arc<dyn LLMProviderAdapter>) {
        let mut adapters = self.adapters.write().unwrap();
        adapters.insert(provider_id, adapter);
    }

    pub fn get(&self, provider_id: &str) -> Option<Arc<dyn LLMProviderAdapter>> {
        let adapters = self.adapters.read().unwrap();
        adapters.get(provider_id).cloned()
    }

    pub fn remove(&self, provider_id: &str) -> bool {
        let mut adapters = self.adapters.write().unwrap();
        adapters.remove(provider_id).is_some()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let adapters = self.adapters.read().unwrap();
        adapters.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
