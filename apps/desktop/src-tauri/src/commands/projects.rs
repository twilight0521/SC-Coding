use serde::{Deserialize, Serialize};
use crate::db::Database;
use tauri::State;
use rusqlite::params;
use chrono::Utc;
use uuid::Uuid;
use std::fs;
use std::path::PathBuf;

// ==================== PROJECT COMMANDS ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub path: String,
    pub project_type: Option<String>,
    pub tech_stack: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_type: Option<String>,
    pub tech_stack: Option<String>,
    pub budget_limit: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

// List all projects
#[tauri::command]
pub fn list_projects(db: State<Database>) -> Result<Vec<ProjectResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, name, path, type, tech_stack, budget_limit, created_at, updated_at
         FROM projects ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let projects = stmt.query_map([], |row| {
        Ok(ProjectResponse {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            project_type: row.get(3)?,
            tech_stack: row.get(4)?,
            budget_limit: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(projects)
}

// Get single project
#[tauri::command]
pub fn get_project(db: State<Database>, id: String) -> Result<ProjectResponse, String> {
    let conn = db.connection();

    conn.query_row(
        "SELECT id, name, path, type, tech_stack, budget_limit, created_at, updated_at
         FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(ProjectResponse {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                project_type: row.get(3)?,
                tech_stack: row.get(4)?,
                budget_limit: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    ).map_err(|e| e.to_string())
}

// Create a new project
#[tauri::command]
pub fn create_project(
    db: State<Database>,
    request: CreateProjectRequest,
) -> Result<ProjectResponse, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Validate path exists
    if !PathBuf::from(&request.path).exists() {
        return Err("Project path does not exist".to_string());
    }

    conn.execute(
        "INSERT INTO projects (id, name, path, type, tech_stack, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
        params![
            id,
            request.name,
            request.path,
            request.project_type,
            request.tech_stack,
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ProjectResponse {
        id,
        name: request.name,
        path: request.path,
        project_type: request.project_type,
        tech_stack: request.tech_stack,
        budget_limit: None,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    })
}

// Update project
#[tauri::command]
pub fn update_project(
    db: State<Database>,
    id: String,
    name: Option<String>,
    project_type: Option<String>,
    tech_stack: Option<String>,
    budget_limit: Option<f64>,
) -> Result<(), String> {
    let conn = db.connection();
    let now = Utc::now();

    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.to_rfc3339())];

    if let Some(n) = name {
        updates.push("name = ?".to_string());
        params_vec.push(Box::new(n));
    }
    if let Some(t) = project_type {
        updates.push("type = ?".to_string());
        params_vec.push(Box::new(t));
    }
    if let Some(ts) = tech_stack {
        updates.push("tech_stack = ?".to_string());
        params_vec.push(Box::new(ts));
    }
    if let Some(bl) = budget_limit {
        updates.push("budget_limit = ?".to_string());
        params_vec.push(Box::new(bl));
    }

    params_vec.push(Box::new(id.clone()));

    let sql = format!("UPDATE projects SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())))
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Delete project
#[tauri::command]
pub fn delete_project(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.connection();

    // Delete related records first
    conn.execute("DELETE FROM tasks WHERE project_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM project_runs WHERE project_id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    // Delete project
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== FILE TREE COMMANDS ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<FileNode>>,
    pub size: Option<u64>,
}

// List directory contents for file tree
#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<FileNode>, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let entries = fs::read_dir(&path)
        .map_err(|e| e.to_string())?;

    let mut nodes = Vec::new();
    let sensitive_patterns = [".env", ".git", "node_modules", "dist", "build", "target", ".DS_Store"];

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip sensitive files/directories
        let is_sensitive = sensitive_patterns.iter().any(|p| file_name.contains(p));
        if is_sensitive {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { None } else { Some(metadata.len()) };

        nodes.push(FileNode {
            name: file_name,
            path: entry.path().to_string_lossy().to_string(),
            is_directory: is_dir,
            children: None,
            size,
        });
    }

    // Sort: directories first, then by name
    nodes.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(nodes)
}

// Read file contents
#[tauri::command]
pub fn read_file(path: String, max_size: Option<u64>) -> Result<String, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
    let size = metadata.len();

    // Limit file size (default 1MB)
    let limit = max_size.unwrap_or(1024 * 1024);
    if size > limit {
        return Err(format!("File too large: {} bytes", size));
    }

    // Check for sensitive extensions
    let sensitive_exts = [".env", ".pem", ".key", ".sqlite", ".db"];
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if sensitive_exts.contains(&format!(".{}", ext).as_str()) {
        return Err("Cannot read sensitive file".to_string());
    }

    fs::read_to_string(&path).map_err(|e| e.to_string())
}

// Get file/directory info
#[tauri::command]
pub fn get_file_info(path: String) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "path": path.to_string_lossy(),
        "is_directory": metadata.is_dir(),
        "size": metadata.len(),
        "is_readonly": metadata.permissions().readonly(),
        "modified": metadata.modified()
            .ok()
            .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339()),
    }))
}

// ==================== TASK COMMANDS ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub task_type: String,
    pub complexity: String,
    pub risk_level: String,
    pub acceptance_criteria: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
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
    pub routing_reason: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// List tasks for a project
#[tauri::command]
pub fn list_tasks(db: State<Database>, project_id: String) -> Result<Vec<TaskResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, description, task_type, complexity, risk_level,
                status, assigned_agent_id, selected_provider_id, routing_reason,
                acceptance_criteria, created_at, updated_at
         FROM tasks WHERE project_id = ?1 ORDER BY created_at"
    ).map_err(|e| e.to_string())?;

    let tasks = stmt.query_map(params![project_id], |row| {
        Ok(TaskResponse {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            task_type: row.get(4)?,
            complexity: row.get(5)?,
            risk_level: row.get(6)?,
            status: row.get(7)?,
            assigned_agent_id: row.get(8)?,
            selected_provider_id: row.get(9)?,
            routing_reason: row.get(10)?,
            acceptance_criteria: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(tasks)
}

// Create a task
#[tauri::command]
pub fn create_task(
    db: State<Database>,
    request: CreateTaskRequest,
) -> Result<TaskResponse, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO tasks
         (id, project_id, title, description, task_type, complexity, risk_level,
          status, acceptance_criteria, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?9, ?9)",
        params![
            id,
            request.project_id,
            request.title,
            request.description,
            request.task_type,
            request.complexity,
            request.risk_level,
            request.acceptance_criteria,
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(TaskResponse {
        id,
        project_id: request.project_id,
        title: request.title,
        description: request.description,
        task_type: request.task_type,
        complexity: request.complexity,
        risk_level: request.risk_level,
        status: "pending".to_string(),
        assigned_agent_id: None,
        selected_provider_id: None,
        routing_reason: None,
        acceptance_criteria: request.acceptance_criteria,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    })
}

// Update task status
#[tauri::command]
pub fn update_task_status(
    db: State<Database>,
    id: String,
    status: String,
    assigned_agent_id: Option<String>,
    selected_provider_id: Option<String>,
    routing_reason: Option<String>,
) -> Result<(), String> {
    let conn = db.connection();
    let now = Utc::now();

    conn.execute(
        "UPDATE tasks SET status = ?1, assigned_agent_id = ?2, selected_provider_id = ?3,
         routing_reason = ?4, updated_at = ?5 WHERE id = ?6",
        params![
            status,
            assigned_agent_id,
            selected_provider_id,
            routing_reason,
            now.to_rfc3339(),
            id
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

// Delete task
#[tauri::command]
pub fn delete_task(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.connection();

    conn.execute("DELETE FROM task_dependencies WHERE task_id = ?1 OR depends_on_task_id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ==================== PROJECT RUN COMMANDS ====================

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRunResponse {
    pub id: String,
    pub project_id: String,
    pub status: String,
    pub progress_percent: i32,
    pub current_phase: Option<String>,
    pub budget_limit: Option<f64>,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

// Create a project run
#[tauri::command]
pub fn create_project_run(
    db: State<Database>,
    project_id: String,
    autonomy_mode: Option<String>,
) -> Result<ProjectRunResponse, String> {
    let conn = db.connection();
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO project_runs
         (id, project_id, status, autonomy_mode, report_cadence, progress_percent, created_at, updated_at)
         VALUES (?1, ?2, 'created', ?3, 'realtime', 0, ?4, ?4)",
        params![
            id,
            project_id,
            autonomy_mode.unwrap_or_else(|| "full".to_string()),
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ProjectRunResponse {
        id,
        project_id,
        status: "created".to_string(),
        progress_percent: 0,
        current_phase: None,
        budget_limit: None,
        estimated_cost: None,
        actual_cost: None,
        started_at: None,
        completed_at: None,
        created_at: now.to_rfc3339(),
    })
}

// Get project runs
#[tauri::command]
pub fn get_project_runs(db: State<Database>, project_id: String) -> Result<Vec<ProjectRunResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, project_id, status, progress_percent, current_phase,
                budget_limit, estimated_cost, actual_cost, started_at, completed_at, created_at
         FROM project_runs WHERE project_id = ?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let runs = stmt.query_map(params![project_id], |row| {
        Ok(ProjectRunResponse {
            id: row.get(0)?,
            project_id: row.get(1)?,
            status: row.get(2)?,
            progress_percent: row.get(3)?,
            current_phase: row.get(4)?,
            budget_limit: row.get(5)?,
            estimated_cost: row.get(6)?,
            actual_cost: row.get(7)?,
            started_at: row.get(8)?,
            completed_at: row.get(9)?,
            created_at: row.get(10)?,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(runs)
}

// Update project run status
#[tauri::command]
pub fn update_project_run_status(
    db: State<Database>,
    id: String,
    status: Option<String>,
    progress_percent: Option<i32>,
    current_phase: Option<String>,
) -> Result<(), String> {
    let conn = db.connection();
    let now = Utc::now();

    let mut updates = vec!["updated_at = ?1".to_string()];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.to_rfc3339())];

    if let Some(s) = status {
        updates.push("status = ?".to_string());
        params_vec.push(Box::new(s));
    }
    if let Some(p) = progress_percent {
        updates.push("progress_percent = ?".to_string());
        params_vec.push(Box::new(p));
    }
    if let Some(c) = current_phase {
        updates.push("current_phase = ?".to_string());
        params_vec.push(Box::new(c));
    }

    params_vec.push(Box::new(id.clone()));

    let sql = format!("UPDATE project_runs SET {} WHERE id = ?", updates.join(", "));

    conn.execute(&sql, rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())))
        .map_err(|e| e.to_string())?;

    Ok(())
}