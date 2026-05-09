# 场景驱动 Agent 组织生成器规范

## 1. 模块定位

场景驱动 Agent 组织生成器是 SuperCompany Coding 从“固定 Agent 团队”升级为“AI 自动组建研发组织”的核心模块。

用户不需要一开始就知道需要多少 Agent、每个 Agent 应该做什么、哪个 Agent 用哪个模型。用户只需要输入场景和目标，系统自动预估：

1. 这个需求大概需要多少个 Agent。
2. 每个 Agent 的岗位职责是什么。
3. 每个 Agent 推荐使用哪个模型。
4. 每个 Agent 为什么推荐这个模型。
5. 每个 Agent 需要什么 Prompt。
6. 哪些 Agent 是必须的，哪些 Agent 是可选的。
7. 用户可以如何增删、合并、拆分、替换 Agent。

最终形成一个可编辑的 **Agent Team Plan**。

---

## 2. 核心产品逻辑

产品从“用户配置 Agent”变为：

```text
用户给出场景
  ↓
AI 识别产品类型、复杂度、风险、技术栈
  ↓
AI 生成 Agent 团队方案
  ↓
AI 推荐每个 Agent 的模型和 Prompt
  ↓
用户自由调整 Agent 数量、职责、模型、Prompt、权限
  ↓
用户确认后启动多 Agent 协作开发
```

---

## 3. 用户输入

### 3.1 基础输入

用户可以输入自然语言，例如：

> 我要做一个 Windows 和 macOS 桌面 App，用来读取本地文件夹里的简历，然后根据岗位画像给出匹配度评分。

系统需要识别：

| 识别项 | 示例 |
|---|---|
| 产品类型 | 桌面端 App |
| 运行平台 | Windows、macOS |
| 核心场景 | 本地简历读取与匹配 |
| 技术复杂度 | 中等 |
| 数据敏感度 | 高，涉及简历和个人信息 |
| 推荐架构 | Tauri + React + SQLite + LLM Adapter |
| 推荐 Agent 数量 | 7-9 个 |
| 是否需要安全 Agent | 需要 |
| 是否需要文档 Agent | 需要 |
| 是否需要长上下文模型 | 可能需要，用于读取大量简历或代码库 |

### 3.2 高级输入

用户可以补充：

- 我想做 Demo / MVP / 生产级产品。
- 我更重视速度 / 质量 / 成本。
- 我已有 Minimax 和 DeepSeek API Key。
- 我希望全部本地运行。
- 我不希望上传敏感文件。
- 我希望前端好看一点。
- 我希望代码适合 Claude Code 后续维护。

---

## 4. 场景分析结果

系统需要输出一个场景分析报告。

### 4.1 报告结构

```json
{
  "scenarioSummary": "用户想做一个跨平台桌面端简历匹配工具",
  "productType": "desktop_app",
  "targetPlatforms": ["windows", "macos"],
  "complexity": "medium",
  "riskLevel": "medium_high",
  "privacyLevel": "high",
  "recommendedTechStack": ["Tauri", "React", "TypeScript", "SQLite"],
  "estimatedAgentCount": 8,
  "estimatedPhases": [
    "需求澄清",
    "架构设计",
    "UI 开发",
    "本地文件读取",
    "简历解析与匹配",
    "测试与修复",
    "安全检查",
    "文档交付"
  ]
}
```

### 4.2 场景复杂度判断

| 复杂度 | 特征 | 推荐 Agent 数量 |
|---|---|---|
| Low | 单页面、小脚本、简单自动化 | 2-4 个 |
| Medium | 小型 App、前后端模块清晰、需要测试 | 5-8 个 |
| High | 多端、多数据源、复杂权限、复杂状态 | 8-12 个 |
| Expert | 企业级系统、多人协作、安全合规、复杂部署 | 12+ 个 |

---

## 5. Agent Team Plan

### 5.1 生成结果

系统根据场景自动生成 Agent 团队方案。

示例：

| Agent | 是否必需 | 职责 | 推荐模型 | 推荐原因 |
|---|---|---|---|---|
| Orchestrator Agent | 必需 | 统筹任务、拆解阶段、协调其他 Agent | Minimax M2.7 / Claude / OpenAI reasoning | 需要稳定多轮规划、任务分配和上下文调度 |
| PM Agent | 必需 | 把用户场景转成 PRD、验收标准 | Minimax / Claude / Qwen | 需要中文表达、产品拆解和需求边界判断 |
| Architect Agent | 必需 | 技术架构、目录结构、模块边界 | Claude / DeepSeek Pro / OpenAI reasoning | 需要较强推理和工程判断 |
| Frontend Agent | 必需 | 页面、组件、交互、状态管理 | Claude / DeepSeek Pro / Qwen / Doubao | 需要代码生成和 UI 结构能力 |
| Backend Agent | 必需 | 本地服务、数据结构、文件系统逻辑 | DeepSeek Pro / OpenAI / Claude | 需要稳定编码和业务逻辑能力 |
| Test Agent | 推荐 | 单测、集成测试、运行检查 | DeepSeek Flash / Qwen / 小模型 | 可用低成本模型批量生成测试 |
| Debug Agent | 推荐 | 读取报错、修复代码 | DeepSeek Pro / Claude / OpenAI reasoning | 需要定位根因和修改代码 |
| Security Agent | 视场景必需 | 检查 API Key、隐私、危险命令 | Claude / OpenAI reasoning / 本地模型 | 涉及敏感文件和命令执行时必须启用 |
| Doc Agent | 可选 | README、使用说明、交付报告 | DeepSeek Flash / Qwen / Minimax | 低成本文档生成 |

---

## 6. Agent 可编辑机制

用户必须能在启动前自由调整 Agent 团队。

### 6.1 用户可操作项

用户可以：

1. 增加 Agent。
2. 删除 Agent。
3. 合并 Agent。
4. 拆分 Agent。
5. 修改 Agent 名称。
6. 修改 Agent 职责。
7. 修改 Agent Prompt。
8. 修改 Agent 使用的主模型。
9. 修改 Agent 的备用模型。
10. 修改 Agent 权限。
11. 修改 Agent 预算。
12. 修改 Agent 是否必须参与。
13. 修改 Agent 的执行阶段。

### 6.2 调整时的 AI 辅助

用户每次调整后，系统应给出影响提示：

| 用户操作 | 系统提示 |
|---|---|
| 删除 Test Agent | 测试覆盖率可能降低，Bug 修复闭环变弱 |
| 删除 Security Agent | 涉及本地文件和 API Key 时风险升高 |
| 把 Architect 和 Backend 合并 | 速度更快，但架构审查会变弱 |
| 把 Debug Agent 换成低成本模型 | 成本下降，但复杂 Bug 修复成功率可能下降 |
| 增加 UI Reviewer Agent | 前端体验更好，但执行时间和成本增加 |

---

## 7. Prompt 自动生成与可编辑

### 7.1 Prompt 生成原则

系统应根据以下信息生成 Prompt：

1. 用户场景。
2. 产品类型。
3. 技术栈。
4. Agent 职责。
5. 绑定模型特点。
6. 项目安全策略。
7. 用户偏好：快、省钱、质量优先。

### 7.2 Prompt 结构

每个 Agent 的 Prompt 应包含：

```text
# Role
你是谁。

# Responsibility
你负责什么。

# Scope
你可以做什么，不可以做什么。

# Input
你会收到什么输入。

# Output
你必须输出什么格式。

# Constraints
技术栈、代码风格、安全限制、成本限制。

# Collaboration Rules
你如何与其他 Agent 协作。

# Acceptance Criteria
什么情况下算完成。
```

### 7.3 Prompt 编辑器

Prompt 编辑器需要支持：

- 查看原始 Prompt。
- AI 优化 Prompt。
- 手动修改 Prompt。
- 恢复默认 Prompt。
- 对比修改前后 Diff。
- 保存为 Agent 模板。
- 保存为当前项目专用 Prompt。

---

## 8. 模型推荐逻辑

### 8.1 推荐依据

模型推荐不应基于品牌偏好，而应基于能力标签和项目上下文：

| 维度 | 说明 |
|---|---|
| reasoning | 是否适合复杂推理、架构判断、Debug |
| coding | 是否适合代码生成和重构 |
| longContext | 是否适合读取大代码库和长文档 |
| speed | 响应速度 |
| lowCost | 成本优势 |
| toolUse | 工具调用稳定性 |
| jsonReliability | 结构化输出稳定性 |
| multimodal | 是否支持图像、截图、PDF 等输入 |
| chinese | 中文需求理解和文档表达 |
| privacy | 是否支持本地部署或私有化 |

### 8.2 推荐输出

推荐时必须向用户解释：

1. 推荐哪个模型。
2. 为什么推荐。
3. 可替代模型有哪些。
4. 成本更低的选择是什么。
5. 质量更高的选择是什么。
6. 用户当前已配置的模型里哪个最接近。

### 8.3 当前用户默认策略

在用户当前只有 Minimax 和 DeepSeek 的情况下：

| Agent | 默认推荐 |
|---|---|
| Orchestrator Agent | Minimax M2.7 |
| PM Agent | Minimax M2.7 |
| Architect Agent | DeepSeek v4 Pro |
| Frontend Agent | DeepSeek v4 Pro / Flash |
| Backend Agent | DeepSeek v4 Pro |
| Test Agent | DeepSeek v4 Flash |
| Debug Agent | DeepSeek v4 Pro |
| Doc Agent | DeepSeek v4 Flash |
| Integration Agent | Minimax M2.7 |

但系统 UI 必须明确提示：

> 这是基于你当前已配置模型的推荐，不代表产品只支持这些模型。你可以接入 Claude、OpenAI、Gemini、Qwen、Kimi、GLM、本地模型或任意 OpenAI-Compatible Provider。

---

## 9. 核心页面

### 9.1 场景输入页

用户输入：

- 你想做什么？
- 目标平台是什么？
- 你希望做到 Demo / MVP / 生产级？
- 更重视速度、成本还是质量？
- 是否涉及敏感数据？
- 是否已有代码项目？

### 9.2 Agent 方案预览页

展示：

- 推荐 Agent 数量。
- 每个 Agent 的职责。
- 每个 Agent 推荐模型。
- 每个 Agent 的成本预估。
- 每个 Agent 是否必需。
- 推荐理由。
- 风险提示。

### 9.3 Agent 编排编辑页

用户可以：

- 拖拽调整执行顺序。
- 增删 Agent。
- 合并 Agent。
- 拆分 Agent。
- 修改模型。
- 修改 Prompt。
- 修改权限。
- 查看成本变化。

### 9.4 Prompt 配置页

展示每个 Agent 的 Prompt。

支持：

- AI 生成 Prompt。
- 用户手动修改。
- Prompt Diff。
- Prompt 版本回滚。
- 保存为模板。

---

## 10. 数据结构

### 10.1 ScenarioPlan

```ts
export interface ScenarioPlan {
  id: string;
  projectId: string;
  userScenario: string;
  productType: string;
  complexity: 'low' | 'medium' | 'high' | 'expert';
  riskLevel: 'low' | 'medium' | 'high';
  privacyLevel: 'low' | 'medium' | 'high';
  recommendedTechStack: string[];
  estimatedAgentCount: number;
  createdAt: string;
}
```

### 10.2 AgentTeamPlan

```ts
export interface AgentTeamPlan {
  id: string;
  scenarioPlanId: string;
  agents: PlannedAgent[];
  estimatedCost: number;
  estimatedDuration: string;
  qualityProfile: 'fast' | 'balanced' | 'high_quality' | 'cheap';
}
```

### 10.3 PlannedAgent

```ts
export interface PlannedAgent {
  id: string;
  name: string;
  role: string;
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
}
```

---

## 11. MVP 实现范围

第一版不需要做到完全智能，但必须跑通核心闭环。

### 11.1 MVP 必须实现

1. 用户输入场景。
2. 系统生成 Agent Team Plan。
3. 系统推荐每个 Agent 的模型。
4. 用户可以增删 Agent。
5. 用户可以修改 Agent 职责。
6. 用户可以修改 Agent 绑定模型。
7. 用户可以查看并修改 Prompt。
8. 用户确认后，把 Agent Team Plan 转成实际任务工作流。

### 11.2 MVP 可简化

1. Agent 数量推荐可以基于规则 + LLM。
2. 模型推荐可以基于能力标签，不需要真实评测。
3. 成本预估可以先粗略计算。
4. Prompt 版本管理可以先只保存最近一次。

---

## 12. 验收标准

1. 输入一个产品场景后，系统能在 30 秒内生成 Agent 团队方案。
2. 方案至少包含 Agent 名称、职责、推荐模型、推荐原因、Prompt。
3. 用户可以删除一个 Agent，并看到风险提示。
4. 用户可以新增一个 Agent，并手动配置职责和模型。
5. 用户可以修改任意 Agent 的 Prompt。
6. 用户点击确认后，系统能基于最终 Agent 团队启动任务拆解。
7. 当前只配置 Minimax 和 DeepSeek 时，系统能自动给出基于这两个模型的最优分配。
8. 后续接入 Claude、OpenAI、Gemini、Qwen 等模型后，无需修改 Agent 逻辑，只需更新模型能力标签。

---

## 13. 设计原则

1. **AI 给建议，人做最终决策。**  
   Agent 数量、职责、模型、Prompt 都可以由 AI 推荐，但最终由用户确认。

2. **模型不是岗位，Agent 才是岗位。**  
   模型只是能力来源，Agent 才承担产品组织中的职责。

3. **先解释，再执行。**  
   系统必须告诉用户为什么需要这些 Agent、为什么推荐这些模型。

4. **默认可用，高级可调。**  
   普通用户可以直接接受推荐；高级用户可以逐项调整。

5. **不要把复杂度暴露给新手。**  
   默认展示“推荐团队方案”，高级设置折叠。

---

## 10. 与自主项目交付流的关系

场景驱动 Agent 组织生成器只负责“启动前组队和配置”。

完整产品主流程还需要继续进入自主项目交付流：

```text
场景输入
  ↓
生成 Agent Team Plan
  ↓
生成模型推荐与 Prompt
  ↓
用户一次性调整
  ↓
生成 Execution Snapshot
  ↓
Orchestrator Agent 自动执行项目
  ↓
阶段汇报 / 风险审批 / 最终交付
```

用户不应该在 Agent Team Plan 之后继续手动逐个启动 Agent。Agent Team Plan 一旦确认，就应该交给 Orchestrator Runtime 自动推进。

详见：`AUTONOMOUS_DELIVERY_SPEC.md`。
