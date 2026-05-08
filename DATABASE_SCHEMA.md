# 数据库设计

数据库：SQLite

## 1. provider_configs

```sql
CREATE TABLE provider_configs (
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
```

`provider_configs` 表保存用户的 Provider Profile。具体模型能力、价格和上下文长度放在 `model_profiles`，避免一个 provider 下多个模型时数据混乱。

## 1.1 model_profiles

```sql
CREATE TABLE model_profiles (
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
```

## 2. model_capabilities

```sql
CREATE TABLE model_capabilities (
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
```

## 3. agents

```sql
CREATE TABLE agents (
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
```

## 4. agent_fallback_providers

```sql
CREATE TABLE agent_fallback_providers (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  provider_id TEXT NOT NULL,
  model_profile_id TEXT,
  priority INTEGER NOT NULL,
  FOREIGN KEY(agent_id) REFERENCES agents(id),
  FOREIGN KEY(provider_id) REFERENCES provider_configs(id),
  FOREIGN KEY(model_profile_id) REFERENCES model_profiles(id)
);
```

## 5. agent_permissions

```sql
CREATE TABLE agent_permissions (
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
```

## 6. projects

```sql
CREATE TABLE projects (
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
```

## 7. tasks

```sql
CREATE TABLE tasks (
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
```

## 8. task_dependencies

```sql
CREATE TABLE task_dependencies (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  depends_on_task_id TEXT NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(id),
  FOREIGN KEY(depends_on_task_id) REFERENCES tasks(id)
);
```

## 9. model_call_logs

```sql
CREATE TABLE model_call_logs (
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
```

## 10. routing_history

```sql
CREATE TABLE routing_history (
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
```

## 11. file_changes

```sql
CREATE TABLE file_changes (
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
```

## 12. team_presets

```sql
CREATE TABLE team_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  config_json TEXT NOT NULL,
  is_default INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

## 13. settings

```sql
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

---

## 14. 场景驱动 Agent 组织生成器数据表

### 14.1 scenario_plans

```sql
CREATE TABLE scenario_plans (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  user_scenario TEXT NOT NULL,
  product_type TEXT,
  complexity TEXT NOT NULL,
  risk_level TEXT NOT NULL,
  privacy_level TEXT NOT NULL,
  recommended_tech_stack TEXT,
  estimated_agent_count INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 14.2 agent_team_plans

```sql
CREATE TABLE agent_team_plans (
  id TEXT PRIMARY KEY,
  scenario_plan_id TEXT NOT NULL,
  quality_profile TEXT NOT NULL,
  estimated_cost REAL,
  estimated_duration_level TEXT,
  risk_notes TEXT,
  is_confirmed INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 14.3 planned_agents

```sql
CREATE TABLE planned_agents (
  id TEXT PRIMARY KEY,
  agent_team_plan_id TEXT NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL,
  is_required INTEGER DEFAULT 1,
  responsibility TEXT NOT NULL,
  recommended_provider_id TEXT,
  recommended_model_profile_id TEXT,
  recommended_model_name TEXT,
  alternative_provider_ids TEXT,
  recommendation_reason TEXT,
  system_prompt TEXT NOT NULL,
  editable_prompt TEXT NOT NULL,
  permissions_json TEXT NOT NULL,
  budget_limit REAL,
  execution_phase TEXT,
  user_modified INTEGER DEFAULT 0,
  sort_order INTEGER DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 14.4 prompt_versions

```sql
CREATE TABLE prompt_versions (
  id TEXT PRIMARY KEY,
  planned_agent_id TEXT NOT NULL,
  version_number INTEGER NOT NULL,
  prompt_content TEXT NOT NULL,
  change_reason TEXT,
  created_by TEXT NOT NULL,
  created_at TEXT NOT NULL
);
```

---

## 15. project_runs

记录一次完整自主项目交付运行。

```sql
CREATE TABLE project_runs (
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
```

## 16. execution_snapshots

记录用户确认后的执行快照，保证一次运行的 Agent、模型、Prompt、权限、预算固定可追溯。

```sql
CREATE TABLE execution_snapshots (
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
```

## 17. orchestrator_reports

记录主控智能体给用户看的汇报。

```sql
CREATE TABLE orchestrator_reports (
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
```

## 18. approval_requests

记录需要用户决策的关键事项。

```sql
CREATE TABLE approval_requests (
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
```

## 19. decision_logs

记录主控智能体的自动决策，方便回溯。

```sql
CREATE TABLE decision_logs (
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
```

## 20. run_events

记录执行过程中的系统级事件。

```sql
CREATE TABLE run_events (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  source TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(project_run_id) REFERENCES project_runs(id)
);
```

---

## 21. Agent 通信与人类干预数据表

### 21.1 agent_threads

记录任务级或会议级 Agent 讨论线程。

```sql
CREATE TABLE agent_threads (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  thread_type TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT,
  related_task_id TEXT,
  status TEXT NOT NULL DEFAULT 'open',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 21.2 agent_messages

记录 Agent 之间的每条消息。

```sql
CREATE TABLE agent_messages (
  id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL,
  project_run_id TEXT NOT NULL,
  from_agent_id TEXT,
  to_agent_id TEXT,
  message_type TEXT NOT NULL,
  title TEXT,
  content TEXT NOT NULL,
  related_files TEXT,
  risk_level TEXT DEFAULT 'low',
  created_at TEXT NOT NULL
);
```

### 21.3 agent_meetings

记录公司式会议。

```sql
CREATE TABLE agent_meetings (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  meeting_type TEXT NOT NULL,
  title TEXT NOT NULL,
  participants TEXT NOT NULL,
  agenda TEXT,
  conclusion TEXT,
  decisions TEXT,
  created_at TEXT NOT NULL
);
```

### 21.4 checkpoints

记录项目检查点。

```sql
CREATE TABLE checkpoints (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  title TEXT NOT NULL,
  checkpoint_type TEXT NOT NULL,
  file_snapshot_ref TEXT,
  task_state_json TEXT,
  agent_config_json TEXT,
  prompt_versions_json TEXT,
  decision_log_snapshot TEXT,
  cost_snapshot_json TEXT,
  created_at TEXT NOT NULL
);
```

### 21.5 human_interventions

记录用户暂停和微调行为。

```sql
CREATE TABLE human_interventions (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  intervention_type TEXT NOT NULL,
  user_instruction TEXT,
  before_state_json TEXT,
  after_state_json TEXT,
  created_at TEXT NOT NULL
);
```

### 21.6 impact_assessments

记录用户调整后的影响评估。

```sql
CREATE TABLE impact_assessments (
  id TEXT PRIMARY KEY,
  project_run_id TEXT NOT NULL,
  intervention_id TEXT NOT NULL,
  affected_agents TEXT,
  affected_tasks TEXT,
  cost_delta TEXT,
  quality_delta TEXT,
  schedule_delta TEXT,
  rerun_required INTEGER DEFAULT 0,
  summary TEXT NOT NULL,
  created_at TEXT NOT NULL
);
```

---

## 22. 索引与保留策略

MVP 必须创建以下索引，避免 Project Command Center、日志页和任务页在数据量增加后卡顿：

```sql
CREATE INDEX idx_model_profiles_provider ON model_profiles(provider_id);
CREATE INDEX idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX idx_routing_history_task ON routing_history(task_id, created_at);
CREATE INDEX idx_model_call_logs_project_created ON model_call_logs(project_id, created_at);
CREATE INDEX idx_run_events_project_run_created ON run_events(project_run_id, created_at);
CREATE INDEX idx_decision_logs_project_run_created ON decision_logs(project_run_id, created_at);
CREATE INDEX idx_agent_threads_run ON agent_threads(project_run_id, updated_at);
CREATE INDEX idx_agent_messages_thread_created ON agent_messages(thread_id, created_at);
```

默认保留策略：

1. `model_call_logs` 保存输入/输出摘要，不保存完整 prompt。
2. `agent_messages` 默认保存摘要型内容，Raw Message 仅 debug 模式保存。
3. 大文件 patch / checkpoint 文件快照保存为文件引用，不直接塞入 SQLite。
4. 所有 JSON 字段必须有对应 TypeScript 类型，不允许无约束随意扩展。
