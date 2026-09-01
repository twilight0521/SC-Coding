use crate::db::Database;
use crate::patches::PatchParser;
use crate::security::is_sensitive_path;
use chrono::Utc;
use rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileChangeResponse {
    pub id: String,
    pub task_id: String,
    pub file_path: String,
    pub change_type: String,
    pub status: String,
    pub created_at: String,
}

// List pending file changes for a project
#[tauri::command]
pub fn list_pending_file_changes(
    db: State<Database>,
    project_id: String,
) -> Result<Vec<FileChangeResponse>, String> {
    let conn = db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT fc.id, fc.task_id, fc.file_path, fc.change_type, fc.status, fc.created_at
             FROM file_changes fc
             JOIN tasks t ON fc.task_id = t.id
             WHERE t.project_id = ?1 AND fc.status = 'pending'
             ORDER BY fc.created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let changes = stmt
        .query_map(params![project_id], |row| {
            Ok(FileChangeResponse {
                id: row.get(0)?,
                task_id: row.get(1)?,
                file_path: row.get(2)?,
                change_type: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(changes)
}

// Preview a file change (original vs modified) for the Diff viewer
#[tauri::command]
pub fn preview_file_change(
    db: State<Database>,
    change_id: String,
) -> Result<serde_json::Value, String> {
    let (file_path, patch) = get_change(&db, &change_id)?;
    let project_path = get_project_path(&db, &change_id)?;
    let abs = resolve_path(&project_path, &file_path)?;

    let original = if abs.exists() {
        fs::read_to_string(&abs).unwrap_or_default()
    } else {
        String::new()
    };

    let modified = PatchParser::apply_patch(&original, &patch)?;

    Ok(serde_json::json!({
        "file_path": file_path,
        "original": original,
        "modified": modified,
    }))
}

// Apply a pending file change to disk
#[tauri::command]
pub fn apply_file_change(db: State<Database>, change_id: String) -> Result<(), String> {
    let (file_path, patch, change_type, task_id) = get_change_with_type(&db, &change_id)?;
    let project_path = get_project_path(&db, &change_id)?;
    let abs = resolve_path(&project_path, &file_path)?;

    let (can_write, can_delete): (i32, i32) = {
        let conn = db.connection();
        conn.query_row(
            "SELECT COALESCE(ap.can_write_files, 0), COALESCE(ap.can_delete_files, 0)
             FROM tasks t
             LEFT JOIN agent_permissions ap ON ap.agent_id = t.assigned_agent_id
             WHERE t.id = ?1",
            params![task_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?
    };

    let original = if abs.exists() {
        fs::read_to_string(&abs).unwrap_or_default()
    } else {
        String::new()
    };

    match change_type.as_str() {
        "delete" => {
            if can_delete != 1 {
                return Err("Agent does not have permission to delete files".to_string());
            }
            if abs.exists() {
                fs::remove_file(&abs).map_err(|e| e.to_string())?;
            }
        }
        _ => {
            if can_write != 1 {
                return Err("Agent does not have permission to write files".to_string());
            }
            let modified = PatchParser::apply_patch(&original, &patch)?;
            if let Some(parent) = abs.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&abs, modified).map_err(|e| e.to_string())?;
        }
    }

    let conn = db.connection();
    conn.execute(
        "UPDATE file_changes SET status = 'applied', created_at = ?1 WHERE id = ?2",
        params![Utc::now().to_rfc3339(), change_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

// Reject a pending file change (keep it marked, do not write to disk)
#[tauri::command]
pub fn reject_file_change(db: State<Database>, change_id: String) -> Result<(), String> {
    let conn = db.connection();
    let updated = conn
        .execute(
            "UPDATE file_changes SET status = 'rejected'
         WHERE id = ?1 AND status = 'pending'",
            params![change_id],
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        return Err("Pending file change not found".to_string());
    }
    Ok(())
}

// ==================== helpers ====================

fn get_change(db: &Database, change_id: &str) -> Result<(String, String), String> {
    let conn = db.connection();
    conn.query_row(
        "SELECT file_path, patch FROM file_changes
         WHERE id = ?1 AND status = 'pending'",
        params![change_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|e| e.to_string())
}

fn get_change_with_type(
    db: &Database,
    change_id: &str,
) -> Result<(String, String, String, String), String> {
    let conn = db.connection();
    conn.query_row(
        "SELECT file_path, patch, change_type, task_id
         FROM file_changes WHERE id = ?1 AND status = 'pending'",
        params![change_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .map_err(|e| e.to_string())
}

fn get_project_path(db: &Database, change_id: &str) -> Result<String, String> {
    let conn = db.connection();
    conn.query_row(
        "SELECT p.path
         FROM projects p
         JOIN tasks t ON t.project_id = p.id
         JOIN file_changes fc ON fc.task_id = t.id
         WHERE fc.id = ?1",
        params![change_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

/// Resolve a (possibly relative) patch file path against the project root,
/// enforcing that the result stays within the project directory.
fn resolve_path(project_path: &str, file_path: &str) -> Result<PathBuf, String> {
    if file_path.contains("..") {
        return Err("Refusing to apply change outside the project directory".to_string());
    }

    let project = Path::new(project_path);
    let absolute = Path::new(file_path).is_absolute();
    let resolved = if absolute {
        PathBuf::from(file_path)
    } else {
        project.join(file_path)
    };

    if is_sensitive_path(&resolved) {
        return Err("Refusing to apply change to a sensitive path".to_string());
    }

    // Enforce containment against the project root.
    let project_norm = project.canonicalize().map_err(|e| e.to_string())?;
    let containment_path = if resolved.exists() {
        resolved.canonicalize().map_err(|e| e.to_string())?
    } else if let Some(parent) = resolved.parent() {
        parent.canonicalize().map_err(|e| e.to_string())?.join(
            resolved
                .file_name()
                .ok_or_else(|| "Invalid file path".to_string())?,
        )
    } else {
        return Err("Invalid file path".to_string());
    };
    if !containment_path.starts_with(&project_norm) {
        return Err("Refusing to apply change outside the project directory".to_string());
    }

    Ok(resolved)
}
