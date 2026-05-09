use serde::{Deserialize, Serialize};
use crate::db::Database;
use tauri::State;
use rusqlite::params;
use chrono::Utc;
use uuid::Uuid;
use std::sync::Mutex;
use std::collections::HashMap;

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
pub fn execute_task(
    db: State<Database>,
    task_id: String,
) -> Result<TaskResult, String> {
    let conn = db.connection();
    let start_time = std::time::Instant::now();

    // Get task details
    let (project_id, task_type, title, description, assigned_agent_id, selected_provider_id) = conn.query_row(
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
    ).map_err(|e| e.to_string())?;

    // Get agent config if assigned
    let agent_config = if let Some(aid) = &assigned_agent_id {
        conn.query_row(
            "SELECT name, role, system_prompt FROM agents WHERE id = ?1",
            params![aid],
            |row| Ok(serde_json::json!({
                "name": row.get::<_, String>(0)?,
                "role": row.get::<_, String>(1)?,
                "system_prompt": row.get::<_, String>(2)?,
            })),
        ).ok()
    } else {
        None
    };

    // Get project path
    let project_path: String = conn.query_row(
        "SELECT path FROM projects WHERE id = ?1",
        params![project_id],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    // For now, simulate task execution
    // In V0.2 full implementation, this would:
    // 1. Build context (relevant files for this task)
    // 2. Assemble prompt with agent config
    // 3. Call the model via provider adapter
    // 4. Parse structured output (patches, reports)
    // 5. Validate output format

    let simulated_output = format!(
        "Task '{}' executed.\nType: {}\nProject: {}\nAgent: {:?}\nProvider: {:?}",
        title, task_type, project_path, agent_config, selected_provider_id
    );

    let duration_ms = start_time.elapsed().as_millis() as i64;
    let cost = calculate_task_cost(&task_type, duration_ms);

    // Log model call
    let log_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO model_call_logs
         (id, task_id, agent_id, provider_id, request_summary, response_summary, estimated_cost, latency_ms, success, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
        params![
            log_id,
            task_id,
            assigned_agent_id,
            selected_provider_id,
            format!("Task: {}", title),
            "Simulated output",
            cost,
            duration_ms,
            Utc::now().to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(TaskResult {
        task_id,
        status: "completed".to_string(),
        output: Some(simulated_output),
        error: None,
        patches: vec![],
        cost,
        duration_ms,
    })
}

// Run tasks sequentially with dependencies
#[tauri::command]
pub fn run_task_sequence(
    db: State<Database>,
    project_id: String,
    task_ids: Vec<String>,
) -> Result<Vec<TaskResult>, String> {
    let conn = db.connection();
    let mut results = Vec::new();

    for task_id in task_ids {
        // Check if task is ready (dependencies completed)
        let deps_completed = check_task_dependencies(&conn, &task_id)?;
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
        match execute_task(db.clone(), task_id.clone()) {
            Ok(result) => {
                // Update task status
                conn.execute(
                    "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![result.status, Utc::now().to_rfc3339(), task_id],
                ).map_err(|e| e.to_string())?;

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
    let mut stmt = conn.prepare(
        "SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1"
    ).map_err(|e| e.to_string())?;

    let deps: Vec<String> = stmt.query_map(params![task_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if deps.is_empty() {
        return Ok(true);
    }

    // Check all dependencies are completed
    for dep_id in deps {
        let status: String = conn.query_row(
            "SELECT status FROM tasks WHERE id = ?1",
            params![dep_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if status != "completed" {
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

    // Check for circular dependency
    if has_circular_dependency(&conn, &task_id, &depends_on_task_id)? {
        return Err("Circular dependency detected".to_string());
    }

    conn.execute(
        "INSERT INTO task_dependencies (id, task_id, depends_on_task_id)
         VALUES (?1, ?2, ?3)",
        params![id, task_id, depends_on_task_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn has_circular_dependency(conn: &rusqlite::Connection, task_id: &str, new_dep: &str) -> Result<bool, String> {
    // Simple DFS to detect cycles
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

        let mut stmt = conn.prepare(
            "SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?1"
        ).map_err(|e| e.to_string())?;

        let deps: Vec<String> = stmt.query_map(params![current], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        stack.extend(deps);
    }

    Ok(false)
}

// Calculate task cost based on type and duration
fn calculate_task_cost(task_type: &str, duration_ms: i64) -> f64 {
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

    // Add time-based cost (rough estimate of token usage)
    let time_factor = duration_ms as f64 / 1000.0;
    base_cost * (1.0 + time_factor * 0.1)
}

// Get task execution history
#[tauri::command]
pub fn get_task_execution_history(
    db: State<Database>,
    task_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, request_summary, response_summary, estimated_cost, latency_ms, success, created_at
         FROM model_call_logs
         WHERE task_id = ?1
         ORDER BY created_at DESC
         LIMIT 50"
    ).map_err(|e| e.to_string())?;

    let history = stmt.query_map(params![task_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "request_summary": row.get::<_, Option<String>>(1)?,
            "response_summary": row.get::<_, Option<String>>(2)?,
            "estimated_cost": row.get::<_, Option<f64>>(3)?,
            "latency_ms": row.get::<_, Option<i64>>(4)?,
            "success": row.get::<_, i32>(5)? == 1,
            "created_at": row.get::<_, String>(6)?,
        }))
    }).map_err(|e| e.to_string())?
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

    let mut stmt = conn.prepare(
        "SELECT id, task_id, decision_type, decision_summary, reason, decided_by,
                risk_level, selected_model_provider_id, estimated_cost, created_at
         FROM decision_logs
         WHERE project_run_id = ?1
         ORDER BY created_at DESC
         LIMIT 100"
    ).map_err(|e| e.to_string())?;

    let logs = stmt.query_map(params![project_run_id], |row| {
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
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(logs)
}