# 模型 Provider 接入规范

## 1. 目标

Provider 模块的目标是让 SuperCompany Coding 可以接入任意模型，而不是只支持固定厂商。

参考 CC Switch / Claude Code Router 类工具的经验，模型接入要分成三层：

1. **Protocol Adapter**：真正处理 API 协议差异。
2. **Provider Preset**：内置厂商默认配置，例如 baseUrl、headers、模型列表、限流建议。
3. **Provider Profile**：用户保存的一组可切换配置，例如 API Key 引用、默认模型、备用模型、预算、代理。

用户应该可以通过以下方式接入模型：

1. 选择内置 Provider。
2. 填写 Base URL + API Key + 模型名。
3. 选择或自动识别能力标签。
4. 测试连接。
5. 将该模型绑定给 Agent。

## 2. Provider 类型

`ProviderType` 表示用户看到的厂商或部署类型，`ProviderProtocol` 表示底层协议。Agent Runtime 和 Router 只依赖 profile 中的能力、价格、限流和协议适配结果，不直接判断厂商品牌。

```ts
export type ProviderProtocol =
  | 'openai_chat_completions'
  | 'anthropic_messages'
  | 'gemini_generate_content'
  | 'custom_http';

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
```

MVP 实现优先级：

| 层级 | MVP 要求 |
|---|---|
| Protocol Adapter | `openai_chat_completions` 必做，`anthropic_messages` / `gemini_generate_content` / `custom_http` 放到 V0.3 |
| Provider Preset | Minimax、DeepSeek、OpenAI-Compatible、Ollama / LM Studio 模板优先 |
| Provider Profile | 用户可创建、测试、切换、导入导出 |
| 厂商专属 Adapter | 仅在协议无法兼容时新增 |

## 3. Provider 配置字段

| 字段 | 必填 | 说明 |
|---|---|---|
| id | 是 | 内部 ID |
| name | 是 | 用户自定义名称 |
| providerType | 是 | Provider 类型 |
| protocol | 是 | 底层协议适配器 |
| presetId | 否 | 内置模板 ID |
| baseUrl | 是 | API 地址 |
| apiKeyRef | 视情况 | Keychain / 加密存储引用，不保存明文 |
| defaultModelId | 是 | 默认模型 ID |
| displayModelName | 否 | 展示名称 |
| contextWindow | 否 | 上下文长度 |
| maxOutputTokens | 否 | 最大输出 |
| supportsStreaming | 否 | 是否支持流式 |
| supportsTools | 否 | 是否支持工具调用 |
| supportsJsonMode | 否 | 是否支持 JSON Mode |
| supportsVision | 否 | 是否支持图片 |
| supportsAudio | 否 | 是否支持音频 |
| supportsVideo | 否 | 是否支持视频 |
| inputPrice | 否 | 输入单价 |
| outputPrice | 否 | 输出单价 |
| maxConcurrency | 否 | 最大并发 |
| rateLimitRpm | 否 | 每分钟请求限制 |
| timeoutMs | 否 | 超时时间 |
| capabilityProfile | 是 | 能力评分 |
| isEnabled | 是 | 是否启用 |
| proxyUrl | 否 | 可选代理 |
| failoverProfileIds | 否 | 失败时可切换的 profile |

## 4. Provider Preset 与 Profile

### 4.1 ProviderPreset

Preset 是产品内置的接入模板，不包含用户密钥。

```ts
export interface ProviderPreset {
  id: string;
  providerType: ProviderType;
  protocol: ProviderProtocol;
  displayName: string;
  defaultBaseUrl: string;
  defaultHeaders?: Record<string, string>;
  modelHints: ModelInfo[];
  defaultCapabilities: Partial<ModelCapability>;
  defaultRateLimit?: RateLimitPolicy;
}
```

例如 Minimax / DeepSeek 在 MVP 阶段优先作为 OpenAI-Compatible preset：

```json
{
  "id": "deepseek-openai-compatible",
  "providerType": "deepseek",
  "protocol": "openai_chat_completions",
  "displayName": "DeepSeek",
  "defaultBaseUrl": "https://api.deepseek.com",
  "modelHints": [
    { "id": "deepseek-chat", "displayName": "DeepSeek Chat" },
    { "id": "deepseek-reasoner", "displayName": "DeepSeek Reasoner" }
  ]
}
```

### 4.2 ProviderProfile

Profile 是用户保存的实际接入配置。

```ts
export interface ProviderProfile {
  id: string;
  name: string;
  presetId?: string;
  providerType: ProviderType;
  protocol: ProviderProtocol;
  baseUrl: string;
  apiKeyRef?: string;
  defaultModelId: string;
  modelProfiles: ModelProfile[];
  proxyUrl?: string;
  failoverProfileIds: string[];
  isEnabled: boolean;
}
```

### 4.3 ModelProfile

能力、价格、上下文长度、工具支持应挂在模型层，而不是只挂在 provider 层。一个 provider profile 可以包含多个模型。

```ts
export interface ModelProfile {
  id: string;
  providerProfileId: string;
  modelId: string;
  displayName?: string;
  capability: ModelCapability;
  contextWindow?: number;
  maxOutputTokens?: number;
  inputPrice?: number;
  outputPrice?: number;
  supportsTools?: boolean;
  supportsJsonMode?: boolean;
  supportsVision?: boolean;
}
```

## 5. OpenAI-Compatible 是最重要的通用入口

大量模型厂商都提供 OpenAI-Compatible API，所以 MVP 必须优先实现该适配器。

用户表单：

```text
Provider 名称
Base URL
API Key
Model Name
```

示例：

```json
{
  "name": "My Custom Coding Model",
  "providerType": "openai_compatible",
  "protocol": "openai_chat_completions",
  "baseUrl": "https://api.example.com/v1",
  "defaultModelId": "custom-coder-pro",
  "apiKey": "sk-..."
}
```

## 6. 适配器接口

```ts
export interface LLMProviderAdapter {
  protocol: ProviderProtocol;
  chat(request: ChatRequest): Promise<ChatResponse>;
  stream?(request: ChatRequest): AsyncIterable<ChatChunk>;
  testConnection(): Promise<ConnectionTestResult>;
  listModels?(): Promise<ModelInfo[]>;
  estimateTokens?(input: string): Promise<number>;
}
```

Adapter 的输入应来自 `ProviderProfile + ModelProfile + ChatRequest`，而不是在 Adapter 内重新读取 UI 状态。

## 7. ChatRequest

```ts
export interface ChatRequest {
  model: string;
  messages: ChatMessage[];
  temperature?: number;
  maxTokens?: number;
  responseFormat?: 'text' | 'json';
  tools?: ToolDefinition[];
  toolChoice?: 'auto' | 'none' | string;
  metadata?: {
    projectId?: string;
    taskId?: string;
    agentId?: string;
  };
}
```

## 8. ChatResponse

```ts
export interface ChatResponse {
  id: string;
  content: string;
  toolCalls?: ToolCall[];
  usage?: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
  };
  finishReason?: string;
  raw?: unknown;
}
```

## 9. 连接测试

连接测试需要验证：

1. Base URL 是否可访问。
2. API Key 是否有效。
3. 模型名是否可用。
4. 是否支持流式。
5. 是否支持工具调用。
6. 是否支持 JSON 输出。

返回结果：

```ts
export interface ConnectionTestResult {
  ok: boolean;
  latencyMs: number;
  modelAvailable: boolean;
  streamingAvailable?: boolean;
  toolsAvailable?: boolean;
  jsonModeAvailable?: boolean;
  errorMessage?: string;
}
```

## 10. Provider UI

### 基础模式

只显示：

- 名称
- Provider preset / 类型
- Base URL
- API Key
- 默认模型
- 测试连接

### 高级模式

显示：

- 能力评分
- 上下文长度
- 单价
- 并发限制
- 超时设置
- 备用模型
- 代理 / 镜像 endpoint
- 自定义 Header
- 自定义 Request Body 映射

## 11. 能力模板

新增 Provider 时，系统根据 Provider 类型自动填默认能力模板。

例如：

```json
{
  "providerType": "deepseek",
  "profilePreset": "coding_reasoning_cost_effective",
  "capability": {
    "reasoning": 5,
    "coding": 5,
    "longContext": 3,
    "speed": 4,
    "lowCost": 4,
    "toolUse": 4,
    "multimodal": 2
  }
}
```

用户可以手动修改评分。

## 12. 自定义 Provider

Custom HTTP Provider 用于接入非标准模型 API。

需要支持：

- 自定义 Header
- 自定义 Body Template
- 自定义 Response Path
- 自定义 Error Path
- 自定义 Token Usage Path

示例：

```json
{
  "headers": {
    "Authorization": "Bearer {{apiKey}}",
    "Content-Type": "application/json"
  },
  "bodyTemplate": {
    "model": "{{modelName}}",
    "messages": "{{messages}}",
    "temperature": "{{temperature}}"
  },
  "responsePath": "choices[0].message.content",
  "usagePath": "usage"
}
```

## 13. 导入导出

支持导出 Provider 配置，但默认不导出 API Key。

导出格式：

```json
{
  "version": "1.0",
  "providers": [
    {
      "name": "DeepSeek Pro",
      "providerType": "deepseek",
      "protocol": "openai_chat_completions",
      "baseUrl": "https://api.deepseek.com",
      "defaultModelId": "deepseek-reasoner",
      "apiKeyRef": null,
      "modelProfiles": []
    }
  ]
}
```

## 14. 错误处理

| 错误 | 处理 |
|---|---|
| API Key 无效 | 提示用户重新填写 |
| 模型名不存在 | 引导用户检查模型列表 |
| Rate Limit | 自动等待或切换备用模型 |
| 余额不足 | 停止任务并提示 |
| JSON 输出失败 | 自动修复或重试 |
| 工具调用不支持 | 降级为文本协议 |
| 网络超时 | 重试，超过次数切备用模型 |

## 15. 接入约束

1. 不要在 Agent Runtime、TaskRunner、UI 页面中写厂商品牌判断。
2. 新厂商优先新增 preset，不优先新增 Adapter。
3. API Key 只保存 `apiKeyRef`，日志和导出配置中不得出现明文。
4. Provider 失败时按 profile 的 `failoverProfileIds` 切换，切换原因写入 `decision_logs`。
5. 所有模型能力使用 `ModelProfile.capability`，Router 不读取厂商品牌作为能力依据。
