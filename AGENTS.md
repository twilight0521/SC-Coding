# AGENTS.md — SuperCompany Coding 开发执行规范

本文档用于指导 Codex / Minimax M2.7 / DeepSeek / 其他 AI Coding Agent 开发本项目。

## 1. 项目目标

开发一个 Windows / macOS 桌面 App：**SuperCompany Coding**。

核心能力：

1. 用户可以配置任意模型 Provider。
2. 用户可以创建多个 Agent。
3. 用户可以给每个 Agent 指定不同模型。
4. 系统可以根据任务类型自动推荐模型。
5. 系统可以多 Agent 协同完成软件开发任务。
6. 系统可以完成代码生成、Diff、测试、修复、文档输出。

## 2. 不可违背的产品原则

### 2.1 模型无关

代码中禁止出现以下错误设计：

```ts
if (model === 'minimax') { ... } else if (model === 'deepseek') { ... }
```

除非是在具体 Adapter 内部。

正确设计：

```ts
interface LLMProviderAdapter {
  id: string;
  type: ProviderType;
  capabilities: ModelCapability[];
  chat(request: ChatRequest): Promise<ChatResponse>;
  stream(request: ChatRequest): AsyncIterable<ChatChunk>;
  testConnection(): Promise<ConnectionTestResult>;
}
```

所有 Agent 只依赖统一接口，不直接依赖厂商。

### 2.2 当前默认支持 Minimax / DeepSeek，但不能写死

MVP 优先实现：

- OpenAI-Compatible Protocol Adapter
- Provider Preset / Provider Profile / Model Profile
- Minimax Preset
- DeepSeek Preset
- Ollama / LM Studio Preset

Minimax / DeepSeek 在 MVP 阶段优先作为 OpenAI-Compatible Preset 接入，不要一开始写成业务层专属分支。架构必须允许继续添加：

- Anthropic Protocol Adapter
- Gemini Protocol Adapter
- Qwen
- Kimi
- Doubao / Seed
- GLM
- Grok
- Mistral
- Cohere
- Ollama
- LM Studio
- vLLM
- Custom HTTP

### 2.3 Agent 与模型解耦

Agent 定义职责，模型提供能力。

错误设计：

```ts
class DeepSeekBackendAgent {}
class MinimaxOrchestratorAgent {}
```

正确设计：

```ts
class AgentRuntime {
  constructor(agentConfig: AgentConfig, provider: LLMProviderAdapter) {}
}
```

## 3. 推荐技术栈

| 模块 | 技术 |
|---|---|
| 桌面框架 | Tauri |
| 前端 | React + TypeScript |
| UI | Tailwind CSS + shadcn/ui |
| 编辑器 | Monaco Editor |
| 终端 | xterm.js |
| 状态管理 | Zustand |
| 数据库 | SQLite |
| 本地后端 | Rust / Tauri Commands |
| 密钥存储 | OS Keychain 优先，fallback 为本地加密 |
| Git | simple-git 或 Rust git2 |
| Diff | diff-match-patch / Monaco Diff Editor |

## 4. 项目目录建议

```text
supercompany-coding/
  apps/
    desktop/
      src/
        app/
        components/
        pages/
        stores/
        services/
        types/
      src-tauri/
        src/
          commands/
          security/
          fs/
          terminal/
          git/
          db/
  packages/
    core/
      src/
        agents/
        router/
        providers/
        workflow/
        tasks/
        context/
        cost/
    shared/
      src/
        types/
        constants/
        utils/
  docs/
    PRD.md
    ARCHITECTURE.md
    AGENT_SPEC.md
```

## 5. 核心模块开发顺序

必须按以下顺序推进：

### Phase 1：基础工程

1. 初始化 Tauri + React + TypeScript。
2. 配置 Tailwind。
3. 配置 SQLite。
4. 搭建基础布局。
5. 实现本地配置存储。

### Phase 2：Provider 系统

1. 定义 `LLMProviderAdapter` 接口。
2. 定义 ProviderPreset / ProviderProfile / ModelProfile。
3. 实现 API Key 安全存储。
4. 实现 OpenAI-Compatible Protocol Adapter。
5. 实现 Minimax / DeepSeek / Ollama / LM Studio preset。
6. 实现 Provider Profile CRUD。
7. 实现连接测试。
8. 实现模型能力标签。

### Phase 3：Agent 系统

1. 定义 Agent 数据结构。
2. 实现 Agent CRUD。
3. 实现 Agent 绑定模型。
4. 实现默认 Agent 模板。
5. 实现 Agent Runtime。

### Phase 4：Router 系统

1. 定义任务类型。
2. 定义模型能力评分。
3. 实现推荐模型算法。
4. 实现用户手动覆盖。
5. 实现失败降级。

### Phase 5：项目工作区

1. 创建项目。
2. 打开本地文件夹。
3. 文件树。
4. 代码编辑器。
5. Diff Viewer。
6. Git 状态展示。

### Phase 6：多 Agent 工作流

1. 需求输入。
2. Orchestrator 任务拆解。
3. 用户确认任务。
4. Coder Agent 写代码。
5. Tester Agent 测试。
6. Debug Agent 修复。
7. Doc Agent 生成 README。

## 6. TypeScript 核心类型

### ProviderType

```ts
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

### ModelCapability

```ts
export interface ModelCapability {
  reasoning: number;       // 0-5
  coding: number;          // 0-5
  codeReview: number;      // 0-5
  longContext: number;     // 0-5
  speed: number;           // 0-5
  lowCost: number;         // 0-5
  toolUse: number;         // 0-5
  jsonReliability: number; // 0-5
  multimodal: number;      // 0-5
  chinese: number;         // 0-5
  localDeploy: number;     // 0-5
  rag: number;             // 0-5
}
```

### AgentRole

```ts
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
```

### TaskType

```ts
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
```

## 7. 开发约束

1. 不要在 UI 层直接调用模型 API。
2. 不要把 API Key 存在 localStorage。
3. 不要把 API Key 写入日志。
4. 不要让 Agent 默认执行高风险命令。
5. 不要让 Agent 默认上传整个项目。
6. 不要把 Provider 和 Agent 强绑定。
7. 不要让单个模型失败导致整个工作流终止。
8. 不要把 Minimax / DeepSeek 作为唯一 Provider 假设。

## 8. 安全规则

以下命令默认禁止，需要用户显式确认：

```text
rm -rf
sudo
curl ... | sh
wget ... | sh
chmod -R 777
npm publish
git push --force
ssh
scp
```

以下文件默认不发送给模型：

```text
.env
.env.*
*.pem
*.key
id_rsa
node_modules/
dist/
build/
.git/
```

## 9. MVP 验收标准

1. 用户可以添加一个 OpenAI-Compatible Provider。
2. 用户可以添加 Minimax Provider。
3. 用户可以添加 DeepSeek Provider。
4. 用户可以创建 Agent 并绑定模型。
5. 用户可以创建项目并输入需求。
6. Orchestrator 可以拆解任务。
7. Coder 可以写入代码。
8. Diff 可以展示修改。
9. Tester 可以运行命令并捕获错误。
10. Debugger 可以根据错误修复。
11. Doc Agent 可以生成 README。
12. 模型失败时可以切换备用模型。
13. API Key 不会明文落盘。

## 10. 给 AI Coding Agent 的执行方式

每次开发必须：

1. 先阅读 `PRD.md`、`ARCHITECTURE.md`、`MODEL_CAPABILITY_MATRIX.md`、`TASKS.md`。
2. 每次只完成一个小任务。
3. 修改前说明会改哪些文件。
4. 修改后运行对应测试。
5. 如果无法运行测试，说明原因。
6. 不要大范围重构未要求修改的模块。
7. 不要删除已有文档。
8. 新增 Provider 时必须走统一 Adapter 接口。
9. 新增 Agent 时必须走统一 Agent Runtime。
10. 新增任务类型时必须更新 Router 权重表。

---

## 12. 场景驱动 Agent 组织生成器开发要求

### 12.1 开发目标

在原有固定 Agent 配置基础上，新增“场景 → Agent Team Plan → 用户调整 → 执行”的完整链路。

### 12.2 实现顺序

1. 先实现数据结构：ScenarioPlan、AgentTeamPlan、PlannedAgent、PromptVersion。
2. 再实现规则版 Agent 推荐，不要一开始追求完全智能。
3. 再接入 LLM 生成 Agent 职责和 Prompt。
4. 再实现前端编辑页面。
5. 最后把确认后的 Agent Team Plan 转成实际 Agent Runtime。

### 12.3 第一版推荐策略

第一版可以使用规则 + LLM 混合策略：

- 规则判断 Agent 数量。
- LLM 生成职责描述和 Prompt。
- Router 根据模型能力标签推荐模型。
- 用户调整后重新计算风险。

### 12.4 必须遵守

- 不要把 Agent 写死成 Minimax / DeepSeek。
- Agent 是岗位，模型是能力来源。
- 用户必须能编辑 Agent Prompt。
- 用户必须能替换每个 Agent 的模型。
- 删除关键 Agent 时必须给出风险提示。
- 确认前不得实际修改项目文件。

### 12.5 推荐实现文件

```text
src/features/scenario/
  ScenarioInputPage.tsx
  ScenarioPlannerService.ts
  AgentTeamPlanPage.tsx
  AgentTeamEditor.tsx
  PromptEditorModal.tsx
  scenarioTypes.ts
  scenarioRules.ts

src/features/router/
  scenarioRouter.ts
  modelRecommendation.ts
  riskEvaluator.ts

src/features/agent/
  plannedAgentToRuntime.ts
```

---

## 10. 自主交付流开发要求

### 10.1 禁止把产品做成手动 Agent 点击器

错误实现：

```text
用户必须点击 PM Agent → 再点击 Architect Agent → 再点击 Frontend Agent → 再点击 Test Agent。
```

正确实现：

```text
用户确认 Execution Plan 后，Orchestrator Runtime 自动调度所有 Agent。
```

Agent 页面、任务看板、日志面板只能作为观察和干预工具，不是主路径。

### 10.2 必须实现 Orchestrator Runtime

核心接口：

```ts
export interface OrchestratorRuntime {
  createExecutionPlan(input: ScenarioInput): Promise<ExecutionPlan>;
  createExecutionSnapshot(plan: ExecutionPlan, overrides: UserOverrides): Promise<ExecutionSnapshot>;
  startRun(snapshotId: string): Promise<ProjectRunId>;
  pauseRun(projectRunId: string, reason: PauseReason): Promise<void>;
  resumeRun(projectRunId: string, approval: ApprovalDecision): Promise<void>;
  emitReport(projectRunId: string, report: OrchestratorReport): Promise<void>;
  requestApproval(projectRunId: string, request: ApprovalRequest): Promise<void>;
}
```

### 10.3 默认主界面是 Project Command Center

必须优先实现：

1. 主控智能体汇报流。
2. 当前阶段展示。
3. 进度条。
4. 成本和预算。
5. 当前活跃 Agent。
6. 风险和阻塞。
7. 待用户决策事项。
8. 最终交付物。

### 10.4 用户确认策略

不要让用户确认所有小事。

默认自动执行：

- 任务分配
- 普通代码生成
- 普通文件修改
- 测试运行
- 低风险 Bug 修复
- README 生成
- 在预设策略内切换备用模型

必须请求确认：

- 删除文件
- 修改密钥和 `.env`
- 执行危险命令
- 上传敏感数据到外部服务
- 超预算
- 重大技术栈变化
- 发布部署
- 连续自动修复失败

### 10.5 必须记录主控决策

任何自动调度、模型切换、任务重排、修复重试都要写入 `decision_logs`，方便用户回看。

---

## 12. 公司式 Agent 协作开发要求

### 12.1 开发目标

实现 SuperCompany Coding 时，不能把多 Agent 做成简单的“多按钮任务执行器”。产品必须体现：

1. Orchestrator Agent 负责自动推进项目。
2. 子 Agent 之间有可记录的通信线程。
3. 用户默认只看主控汇报。
4. 用户可展开查看内部讨论。
5. 用户可随时暂停和修改团队。
6. 修改后必须重新评估影响，再继续执行。

### 12.2 抽象分期

MVP 必须实现：

```ts
ProjectRun
DecisionLog
RunEvent
```

公司式协作版本再实现：

```ts
AgentThread
AgentMessage
AgentMeeting
HumanIntervention
ImpactAssessment
PromptVersion
Checkpoint
```

### 12.3 运行时行为要求

项目运行时禁止要求用户逐个点击 Agent。

正确方式：

1. 用户确认方案后，创建 ProjectRun。
2. Orchestrator Agent 自动创建 TaskThread 摘要；V0.6 后再创建 AgentMeeting。
3. Orchestrator Agent 自动分配任务。
4. 子 Agent 输出在 MVP 先通过 run_events / decision_logs 记录摘要；V0.6 后再写 AgentMessage。
5. Orchestrator 定期生成 CEO Report。
6. UI 默认展示 CEO Report。
7. 用户需要时再展开 Thread 和 Raw Messages。

### 12.4 暂停机制要求

实现 `pauseProjectRun()` 时必须：

1. 停止派发新任务。
2. 保存当前任务队列。
3. 保存 Agent 通信上下文。
4. 保存未提交文件变更。
5. 保存未应用 patch / Git diff 引用；V0.7 后再创建完整 Checkpoint。
6. 生成暂停报告。

实现 `resumeProjectRun()` 时必须：

1. 读取最新 ProjectRun 状态和未应用 patch；V0.7 后再读取完整 Checkpoint。
2. 检查用户是否修改 Agent、Prompt、模型、任务范围。
3. 生成 ImpactAssessment。
4. 等用户确认后继续执行。

### 12.5 UI 开发要求

项目运行页必须包括：

1. 主控汇报区。
2. Agent 内部讨论区。
3. 任务状态区。
4. 决策日志区。
5. 暂停 / 继续按钮。
6. 微调 Agent 团队入口。
7. Prompt 编辑入口。
8. 轻量回滚入口；完整 Checkpoint 回滚后续实现。

### 12.6 文档优先级

开发公司式 Agent 协作能力时，优先阅读：

1. `INTER_AGENT_COMMUNICATION_SPEC.md`
2. `HUMAN_CONTROL_LOOP_SPEC.md`
3. `AUTONOMOUS_DELIVERY_SPEC.md`
4. `ORCHESTRATOR_REPORTING_SPEC.md`
5. `AGENT_SPEC.md`
6. `DATABASE_SCHEMA.md`
7. `UI_SPEC.md`
