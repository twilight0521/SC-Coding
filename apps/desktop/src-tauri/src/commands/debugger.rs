use crate::db::Database;
use tauri::State;
use rusqlite::params;
use chrono::Utc;
use uuid::Uuid;
use std::path::Path;
use std::process::Command;

// ==================== DEBUGGER COMMANDS ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebugSession {
    pub session_id: String,
    pub task_id: String,
    pub status: String,
    pub current_round: i32,
    pub max_rounds: i32,
    pub fix_history: Vec<FixAttempt>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FixAttempt {
    pub round: i32,
    pub error_message: String,
    pub attempted_fix: String,
    pub files_modified: Vec<String>,
    pub test_result: Option<TestOutcome>,
    pub success: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestOutcome {
    pub passed: bool,
    pub output: String,
    pub duration_ms: i64,
}

// Start a debugging session for a task
#[tauri::command]
pub fn start_debug_session(
    db: State<Database>,
    task_id: String,
    error_description: String,
    max_rounds: Option<i32>,
) -> Result<DebugSession, String> {
    let conn = db.connection();

    // Verify task exists
    let (_project_id, title): (String, String) = conn.query_row(
        "SELECT project_id, title FROM tasks WHERE id = ?1",
        params![task_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    let session_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let max = max_rounds.unwrap_or(3);

    conn.execute(
        "INSERT INTO debug_sessions
         (id, task_id, error_description, status, current_round, max_rounds, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            session_id,
            task_id,
            error_description,
            "active",
            0,
            max,
            now.to_rfc3339()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(DebugSession {
        session_id,
        task_id,
        status: "active".to_string(),
        current_round: 0,
        max_rounds: max,
        fix_history: vec![],
        created_at: now.to_rfc3339(),
    })
}

// Record a fix attempt in the session
#[tauri::command]
pub fn record_fix_attempt(
    db: State<Database>,
    session_id: String,
    error_message: String,
    attempted_fix: String,
    files_modified: Vec<String>,
    test_passed: bool,
    test_output: String,
    test_duration_ms: i64,
) -> Result<DebugSession, String> {
    let conn = db.connection();

    // Get current session state
    let (task_id, current_round, max_rounds, existing_history): (String, i32, i32, String) =
        conn.query_row(
            "SELECT task_id, current_round, max_rounds, fix_history_json
             FROM debug_sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).map_err(|e| e.to_string())?;

    let new_round = current_round + 1;
    let status = if test_passed {
        "success".to_string()
    } else if new_round >= max_rounds {
        "max_rounds_reached".to_string()
    } else {
        "active".to_string()
    };

    let test_result = TestOutcome {
        passed: test_passed,
        output: test_output,
        duration_ms: test_duration_ms,
    };

    let attempt = FixAttempt {
        round: new_round,
        error_message,
        attempted_fix,
        files_modified,
        test_result: Some(test_result),
        success: test_passed,
    };

    // Build updated history
    let mut history: Vec<FixAttempt> = if existing_history.is_empty() {
        vec![]
    } else {
        serde_json::from_str(&existing_history).unwrap_or_default()
    };
    history.push(attempt);

    let history_json = serde_json::to_string(&history).unwrap_or_default();

    conn.execute(
        "UPDATE debug_sessions
         SET current_round = ?1, status = ?2, fix_history_json = ?3
         WHERE id = ?4",
        params![new_round, status, history_json, session_id],
    ).map_err(|e| e.to_string())?;

    Ok(DebugSession {
        session_id,
        task_id,
        status,
        current_round: new_round,
        max_rounds,
        fix_history: history,
        created_at: Utc::now().to_rfc3339(),
    })
}

// Get debug session details
#[tauri::command]
pub fn get_debug_session(
    db: State<Database>,
    session_id: String,
) -> Result<Option<DebugSession>, String> {
    let conn = db.connection();

    let result: Option<(String, String, String, i32, i32, String, String)> =
        conn.query_row(
            "SELECT id, task_id, status, current_round, max_rounds, fix_history_json, created_at
             FROM debug_sessions WHERE id = ?1",
            params![session_id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?
            )),
        ).ok();

    match result {
        Some((id, task_id, status, current_round, max_rounds, history_json, created_at)) => {
            let fix_history: Vec<FixAttempt> = if history_json.is_empty() {
                vec![]
            } else {
                serde_json::from_str(&history_json).unwrap_or_default()
            };

            Ok(Some(DebugSession {
                session_id: id,
                task_id,
                status,
                current_round,
                max_rounds,
                fix_history,
                created_at,
            }))
        }
        None => Ok(None),
    }
}

// Get active debug sessions for a task
#[tauri::command]
pub fn get_active_debug_sessions(
    db: State<Database>,
    task_id: String,
) -> Result<Vec<DebugSession>, String> {
    let conn = db.connection();

    let mut stmt = conn.prepare(
        "SELECT id, task_id, status, current_round, max_rounds, fix_history_json, created_at
         FROM debug_sessions
         WHERE task_id = ?1 AND status = 'active'
         ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;

    let sessions = stmt.query_map(params![task_id], |row| {
        let history_json: String = row.get(5)?;
        let created_at: String = row.get(6)?;
        let fix_history: Vec<FixAttempt> = if history_json.is_empty() {
            vec![]
        } else {
            serde_json::from_str(&history_json).unwrap_or_default()
        };

        Ok(DebugSession {
            session_id: row.get(0)?,
            task_id: row.get(1)?,
            status: row.get(2)?,
            current_round: row.get(3)?,
            max_rounds: row.get(4)?,
            fix_history,
            created_at,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(sessions)
}

// Analyze error from test output
#[tauri::command]
pub fn analyze_error(
    project_id: String,
    error_output: String,
) -> Result<serde_json::Value, String> {
    // Simple pattern-based error analysis
    let analysis = if error_output.contains("SyntaxError") {
        serde_json::json!({
            "error_type": "syntax",
            "likely_cause": "Code has syntax errors",
            "suggestion": "Check the error line for missing brackets, semicolons, or typos"
        })
    } else if error_output.contains("ReferenceError") || error_output.contains("NameError") {
        serde_json::json!({
            "error_type": "reference",
            "likely_cause": "Variable or function not defined",
            "suggestion": "Check if the variable is declared and imported correctly"
        })
    } else if error_output.contains("TypeError") {
        serde_json::json!({
            "error_type": "type",
            "likely_cause": "Incorrect type used",
            "suggestion": "Check the type of variables and function parameters"
        })
    } else if error_output.contains("AssertionError") || error_output.contains("Assertion failed") {
        serde_json::json!({
            "error_type": "assertion",
            "likely_cause": "Test assertion failed",
            "suggestion": "Review test expectations vs actual behavior"
        })
    } else if error_output.contains("connection refused") || error_output.contains("ECONNREFUSED") {
        serde_json::json!({
            "error_type": "network",
            "likely_cause": "Network connection failed",
            "suggestion": "Check if the server is running and the port is correct"
        })
    } else {
        serde_json::json!({
            "error_type": "unknown",
            "likely_cause": "Unknown error",
            "suggestion": "Review the full error output for clues"
        })
    };

    Ok(analysis)
}

// Get debug history for a project
#[tauri::command]
pub fn get_debug_history(
    db: State<Database>,
    project_id: String,
    limit: Option<i32>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();
    let limit = limit.unwrap_or(50);

    let mut stmt = conn.prepare(
        "SELECT ds.id, ds.task_id, ds.status, ds.current_round, ds.max_rounds,
                ds.fix_history_json, ds.created_at, t.title
         FROM debug_sessions ds
         JOIN tasks t ON ds.task_id = t.id
         WHERE t.project_id = ?1
         ORDER BY ds.created_at DESC
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;

    let history = stmt.query_map(params![project_id, limit], |row| {
        let history_json: String = row.get(5)?;
        let created_at: String = row.get(6)?;
        let fix_history: Vec<FixAttempt> = if history_json.is_empty() {
            vec![]
        } else {
            serde_json::from_str(&history_json).unwrap_or_default()
        };

        Ok(serde_json::json!({
            "session_id": row.get::<_, String>(0)?,
            "task_id": row.get::<_, String>(1)?,
            "task_title": row.get::<_, String>(7)?,
            "status": row.get::<_, String>(2)?,
            "current_round": row.get::<_, i32>(3)?,
            "max_rounds": row.get::<_, i32>(4)?,
            "fix_history": fix_history,
            "created_at": created_at,
        }))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(history)
}

// Close a debug session
#[tauri::command]
pub fn close_debug_session(
    db: State<Database>,
    session_id: String,
    resolution: String,
) -> Result<(), String> {
    let conn = db.connection();

    conn.execute(
        "UPDATE debug_sessions
         SET status = ?1, resolution = ?2, completed_at = ?3
         WHERE id = ?4",
        params![
            "closed",
            resolution,
            Utc::now().to_rfc3339(),
            session_id
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
