// Provider types supported by the system
export type ProviderType =
  | 'openai_compatible'
  | 'openai'
  | 'anthropic'
  | 'gemini'
  | 'minimax'
  | 'deepseek'
  | 'qwen'
  | 'kimi'
  | 'doubao_seed'
  | 'zhipu_glm'
  | 'xai_grok'
  | 'mistral'
  | 'cohere'
  | 'ollama'
  | 'lmstudio'
  | 'vllm'
  | 'custom_http';

// Model capabilities scoring (0-5)
export interface ModelCapability {
  reasoning: number;
  coding: number;
  codeReview: number;
  longContext: number;
  speed: number;
  lowCost: number;
  toolUse: number;
  jsonReliability: number;
  multimodal: number;
  chinese: number;
  localDeploy: number;
  rag: number;
}

// Default capability values
export const DEFAULT_CAPABILITY: ModelCapability = {
  reasoning: 0,
  coding: 0,
  codeReview: 0,
  longContext: 0,
  speed: 0,
  lowCost: 0,
  toolUse: 0,
  jsonReliability: 0,
  multimodal: 0,
  chinese: 0,
  localDeploy: 0,
  rag: 0,
};

// Agent roles
export type AgentRole =
  | 'orchestrator'
  | 'product_manager'
  | 'architect'
  | 'frontend_engineer'
  | 'backend_engineer'
  | 'fullstack_engineer'
  | 'test_engineer'
  | 'debug_engineer'
  | 'security_reviewer'
  | 'code_reviewer'
  | 'integration_engineer'
  | 'document_writer'
  | 'researcher'
  | 'cost_controller';

// Task types
export type TaskType =
  | 'requirement_analysis'
  | 'architecture_design'
  | 'repo_understanding'
  | 'frontend_coding'
  | 'backend_coding'
  | 'database_design'
  | 'test_generation'
  | 'debugging'
  | 'code_review'
  | 'security_review'
  | 'documentation'
  | 'refactoring'
  | 'multimodal_parsing'
  | 'research'
  | 'integration';

// Protocol adapter types
export type ProtocolType =
  | 'openai_chat_completions'
  | 'anthropic_messages'
  | 'gemini_generate_content'
  | 'custom_http';

// Chat message structure
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  name?: string;
  toolCallId?: string;
}

// Chat request
export interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
  tools?: ToolDefinition[];
}

// Chat response
export interface ChatResponse {
  id: string;
  model: string;
  choices: {
    index: number;
    message: ChatMessage;
    finishReason: string;
  }[];
  usage?: {
    promptTokens: number;
    completionTokens: number;
    totalTokens: number;
  };
}

// Tool definition
export interface ToolDefinition {
  type: 'function';
  function: {
    name: string;
    description: string;
    parameters: Record<string, unknown>;
  };
}

// Connection test result
export interface ConnectionTestResult {
  success: boolean;
  latency?: number;
  error?: string;
  modelList?: string[];
}

// Provider preset configuration
export interface ProviderPreset {
  id: string;
  name: string;
  providerType: ProviderType;
  baseUrl: string;
  defaultModel: string;
  supportsStreaming: boolean;
  authenticationType: 'api_key' | 'oauth' | 'none';
}

// Provider profile configuration
export interface ProviderProfile {
  id: string;
  name: string;
  providerType: ProviderType;
  baseUrl: string;
  apiKeyRef: string; // Reference to secure storage, not actual key
  models: ModelProfile[];
  enabled: boolean;
  rateLimit?: {
    requestsPerMinute: number;
    requestsPerDay: number;
  };
  proxyUrl?: string;
}

// Model profile
export interface ModelProfile {
  id: string;
  modelId: string; // Provider's model ID
  name: string;
  capabilities: ModelCapability;
  contextWindow: number;
  inputCostPer1M?: number; // USD per 1M tokens
  outputCostPer1M?: number;
  supportedProtocols: ProtocolType[];
  fallbackModelId?: string;
}

// Agent configuration
export interface AgentConfig {
  id: string;
  name: string;
  role: AgentRole;
  description: string;
  systemPrompt: string;
  providerProfileId: string;
  modelProfileId: string;
  fallbackProviderProfileId?: string;
  fallbackModelProfileId?: string;
  permissions: AgentPermission[];
  maxRetries: number;
  temperature: number;
  maxTokens: number;
}

// Agent permission
export interface AgentPermission {
  action: PermissionAction;
  resource: string;
  conditions?: Record<string, unknown>;
}

export type PermissionAction =
  | 'read'
  | 'write'
  | 'delete'
  | 'execute'
  | 'execute_shell'
  | 'network'
  | 'git_push';