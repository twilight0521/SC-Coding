# SuperCompany Coding 技术架构

## 1. 架构目标

SuperCompany Coding 的架构必须满足：

1. 模型无关。
2. Agent 与模型解耦。
3. Provider 可扩展。
4. 支持本地优先。
5. 支持 Windows / macOS。
6. 支持多 Agent 并行与串行工作流。
7. 支持代码生成、测试、修复闭环。

## 2. 总体架构

参考 opencode 的做法，SuperCompany Coding 不应把桌面 UI、Agent 执行、模型调用、文件工具揉在一个前端进程里。核心应是一个本地优先的 **Project Run Core**，UI 只是客户端。这样后续可以自然支持桌面端、CLI、Web 控制台、IDE 插件，而不重写 Agent Runtime。

```text
Client Layer
  ├─ Desktop UI (Tauri + React)
  ├─ Future CLI
  ├─ Future Web Console
  └─ Future IDE Extension

Local Core API Layer
  ├─ Project Run API
  ├─ Session / Event API
  ├─ Provider / Model API
  ├─ Agent / Task API
  ├─ File / Diff API
  ├─ Tool / Terminal API
  └─ Log / Cost API

Project Run Core
  ├─ ProjectRunService
  ├─ TaskRunner
  ├─ OrchestratorRuntime
  ├─ AgentRuntime
  ├─ ContextBuilder
  ├─ CapabilityRouter
  ├─ ToolExecutor
  ├─ ResultEvaluator
  └─ DebugLoop

Provider Runtime
  ├─ ProviderPresetRegistry
  ├─ ProviderProfileService
  ├─ OpenAICompatibleAdapter
  ├─ AnthropicProtocolAdapter
  ├─ GeminiProtocolAdapter
  ├─ CustomHttpAdapter
  └─ ProviderProxy / FailoverPolicy

Infrastructure
  ├─ SQLite
  ├─ Secret Storage
  ├─ File System
  ├─ Git
  ├─ Terminal Sandbox
  ├─ Token Counter
  └─ Local Logs
```

## 3. 核心设计思想

### 3.1 本地核心服务优先

本地核心服务负责项目状态、任务执行、工具调用、事件流和数据持久化。桌面 UI 通过 Tauri Commands 或本地 HTTP/SSE 与核心通信。关键收益：

1. UI 崩溃不应破坏正在运行的 ProjectRun。
2. 后续新增 CLI / Web / IDE 客户端时复用同一套 Core。
3. 日志、事件、权限、工具调用都从一个入口治理。
4. Agent Runtime 不直接依赖 React 组件或页面状态。

### 3.2 Provider Runtime

Provider 接入参考 CC Switch / CCS 的配置管理思路：**Provider Profile 是用户可切换的模型接入方案，Adapter 只是协议实现**。不要为每个模型厂商都复制完整业务逻辑。

```ts
export interface LLMProviderAdapter {
  protocol: ProviderProtocol;
  chat(request: ChatRequest): Promise<ChatResponse>;
  stream(request: ChatRequest): AsyncIterable<ChatChunk>;
  embed?(request: EmbeddingRequest): Promise<EmbeddingResponse>;
  testConnection(): Promise<ConnectionTestResult>;
  listModels?(): Promise<ModelInfo[]>;
}
```

V0.1 / V0.2 只需要稳定实现：

1. `openai_compatible` 协议适配器。
2. Provider preset / profile / model profile 配置。
3. Minimax、DeepSeek、OpenRouter、Ollama、LM Studio 等作为 preset，而不是一开始都写成独立业务分支。

V0.3 再实现 Anthropic / Gemini / Custom HTTP 等协议适配器。早期可以先保留协议枚举和表结构，不要求 UI 与运行时完整打通。

### 3.3 Agent Runtime

Agent 不关心模型厂商，只关心任务。

```ts
export interface AgentRuntimeInput {
  agent: AgentConfig;
  task: Task;
  projectContext: ProjectContext;
  provider: LLMProviderAdapter;
}
```

### 3.4 Capability Router

Router 根据任务和模型能力决定调用哪个模型。

```text
任务类型 + 成本限制 + 速度要求 + 质量要求 + 用户偏好
→ 候选模型评分
→ 选择主模型
→ 设置备用模型
→ 执行任务
→ 记录结果
```

## 4. 数据流

### 4.0 本地核心事件流

```text
UI 发起操作
→ Local Core API 创建命令
→ ProjectRunService 更新状态
→ TaskRunner / AgentRuntime 执行
→ EventBus 写入 run_events
→ SSE / Tauri Event 推送给 UI
→ UI 按需分页读取详情
```

MVP 事件只记录摘要和引用，不默认保存完整模型上下文。Raw Message 和完整工具调用 trace 仅在 debug 模式或用户显式开启时保存。

### 4.1 用户创建项目

```text
用户输入需求
→ Project Service 创建项目
→ Orchestrator Agent 分析需求
→ Task Planner 生成任务
→ Router 推荐 Agent / Model
→ 用户确认
→ Workflow Service 执行
```

### 4.2 Agent 执行任务

```text
Task
→ Context Builder 收集上下文
→ Router 选择 Provider
→ Agent Runtime 生成 Prompt
→ Provider Adapter 调用模型
→ Result Evaluator 检查结果
→ File System 写入临时变更
→ Diff Viewer 展示
```

### 4.3 测试修复闭环

```text
Integration Agent 合并代码
→ Terminal Runner 执行测试
→ 捕获错误日志
→ Debug Agent 分析错误
→ 修改代码
→ 再次测试
→ 成功后生成报告
```

## 5. 关键模块

### 5.0 ProjectRunService

职责：

- 创建 / 启动 / 暂停 / 取消 ProjectRun
- 管理 run 状态机
- 写入 run_events 和 decision_logs
- 聚合任务进度、成本、风险
- 为 UI 提供 Project Command Center 数据

### 5.1 Provider Service

职责：

- Provider CRUD
- API Key 加密存储
- Provider preset / profile 管理
- Base URL 测试与延迟测速
- 模型列表拉取
- 能力标签管理
- 单价配置
- 调用日志记录
- profile 导入 / 导出，默认不导出 API Key

### 5.2 Agent Service

职责：

- Agent CRUD
- Agent Prompt 管理
- Agent 权限管理
- Agent 与模型绑定
- 默认 Agent 模板
- Team Preset 管理

### 5.3 TaskRunner

职责：

- 任务队列
- 串行 / 并行执行
- 依赖管理
- 失败重试
- 备用模型切换
- 任务状态更新
- Agent 输出落成 patch / command / report

### 5.4 Context Builder

职责：

- 根据任务收集文件上下文
- 排除敏感文件
- 控制 Token 长度
- 生成上下文摘要
- 长上下文模型优先读取大文件
- 短上下文模型只读取必要片段

### 5.5 Result Evaluator

职责：

- 检查模型输出是否符合 JSON Schema
- 检查是否包含可应用 patch
- 检查是否违反安全规则
- 判断是否需要重试
- 判断是否升级到更强模型

## 6. Provider 扩展方式

新增模型厂商时，只需要：

1. 优先判断是否可用现有协议适配器：`openai_compatible` / `anthropic_protocol` / `gemini_protocol` / `custom_http`。
2. 新增 Provider preset，配置默认 baseUrl、headers、模型列表、能力模板和限流策略。
3. 只有协议确实不同，才新增 Adapter。
4. 编写连接测试和最小模型调用测试。
5. 增加 UI preset 表单字段。

不要改 Agent Runtime，不要改 Workflow 主逻辑。

### 6.1 禁止的扩展方式

```ts
if (providerType === 'minimax') {
  // 在 AgentRuntime 或 Router 内写厂商逻辑
}
```

Minimax / DeepSeek / Qwen / Kimi 等默认应是 ProviderPreset + OpenAICompatibleAdapter，除非其 API 协议无法兼容。

## 7. 本地优先架构

默认所有数据保存在本地：

- Provider 配置：SQLite
- API Key：系统 Keychain / 加密存储
- 项目文件：用户本地文件夹
- 执行日志：本地日志
- Token 统计：SQLite
- Agent 配置：SQLite

不默认上传到 SuperCompany 云端。

## 8. 技术风险

| 风险 | 解决方案 |
|---|---|
| 不同模型 API 格式差异 | Adapter 层统一抽象 |
| 模型 JSON 输出不稳定 | Schema 修复器 + 重试策略 |
| 多 Agent 改同一文件冲突 | 临时分支 / patch 队列 / Integration Agent |
| 任务成本失控 | 预算上限 + 低成本模式 + 自动中断 |
| 代码无法运行 | 测试修复闭环 |
| API Key 泄露 | Keychain + 脱敏日志 + 禁止明文导出 |
| 本地命令危险 | 命令分级和用户确认 |

## 9. 复杂度收敛原则

以下能力是长期方向，但不得阻塞 MVP：

| 能力 | MVP 策略 | 后续版本 |
|---|---|---|
| Agent 内部会议 | 只记录摘要型 decision_logs | V1.5 引入 agent_threads / meetings |
| Raw Message 全量追踪 | 默认关闭，只保留摘要与引用 | Debug 模式可开启 |
| Checkpoint 回滚 | 先用 Git diff / patch 队列 | V2 做完整 checkpoint |
| 历史成功率学习 | 先记录数据，不参与复杂评分 | 有样本后启用 |
| 多客户端 | Core API 先留边界 | CLI / Web / IDE 后续实现 |
