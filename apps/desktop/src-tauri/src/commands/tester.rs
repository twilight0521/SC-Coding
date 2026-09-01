use crate::db::Database;
use crate::security::{validate_command, validate_command_argument};
use chrono::Utc;
use rusqlite::params;
use std::path::Path;
use std::process::Command;
use tauri::State;
use uuid::Uuid;

// ==================== TESTER COMMANDS ====================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub status: String,
    pub duration_ms: i64,
    pub output: String,
    pub error: Option<String>,
    pub passed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestRun {
    pub run_id: String,
    pub project_id: String,
    pub command: String,
    pub working_directory: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub status: String,
}

// Detect test framework from project structure
fn detect_test_framework(project_path: &str) -> Option<String> {
    let path = Path::new(project_path);

    if path.join("package.json").exists() {
        if path.join("vitest.config.ts").exists() || path.join("vitest.config.js").exists() {
            return Some("vitest".to_string());
        }
        if path.join("jest.config.js").exists() || path.join("jest.config.ts").exists() {
            return Some("jest".to_string());
        }
        if path.join("package.json").exists() {
            if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
                if content.contains("\"test\"")
                    || content.contains("\"vitest\"")
                    || content.contains("\"jest\"")
                {
                    return Some("npm".to_string());
                }
            }
        }
    }

    if path.join("Cargo.toml").exists() {
        return Some("cargo".to_string());
    }

    if path.join("pytest.ini").exists() || path.join("pyproject.toml").exists() {
        return Some("pytest".to_string());
    }

    if path.join("go.mod").exists() {
        return Some("go".to_string());
    }

    None
}

// Build test command based on framework
fn build_test_command(
    framework: &str,
    project_path: &str,
    test_filter: Option<&str>,
) -> (String, String) {
    let working_dir = project_path.to_string();

    let command = match framework {
        "vitest" => {
            if let Some(filter) = test_filter {
                format!("npx vitest run {}", filter)
            } else {
                "npx vitest run".to_string()
            }
        }
        "jest" => {
            if let Some(filter) = test_filter {
                format!("npx jest {}", filter)
            } else {
                "npx jest".to_string()
            }
        }
        "npm" => "npm test -- --passWithNoTests".to_string(),
        "cargo" => {
            if let Some(filter) = test_filter {
                format!("cargo test {}", filter)
            } else {
                "cargo test".to_string()
            }
        }
        "pytest" => {
            if let Some(filter) = test_filter {
                format!("pytest {}", filter)
            } else {
                "pytest".to_string()
            }
        }
        "go" => {
            if let Some(filter) = test_filter {
                format!("go test -v {}", filter)
            } else {
                "go test -v ./...".to_string()
            }
        }
        _ => "echo 'No test framework detected'".to_string(),
    };

    (command, working_dir)
}

// Run tests with specified framework
#[tauri::command]
pub fn run_tests(
    db: State<Database>,
    project_id: String,
    test_filter: Option<String>,
) -> Result<TestResult, String> {
    let conn = db.connection();

    // Get project path
    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Detect test framework
    let framework = detect_test_framework(&project_path)
        .ok_or_else(|| "No test framework detected in project".to_string())?;

    let (command, working_dir) =
        build_test_command(&framework, &project_path, test_filter.as_deref());

    validate_command(&command).map_err(|e| e.to_string())?;
    if let Some(filter) = test_filter.as_deref() {
        validate_command_argument(filter).map_err(|e| e.to_string())?;
    }

    let test_id = Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();

    // Execute test command
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        Command::new("sh")
            .args(["-c", &command])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| e.to_string())?
    };

    let duration_ms = start_time.elapsed().as_millis() as i64;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined_output = if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    let passed = output.status.success();
    let status = if passed { "passed" } else { "failed" };

    // Log test execution
    conn.execute(
        "INSERT INTO test_runs
         (id, project_id, command, working_directory, start_time, end_time, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            test_id,
            project_id,
            command,
            working_dir,
            Utc::now().to_rfc3339(),
            Utc::now().to_rfc3339(),
            status
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(TestResult {
        test_id,
        test_name: test_filter.unwrap_or_else(|| "all tests".to_string()),
        status: status.to_string(),
        duration_ms,
        output: combined_output,
        error: if passed {
            None
        } else {
            Some("Tests failed".to_string())
        },
        passed,
    })
}

// Get test history for a project
#[tauri::command]
pub fn get_test_history(
    db: State<Database>,
    project_id: String,
    limit: Option<i32>,
) -> Result<Vec<serde_json::Value>, String> {
    let conn = db.connection();
    let limit = limit.unwrap_or(50);

    let mut stmt = conn
        .prepare(
            "SELECT id, command, working_directory, start_time, end_time,
                status, passed_tests, failed_tests, total_tests
         FROM test_runs
         WHERE project_id = ?1
         ORDER BY start_time DESC
         LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;

    let history = stmt
        .query_map(params![project_id, limit], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "command": row.get::<_, Option<String>>(1)?,
                "working_directory": row.get::<_, Option<String>>(2)?,
                "start_time": row.get::<_, String>(3)?,
                "end_time": row.get::<_, Option<String>>(4)?,
                "status": row.get::<_, Option<String>>(5)?,
                "passed_tests": row.get::<_, Option<i32>>(6)?,
                "failed_tests": row.get::<_, Option<i32>>(7)?,
                "total_tests": row.get::<_, Option<i32>>(8)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(history)
}

// Get detected test framework info
#[tauri::command]
pub fn get_test_framework_info(
    db: State<Database>,
    project_id: String,
) -> Result<serde_json::Value, String> {
    let conn = db.connection();

    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let framework = detect_test_framework(&project_path)
        .ok_or_else(|| "No test framework detected".to_string())?;

    let (command, _) = build_test_command(&framework, &project_path, None);

    Ok(serde_json::json!({
        "framework": framework,
        "suggested_command": command,
    }))
}

// Quick test single file or function
#[tauri::command]
pub fn quick_test(
    db: State<Database>,
    project_id: String,
    target: String,
) -> Result<TestResult, String> {
    let conn = db.connection();

    let project_path: String = conn
        .query_row(
            "SELECT path FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let framework = detect_test_framework(&project_path)
        .ok_or_else(|| "No test framework detected".to_string())?;

    // Build focused test command
    let (command, working_dir) = match framework.as_str() {
        "vitest" => (format!("npx vitest run {}", target), project_path),
        "jest" => (format!("npx jest {}", target), project_path),
        "cargo" => (format!("cargo test {}", target), project_path),
        "pytest" => (format!("pytest -k {}", target), project_path),
        _ => return Err("Framework does not support quick test".to_string()),
    };

    validate_command(&command).map_err(|e| e.to_string())?;
    validate_command_argument(&target).map_err(|e| e.to_string())?;

    let test_id = Uuid::new_v4().to_string();
    let start_time = std::time::Instant::now();

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        Command::new("sh")
            .args(["-c", &command])
            .current_dir(&working_dir)
            .output()
            .map_err(|e| e.to_string())?
    };

    let duration_ms = start_time.elapsed().as_millis() as i64;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined_output = if stderr.is_empty() {
        stdout.to_string()
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    let passed = output.status.success();

    Ok(TestResult {
        test_id,
        test_name: target,
        status: if passed { "passed" } else { "failed" }.to_string(),
        duration_ms,
        output: combined_output,
        error: if passed {
            None
        } else {
            Some("Quick test failed".to_string())
        },
        passed,
    })
}
