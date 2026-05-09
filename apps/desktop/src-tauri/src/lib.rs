mod commands;
mod db;
mod models;
mod security;

use db::Database;
use security::SecretStore;
use std::panic;
use std::path::PathBuf;
use tauri::Manager;

fn get_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("supercompany-coding")
        .join("data")
        .join("supercompany.db")
}

fn setup_panic_handler() {
    panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        eprintln!("[PANIC] {} at {}", msg, location);
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_panic_handler();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = get_db_path();

            // Ensure directory exists
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            // Initialize database
            let db = Database::new(db_path)
                .expect("Failed to initialize database");
            db.initialize()
                .expect("Failed to initialize database schema");

            app.manage(db);

            // Initialize secret store
            let secret_store = SecretStore::new()
                .expect("Failed to initialize secret store");
            app.manage(secret_store);

            println!("SuperCompany Coding initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Provider commands
            commands::list_providers,
            commands::get_provider_presets,
            commands::create_provider,
            commands::update_provider,
            commands::delete_provider,
            commands::test_provider_connection,
            // Agent commands
            commands::list_agents,
            commands::get_agent,
            commands::create_agent,
            commands::update_agent,
            commands::delete_agent,
            commands::get_default_agent_templates,
            commands::create_default_agents,
            commands::update_agent_permissions,
            commands::get_agent_permissions,
            // Router commands
            commands::get_task_types,
            commands::route_task,
            commands::save_routing_history,
            commands::get_routing_history,
            commands::get_available_models_for_routing,
            // Project commands
            commands::list_projects,
            commands::get_project,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::list_directory,
            commands::read_file,
            commands::get_file_info,
            commands::list_tasks,
            commands::create_task,
            commands::update_task_status,
            commands::delete_task,
            commands::create_project_run,
            commands::get_project_runs,
            commands::update_project_run_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}