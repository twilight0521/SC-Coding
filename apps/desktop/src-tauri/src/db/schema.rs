pub const INIT_SQL: &str = r#"
-- Provider configurations
CREATE TABLE IF NOT EXISTS provider_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    protocol TEXT NOT NULL DEFAULT 'openai_chat_completions',
    preset_id TEXT,
    base_url TEXT NOT NULL,
    api_key_ref TEXT,
    default_model_id TEXT NOT NULL,
    display_model_name TEXT,
    max_concurrency INTEGER DEFAULT 1,
    rate_limit_rpm INTEGER,
    timeout_ms INTEGER DEFAULT 120000,
    proxy_url TEXT,
    failover_provider_ids TEXT,
    is_enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Model profiles
CREATE TABLE IF NOT EXISTS model_profiles (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_model_name TEXT,
    context_window INTEGER,
    max_output_tokens INTEGER,
    supports_streaming INTEGER DEFAULT 1,
    supports_tools INTEGER DEFAULT 0,
    supports_json_mode INTEGER DEFAULT 0,
    supports_vision INTEGER DEFAULT 0,
    supports_audio INTEGER DEFAULT 0,
    supports_video INTEGER DEFAULT 0,
    input_price REAL,
    output_price REAL,
    is_default INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(provider_id) REFERENCES provider_configs(id)
);

-- Model capabilities
CREATE TABLE IF NOT EXISTS model_capabilities (
    model_profile_id TEXT PRIMARY KEY,
    reasoning INTEGER DEFAULT 3,
    coding INTEGER DEFAULT 3,
    code_review INTEGER DEFAULT 3,
    long_context INTEGER DEFAULT 3,
    speed INTEGER DEFAULT 3,
    low_cost INTEGER DEFAULT 3,
    tool_use INTEGER DEFAULT 3,
    json_reliability INTEGER DEFAULT 3,
    multimodal INTEGER DEFAULT 0,
    chinese INTEGER DEFAULT 3,
    local_deploy INTEGER DEFAULT 0,
    rag INTEGER DEFAULT 0,
    realtime INTEGER DEFAULT 0,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(model_profile_id) REFERENCES model_profiles(id)
);

-- Agents
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL,
    description TEXT,
    system_prompt TEXT NOT NULL,
    primary_provider_id TEXT,
    primary_model_profile_id TEXT,
    budget_limit REAL,
    max_runtime_ms INTEGER,
    is_enabled INTEGER DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(primary_provider_id) REFERENCES provider_configs(id),
    FOREIGN KEY(primary_model_profile_id) REFERENCES model_profiles(id)
);

-- Agent fallback providers
CREATE TABLE IF NOT EXISTS agent_fallback_providers (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_profile_id TEXT,
    priority INTEGER NOT NULL,
    FOREIGN KEY(agent_id) REFERENCES agents(id),
    FOREIGN KEY(provider_id) REFERENCES provider_configs(id),
    FOREIGN KEY(model_profile_id) REFERENCES model_profiles(id)
);

-- Agent permissions
CREATE TABLE IF NOT EXISTS agent_permissions (
    agent_id TEXT PRIMARY KEY,
    can_read_files INTEGER DEFAULT 1,
    can_write_files INTEGER DEFAULT 0,
    can_execute_commands INTEGER DEFAULT 0,
    can_install_dependencies INTEGER DEFAULT 0,
    can_access_network INTEGER DEFAULT 0,
    can_modify_env_files INTEGER DEFAULT 0,
    can_delete_files INTEGER DEFAULT 0,
    FOREIGN KEY(agent_id) REFERENCES agents(id)
);

-- Projects
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    type TEXT,
    tech_stack TEXT,
    default_team_preset_id TEXT,
    budget_limit REAL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tasks
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    task_type TEXT NOT NULL,
    complexity TEXT DEFAULT 'medium',
    risk_level TEXT DEFAULT 'medium',
    status TEXT DEFAULT 'pending',
    assigned_agent_id TEXT,
    selected_provider_id TEXT,
    selected_model_profile_id TEXT,
    routing_reason TEXT,
    acceptance_criteria TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id),
    FOREIGN KEY(assigned_agent_id) REFERENCES agents(id),
    FOREIGN KEY(selected_provider_id) REFERENCES provider_configs(id),
    FOREIGN KEY(selected_model_profile_id) REFERENCES model_profiles(id)
);

-- Task dependencies
CREATE TABLE IF NOT EXISTS task_dependencies (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    depends_on_task_id TEXT NOT NULL,
    FOREIGN KEY(task_id) REFERENCES tasks(id),
    FOREIGN KEY(depends_on_task_id) REFERENCES tasks(id)
);

-- Model call logs
CREATE TABLE IF NOT EXISTS model_call_logs (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    task_id TEXT,
    agent_id TEXT,
    provider_id TEXT NOT NULL,
    model_profile_id TEXT,
    model_id TEXT,
    request_summary TEXT,
    response_summary TEXT,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    estimated_cost REAL DEFAULT 0,
    latency_ms INTEGER,
    success INTEGER DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(provider_id) REFERENCES provider_configs(id),
    FOREIGN KEY(model_profile_id) REFERENCES model_profiles(id)
);

-- Routing history
CREATE TABLE IF NOT EXISTS routing_history (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    selected_provider_id TEXT NOT NULL,
    selected_model_profile_id TEXT,
    fallback_provider_ids TEXT,
    capability_score REAL,
    cost_score REAL,
    latency_score REAL,
    final_score REAL,
    reason TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(task_id) REFERENCES tasks(id),
    FOREIGN KEY(selected_provider_id) REFERENCES provider_configs(id),
    FOREIGN KEY(selected_model_profile_id) REFERENCES model_profiles(id)
);

-- File changes
CREATE TABLE IF NOT EXISTS file_changes (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    change_type TEXT NOT NULL,
    old_content_hash TEXT,
    new_content_hash TEXT,
    patch TEXT,
    status TEXT DEFAULT 'pending',
    created_at TEXT NOT NULL,
    FOREIGN KEY(task_id) REFERENCES tasks(id)
);

-- Team presets
CREATE TABLE IF NOT EXISTS team_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    config_json TEXT NOT NULL,
    is_default INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Project runs
CREATE TABLE IF NOT EXISTS project_runs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    scenario_plan_id TEXT,
    execution_snapshot_id TEXT,
    status TEXT NOT NULL,
    autonomy_mode TEXT NOT NULL,
    report_cadence TEXT NOT NULL,
    progress_percent INTEGER DEFAULT 0,
    current_phase TEXT,
    budget_limit REAL,
    estimated_cost REAL,
    actual_cost REAL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id)
);

-- Execution snapshots
CREATE TABLE IF NOT EXISTS execution_snapshots (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    scenario_summary TEXT NOT NULL,
    agent_team_plan_json TEXT NOT NULL,
    model_routing_policy_json TEXT NOT NULL,
    autonomy_policy_json TEXT NOT NULL,
    prompt_versions_json TEXT NOT NULL,
    budget_policy_json TEXT,
    security_policy_json TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES projects(id)
);

-- Orchestrator reports
CREATE TABLE IF NOT EXISTS orchestrator_reports (
    id TEXT PRIMARY KEY,
    project_run_id TEXT NOT NULL,
    report_type TEXT NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    completed_items_json TEXT,
    current_risks_json TEXT,
    next_actions_json TEXT,
    progress_percent INTEGER,
    used_agents_json TEXT,
    used_models_json TEXT,
    estimated_cost REAL,
    actual_cost REAL,
    requires_user_decision INTEGER DEFAULT 0,
    approval_request_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY(project_run_id) REFERENCES project_runs(id)
);

-- Approval requests
CREATE TABLE IF NOT EXISTS approval_requests (
    id TEXT PRIMARY KEY,
    project_run_id TEXT NOT NULL,
    title TEXT NOT NULL,
    reason TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    options_json TEXT NOT NULL,
    recommended_option_id TEXT,
    status TEXT NOT NULL,
    selected_option_id TEXT,
    user_comment TEXT,
    created_at TEXT NOT NULL,
    resolved_at TEXT,
    FOREIGN KEY(project_run_id) REFERENCES project_runs(id)
);

-- Decision logs
CREATE TABLE IF NOT EXISTS decision_logs (
    id TEXT PRIMARY KEY,
    project_run_id TEXT NOT NULL,
    task_id TEXT,
    decision_type TEXT NOT NULL,
    decision_summary TEXT NOT NULL,
    reason TEXT NOT NULL,
    decided_by TEXT DEFAULT 'orchestrator',
    alternatives_json TEXT,
    risk_level TEXT,
    affected_agents_json TEXT,
    affected_files_json TEXT,
    selected_model_provider_id TEXT,
    selected_model_profile_id TEXT,
    estimated_cost REAL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(project_run_id) REFERENCES project_runs(id)
);

-- Run events
CREATE TABLE IF NOT EXISTS run_events (
    id TEXT PRIMARY KEY,
    project_run_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    source TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(project_run_id) REFERENCES project_runs(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_model_profiles_provider ON model_profiles(provider_id);
CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX IF NOT EXISTS idx_routing_history_task ON routing_history(task_id, created_at);
CREATE INDEX IF NOT EXISTS idx_model_call_logs_project_created ON model_call_logs(project_id, created_at);
CREATE INDEX IF NOT EXISTS idx_run_events_project_run_created ON run_events(project_run_id, created_at);
CREATE INDEX IF NOT EXISTS idx_decision_logs_project_run_created ON decision_logs(project_run_id, created_at);
"#;