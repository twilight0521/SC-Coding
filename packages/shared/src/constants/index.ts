// Protocol type constants
export const PROTOCOL_OPENAI = 'openai_chat_completions';
export const PROTOCOL_ANTHROPIC = 'anthropic_messages';
export const PROTOCOL_GEMINI = 'gemini_generate_content';
export const PROTOCOL_CUSTOM_HTTP = 'custom_http';

// Default agent roles
export const DEFAULT_AGENT_ROLES = [
  'orchestrator',
  'product_manager',
  'architect',
  'frontend_engineer',
  'backend_engineer',
  'fullstack_engineer',
  'test_engineer',
  'debug_engineer',
  'security_reviewer',
  'code_reviewer',
  'integration_engineer',
  'document_writer',
  'researcher',
  'cost_controller',
] as const;

// Task type weights for routing
export const TASK_TYPE_WEIGHTS: Record<string, Record<string, number>> = {
  requirement_analysis: { reasoning: 5, coding: 1, chinese: 3 },
  architecture_design: { reasoning: 5, coding: 2, longContext: 3 },
  frontend_coding: { coding: 5, toolUse: 3, speed: 3 },
  backend_coding: { coding: 5, reasoning: 3, speed: 2 },
  test_generation: { coding: 4, toolUse: 4, jsonReliability: 3 },
  debugging: { reasoning: 5, coding: 4, jsonReliability: 2 },
  code_review: { codeReview: 5, reasoning: 3, coding: 2 },
  documentation: { coding: 2, chinese: 4, speed: 3 },
};

// Dangerous commands that require confirmation
export const DANGEROUS_COMMANDS = [
  'rm -rf',
  'sudo',
  'curl ... | sh',
  'wget ... | sh',
  'chmod -R 777',
  'npm publish',
  'git push --force',
  'ssh',
  'scp',
];

// Sensitive file patterns to exclude from AI context
export const SENSITIVE_PATTERNS = [
  '.env',
  '.env.*',
  '*.pem',
  '*.key',
  'id_rsa',
  'node_modules/',
  'dist/',
  'build/',
  '.git/',
];

// Default provider presets
export const DEFAULT_PRESETS = {
  minimax: {
    id: 'minimax',
    name: 'Minimax',
    providerType: 'minimax',
    baseUrl: 'https://api.minimax.chat/v1',
    defaultModel: 'MiniMax-Text-01',
    supportsStreaming: true,
    authenticationType: 'api_key' as const,
  },
  deepseek: {
    id: 'deepseek',
    name: 'DeepSeek',
    providerType: 'deepseek',
    baseUrl: 'https://api.deepseek.com/v1',
    defaultModel: 'deepseek-chat',
    supportsStreaming: true,
    authenticationType: 'api_key' as const,
  },
  ollama: {
    id: 'ollama',
    name: 'Ollama',
    providerType: 'ollama',
    baseUrl: 'http://localhost:11434/v1',
    defaultModel: 'llama3',
    supportsStreaming: true,
    authenticationType: 'none' as const,
  },
  lmstudio: {
    id: 'lmstudio',
    name: 'LM Studio',
    providerType: 'lmstudio',
    baseUrl: 'http://localhost:1234/v1',
    defaultModel: 'local-model',
    supportsStreaming: true,
    authenticationType: 'none' as const,
  },
};