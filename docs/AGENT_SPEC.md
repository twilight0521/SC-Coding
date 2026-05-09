# Agent 设计规范

## 1. Agent 核心定义

Agent 是“岗位”，模型是“员工能力”。

同一个 Agent 可以使用不同模型；同一个模型也可以服务多个 Agent。

## 2. Agent 配置字段

```ts
export interface AgentConfig {
  id: string;
  name: string;
  role: AgentRole;
  description: string;
  systemPrompt: string;
  primaryProviderId: string;
  primaryModelProfileId: string;
  fallbackProviderIds: string[];
  fallbackModelProfileIds?: string[];
  permissions: AgentPermissions;
  budgetLimit?: number;
  maxRuntimeMs?: number;
  contextPolicy: ContextPolicy;
  routingPolicy: RoutingPolicy;
}
```

## 3. 默认 Agent 团队

### 3.1 Orchestrator Agent

职责：

- 理解用户目标
- 拆解任务
- 分配 Agent
- 控制任务依赖
- 决定是否升级模型
- 生成最终交付报告

推荐模型：

- Minimax M2.7
- OpenAI reasoning
- Claude
- GLM

### 3.2 Product Manager Agent

职责：

- 把用户想法转成 PRD
- 生成用户故事
- 生成验收标准
- 判断 MVP 范围

推荐模型：

- Minimax
- Claude
- OpenAI
- Gemini
- Qwen

### 3.3 Architect Agent

职责：

- 技术选型
- 架构设计
- 模块边界
- 数据结构设计
- 风险判断

推荐模型：

- Claude
- OpenAI reasoning
- DeepSeek Pro
- GLM
- Kimi

### 3.4 Frontend Agent

职责：

- 页面开发
- 组件设计
- 状态管理
- 样式实现
- 前端交互

推荐模型：

- Claude
- DeepSeek Pro
- OpenAI
- Qwen
- Doubao / Seed

### 3.5 Backend Agent

职责：

- API 设计
- 业务逻辑
- 数据库操作
- 文件系统操作
- 权限逻辑

推荐模型：

- DeepSeek Pro
- OpenAI
- Claude
- Qwen
- GLM

### 3.6 Test Agent

职责：

- 单元测试
- 集成测试
- E2E 测试
- 测试命令执行
- 测试报告

推荐模型：

- DeepSeek Flash
- Qwen
- Mistral
- OpenAI mini
- Claude Haiku / 低成本模型

### 3.7 Debug Agent

职责：

- 读取错误日志
- 定位根因
- 修改代码
- 验证修复

推荐模型：

- DeepSeek Pro
- OpenAI reasoning
- Claude
- GLM

### 3.8 Integration Agent

职责：

- 合并多个 Agent 的修改
- 处理接口不一致
- 处理文件冲突
- 统一代码风格
- 生成 commit message

推荐模型：

- Minimax M2.7
- Claude
- OpenAI
- DeepSeek Pro

### 3.9 Code Reviewer Agent

职责：

- 审查代码质量
- 检查潜在 Bug
- 检查可维护性
- 提出修改建议

推荐模型：

- Claude
- OpenAI reasoning
- DeepSeek Pro
- GLM

### 3.10 Security Agent

职责：

- 检查 API Key 泄露
- 检查命令风险
- 检查依赖风险
- 检查注入风险
- 检查权限越界

推荐模型：

- OpenAI reasoning
- Claude
- GLM
- 本地模型 + 静态扫描工具

### 3.11 Doc Agent

职责：

- README
- 使用说明
- API 文档
- 变更日志
- 交付报告

推荐模型：

- DeepSeek Flash
- Minimax
- Claude
- Qwen

### 3.12 Research Agent

职责：

- 查询外部信息
- 技术选型调研
- 竞品分析
- 依赖文档阅读

推荐模型：

- Grok
- Gemini
- OpenAI with search
- Cohere RAG

### 3.13 Long Context Reader Agent

职责：

- 读取大型代码库
- 读取长 PDF
- 读取历史文档
- 生成项目理解报告

推荐模型：

- Gemini
- Kimi
- Claude
- OpenAI long-context

## 4. Agent 执行协议

每个 Agent 执行任务必须输出：

```json
{
  "summary": "本次任务做了什么",
  "changedFiles": ["src/example.ts"],
  "patches": [],
  "commandsToRun": [],
  "risks": [],
  "needsUserConfirmation": false,
  "nextSteps": []
}
```

## 5. Agent 权限

```ts
export interface AgentPermissions {
  canReadFiles: boolean;
  canWriteFiles: boolean;
  canExecuteCommands: boolean;
  canInstallDependencies: boolean;
  canAccessNetwork: boolean;
  canModifyEnvFiles: boolean;
  canDeleteFiles: boolean;
}
```

默认权限：

| Agent | 读文件 | 写文件 | 执行命令 | 安装依赖 | 删除文件 |
|---|---|---|---|---|---|
| Orchestrator | 是 | 否 | 否 | 否 | 否 |
| PM | 是 | 否 | 否 | 否 | 否 |
| Architect | 是 | 可选 | 否 | 否 | 否 |
| Coder | 是 | 是 | 否 | 否 | 否 |
| Tester | 是 | 可选 | 是 | 否 | 否 |
| Debug | 是 | 是 | 是 | 否 | 否 |
| Integration | 是 | 是 | 是 | 可选 | 否 |
| Doc | 是 | 是 | 否 | 否 | 否 |
| Security | 是 | 否 | 可选 | 否 | 否 |

## 6. Agent Team Preset

### 6.1 当前用户默认 Preset

```json
{
  "name": "Minimax + DeepSeek Coding Team",
  "agents": {
    "orchestrator": "minimax-m2.7",
    "pm": "minimax-m2.7",
    "architect": "deepseek-v4-pro",
    "frontend": "deepseek-v4-pro",
    "backend": "deepseek-v4-pro",
    "tester": "deepseek-v4-flash",
    "debugger": "deepseek-v4-pro",
    "doc": "deepseek-v4-flash",
    "integration": "minimax-m2.7"
  }
}
```

### 6.2 全模型高级 Preset

```json
{
  "name": "Best-of-All-Models Team",
  "agents": {
    "orchestrator": "minimax-or-openai-reasoning",
    "pm": "claude-or-minimax",
    "architect": "claude-or-openai-reasoning",
    "longContextReader": "gemini-or-kimi",
    "frontend": "claude-or-deepseek-pro",
    "backend": "deepseek-pro-or-openai",
    "tester": "deepseek-flash-or-qwen",
    "debugger": "deepseek-pro-or-claude",
    "security": "openai-reasoning-or-claude",
    "research": "grok-or-gemini",
    "doc": "claude-or-minimax-or-flash",
    "integration": "minimax-or-claude"
  }
}
```

## 7. Agent 协作流程

### 7.1 标准流程

```text
用户需求
→ Orchestrator 拆解
→ PM 补充验收标准
→ Architect 设计架构
→ Frontend / Backend 并行开发
→ Integration 合并
→ Tester 测试
→ Debug 修复
→ Reviewer 审查
→ Doc 生成文档
→ Orchestrator 交付总结
```

### 7.2 复杂问题流程

```text
问题输入
→ Orchestrator 判断复杂度
→ Long Context Reader 收集上下文
→ Architect 分析原因
→ Debug Agent 修复
→ Code Reviewer 复查
→ Tester 验证
```

## 8. Agent 失败处理

| 失败类型 | 处理方式 |
|---|---|
| 输出格式不合法 | 原模型重试一次 |
| 代码无法应用 | 交给 Integration Agent 修复 patch |
| 测试失败 | 交给 Debug Agent |
| 连续失败 2 次 | 切换备用模型 |
| 成本超限 | 暂停任务，请用户确认 |
| 高风险操作 | 强制用户确认 |

---

## 7. 场景驱动动态 Agent 生成

### 7.1 核心变化

早期版本允许用户手动创建 Agent。后续版本必须支持由用户输入场景后，系统自动生成 Agent 团队方案。

Agent 不再只是固定模板，而是根据场景动态生成。

```text
用户场景 → 场景分析 → Agent Team Plan → 用户调整 → Agent Runtime
```

### 7.2 动态 Agent 生成维度

系统生成 Agent 时需要考虑：

| 维度 | 示例 |
|---|---|
| 产品类型 | 桌面 App、Web App、后端服务、移动端、插件、脚本 |
| 复杂度 | Demo、MVP、生产级、企业级 |
| 技术栈 | Tauri、Electron、React、Next.js、FastAPI、Go、Android |
| 数据敏感度 | 是否涉及简历、合同、API Key、用户隐私 |
| 交付目标 | Demo、源码、可执行文件、部署包、文档 |
| 用户偏好 | 快速、省钱、质量优先、私有化优先 |

### 7.3 Agent Team Plan 字段

```ts
export interface AgentTeamPlan {
  id: string;
  scenarioPlanId: string;
  agents: PlannedAgent[];
  estimatedCost: number;
  estimatedDurationLevel: 'short' | 'medium' | 'long';
  qualityProfile: 'fast' | 'balanced' | 'high_quality' | 'cheap';
  riskNotes: string[];
}
```

### 7.4 PlannedAgent 字段

```ts
export interface PlannedAgent {
  id: string;
  name: string;
  role: AgentRole | string;
  isRequired: boolean;
  responsibility: string;
  recommendedProviderId?: string;
  recommendedModelName?: string;
  alternativeProviderIds: string[];
  recommendationReason: string;
  systemPrompt: string;
  editablePrompt: string;
  permissions: AgentPermissions;
  budgetLimit?: number;
  executionPhase: string;
  userModified: boolean;
}
```

### 7.5 用户调整规则

用户调整 Agent 后，系统必须重新计算：

1. 成本预估。
2. 风险提示。
3. 执行计划。
4. 模型调用计划。
5. 上下文分发范围。

示例：

- 删除 Test Agent：提示测试风险上升。
- 删除 Security Agent：提示隐私和命令执行风险上升。
- 合并 Architect 与 Backend：提示速度提升但架构质量可能下降。
- 把 Debug Agent 从 Pro 模型换成 Flash 模型：提示成本下降但复杂 Bug 修复能力下降。

### 7.6 Prompt 生成规则

每个动态 Agent 的 Prompt 必须由系统生成，并允许用户修改。

Prompt 必须包含：

1. Role：身份。
2. Responsibility：职责。
3. Scope：工作范围。
4. Input：输入内容。
5. Output：输出格式。
6. Constraints：技术和安全约束。
7. Collaboration Rules：与其他 Agent 的协作规则。
8. Acceptance Criteria：完成标准。

---

## 9. 自主交付模式下的 Agent 协作

### 9.1 默认沟通对象

用户默认只和 Orchestrator Agent 沟通。

其他 Agent 的输出默认进入内部工作区，由 Orchestrator Agent 汇总、评估和转述。

### 9.2 Orchestrator Agent 的新增职责

在自主交付模式下，Orchestrator Agent 必须承担项目负责人职责：

1. 生成执行计划。
2. 创建 Agent Team Plan。
3. 生成每个 Agent 的 Prompt。
4. 根据用户修改生成 Execution Snapshot。
5. 调度任务队列。
6. 管理任务依赖。
7. 分配上下文。
8. 汇总子 Agent 输出。
9. 判断是否需要返工。
10. 触发 Review / Test / Debug。
11. 控制预算。
12. 触发用户审批。
13. 阶段汇报。
14. 最终交付。

### 9.3 子 Agent 输出规则

子 Agent 不直接打扰用户。子 Agent 输出必须进入以下结构：

```ts
export interface AgentWorkResult {
  agentId: string;
  taskId: string;
  status: 'completed' | 'failed' | 'needs_review';
  summary: string;
  changedFiles: string[];
  artifacts: ArtifactRef[];
  risks: string[];
  nextSuggestedActions: string[];
}
```

Orchestrator Agent 根据这些结果生成用户可读汇报。

### 9.4 Agent 内部协作空间

需要提供内部共享空间：

- Project Brief
- Technical Plan
- Task Queue
- Decision Log
- Artifact Registry
- Error Log
- Test Report
- Final Delivery Report

子 Agent 通过这些共享对象协作，不通过用户手动转发信息。

---

## 13. Agent 之间的公司式协作机制

### 13.1 基本原则

Agent 不能只各自独立输出结果，必须像公司团队一样协作：

1. PM Agent 可以向 Architect Agent 提问。
2. Architect Agent 可以反驳 Backend Agent 的实现方案。
3. Security Agent 可以阻止高风险实现。
4. Test Agent 可以把失败原因反馈给 Debug Agent。
5. Reviewer Agent 可以要求 Coder Agent 返工。
6. Orchestrator Agent 负责总结、仲裁和决策。

### 13.2 Agent 通信角色

| Agent | 通信职责 |
|---|---|
| Orchestrator Agent | 组织会议、分配话题、总结结论、请求用户确认 |
| PM Agent | 解释需求、判断范围、维护验收标准 |
| Architect Agent | 解释架构取舍、定义模块边界 |
| Frontend Agent | 向 PM / UX / Backend 对齐页面和接口 |
| Backend Agent | 向 Architect / Frontend 对齐数据结构和 API |
| Test Agent | 向开发 Agent 反馈测试失败和覆盖不足 |
| Debug Agent | 向 Test Agent 追问错误复现条件 |
| Security Agent | 审查 API Key、文件权限、危险命令和隐私数据 |
| Reviewer Agent | 对代码质量、可维护性、风格一致性提出意见 |
| Doc Agent | 向 PM / Architect 确认文档口径 |

### 13.3 通信输出要求

每条 Agent 消息必须尽量结构化：

1. 结论。
2. 理由。
3. 影响范围。
4. 风险。
5. 是否需要主控决策。
6. 是否需要用户确认。

### 13.4 用户微调后的 Agent 行为

用户修改 Agent 后：

1. Orchestrator Agent 重新读取 Agent 配置。
2. 评估受影响任务。
3. 必要时重写相关 Prompt。
4. 重新分配任务。
5. 保留旧 Agent 的历史通信记录。
6. 新 Agent 必须读取项目摘要和相关上下文后再工作。
