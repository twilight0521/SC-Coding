use crate::commands::task_runner::{execute_task, TaskResult};
use crate::db::Database;
use crate::providers::ProviderRegistry;
use crate::security::SecretStore;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

// ==================== Runtime State ====================

pub struct OrchestratorState {
    pub active_runs: HashMap<String, RunSession>,
    pub paused: HashMap<String, PauseInfo>,
}

pub struct RunSession {
    pub project_run_id: String,
    pub current_phase: String,
    pub progress_percent: i32,
    pub active_tasks: Vec<String>,
    pub pending_approvals: Vec<String>,
}

pub struct PauseInfo {
    pub paused_at: chrono::DateTime<Utc>,
    pub reason: String,
    pub unfinished_tasks: Vec<String>,
    pub pending_decisions: Vec<String>,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            active_runs: HashMap::new(),
            paused: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub project_run_id: String,
    pub scenario_plan_id: Option<String>,
    pub phases: Vec<ExecutionPhase>,
    pub estimated_total_cost: f64,
    pub estimated_duration_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPhase {
    pub phase_name: String,
    pub task_ids: Vec<String>,
    pub dependencies: Vec<String>,
    pub parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub project_run_id: String,
    pub title: String,
    pub reason: String,
    pub risk_level: String,
    pub options: Vec<ApprovalOption>,
    pub recommended_option_id: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub estimated_cost: f64,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    pub option_id: String,
    pub user_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PauseReason {
    UserRequested,
    ApprovalRequired,
    BudgetExceeded,
    ErrorThresholdReached,
    Unknown(String),
}

// ==================== Orchestrator Runtime ====================

/// Create an execution plan from a scenario plan
#[tauri::command]
pub fn create_execution_plan(
    db: State<Database>,
    project_run_id: String,
    scenario_plan_id: Option<String>,
) -> Result<ExecutionPlan, String> {
    let conn = db.connection();

    // Get project_id from project_runs
    let project_id: String = conn
        .query_row(
            "SELECT project_id FROM project_runs WHERE id = ?1",
            params![project_run_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Build phases based on existing tasks
    let mut stmt = conn
        .prepare(
            "SELECT id, task_type, complexity FROM tasks
         WHERE project_id = ?1
         ORDER BY task_type, created_at",
        )
        .map_err(|e| e.to_string())?;

    let tasks: Vec<(String, String, String)> = stmt
        .query_map(params![project_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Organize into phases based on task type
    let mut phases_map: HashMap<String, Vec<String>> = HashMap::new();
    let phase_order = vec![
        "requirement_analysis",
        "architecture_design",
        "repo_understanding",
        "frontend_coding",
        "backend_coding",
        "database_design",
        "test_generation",
        "code_review",
        "debugging",
        "refactoring",
        "security_review",
        "documentation",
        "integration",
    ];

    for (task_id, task_type, _complexity) in &tasks {
        let phase = match task_type.as_str() {
            "requirement_analysis" => "1. Analysis",
            "architecture_design" | "repo_understanding" => "2. Design",
            "frontend_coding" | "backend_coding" | "database_design" => "3. Implementation",
            "test_generation" | "code_review" | "security_review" => "4. Quality Assurance",
            "debugging" | "refactoring" => "5. Refinement",
            "documentation" => "6. Documentation",
            _ => "7. Integration",
        };

        phases_map
            .entry(phase.to_string())
            .or_insert_with(Vec::new)
            .push(task_id.clone());
    }

    let mut phases: Vec<ExecutionPhase> = Vec::new();
    let mut prev_phase_tasks: Vec<String> = Vec::new();

    for phase_name in &phase_order {
        let mapped_phase = match *phase_name {
            "requirement_analysis" => "1. Analysis",
            "architecture_design" | "repo_understanding" => "2. Design",
            "frontend_coding" | "backend_coding" | "database_design" => "3. Implementation",
            "test_generation" | "code_review" | "security_review" => "4. Quality Assurance",
            "debugging" | "refactoring" => "5. Refinement",
            "documentation" => "6. Documentation",
            _ => "7. Integration",
        };

        if let Some(task_ids) = phases_map.get(mapped_phase) {
            let dependencies = if prev_phase_tasks.is_empty() {
                vec![]
            } else {
                prev_phase_tasks.clone()
            };

            phases.push(ExecutionPhase {
                phase_name: mapped_phase.to_string(),
                task_ids: task_ids.clone(),
                dependencies,
                parallel: task_ids.len() > 1,
            });

            prev_phase_tasks = task_ids.clone();
        }
    }

    let estimated_total_cost = phases.iter().flat_map(|p| p.task_ids.iter()).count() as f64 * 0.05;
    let estimated_duration_minutes = (phases.len() as i32) * 10;

    Ok(ExecutionPlan {
        project_run_id,
        scenario_plan_id,
        phases,
        estimated_total_cost,
        estimated_duration_minutes,
    })
}

/// Start a project run - executes the plan
#[tauri::command]
pub async fn start_run(
    db: State<'_, Database>,
    secret_store: State<'_, SecretStore>,
    registry: State<'_, Arc<ProviderRegistry>>,
    state: State<'_, Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
) -> Result<String, String> {
    let now = Utc::now();

    // Update project_run status to running
    {
        let conn = db.connection();
        conn.execute(
            "UPDATE project_runs SET status = 'running', started_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), project_run_id],
        ).map_err(|e| e.to_string())?;
    }

    // Create run session
    {
        let mut state = state.lock().unwrap();
        state.active_runs.insert(
            project_run_id.clone(),
            RunSession {
                project_run_id: project_run_id.clone(),
                current_phase: "Initializing".to_string(),
                progress_percent: 0,
                active_tasks: vec![],
                pending_approvals: vec![],
            },
        );
    }

    // Log decision
    {
        let conn = db.connection();
        let log_id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO decision_logs (id, project_run_id, decision_type, decision_summary, reason, decided_by, created_at)
             VALUES (?1, ?2, 'run_start', 'Project run started', 'User initiated run', 'user', ?3)",
            params![log_id, project_run_id, now.to_rfc3339()],
        );
    }

    // Get plan and execute
    let plan = create_execution_plan(db.clone(), project_run_id.clone(), None)?;

    let total_tasks: Vec<String> = plan
        .phases
        .iter()
        .flat_map(|p| p.task_ids.clone())
        .collect();

    let total = total_tasks.len();

    // Execute tasks phase by phase
    for phase in &plan.phases {
        // Update current phase
        {
            let conn = db.connection();
            let _ = conn.execute(
                "UPDATE project_runs SET current_phase = ?1, updated_at = ?2 WHERE id = ?3",
                params![phase.phase_name, Utc::now().to_rfc3339(), project_run_id],
            );

            // Update state
            let mut state = state.lock().unwrap();
            if let Some(session) = state.active_runs.get_mut(&project_run_id) {
                session.current_phase = phase.phase_name.clone();
            }
        }

        // Execute tasks in this phase
        let mut phase_results = Vec::new();
        for task_id in &phase.task_ids {
            // Check if run was paused
            {
                let state = state.lock().unwrap();
                if state.paused.contains_key(&project_run_id) {
                    return Ok(format!("Run paused during phase {}", phase.phase_name));
                }
                if !state.active_runs.contains_key(&project_run_id) {
                    return Ok("Run stopped".to_string());
                }
            }

            let result = execute_task(
                db.clone(),
                secret_store.clone(),
                registry.clone(),
                task_id.clone(),
            )
            .await?;

            // Update progress
            let completed = phase_results.len() + 1;
            let overall_completed = total_tasks.iter().position(|t| t == task_id).unwrap_or(0) + 1;
            let progress = ((overall_completed as f64 / total as f64) * 100.0) as i32;

            {
                let conn = db.connection();
                let _ = conn.execute(
                    "UPDATE project_runs SET progress_percent = ?1, updated_at = ?2 WHERE id = ?3",
                    params![progress, Utc::now().to_rfc3339(), project_run_id],
                );
            }

            phase_results.push(result);
        }
    }

    // Mark run as completed
    let now = Utc::now();
    {
        let conn = db.connection();
        conn.execute(
            "UPDATE project_runs SET status = 'completed', completed_at = ?1, progress_percent = 100, updated_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), project_run_id],
        ).map_err(|e| e.to_string())?;
    }

    // Remove from active runs
    {
        let mut state = state.lock().unwrap();
        state.active_runs.remove(&project_run_id);
    }

    Ok("Run completed".to_string())
}

/// Pause a project run
#[tauri::command]
pub fn pause_run(
    db: State<Database>,
    state: State<Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
    reason: String,
) -> Result<(), String> {
    let now = Utc::now();

    // Update database
    {
        let conn = db.connection();
        conn.execute(
            "UPDATE project_runs SET status = 'paused', updated_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), project_run_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update state
    {
        let mut state = state.lock().unwrap();

        let unfinished_tasks = if let Some(session) = state.active_runs.get(&project_run_id) {
            session.active_tasks.clone()
        } else {
            vec![]
        };

        state.paused.insert(
            project_run_id.clone(),
            PauseInfo {
                paused_at: now,
                reason: reason.clone(),
                unfinished_tasks,
                pending_decisions: vec![],
            },
        );

        state.active_runs.remove(&project_run_id);
    }

    // Log decision
    {
        let conn = db.connection();
        let log_id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO decision_logs (id, project_run_id, decision_type, decision_summary, reason, decided_by, created_at)
             VALUES (?1, ?2, 'run_paused', 'Project run paused', ?3, 'user', ?4)",
            params![log_id, project_run_id, reason, now.to_rfc3339()],
        );
    }

    Ok(())
}

/// Resume a project run
#[tauri::command]
pub fn resume_run(
    db: State<Database>,
    state: State<Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
) -> Result<(), String> {
    let now = Utc::now();

    // Update database
    {
        let conn = db.connection();
        conn.execute(
            "UPDATE project_runs SET status = 'running', updated_at = ?1 WHERE id = ?2",
            params![now.to_rfc3339(), project_run_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Update state
    {
        let mut state = state.lock().unwrap();
        state.paused.remove(&project_run_id);
        state.active_runs.insert(
            project_run_id.clone(),
            RunSession {
                project_run_id: project_run_id.clone(),
                current_phase: "Resuming".to_string(),
                progress_percent: 0,
                active_tasks: vec![],
                pending_approvals: vec![],
            },
        );
    }

    Ok(())
}

/// Request approval for a risky operation
#[tauri::command]
pub fn request_approval(
    db: State<Database>,
    state: State<Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
    title: String,
    reason: String,
    risk_level: String,
    options: Vec<ApprovalOption>,
    recommended_option_id: Option<String>,
) -> Result<String, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let options_json = serde_json::to_string(&options).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO approval_requests
         (id, project_run_id, title, reason, risk_level, options_json, recommended_option_id, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)",
        params![id, project_run_id, title, reason, risk_level, options_json, recommended_option_id, now.to_rfc3339()],
    ).map_err(|e| e.to_string())?;

    // Pause run
    let _ = pause_run(
        db.clone(),
        state.clone(),
        project_run_id,
        format!("Awaiting approval: {}", title),
    );

    Ok(id)
}

/// Emit an orchestrator report
#[tauri::command]
pub fn emit_report(
    db: State<Database>,
    project_run_id: String,
    report_type: String,
    title: String,
    summary: String,
    completed_items: Vec<String>,
    current_risks: Vec<String>,
    next_actions: Vec<String>,
    progress_percent: i32,
) -> Result<String, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let completed_json = serde_json::to_string(&completed_items).unwrap_or_default();
    let risks_json = serde_json::to_string(&current_risks).unwrap_or_default();
    let actions_json = serde_json::to_string(&next_actions).unwrap_or_default();

    conn.execute(
        "INSERT INTO orchestrator_reports
         (id, project_run_id, report_type, title, summary,
          completed_items_json, current_risks_json, next_actions_json,
          progress_percent, requires_user_decision, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10)",
        params![
            id,
            project_run_id,
            report_type,
            title,
            summary,
            completed_json,
            risks_json,
            actions_json,
            progress_percent,
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// Get current orchestrator state
#[tauri::command]
pub fn get_run_state(
    state: State<Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
) -> Result<serde_json::Value, String> {
    let state = state.lock().unwrap();

    let active = state.active_runs.get(&project_run_id).map(|s| {
        serde_json::json!({
            "project_run_id": s.project_run_id,
            "current_phase": s.current_phase,
            "progress_percent": s.progress_percent,
            "active_tasks": s.active_tasks,
        })
    });

    let paused = state.paused.get(&project_run_id).map(|p| {
        serde_json::json!({
            "paused_at": p.paused_at.to_rfc3339(),
            "reason": p.reason,
            "unfinished_tasks": p.unfinished_tasks,
        })
    });

    Ok(serde_json::json!({
        "active": active,
        "paused": paused,
    }))
}
