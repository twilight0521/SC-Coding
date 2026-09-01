use crate::db::Database;
use crate::models::get_default_presets;
use crate::providers::ProviderRegistry;
use crate::runtime;
use crate::security::SecretStore;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_model_id: String,
    pub display_model_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub available_models: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub default_model_id: String,
    pub display_model_name: Option<String>,
    pub is_enabled: bool,
}

// List all providers
#[tauri::command]
pub fn list_providers(db: State<Database>) -> Result<Vec<ProviderResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, base_url, default_model_id, display_model_name, is_enabled
         FROM provider_configs ORDER BY name"
    ).map_err(|e| e.to_string())?;

    let providers = stmt
        .query_map([], |row| {
            Ok(ProviderResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                default_model_id: row.get(4)?,
                display_model_name: row.get(5)?,
                is_enabled: row.get::<_, i32>(6)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(providers)
}

// Get provider presets
#[tauri::command]
pub fn get_provider_presets() -> Vec<crate::models::ProviderPreset> {
    get_default_presets()
}

// Create a new provider
#[tauri::command]
pub fn create_provider(
    db: State<Database>,
    secret_store: State<SecretStore>,
    registry: State<Arc<ProviderRegistry>>,
    request: CreateProviderRequest,
) -> Result<ProviderResponse, String> {
    let conn = db.connection();
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();
    let api_key_ref = request
        .api_key
        .as_ref()
        .map(|key| secret_store.store(key).map_err(|e| e.to_string()))
        .transpose()?;

    conn.execute(
        "INSERT INTO provider_configs
         (id, name, provider_type, protocol, base_url, api_key_ref, default_model_id,
          display_model_name, is_enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'openai_chat_completions', ?4, ?5, ?6, ?7, 1, ?8, ?8)",
        params![
            id,
            request.name,
            request.provider_type,
            request.base_url,
            api_key_ref,
            request.default_model_id,
            request.display_model_name,
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    // Create a default model profile + capabilities so the capability
    // router has data to score. Without this, routing always returns empty.
    let model_profile_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO model_profiles
         (id, provider_id, model_id, display_model_name, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
        params![
            model_profile_id,
            id,
            request.default_model_id,
            request.display_model_name,
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO model_capabilities (model_profile_id, updated_at) VALUES (?1, ?2)",
        params![model_profile_id, now.to_rfc3339()],
    )
    .map_err(|e| e.to_string())?;

    // Register the new adapter in the registry
    let api_key = request.api_key.clone();
    runtime::update_provider_adapter(&registry, &id, request.base_url.clone(), api_key);

    Ok(ProviderResponse {
        id,
        name: request.name,
        provider_type: request.provider_type,
        base_url: request.base_url,
        default_model_id: request.default_model_id,
        display_model_name: request.display_model_name,
        is_enabled: true,
    })
}

// Update provider
#[tauri::command]
pub fn update_provider(
    db: State<Database>,
    secret_store: State<SecretStore>,
    registry: State<Arc<ProviderRegistry>>,
    id: String,
    name: Option<String>,
    base_url: Option<String>,
    default_model_id: Option<String>,
    display_model_name: Option<String>,
    is_enabled: Option<bool>,
) -> Result<(), String> {
    let conn = db.connection();
    let now = Utc::now();

    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.to_rfc3339())];

    if let Some(n) = name {
        updates.push("name = ?".to_string());
        params_vec.push(Box::new(n));
    }
    if let Some(url) = base_url {
        updates.push("base_url = ?".to_string());
        params_vec.push(Box::new(url));
    }
    if let Some(model) = default_model_id {
        updates.push("default_model_id = ?".to_string());
        params_vec.push(Box::new(model));
    }
    if let Some(dname) = display_model_name {
        updates.push("display_model_name = ?".to_string());
        params_vec.push(Box::new(dname));
    }
    if let Some(enabled) = is_enabled {
        updates.push("is_enabled = ?".to_string());
        params_vec.push(Box::new(if enabled { 1 } else { 0 }));
    }

    params_vec.push(Box::new(id.clone()));

    let sql = format!(
        "UPDATE provider_configs SET {} WHERE id = ?",
        updates.join(", ")
    );

    conn.execute(
        &sql,
        rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
    )
    .map_err(|e| e.to_string())?;

    // Update the registry - fetch current config and rebuild adapter
    let new_base_url: String = conn
        .query_row(
            "SELECT base_url FROM provider_configs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let api_key_ref: Option<String> = conn
        .query_row(
            "SELECT api_key_ref FROM provider_configs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .ok();

    let api_key = api_key_ref.and_then(|key_ref| secret_store.retrieve(&key_ref).ok());

    if let Some(enabled) = is_enabled {
        if !enabled {
            runtime::remove_provider_adapter(&registry, &id);
            return Ok(());
        }
    }

    runtime::update_provider_adapter(&registry, &id, new_base_url, api_key);

    Ok(())
}

// Delete provider
#[tauri::command]
pub fn delete_provider(
    db: State<Database>,
    secret_store: State<SecretStore>,
    registry: State<Arc<ProviderRegistry>>,
    id: String,
) -> Result<(), String> {
    let conn = db.connection();

    // First get the api_key_ref to delete from secret store
    let api_key_ref: Option<String> = conn
        .query_row(
            "SELECT api_key_ref FROM provider_configs WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .ok();

    // Delete from secret store if exists
    if let Some(key_ref) = api_key_ref {
        let _ = secret_store.delete(&key_ref);
    }

    // Delete the provider (cascade will handle model_profiles and model_capabilities)
    conn.execute("DELETE FROM provider_configs WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    // Remove from registry
    runtime::remove_provider_adapter(&registry, &id);

    Ok(())
}

// Test connection to a provider
#[tauri::command]
pub async fn test_provider_connection(
    base_url: String,
    api_key: Option<String>,
    model_id: String,
) -> Result<ConnectionTestResult, String> {
    use std::time::Instant;

    let client = reqwest::Client::new();
    let start = Instant::now();

    let request_body = serde_json::json!({
        "model": model_id,
        "messages": [{"role": "user", "content": "test"}],
        "max_tokens": 5
    });

    let mut request = client
        .post(format!("{}/chat/completions", base_url))
        .header("Content-Type", "application/json");

    if let Some(key) = api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let latency = start.elapsed().as_millis() as u64;

    if response.status().is_success() {
        Ok(ConnectionTestResult {
            success: true,
            latency_ms: Some(latency),
            error: None,
            available_models: None,
        })
    } else {
        Ok(ConnectionTestResult {
            success: false,
            latency_ms: Some(latency),
            error: Some(format!("HTTP {}", response.status())),
            available_models: None,
        })
    }
}

// ==================== AGENT COMMANDS ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub primary_provider_id: Option<String>,
    pub primary_model_profile_id: Option<String>,
    pub budget_limit: Option<f64>,
    pub max_runtime_ms: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
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
}

// List all agents
#[tauri::command]
pub fn list_agents(db: State<Database>) -> Result<Vec<AgentResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, name, role, description, system_prompt, primary_provider_id,
                primary_model_profile_id, budget_limit, max_runtime_ms, is_enabled
         FROM agents ORDER BY name",
        )
        .map_err(|e| e.to_string())?;

    let agents = stmt
        .query_map([], |row| {
            Ok(AgentResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                description: row.get(3)?,
                system_prompt: row.get(4)?,
                primary_provider_id: row.get(5)?,
                primary_model_profile_id: row.get(6)?,
                budget_limit: row.get(7)?,
                max_runtime_ms: row.get(8)?,
                is_enabled: row.get::<_, i32>(9)? == 1,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(agents)
}

// Get agent by ID
#[tauri::command]
pub fn get_agent(db: State<Database>, id: String) -> Result<AgentResponse, String> {
    let conn = db.connection();

    conn.query_row(
        "SELECT id, name, role, description, system_prompt, primary_provider_id,
                primary_model_profile_id, budget_limit, max_runtime_ms, is_enabled
         FROM agents WHERE id = ?1",
        params![id],
        |row| {
            Ok(AgentResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                description: row.get(3)?,
                system_prompt: row.get(4)?,
                primary_provider_id: row.get(5)?,
                primary_model_profile_id: row.get(6)?,
                budget_limit: row.get(7)?,
                max_runtime_ms: row.get(8)?,
                is_enabled: row.get::<_, i32>(9)? == 1,
            })
        },
    )
    .map_err(|e| e.to_string())
}

// Create a new agent
#[tauri::command]
pub fn create_agent(
    db: State<Database>,
    request: CreateAgentRequest,
) -> Result<AgentResponse, String> {
    let conn = db.connection();
    let id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO agents
         (id, name, role, description, system_prompt, primary_provider_id,
          primary_model_profile_id, budget_limit, max_runtime_ms, is_enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?10)",
        params![
            id,
            request.name,
            request.role,
            request.description,
            request.system_prompt,
            request.primary_provider_id,
            request.primary_model_profile_id,
            request.budget_limit,
            request.max_runtime_ms,
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    // Create role-based least-privilege defaults.
    let (can_read_files, can_write_files, can_execute_commands, can_delete_files) =
        match request.role.as_str() {
            "coder" | "frontend_engineer" | "backend_engineer" | "fullstack_engineer"
            | "debugger" | "debug_engineer" => (true, true, false, false),
            "tester" | "test_engineer" => (true, false, true, false),
            _ => (true, false, false, false),
        };

    conn.execute(
        "INSERT INTO agent_permissions (agent_id, can_read_files, can_write_files, can_execute_commands,
         can_install_dependencies, can_access_network, can_modify_env_files, can_delete_files)
         VALUES (?1, ?2, ?3, ?4, 0, 0, 0, ?5)",
        params![id, can_read_files as i32, can_write_files as i32, can_execute_commands as i32, can_delete_files as i32],
    ).map_err(|e| e.to_string())?;

    Ok(AgentResponse {
        id,
        name: request.name,
        role: request.role,
        description: request.description,
        system_prompt: request.system_prompt,
        primary_provider_id: request.primary_provider_id,
        primary_model_profile_id: request.primary_model_profile_id,
        budget_limit: request.budget_limit,
        max_runtime_ms: request.max_runtime_ms,
        is_enabled: true,
    })
}

// Update agent
#[tauri::command]
pub fn update_agent(
    db: State<Database>,
    id: String,
    name: Option<String>,
    role: Option<String>,
    description: Option<String>,
    system_prompt: Option<String>,
    primary_provider_id: Option<String>,
    primary_model_profile_id: Option<String>,
    budget_limit: Option<f64>,
    max_runtime_ms: Option<i64>,
    is_enabled: Option<bool>,
) -> Result<(), String> {
    let conn = db.connection();
    let now = Utc::now();

    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.to_rfc3339())];

    if let Some(n) = name {
        updates.push("name = ?".to_string());
        params_vec.push(Box::new(n));
    }
    if let Some(r) = role {
        updates.push("role = ?".to_string());
        params_vec.push(Box::new(r));
    }
    if let Some(d) = description {
        updates.push("description = ?".to_string());
        params_vec.push(Box::new(d));
    }
    if let Some(sp) = system_prompt {
        updates.push("system_prompt = ?".to_string());
        params_vec.push(Box::new(sp));
    }
    if let Some(ppid) = primary_provider_id {
        updates.push("primary_provider_id = ?".to_string());
        params_vec.push(Box::new(ppid));
    }
    if let Some(mpid) = primary_model_profile_id {
        updates.push("primary_model_profile_id = ?".to_string());
        params_vec.push(Box::new(mpid));
    }
    if let Some(bl) = budget_limit {
        updates.push("budget_limit = ?".to_string());
        params_vec.push(Box::new(bl));
    }
    if let Some(mrt) = max_runtime_ms {
        updates.push("max_runtime_ms = ?".to_string());
        params_vec.push(Box::new(mrt));
    }
    if let Some(enabled) = is_enabled {
        updates.push("is_enabled = ?".to_string());
        params_vec.push(Box::new(if enabled { 1 } else { 0 }));
    }

    params_vec.push(Box::new(id.clone()));

    let sql = format!("UPDATE agents SET {} WHERE id = ?", updates.join(", "));

    conn.execute(
        &sql,
        rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Delete agent
#[tauri::command]
pub fn delete_agent(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.connection();

    // Delete related records first
    conn.execute(
        "DELETE FROM agent_permissions WHERE agent_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM agent_fallback_providers WHERE agent_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;

    // Delete agent
    conn.execute("DELETE FROM agents WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Get default agent templates
#[tauri::command]
pub fn get_default_agent_templates() -> Vec<CreateAgentRequest> {
    vec![
        CreateAgentRequest {
            name: "Orchestrator".to_string(),
            role: "orchestrator".to_string(),
            description: Some("Project planning and coordination".to_string()),
            system_prompt: "You are the Orchestrator of a software development project. Your responsibilities include:\n1. Analyzing project requirements and creating task breakdown\n2. Coordinating between different agents\n3. Monitoring progress and adjusting plans\n4. Generating progress reports for the user\n5. Making decisions about task sequencing and prioritization\n\nAlways prioritize clarity in communication and ensure all decisions are logged.".to_string(),
            primary_provider_id: None,
            primary_model_profile_id: None,
            budget_limit: Some(10.0),
            max_runtime_ms: Some(300000),
        },
        CreateAgentRequest {
            name: "Coder".to_string(),
            role: "coder".to_string(),
            description: Some("Code generation and implementation".to_string()),
            system_prompt: "You are the Coder agent responsible for implementing code features. Your responsibilities include:\n1. Writing clean, efficient, and maintainable code\n2. Following project conventions and best practices\n3. Creating necessary tests for your code\n4. Documenting your implementation\n\nGenerate code as patches that can be reviewed and applied.".to_string(),
            primary_provider_id: None,
            primary_model_profile_id: None,
            budget_limit: Some(5.0),
            max_runtime_ms: Some(180000),
        },
        CreateAgentRequest {
            name: "Tester".to_string(),
            role: "tester".to_string(),
            description: Some("Test execution and validation".to_string()),
            system_prompt: "You are the Tester agent responsible for running tests and validation. Your responsibilities include:\n1. Running test suites and capturing results\n2. Analyzing test failures and reporting issues\n3. Verifying that fixes resolve the reported problems\n4. Ensuring code quality meets standards\n\nReport all test results clearly with any error messages captured.".to_string(),
            primary_provider_id: None,
            primary_model_profile_id: None,
            budget_limit: Some(3.0),
            max_runtime_ms: Some(120000),
        },
        CreateAgentRequest {
            name: "Debugger".to_string(),
            role: "debugger".to_string(),
            description: Some("Bug detection and fixing".to_string()),
            system_prompt: "You are the Debugger agent responsible for fixing bugs. Your responsibilities include:\n1. Analyzing error messages and identifying root causes\n2. Fixing bugs in existing code\n3. Verifying fixes work correctly\n4. Documenting any changes made\n\nYou can auto-fix up to 3 rounds of debugging.".to_string(),
            primary_provider_id: None,
            primary_model_profile_id: None,
            budget_limit: Some(3.0),
            max_runtime_ms: Some(180000),
        },
        CreateAgentRequest {
            name: "Document Writer".to_string(),
            role: "document_writer".to_string(),
            description: Some("Documentation generation".to_string()),
            system_prompt: "You are the Document Writer agent responsible for creating documentation. Your responsibilities include:\n1. Writing clear README files\n2. Documenting code functionality\n3. Creating API documentation\n4. Summarizing project structure\n\nGenerate documentation in Markdown format.".to_string(),
            primary_provider_id: None,
            primary_model_profile_id: None,
            budget_limit: Some(2.0),
            max_runtime_ms: Some(60000),
        },
    ]
}

// Create agents from templates
#[tauri::command]
pub fn create_default_agents(db: State<Database>) -> Result<Vec<AgentResponse>, String> {
    let templates = get_default_agent_templates();
    let mut created = Vec::new();

    for template in templates {
        let agent = create_agent(db.clone(), template)?;
        created.push(agent);
    }

    Ok(created)
}

// Update agent permissions
#[tauri::command]
pub fn update_agent_permissions(
    db: State<Database>,
    agent_id: String,
    can_read_files: Option<bool>,
    can_write_files: Option<bool>,
    can_execute_commands: Option<bool>,
    can_install_dependencies: Option<bool>,
    can_access_network: Option<bool>,
    can_modify_env_files: Option<bool>,
    can_delete_files: Option<bool>,
) -> Result<(), String> {
    let conn = db.connection();

    let mut updates = vec![];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(v) = can_read_files {
        updates.push("can_read_files = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_write_files {
        updates.push("can_write_files = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_execute_commands {
        updates.push("can_execute_commands = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_install_dependencies {
        updates.push("can_install_dependencies = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_access_network {
        updates.push("can_access_network = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_modify_env_files {
        updates.push("can_modify_env_files = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }
    if let Some(v) = can_delete_files {
        updates.push("can_delete_files = ?".to_string());
        params_vec.push(Box::new(if v { 1 } else { 0 }));
    }

    if updates.is_empty() {
        return Ok(());
    }

    params_vec.push(Box::new(agent_id.clone()));

    let sql = format!(
        "UPDATE agent_permissions SET {} WHERE agent_id = ?",
        updates.join(", ")
    );

    conn.execute(
        &sql,
        rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Get agent permissions
#[tauri::command]
pub fn get_agent_permissions(
    db: State<Database>,
    agent_id: String,
) -> Result<serde_json::Value, String> {
    let conn = db.connection();

    conn.query_row(
        "SELECT can_read_files, can_write_files, can_execute_commands, can_install_dependencies,
                can_access_network, can_modify_env_files, can_delete_files
         FROM agent_permissions WHERE agent_id = ?1",
        params![agent_id],
        |row| {
            Ok(serde_json::json!({
                "can_read_files": row.get::<_, i32>(0)? == 1,
                "can_write_files": row.get::<_, i32>(1)? == 1,
                "can_execute_commands": row.get::<_, i32>(2)? == 1,
                "can_install_dependencies": row.get::<_, i32>(3)? == 1,
                "can_access_network": row.get::<_, i32>(4)? == 1,
                "can_modify_env_files": row.get::<_, i32>(5)? == 1,
                "can_delete_files": row.get::<_, i32>(6)? == 1,
            }))
        },
    )
    .map_err(|e| e.to_string())
}
