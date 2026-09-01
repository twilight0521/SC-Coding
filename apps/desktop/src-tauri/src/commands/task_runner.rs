use crate::agents::{extract_keywords, AgentConfig, PromptBuilder};
use crate::context::ContextBuilder;
use crate::db::Database;
use crate::patches::{ParsedPatch, PatchParseResult, PatchParser};
use crate::providers::{ChatRequest, ProviderError, ProviderRegistry};
use crate::security::SecretStore;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::State;
use uuid::Uuid;

// ==================== TASK RUNNER COMMANDS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: String,
    pub project_id: String,
    pub task_type: String,
    pub assigned_agent_id: Option<String>,
    pub selected_provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub patches: Vec<FilePatch>,
    pub cost: f64,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    pub file_path: String,
    pub patch: String,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLog {
    pub id: String,
    pub project_run_id: String,
    pub task_id: Option<String>,
    pub decision_type: String,
    pub decision_summary: String,
    pub reason: String,
    pub decided_by: String,
    pub alternatives_json: Option<String>,
    pub risk_level: Option<String>,
    pub selected_model_provider_id: Option<String>,
    pub selected_model_profile_id: Option<String>,
    pub estimated_cost: Option<f64>,
    pub created_at: String,
}

// Task runner state
pub struct TaskRunnerState {
    pub running: bool,
    pub current_task_id: Option<String>,
    pub pause_requested: bool,
}

impl Default for TaskRunnerState {
    fn default() -> Self {
        Self {
            running: false,
            current_task_id: None,
            pause_requested: false,
        }
    }
}

// Execute a single task with the assigned agent
#[tauri::command]
pub async fn execute_task(
    db: State<'_, Database>,
    secret_store: State<'_, SecretStore>,
    registry: State<'_, Arc<ProviderRegistry>>,
    task_id: String,
) -> Result<TaskResult, String> {
    let start_time = Instant::now();

    // 1. Get task details
    let (project_id, task_type, title, description, assigned_agent_id, selected_provider_id) = {
        let conn = db.connection();
        conn.query_row(
            "SELECT project_id, task_type, title, description, assigned_agent_id, selected_provider_id
             FROM tasks WHERE id = ?1",
            params![task_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            )),
        ).map_err(|e| e.to_string())?
    };

    // 2. Get agent config
    let agent_config = if let Some(aid) = &assigned_agent_id {
        let conn = db.connection();
        conn.query_row(
            "SELECT name, role, system_prompt, description FROM agents WHERE id = ?1",
            params![aid],
            |row| {
                Ok(AgentConfig {
                    name: row.get(0)?,
                    role: row.get(1)?,
                    system_prompt: row.get(2)?,
                    description: row.get(3)?,
                })
            },
        )
        .ok()
    } else {
        None
    };

    let agent_config = agent_config.unwrap_or_else(|| AgentConfig {
        name: "Default Agent".to_string(),
        role: "coder".to_string(),
        system_prompt:
            "You are a helpful AI coding assistant. Provide clear, well-structured code."
                .to_string(),
        description: None,
    });

    // 3. Get project path
    let project_path: String = {
        let conn = db.connection();
        conn.query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
    };

    // 4. Build context only when the assigned agent can read project files.
    let can_read_files = if let Some(agent_id) = &assigned_agent_id {
        let conn = db.connection();
        conn.query_row(
            "SELECT COALESCE(can_read_files, 0) FROM agent_permissions WHERE agent_id = ?1",
            params![agent_id],
            |row| row.get::<_, i32>(0),
        )
        .map_err(|e| e.to_string())?
            == 1
    } else {
        false
    };

    let keywords = extract_keywords(&title, description.as_deref());
    let task_context = if can_read_files {
        let context_builder = ContextBuilder::new(project_path.clone());
        context_builder
            .build(&task_id, &keywords)
            .unwrap_or_else(|_| crate::context::TaskContext {
                task_id: task_id.clone(),
                project_path: project_path.clone(),
                files: vec![],
                total_size: 0,
            })
    } else {
        crate::context::TaskContext {
            task_id: task_id.clone(),
            project_path: project_path.clone(),
            files: vec![],
            total_size: 0,
        }
    };

    // 5. Build prompt
    let prompt = PromptBuilder::new(
        agent_config,
        task_type.clone(),
        title.clone(),
        description.clone(),
        project_path.clone(),
    )
    .with_context(task_context)
    .build_split();

    // 6. Get provider and call model
    // If no provider was explicitly assigned, fall back to the first enabled provider.
    let provider_id = if let Some(pid) = selected_provider_id {
        pid
    } else {
        {
            let conn = db.connection();
            conn.query_row(
                "SELECT id FROM provider_configs WHERE is_enabled = 1 ORDER BY created_at LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .map_err(|e| format!("No enabled provider configured: {}", e))?
        }
    };
    let adapter = match registry.get(&provider_id) {
        Some(a) => a,
        None => {
            return Ok(TaskResult {
                task_id,
                status: "failed".to_string(),
                output: None,
                error: Some(format!("Provider not found: {}", provider_id)),
                patches: vec![],
                cost: 0.0,
                duration_ms: start_time.elapsed().as_millis() as i64,
            });
        }
    };

    // Get default model from provider
    let model_id = {
        let conn = db.connection();
        conn.query_row(
            "SELECT default_model_id FROM provider_configs WHERE id = ?1",
            params![provider_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "default".to_string())
    };

    let chat_request = ChatRequest {
        model: model_id.clone(),
        messages: vec![
            crate::providers::ChatMessage {
                role: "system".to_string(),
                content: prompt.system_prompt,
                name: None,
            },
            crate::providers::ChatMessage {
                role: "user".to_string(),
                content: prompt.user_prompt,
                name: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(4096),
        top_p: None,
        stream: Some(false),
        tools: None,
    };

    let chat_response = match adapter.chat(chat_request).await {
        Ok(r) => r,
        Err(e) => {
            return Ok(TaskResult {
                task_id,
                status: "failed".to_string(),
                output: None,
                error: Some(format!("Model call failed: {}", e)),
                patches: vec![],
                cost: 0.0,
                duration_ms: start_time.elapsed().as_millis() as i64,
            });
        }
    };

    // 7. Parse output
    let output_text = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    let parse_result = PatchParser::parse(&output_text);

    let duration_ms = start_time.elapsed().as_millis() as i64;
    let cost = calculate_cost_from_usage(chat_response.usage.as_ref(), &task_type);

    let patches: Vec<FilePatch> = parse_result
        .patches
        .iter()
        .map(|p| FilePatch {
            file_path: p.file_path.clone(),
            patch: p.patch.clone(),
            old_hash: None,
            new_hash: None,
        })
        .collect();

    // Persist generated patches as pending file changes so the UI can
    // preview the diff and apply/reject each change.
    {
        let conn = db.connection();
        for parsed in &parse_result.patches {
            let change_id = Uuid::new_v4().to_string();
            let change_type = match parsed.change_type {
                crate::patches::PatchChangeType::Create => "create",
                crate::patches::PatchChangeType::Modify => "modify",
                crate::patches::PatchChangeType::Delete => "delete",
            };
            let _ = conn.execute(
                "INSERT INTO file_changes
                 (id, task_id, file_path, change_type, patch, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
                params![
                    change_id,
                    task_id,
                    parsed.file_path,
                    change_type,
                    parsed.patch,
                    Utc::now().to_rfc3339()
                ],
            );
        }
    }

    // 8. Log model call
    {
        let conn = db.connection();
        let log_id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO model_call_logs
             (id, task_id, agent_id, provider_id, request_summary, response_summary,
              input_tokens, output_tokens, total_tokens, estimated_cost, latency_ms, success, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                log_id,
                task_id,
                assigned_agent_id,
                provider_id,
                format!("Task: {}", title),
                if output_text.len() > 200 { &output_text[..200] } else { &output_text },
                chat_response.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0) as i64,
                chat_response.usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0) as i64,
                chat_response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0) as i64,
                cost,
                duration_ms,
                if parse_result.has_errors { 0 } else { 1 },
                Utc::now().to_rfc3339()
            ],
        );
    }

    let status = if parse_result.has_errors {
        "completed_with_warnings"
    } else {
        "completed"
    };

    Ok(TaskResult {
        task_id,
        status: status.to_string(),
        output: Some(output_text),
        error: parse_result.error_message,
        patches,
        cost,
        duration_ms,
    })
}

fn calculate_cost_from_usage(usage: Option<&crate::providers::TokenUsage>, task_type: &str) -> f64 {
    let base_cost = match task_type {
        "requirement_analysis" => 0.05,
        "architecture_design" => 0.08,
        "frontend_coding" => 0.03,
        "backend_coding" => 0.04,
        "database_design" => 0.03,
        "test_generation" => 0.02,
        "debugging" => 0.04,
        "code_review" => 0.02,
        "security_review" => 0.05,
        "documentation" => 0.01,
        "refactoring" => 0.03,
        "integration" => 0.04,
        _ => 0.02,
    };

    if let Some(u) = usage {
        // Rough estimate: 1 token ≈ $0.00001
        base_cost + (u.total_tokens as f64) * 0.00001
    } else {
        base_cost
    }
}

// Run tasks sequentially with dependencies
#[tauri::command]
pub async fn run_task_sequence(
    db: State<'_, Database>,
    secret_store: State<'_, SecretStore>,
    registry: State<'_, Arc<ProviderRegistry>>,
    project_id: String,
    task_ids: Vec<String>,
) -> Result<Vec<TaskResult>, String> {
    let mut results = Vec::new();

    for task_id in task_ids {
        // Check dependencies
        let deps_completed = {
            let conn = db.connection();
            check_task_dependencies(&conn, &task_id)?
        };

        if !deps_completed {
            results.push(TaskResult {
                task_id: task_id.clone(),
                status: "blocked".to_string(),
                output: None,
                error: Some("Dependencies not completed".to_string()),
                patches: vec![],
                cost: 0.0,
                duration_ms: 0,
            });
            continue;
        }

        // Execute task
        match execute_task(
            db.clone(),
            secret_store.clone(),
            registry.clone(),
            task_id.clone(),
        )
        .await
        {
            Ok(result) => {
                // Update task status
                {
                    let conn = db.connection();
                    let _ = conn.execute(
                        "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
                        params![result.status, Utc::now().to_rfc3339(), task_id],
                    );
                }
                results.push(result);
            }
            Err(e) => {
                results.push(TaskResult {
                    task_id: task_id.clone(),
                    status: "failed".to_string(),
                    output: None,
                    error: Some(e),
                    patches: vec![],
                    cost: 0.0,
                    duration_ms: 0,
                });
            }
        }
    }

    Ok(results)
}

// Check if task dependencies are completed
fn check_task_dependencies(conn: &rusqlite::Connection, task_id: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1")
        .map_err(|e| e.to_string())?;

    let deps: Vec<String> = stmt
        .query_map(params![task_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if deps.is_empty() {
        return Ok(true);
    }

    for dep_id in deps {
        let status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1",
                params![dep_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if status != "completed" && status != "completed_with_warnings" {
            return Ok(false);
        }
    }

    Ok(true)
}

// Add task dependency
#[tauri::command]
pub fn add_task_dependency(
    db: State<Database>,
    task_id: String,
    depends_on_task_id: String,
) -> Result<(), String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();

    if has_circular_dependency(&conn, &task_id, &depends_on_task_id)? {
        return Err("Circular dependency detected".to_string());
    }

    conn.execute(
        "INSERT INTO task_dependencies (id, task_id, depends_on_task_id)
         VALUES (?1, ?2, ?3)",
        params![id, task_id, depends_on_task_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn has_circular_dependency(
    conn: &rusqlite::Connection,
    task_id: &str,
    new_dep: &str,
) -> Result<bool, String> {
    let mut visited = std::collections::HashSet::new();
    let mut stack = vec![new_dep.to_string()];

    while let Some(current) = stack.pop() {
        if current == task_id {
            return Ok(true);
        }
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());

        let mut stmt = conn
            .prepare("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1")
            .map_err(|e| e.to_string())?;

        let deps: Vec<String> = stmt
            .query_map(params![current], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        stack.extend(deps);
    }

    Ok(false)
}

// Get task execution history
#[tauri::command]
pub fn get_task_execution_history(
    db: State<Database>,
    task_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, request_summary, response_summary, input_tokens, output_tokens, total_tokens,
                estimated_cost, latency_ms, success, created_at
         FROM model_call_logs
         WHERE task_id = ?1
         ORDER BY created_at DESC
         LIMIT 50"
    ).map_err(|e| e.to_string())?;

    let history = stmt
        .query_map(params![task_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "request_summary": row.get::<_, Option<String>>(1)?,
                "response_summary": row.get::<_, Option<String>>(2)?,
                "input_tokens": row.get::<_, Option<i64>>(3)?,
                "output_tokens": row.get::<_, Option<i64>>(4)?,
                "total_tokens": row.get::<_, Option<i64>>(5)?,
                "estimated_cost": row.get::<_, Option<f64>>(6)?,
                "latency_ms": row.get::<_, Option<i64>>(7)?,
                "success": row.get::<_, i32>(8)? == 1,
                "created_at": row.get::<_, String>(9)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(history)
}

// Save decision log
#[tauri::command]
pub fn save_decision_log(
    db: State<Database>,
    project_run_id: String,
    decision_type: String,
    decision_summary: String,
    reason: String,
    decided_by: String,
    task_id: Option<String>,
    risk_level: Option<String>,
    selected_provider_id: Option<String>,
    selected_model_profile_id: Option<String>,
    estimated_cost: Option<f64>,
) -> Result<String, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO decision_logs
         (id, project_run_id, task_id, decision_type, decision_summary, reason, decided_by,
          risk_level, selected_model_provider_id, selected_model_profile_id, estimated_cost, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            id,
            project_run_id,
            task_id,
            decision_type,
            decision_summary,
            reason,
            decided_by,
            risk_level,
            selected_provider_id,
            selected_model_profile_id,
            estimated_cost,
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(id)
}

// Get decision logs for a project run
#[tauri::command]
pub fn get_decision_logs(
    db: State<Database>,
    project_run_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, task_id, decision_type, decision_summary, reason, decided_by,
                risk_level, selected_model_provider_id, estimated_cost, created_at
         FROM decision_logs
         WHERE project_run_id = ?1
         ORDER BY created_at DESC
         LIMIT 100",
        )
        .map_err(|e| e.to_string())?;

    let logs = stmt
        .query_map(params![project_run_id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "task_id": row.get::<_, Option<String>>(1)?,
                "decision_type": row.get::<_, String>(2)?,
                "decision_summary": row.get::<_, String>(3)?,
                "reason": row.get::<_, String>(4)?,
                "decided_by": row.get::<_, String>(5)?,
                "risk_level": row.get::<_, Option<String>>(6)?,
                "selected_model_provider_id": row.get::<_, Option<String>>(7)?,
                "estimated_cost": row.get::<_, Option<f64>>(8)?,
                "created_at": row.get::<_, String>(9)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}
