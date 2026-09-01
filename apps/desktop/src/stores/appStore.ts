import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

// Types
export interface Provider {
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  default_model_id: string;
  display_model_name?: string;
  is_enabled: boolean;
}

export interface Agent {
  id: string;
  name: string;
  role: string;
  description?: string;
  system_prompt: string;
  primary_provider_id?: string;
  primary_model_profile_id?: string;
  budget_limit?: number;
  max_runtime_ms?: number;
  is_enabled: boolean;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  project_type?: string;
  tech_stack?: string;
  budget_limit?: number;
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: string;
  project_id: string;
  title: string;
  description?: string;
  task_type: string;
  complexity: string;
  risk_level: string;
  status: string;
  assigned_agent_id?: string;
  selected_provider_id?: string;
  routing_reason?: string;
  acceptance_criteria?: string;
  created_at: string;
  updated_at: string;
}

export interface ProjectRun {
  id: string;
  project_id: string;
  status: string;
  progress_percent: number;
  current_phase?: string;
  budget_limit?: number;
  estimated_cost?: number;
  actual_cost?: number;
  started_at?: string;
  completed_at?: string;
  created_at: string;
}

export interface FileNode {
  name: string;
  path: string;
  is_directory: boolean;
  children?: FileNode[];
  size?: number;
}

export interface RoutingDecision {
  provider_id: string;
  model_profile_id: string;
  model_name: string;
  score: number;
  reasoning: string;
  fallback_provider_id?: string;
}

export interface TaskResult {
  task_id: string;
  status: string;
  output?: string;
  error?: string;
  patches: FilePatch[];
  cost: number;
  duration_ms: number;
}

export interface FilePatch {
  file_path: string;
  patch: string;
  old_hash?: string;
  new_hash?: string;
}

export interface DecisionLog {
  id: string;
  project_run_id: string;
  task_id?: string;
  decision_type: string;
  decision_summary: string;
  reason: string;
  decided_by: string;
  alternatives_json?: string;
  risk_level?: string;
  selected_model_provider_id?: string;
  selected_model_profile_id?: string;
  estimated_cost?: number;
  created_at: string;
}

export interface ProviderPreset {
  id: string;
  name: string;
  provider_type: string;
  base_url: string;
  default_model: string;
  supports_streaming: boolean;
}

export interface ProviderTypeOption {
  id: string;
  name: string;
  source: 'preset' | 'existing' | 'fallback';
  default_model?: string;
  base_url?: string;
}

export interface CreateProviderRequest {
  name: string;
  provider_type: string;
  base_url: string;
  api_key?: string;
  default_model_id: string;
  display_model_name?: string;
}

export interface CreateAgentRequest {
  name: string;
  role: string;
  description?: string;
  system_prompt: string;
  primary_provider_id?: string;
  primary_model_profile_id?: string;
  budget_limit?: number;
  max_runtime_ms?: number;
}

export interface CreateProjectRequest {
  name: string;
  path: string;
  project_type?: string;
  tech_stack?: string;
}

export interface CreateTaskRequest {
  project_id: string;
  title: string;
  description?: string;
  task_type: string;
  complexity: string;
  risk_level: string;
  acceptance_criteria?: string;
}

interface AppState {
  // Data
  providers: Provider[];
  agents: Agent[];
  presets: ProviderPreset[];
  projects: Project[];
  currentProject: Project | null;
  tasks: Task[];
  projectRuns: ProjectRun[];
  taskTypes: { id: string; name: string; description: string }[];
  availableModels: Record<string, unknown>[];

  // UI State
  loading: boolean;
  error: string | null;
  selectedFilePath: string | null;
  fileTree: FileNode[];
  taskResults: TaskResult[];
  decisionLogs: DecisionLog[];

  // Derived helpers (computed on demand, not stored)
  getProviderTypes: () => ProviderTypeOption[];

  // Provider actions
  fetchProviders: () => Promise<void>;
  fetchPresets: () => Promise<void>;
  createProvider: (request: CreateProviderRequest) => Promise<Provider>;
  updateProvider: (id: string, updates: Partial<Provider>) => Promise<void>;
  deleteProvider: (id: string) => Promise<void>;
  testConnection: (baseUrl: string, apiKey: string | null, modelId: string) => Promise<{
    success: boolean;
    latency_ms?: number;
    error?: string;
  }>;

  // Agent actions
  fetchAgents: () => Promise<void>;
  createAgent: (request: CreateAgentRequest) => Promise<Agent>;
  updateAgent: (id: string, updates: Partial<Agent>) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  createDefaultAgents: () => Promise<Agent[]>;

  // Project actions
  fetchProjects: () => Promise<void>;
  fetchProject: (id: string) => Promise<void>;
  createProject: (request: CreateProjectRequest) => Promise<Project>;
  updateProject: (id: string, updates: Partial<Project>) => Promise<void>;
  deleteProject: (id: string) => Promise<void>;

  // File actions
  fetchFileTree: (path: string) => Promise<void>;
  readFile: (path: string) => Promise<string>;
  selectFile: (path: string | null) => void;

  // Task actions
  fetchTaskTypes: () => Promise<void>;
  fetchTasks: (projectId: string) => Promise<void>;
  createTask: (request: CreateTaskRequest) => Promise<Task>;
  updateTaskStatus: (id: string, status: string) => Promise<void>;
  deleteTask: (id: string) => Promise<void>;

  // Project Run actions
  fetchProjectRuns: (projectId: string) => Promise<void>;
  createProjectRun: (projectId: string) => Promise<ProjectRun>;
  updateProjectRunStatus: (id: string, updates: Partial<ProjectRun>) => Promise<void>;

  // Routing actions
  fetchAvailableModels: () => Promise<void>;
  routeTask: (taskType: string, complexity: string) => Promise<RoutingDecision[]>;

  // Task Runner actions
  executeTask: (taskId: string) => Promise<TaskResult>;
  runTaskSequence: (projectId: string, taskIds: string[]) => Promise<TaskResult[]>;
  addTaskDependency: (taskId: string, dependsOnTaskId: string) => Promise<void>;
  getTaskExecutionHistory: (taskId: string) => Promise<unknown[]>;
  saveDecisionLog: (
    projectRunId: string,
    decisionType: string,
    decisionSummary: string,
    reason: string,
    decidedBy: string,
    taskId?: string,
    riskLevel?: string,
    selectedProviderId?: string,
    selectedModelProfileId?: string,
    estimatedCost?: number
  ) => Promise<string>;
  getDecisionLogs: (projectRunId: string) => Promise<DecisionLog[]>;
}

export const useAppStore = create<AppState>((set, get) => ({
  providers: [],
  agents: [],
  presets: [],
  projects: [],
  currentProject: null,
  tasks: [],
  projectRuns: [],
  taskTypes: [],
  availableModels: [],
  loading: false,
  error: null,
  selectedFilePath: null,
  fileTree: [],
  taskResults: [],
  decisionLogs: [],

  // Derived: build provider type options from presets + existing providers + a fallback
  getProviderTypes: () => {
    const state = get();
    const seen = new Set<string>();
    const options: ProviderTypeOption[] = [];

    // From presets (preferred — known good defaults)
    for (const preset of state.presets) {
      if (!seen.has(preset.provider_type)) {
        seen.add(preset.provider_type);
        options.push({
          id: preset.provider_type,
          name: preset.name,
          source: 'preset',
          default_model: preset.default_model,
          base_url: preset.base_url,
        });
      }
    }

    // From existing providers (for any types not in presets)
    for (const provider of state.providers) {
      if (!seen.has(provider.provider_type)) {
        seen.add(provider.provider_type);
        options.push({
          id: provider.provider_type,
          name: provider.provider_type,
          source: 'existing',
        });
      }
    }

    // Fallback for OpenAI-compatible
    if (!seen.has('openai_compatible')) {
      options.push({
        id: 'openai_compatible',
        name: 'OpenAI Compatible (Custom)',
        source: 'fallback',
      });
    }

    return options;
  },

  // Provider
  fetchProviders: async () => {
    set({ loading: true, error: null });
    try {
      const providers = await invoke<Provider[]>('list_providers');
      set({ providers, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchPresets: async () => {
    try {
      const presets = await invoke<ProviderPreset[]>('get_provider_presets');
      set({ presets });
    } catch (e) {
      console.error('Failed to fetch presets:', e);
    }
  },

  createProvider: async (request) => {
    set({ loading: true, error: null });
    try {
      const provider = await invoke<Provider>('create_provider', { request });
      set((state) => ({ providers: [...state.providers, provider], loading: false }));
      return provider;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  updateProvider: async (id, updates) => {
    set({ loading: true, error: null });
    try {
      await invoke('update_provider', { id, ...updates });
      set((state) => ({
        providers: state.providers.map((p) => (p.id === id ? { ...p, ...updates } : p)),
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  deleteProvider: async (id) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_provider', { id });
      set((state) => ({ providers: state.providers.filter((p) => p.id !== id), loading: false }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  testConnection: async (baseUrl, apiKey, modelId) => {
    try {
      return await invoke<{ success: boolean; latency_ms?: number; error?: string }>(
        'test_provider_connection', { baseUrl, apiKey, modelId }
      );
    } catch (e) {
      return { success: false, error: String(e) };
    }
  },

  // Agent
  fetchAgents: async () => {
    set({ loading: true, error: null });
    try {
      const agents = await invoke<Agent[]>('list_agents');
      set({ agents, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  createAgent: async (request) => {
    set({ loading: true, error: null });
    try {
      const agent = await invoke<Agent>('create_agent', { request });
      set((state) => ({ agents: [...state.agents, agent], loading: false }));
      return agent;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  updateAgent: async (id, updates) => {
    set({ loading: true, error: null });
    try {
      await invoke('update_agent', { id, ...updates });
      set((state) => ({
        agents: state.agents.map((a) => (a.id === id ? { ...a, ...updates } : a)),
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  deleteAgent: async (id) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_agent', { id });
      set((state) => ({ agents: state.agents.filter((a) => a.id !== id), loading: false }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  createDefaultAgents: async () => {
    set({ loading: true, error: null });
    try {
      const agents = await invoke<Agent[]>('create_default_agents');
      set((state) => ({ agents: [...state.agents, ...agents], loading: false }));
      return agents;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  // Project
  fetchProjects: async () => {
    set({ loading: true, error: null });
    try {
      const projects = await invoke<Project[]>('list_projects');
      set({ projects, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchProject: async (id) => {
    try {
      const project = await invoke<Project>('get_project', { id });
      set({ currentProject: project });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createProject: async (request) => {
    set({ loading: true, error: null });
    try {
      const project = await invoke<Project>('create_project', { request });
      set((state) => ({ projects: [project, ...state.projects], loading: false }));
      return project;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  updateProject: async (id, updates) => {
    set({ loading: true, error: null });
    try {
      await invoke('update_project', { id, ...updates });
      set((state) => ({
        projects: state.projects.map((p) => (p.id === id ? { ...p, ...updates } : p)),
        currentProject: state.currentProject?.id === id ? { ...state.currentProject, ...updates } : state.currentProject,
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  deleteProject: async (id) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_project', { id });
      set((state) => ({
        projects: state.projects.filter((p) => p.id !== id),
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  // File
  fetchFileTree: async (path) => {
    try {
      const fileTree = await invoke<FileNode[]>('list_directory', { path });
      set({ fileTree });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  readFile: async (path) => {
    try {
      return await invoke<string>('read_file', { path });
    } catch (e) {
      throw new Error(String(e));
    }
  },

  selectFile: (path) => set({ selectedFilePath: path }),

  // Task
  fetchTaskTypes: async () => {
    try {
      const taskTypes = await invoke<{ id: string; name: string; description: string }[]>('get_task_types');
      set({ taskTypes });
    } catch (e) {
      console.error('Failed to fetch task types:', e);
    }
  },

  fetchTasks: async (projectId) => {
    try {
      const tasks = await invoke<Task[]>('list_tasks', { projectId });
      set({ tasks });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createTask: async (request) => {
    try {
      const task = await invoke<Task>('create_task', { request });
      set((state) => ({ tasks: [...state.tasks, task] }));
      return task;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  updateTaskStatus: async (id, status) => {
    try {
      await invoke('update_task_status', { id, status });
      set((state) => ({
        tasks: state.tasks.map((t) => (t.id === id ? { ...t, status } : t)),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  deleteTask: async (id) => {
    try {
      await invoke('delete_task', { id });
      set((state) => ({ tasks: state.tasks.filter((t) => t.id !== id) }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  // Project Run
  fetchProjectRuns: async (projectId) => {
    try {
      const projectRuns = await invoke<ProjectRun[]>('get_project_runs', { projectId });
      set({ projectRuns });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createProjectRun: async (projectId) => {
    try {
      const run = await invoke<ProjectRun>('create_project_run', { projectId });
      set((state) => ({ projectRuns: [run, ...state.projectRuns] }));
      return run;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  updateProjectRunStatus: async (id, updates) => {
    try {
      await invoke('update_project_run_status', { id, ...updates });
      set((state) => ({
        projectRuns: state.projectRuns.map((r) => (r.id === id ? { ...r, ...updates } : r)),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  // Routing
  fetchAvailableModels: async () => {
    try {
      const models = await invoke<Record<string, unknown>[]>('get_available_models_for_routing');
      set({ availableModels: models });
    } catch (e) {
      console.error('Failed to fetch models:', e);
    }
  },

  routeTask: async (taskType, complexity) => {
    try {
      return await invoke<RoutingDecision[]>('route_task', {
        input: {
          task_type: taskType,
          task_complexity: complexity,
          required_capabilities: [],
          max_cost: null,
          preferred_speed: 'balanced',
        },
      });
    } catch (e) {
      set({ error: String(e) });
      return [];
    }
  },

  // Task Runner
  executeTask: async (taskId) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<TaskResult>('execute_task', { taskId });
      set((state) => ({ taskResults: [...state.taskResults, result], loading: false }));
      return result;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  runTaskSequence: async (projectId, taskIds) => {
    set({ loading: true, error: null });
    try {
      const results = await invoke<TaskResult[]>('run_task_sequence', { projectId, taskIds });
      set((state) => ({ taskResults: [...state.taskResults, ...results], loading: false }));
      return results;
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  addTaskDependency: async (taskId, dependsOnTaskId) => {
    try {
      await invoke('add_task_dependency', { taskId, dependsOnTaskId });
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  getTaskExecutionHistory: async (taskId) => {
    try {
      return await invoke<unknown[]>('get_task_execution_history', { taskId });
    } catch (e) {
      set({ error: String(e) });
      return [];
    }
  },

  saveDecisionLog: async (
    projectRunId,
    decisionType,
    decisionSummary,
    reason,
    decidedBy,
    taskId,
    riskLevel,
    selectedProviderId,
    selectedModelProfileId,
    estimatedCost
  ) => {
    try {
      const id = await invoke<string>('save_decision_log', {
        projectRunId,
        decisionType,
        decisionSummary,
        reason,
        decidedBy,
        taskId,
        riskLevel,
        selectedProviderId,
        selectedModelProfileId,
        estimatedCost,
      });
      return id;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  getDecisionLogs: async (projectRunId) => {
    try {
      const logs = await invoke<DecisionLog[]>('get_decision_logs', { projectRunId });
      set({ decisionLogs: logs });
      return logs;
    } catch (e) {
      set({ error: String(e) });
      return [];
    }
  },
}));