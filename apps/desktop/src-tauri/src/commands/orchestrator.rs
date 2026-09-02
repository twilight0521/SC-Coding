use crate::db::Database;
use chrono::Utc;
use rusqlite::params;
use tauri::State;
use uuid::Uuid;

// ==================== ORCHESTRATOR COMMANDS ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioPlan {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: String,
    pub complexity: String,
    pub estimated_tasks: i32,
    pub estimated_duration_minutes: i32,
    pub agent_team_json: String,
    pub routing_policy_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskBreakdown {
    pub id: String,
    pub scenario_plan_id: String,
    pub original_task_title: String,
    pub subtasks: Vec<Subtask>,
    pub execution_order: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Subtask {
    pub task_id: String,
    pub title: String,
    pub task_type: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub complexity: String,
    pub risk_level: String,
    pub estimated_cost: f64,
    pub suggested_agent_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OrchestratorReport {
    pub id: String,
    pub project_run_id: String,
    pub report_type: String,
    pub title: String,
    pub summary: String,
    pub completed_items: Vec<String>,
    pub current_risks: Vec<String>,
    pub next_actions: Vec<String>,
    pub progress_percent: i32,
    pub used_agents: Vec<String>,
    pub used_models: Vec<String>,
    pub estimated_cost: f64,
    pub actual_cost: f64,
    pub requires_user_decision: bool,
    pub created_at: String,
}

// Create a scenario plan for the project
#[tauri::command]
pub fn create_scenario_plan(
    db: State<Database>,
    project_id: String,
    name: String,
    description: String,
    complexity: String,
) -> Result<ScenarioPlan, String> {
    let conn = db.connection();

    // Calculate estimates based on complexity
    let (estimated_tasks, estimated_duration) = match complexity.as_str() {
        "simple" => (3, 15),
        "medium" => (7, 45),
        "complex" => (15, 120),
        _ => (5, 30),
    };

    // Default agent team
    let agent_team = serde_json::json!([
        { "role": "Architect", "count": 1 },
        { "role": "Coder", "count": 2 },
        { "role": "Tester", "count": 1 }
    ]);

    // Default routing policy
    let routing_policy = serde_json::json!({
        "requirement_analysis": { "prefer_reasoning": true },
        "architecture_design": { "prefer_reasoning": true },
        "frontend_coding": { "prefer_coding": true },
        "backend_coding": { "prefer_coding": true },
        "test_generation": { "prefer_coding": true }
    });

    let plan_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO scenario_plans
         (id, project_id, name, description, complexity, estimated_tasks,
          estimated_duration_minutes, agent_team_json, routing_policy_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            plan_id,
            project_id,
            name,
            description,
            complexity,
            estimated_tasks,
            estimated_duration,
            agent_team.to_string(),
            routing_policy.to_string(),
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(ScenarioPlan {
        id: plan_id,
        project_id,
        name,
        description,
        complexity,
        estimated_tasks,
        estimated_duration_minutes: estimated_duration,
        agent_team_json: agent_team.to_string(),
        routing_policy_json: routing_policy.to_string(),
        created_at: now.to_rfc3339(),
    })
}

// Get scenario plans for a project
#[tauri::command]
pub fn get_scenario_plans(
    db: State<Database>,
    project_id: String,
) -> Result<Vec<ScenarioPlan>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, name, description, complexity, estimated_tasks,
                estimated_duration_minutes, agent_team_json, routing_policy_json, created_at
         FROM scenario_plans
         WHERE project_id = ?1
         ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let plans = stmt
        .query_map(params![project_id], |row| {
            Ok(ScenarioPlan {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                complexity: row.get(4)?,
                estimated_tasks: row.get(5)?,
                estimated_duration_minutes: row.get(6)?,
                agent_team_json: row.get(7)?,
                routing_policy_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(plans)
}

// Break down a high-level task into subtasks
#[tauri::command]
pub fn breakdown_task(
    db: State<Database>,
    scenario_plan_id: String,
    original_task_title: String,
    task_type: String,
    complexity: String,
) -> Result<TaskBreakdown, String> {
    let conn = db.connection();

    // Generate subtasks based on task type and complexity
    let subtasks = generate_subtasks(&task_type, &complexity);

    let breakdown_id = Uuid::new_v4().to_string();
    let execution_order: Vec<String> = subtasks.iter().map(|s| s.task_id.clone()).collect();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO task_breakdowns
         (id, scenario_plan_id, original_task_title, subtasks_json,
          execution_order_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            breakdown_id,
            scenario_plan_id,
            original_task_title,
            serde_json::to_string(&subtasks).unwrap_or_default(),
            serde_json::to_string(&execution_order).unwrap_or_default(),
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(TaskBreakdown {
        id: breakdown_id,
        scenario_plan_id,
        original_task_title,
        subtasks,
        execution_order,
        created_at: now.to_rfc3339(),
    })
}

fn generate_subtasks(task_type: &str, complexity: &str) -> Vec<Subtask> {
    let mut tasks = Vec::new();
    let multiplier = match complexity {
        "simple" => 1,
        "medium" => 2,
        "complex" => 3,
        _ => 2,
    };

    // Base subtasks based on type
    match task_type {
        "requirement_analysis" => {
            let base = vec![
                (
                    "Analyze Requirements",
                    "requirement_analysis",
                    "Analyze and document requirements",
                    "low",
                    "low",
                ),
                (
                    "Identify Stakeholders",
                    "requirement_analysis",
                    "Identify key stakeholders and their needs",
                    "low",
                    "low",
                ),
                (
                    "Create Spec Document",
                    "documentation",
                    "Create detailed specification document",
                    "medium",
                    "medium",
                ),
            ];
            for (i, (title, ttype, desc, risk, cost)) in
                base.into_iter().take(multiplier * 2 + 1).enumerate()
            {
                tasks.push(Subtask {
                    task_id: format!("subtask-{}", i + 1),
                    title: title.to_string(),
                    task_type: ttype.to_string(),
                    description: desc.to_string(),
                    dependencies: vec![],
                    complexity: "medium".to_string(),
                    risk_level: risk.to_string(),
                    estimated_cost: 0.05,
                    suggested_agent_id: None,
                });
            }
        }
        "architecture_design" => {
            tasks.push(Subtask {
                task_id: "subtask-1".to_string(),
                title: "System Overview".to_string(),
                task_type: "architecture_design".to_string(),
                description: "Design high-level system architecture".to_string(),
                dependencies: vec![],
                complexity: "high".to_string(),
                risk_level: "high".to_string(),
                estimated_cost: 0.15,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-2".to_string(),
                title: "Component Design".to_string(),
                task_type: "architecture_design".to_string(),
                description: "Design individual component interfaces".to_string(),
                dependencies: vec!["subtask-1".to_string()],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.10,
                suggested_agent_id: None,
            });
            if multiplier >= 2 {
                tasks.push(Subtask {
                    task_id: "subtask-3".to_string(),
                    title: "Database Schema".to_string(),
                    task_type: "database_design".to_string(),
                    description: "Design database schema and models".to_string(),
                    dependencies: vec!["subtask-1".to_string()],
                    complexity: "medium".to_string(),
                    risk_level: "medium".to_string(),
                    estimated_cost: 0.08,
                    suggested_agent_id: None,
                });
            }
        }
        "frontend_coding" => {
            tasks.push(Subtask {
                task_id: "subtask-1".to_string(),
                title: "Setup Structure".to_string(),
                task_type: "frontend_coding".to_string(),
                description: "Setup project structure and dependencies".to_string(),
                dependencies: vec![],
                complexity: "low".to_string(),
                risk_level: "low".to_string(),
                estimated_cost: 0.02,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-2".to_string(),
                title: "Core Components".to_string(),
                task_type: "frontend_coding".to_string(),
                description: "Implement core UI components".to_string(),
                dependencies: vec!["subtask-1".to_string()],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.10,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-3".to_string(),
                title: "Integration".to_string(),
                task_type: "integration".to_string(),
                description: "Connect to backend API".to_string(),
                dependencies: vec!["subtask-2".to_string()],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.05,
                suggested_agent_id: None,
            });
        }
        "backend_coding" => {
            tasks.push(Subtask {
                task_id: "subtask-1".to_string(),
                title: "API Structure".to_string(),
                task_type: "backend_coding".to_string(),
                description: "Setup API routes and structure".to_string(),
                dependencies: vec![],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.05,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-2".to_string(),
                title: "Business Logic".to_string(),
                task_type: "backend_coding".to_string(),
                description: "Implement business logic".to_string(),
                dependencies: vec!["subtask-1".to_string()],
                complexity: "high".to_string(),
                risk_level: "high".to_string(),
                estimated_cost: 0.12,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-3".to_string(),
                title: "Database Integration".to_string(),
                task_type: "database_design".to_string(),
                description: "Implement database models and queries".to_string(),
                dependencies: vec!["subtask-2".to_string()],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.08,
                suggested_agent_id: None,
            });
        }
        _ => {
            tasks.push(Subtask {
                task_id: "subtask-1".to_string(),
                title: "Analysis".to_string(),
                task_type: task_type.to_string(),
                description: "Analyze the task".to_string(),
                dependencies: vec![],
                complexity: "medium".to_string(),
                risk_level: "low".to_string(),
                estimated_cost: 0.05,
                suggested_agent_id: None,
            });
            tasks.push(Subtask {
                task_id: "subtask-2".to_string(),
                title: "Implementation".to_string(),
                task_type: "frontend_coding".to_string(),
                description: "Implement the solution".to_string(),
                dependencies: vec!["subtask-1".to_string()],
                complexity: "medium".to_string(),
                risk_level: "medium".to_string(),
                estimated_cost: 0.08,
                suggested_agent_id: None,
            });
        }
    }

    // Add test task at the end
    let last_dep = tasks.last().map(|t| t.task_id.clone()).unwrap_or_default();
    tasks.push(Subtask {
        task_id: format!("subtask-{}", tasks.len() + 1),
        title: "Test & Verify".to_string(),
        task_type: "test_generation".to_string(),
        description: "Write and run tests to verify implementation".to_string(),
        dependencies: if last_dep.is_empty() {
            vec![]
        } else {
            vec![last_dep]
        },
        complexity: "medium".to_string(),
        risk_level: "medium".to_string(),
        estimated_cost: 0.04,
        suggested_agent_id: None,
    });

    tasks
}

// Create orchestrator report
#[tauri::command]
pub fn create_orchestrator_report(
    db: State<Database>,
    project_run_id: String,
    report_type: String,
    title: String,
    summary: String,
    completed_items: Vec<String>,
    current_risks: Vec<String>,
    next_actions: Vec<String>,
    progress_percent: i32,
) -> Result<OrchestratorReport, String> {
    let conn = db.connection();

    let report_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO orchestrator_reports
         (id, project_run_id, report_type, title, summary,
          completed_items_json, current_risks_json, next_actions_json,
          progress_percent, requires_user_decision, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            report_id,
            project_run_id,
            report_type,
            title,
            summary,
            serde_json::to_string(&completed_items).unwrap_or_default(),
            serde_json::to_string(&current_risks).unwrap_or_default(),
            serde_json::to_string(&next_actions).unwrap_or_default(),
            progress_percent,
            false,
            now.to_rfc3339()
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(OrchestratorReport {
        id: report_id,
        project_run_id,
        report_type,
        title,
        summary,
        completed_items,
        current_risks,
        next_actions,
        progress_percent,
        used_agents: vec![],
        used_models: vec![],
        estimated_cost: 0.0,
        actual_cost: 0.0,
        requires_user_decision: false,
        created_at: now.to_rfc3339(),
    })
}

// Get orchestrator reports for a project run
#[tauri::command]
pub fn get_orchestrator_reports(
    db: State<Database>,
    project_run_id: String,
) -> Result<Vec<OrchestratorReport>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_run_id, report_type, title, summary,
                COALESCE(completed_items_json, '[]'), COALESCE(current_risks_json, '[]'), COALESCE(next_actions_json, '[]'),
                progress_percent, COALESCE(used_agents_json, '[]'), COALESCE(used_models_json, '[]'),
                estimated_cost, actual_cost, requires_user_decision, created_at
         FROM orchestrator_reports
         WHERE project_run_id = ?1
         ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let reports = stmt
        .query_map(params![project_run_id], |row| {
            let completed_json: String = row.get(5)?;
            let risks_json: String = row.get(6)?;
            let actions_json: String = row.get(7)?;
            let agents_json: String = row.get(9)?;
            let models_json: String = row.get(10)?;

            Ok(OrchestratorReport {
                id: row.get(0)?,
                project_run_id: row.get(1)?,
                report_type: row.get(2)?,
                title: row.get(3)?,
                summary: row.get(4)?,
                completed_items: serde_json::from_str(&completed_json).unwrap_or_default(),
                current_risks: serde_json::from_str(&risks_json).unwrap_or_default(),
                next_actions: serde_json::from_str(&actions_json).unwrap_or_default(),
                progress_percent: row.get(8)?,
                used_agents: serde_json::from_str(&agents_json).unwrap_or_default(),
                used_models: serde_json::from_str(&models_json).unwrap_or_default(),
                estimated_cost: row.get(11)?,
                actual_cost: row.get(12)?,
                requires_user_decision: row.get::<_, i32>(13)? == 1,
                created_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(reports)
}

// Get task breakdown by ID
#[tauri::command]
pub fn get_task_breakdown(
    db: State<Database>,
    breakdown_id: String,
) -> Result<Option<TaskBreakdown>, String> {
    let conn = db.connection();

    let result: Option<(String, String, String, String, String)> = conn
        .query_row(
            "SELECT id, scenario_plan_id, original_task_title, subtasks_json, execution_order_json
         FROM task_breakdowns WHERE id = ?1",
            params![breakdown_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .ok();

    match result {
        Some((id, scenario_plan_id, original_task_title, subtasks_json, execution_order_json)) => {
            let subtasks: Vec<Subtask> = serde_json::from_str(&subtasks_json).unwrap_or_default();
            let execution_order: Vec<String> =
                serde_json::from_str(&execution_order_json).unwrap_or_default();

            Ok(Some(TaskBreakdown {
                id,
                scenario_plan_id,
                original_task_title,
                subtasks,
                execution_order,
                created_at: Utc::now().to_rfc3339(),
            }))
        }
        None => Ok(None),
    }
}
