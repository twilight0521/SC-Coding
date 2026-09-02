use crate::commands::task_runner::execute_task;
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
    // Use one canonical entry per phase. The task-type aliases below all map
    // to the same phase; iterating aliases would otherwise enqueue a phase
    // multiple times and execute every task repeatedly.
    let phase_order = vec![
        "1. Analysis",
        "2. Design",
        "3. Implementation",
        "4. Quality Assurance",
        "5. Refinement",
        "6. Documentation",
        "7. Integration",
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
        if let Some(task_ids) = phases_map.get(*phase_name) {
            let dependencies = if prev_phase_tasks.is_empty() {
                vec![]
            } else {
                prev_phase_tasks.clone()
            };

            phases.push(ExecutionPhase {
                phase_name: phase_name.to_string(),
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

    // Reject duplicate starts and invalid lifecycle transitions before any
    // runtime state is created. A failed run may be retried; completed runs
    // must be started by creating a new project run.
    {
        let runtime_state = state.lock().unwrap();
        if runtime_state.active_runs.contains_key(&project_run_id) {
            return Err("Run is already running".to_string());
        }
    }

    let current_status: String = {
        let conn = db.connection();
        conn.query_row(
            "SELECT status FROM project_runs WHERE id = ?1",
            params![project_run_id],
            |row| row.get(0),
        )
        .map_err(|_| "Project run not found".to_string())?
    };
    if !matches!(current_status.as_str(), "created" | "paused" | "failed") {
        return Err(format!(
            "Cannot start run from status '{}'.",
            current_status
        ));
    }

    // Update project_run status to running
    {
        let conn = db.connection();
        let updated = conn
            .execute(
                "UPDATE project_runs
             SET status = 'running',
                 started_at = COALESCE(started_at, ?1),
                 completed_at = NULL,
                 updated_at = ?1
             WHERE id = ?2 AND status IN ('created', 'paused', 'failed')",
                params![now.to_rfc3339(), project_run_id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            return Err("Run state changed before it could be started".to_string());
        }
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
    let scenario_plan_id: Option<String> = {
        let conn = db.connection();
        conn.query_row(
            "SELECT scenario_plan_id FROM project_runs WHERE id = ?1",
            params![project_run_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?
    };
    let plan = match create_execution_plan(db.clone(), project_run_id.clone(), scenario_plan_id) {
        Ok(plan) => plan,
        Err(error) => {
            let _ = mark_run_failed(&db, &state, &project_run_id);
            return Err(error);
        }
    };

    // A resumed run must not execute tasks that already completed before the
    // pause (or after a transient command failure).
    let completed_tasks: std::collections::HashSet<String> = {
        let conn = db.connection();
        let mut stmt = conn
            .prepare("SELECT id FROM tasks WHERE project_id = (SELECT project_id FROM project_runs WHERE id = ?1) AND status IN ('completed', 'completed_with_warnings')")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![project_run_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<std::collections::HashSet<String>, _>>();
        rows.map_err(|e| e.to_string())?
    };
    let total_tasks: Vec<String> = plan
        .phases
        .iter()
        .flat_map(|p| {
            p.task_ids
                .iter()
                .filter(|id| !completed_tasks.contains(*id))
                .cloned()
        })
        .collect();

    let total = total_tasks.len();

    if total == 0 {
        let now = Utc::now();
        db.connection()
            .execute(
                "UPDATE project_runs SET status = 'completed', completed_at = ?1, progress_percent = 100, updated_at = ?1 WHERE id = ?2",
                params![now.to_rfc3339(), project_run_id],
            )
            .map_err(|e| e.to_string())?;
        state.lock().unwrap().active_runs.remove(&project_run_id);
        return Ok("Run completed".to_string());
    }

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
        for task_id in phase
            .task_ids
            .iter()
            .filter(|id| !completed_tasks.contains(*id))
        {
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

            // Enforce explicit task dependencies before executing a task.
            let dependencies_completed: bool = {
                let conn = db.connection();
                conn.query_row(
                    "SELECT NOT EXISTS (
                       SELECT 1 FROM task_dependencies d
                       JOIN tasks dependency ON dependency.id = d.depends_on_task_id
                       WHERE d.task_id = ?1
                         AND dependency.status NOT IN ('completed', 'completed_with_warnings')
                     )",
                    params![task_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?
            };
            if !dependencies_completed {
                let error = format!("Task {} has incomplete dependencies", task_id);
                let _ = mark_run_failed(&db, &state, &project_run_id);
                return Err(error);
            }

            let result = execute_task(
                db.clone(),
                secret_store.clone(),
                registry.clone(),
                task_id.clone(),
            )
            .await;
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    let _ = mark_run_failed(&db, &state, &project_run_id);
                    return Err(error);
                }
            };

            // execute_task reports model/provider failures as a TaskResult so
            // standalone callers can display structured failure details. The
            // orchestrator must still persist that status and fail the run.
            {
                let conn = db.connection();
                if let Err(error) = conn.execute(
                    "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![result.status, Utc::now().to_rfc3339(), task_id],
                ) {
                    let error = error.to_string();
                    let _ = mark_run_failed(&db, &state, &project_run_id);
                    return Err(error);
                }
            }
            if result.status == "failed" {
                let error = result
                    .error
                    .unwrap_or_else(|| format!("Task {} failed", task_id));
                let _ = mark_run_failed(&db, &state, &project_run_id);
                return Err(error);
            }

            // Update progress
            let overall_completed = total_tasks.iter().position(|t| t == task_id).unwrap_or(0) + 1;
            let progress = ((overall_completed as f64 / total as f64) * 100.0) as i32;

            {
                let conn = db.connection();
                let _ = conn.execute(
                    "UPDATE project_runs SET progress_percent = ?1, updated_at = ?2 WHERE id = ?3",
                    params![progress, Utc::now().to_rfc3339(), project_run_id],
                );
            }
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

fn mark_run_failed(
    db: &State<'_, Database>,
    state: &State<'_, Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: &str,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    db.connection()
        .execute(
            "UPDATE project_runs SET status = 'failed', updated_at = ?1 WHERE id = ?2",
            params![now, project_run_id],
        )
        .map_err(|e| e.to_string())?;
    state.lock().unwrap().active_runs.remove(project_run_id);
    Ok(())
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
        let updated = conn
            .execute(
                "UPDATE project_runs SET status = 'paused', updated_at = ?1
             WHERE id = ?2 AND status = 'running'",
                params![now.to_rfc3339(), project_run_id],
            )
            .map_err(|e| e.to_string())?;
        if updated == 0 {
            let status: Option<String> = conn
                .query_row(
                    "SELECT status FROM project_runs WHERE id = ?1",
                    params![project_run_id],
                    |row| row.get(0),
                )
                .ok();
            return Err(match status {
                Some(status) => format!("Cannot pause run from status '{}'.", status),
                None => "Project run not found".to_string(),
            });
        }
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
pub async fn resume_run(
    db: State<'_, Database>,
    secret_store: State<'_, SecretStore>,
    registry: State<'_, Arc<ProviderRegistry>>,
    state: State<'_, Arc<std::sync::Mutex<OrchestratorState>>>,
    project_run_id: String,
) -> Result<String, String> {
    let status: String = {
        let conn = db.connection();
        conn.query_row(
            "SELECT status FROM project_runs WHERE id = ?1",
            params![project_run_id],
            |row| row.get(0),
        )
        .map_err(|_| "Project run not found".to_string())?
    };
    if status != "paused" {
        return Err(format!("Cannot resume run from status '{}'.", status));
    }

    // Runtime state is intentionally best-effort: a paused run must remain
    // resumable after an application restart, when this in-memory entry no
    // longer exists.
    state.lock().unwrap().paused.remove(&project_run_id);

    // Re-enter the same execution path so resume actually performs pending
    // work instead of only changing in-memory status.
    start_run(db, secret_store, registry, state, project_run_id).await
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
