use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub protocol: String,
    pub preset_id: Option<String>,
    pub base_url: String,
    pub api_key_ref: Option<String>,
    pub default_model_id: String,
    pub display_model_name: Option<String>,
    pub max_concurrency: i32,
    pub rate_limit_rpm: Option<i32>,
    pub timeout_ms: i32,
    pub proxy_url: Option<String>,
    pub failover_provider_ids: Option<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProfile {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub display_model_name: Option<String>,
    pub context_window: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_json_mode: bool,
    pub supports_vision: bool,
    pub supports_audio: bool,
    pub supports_video: bool,
    pub input_price: Option<f64>,
    pub output_price: Option<f64>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapability {
    pub model_profile_id: String,
    pub reasoning: i32,
    pub coding: i32,
    pub code_review: i32,
    pub long_context: i32,
    pub speed: i32,
    pub low_cost: i32,
    pub tool_use: i32,
    pub json_reliability: i32,
    pub multimodal: i32,
    pub chinese: i32,
    pub local_deploy: i32,
    pub rag: i32,
    pub realtime: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub primary_provider_id: Option<String>,
    pub primary_model_profile_id: Option<String>,
    pub budget_limit: Option<f64>,
    pub max_runtime_ms: Option<i64>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermission {
    pub agent_id: String,
    pub can_read_files: bool,
    pub can_write_files: bool,
    pub can_execute_commands: bool,
    pub can_install_dependencies: bool,
    pub can_access_network: bool,
    pub can_modify_env_files: bool,
    pub can_delete_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub r#type: Option<String>,
    pub tech_stack: Option<String>,
    pub default_team_preset_id: Option<String>,
    pub budget_limit: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub task_type: String,
    pub complexity: String,
    pub risk_level: String,
    pub status: String,
    pub assigned_agent_id: Option<String>,
    pub selected_provider_id: Option<String>,
    pub selected_model_profile_id: Option<String>,
    pub routing_reason: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Provider preset definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPreset {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub default_model: String,
    pub supports_streaming: bool,
}

pub fn get_default_presets() -> Vec<ProviderPreset> {
    vec![
        ProviderPreset {
            id: "minimax".to_string(),
            name: "Minimax".to_string(),
            provider_type: "minimax".to_string(),
            base_url: "https://api.minimax.chat/v1".to_string(),
            default_model: "MiniMax-Text-01".to_string(),
            supports_streaming: true,
        },
        ProviderPreset {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            provider_type: "deepseek".to_string(),
            base_url: "https://api.deepseek.com/v1".to_string(),
            default_model: "deepseek-chat".to_string(),
            supports_streaming: true,
        },
        ProviderPreset {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            provider_type: "ollama".to_string(),
            base_url: "http://localhost:11434/v1".to_string(),
            default_model: "llama3".to_string(),
            supports_streaming: true,
        },
        ProviderPreset {
            id: "lmstudio".to_string(),
            name: "LM Studio".to_string(),
            provider_type: "lmstudio".to_string(),
            base_url: "http://localhost:1234/v1".to_string(),
            default_model: "local-model".to_string(),
            supports_streaming: true,
        },
    ]
}
