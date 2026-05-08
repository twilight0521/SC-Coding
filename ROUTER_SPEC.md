# 模型路由与成本控制规范

## 1. Router 目标

Router 的目标是：

> 根据任务类型、模型能力、用户偏好、成本预算、速度要求，选择最合适的 Agent 和模型。

Router 不应该只根据品牌选择模型，而应该根据能力评分选择。

## 2. 输入

```ts
export interface RoutingInput {
  taskType: TaskType;
  complexity: 'low' | 'medium' | 'high';
  riskLevel: 'low' | 'medium' | 'high';
  latencyPreference: 'fast' | 'balanced' | 'quality';
  costPreference: 'cheap' | 'balanced' | 'quality';
  contextSize: number;
  requiresTools: boolean;
  requiresJson: boolean;
  requiresMultimodal: boolean;
  preferredProviderIds?: string[];
  preferredModelProfileIds?: string[];
  blockedProviderIds?: string[];
  blockedModelProfileIds?: string[];
}
```

## 3. 输出

```ts
export interface RoutingDecision {
  primaryProviderId: string;
  primaryModelProfileId: string;
  fallbackProviderIds: string[];
  fallbackModelProfileIds: string[];
  reason: string;
  estimatedCost?: number;
  estimatedLatency?: number;
  capabilityScore: number;
}
```

Router 评分对象是 `ModelProfile`，不是厂商品牌。Provider 只提供协议、密钥、限流和 endpoint；模型能力、价格、上下文长度都来自 `ModelProfile`。

## 4. 任务权重

### 4.1 架构设计

```json
{
  "reasoning": 0.35,
  "coding": 0.2,
  "longContext": 0.15,
  "toolUse": 0.1,
  "codeReview": 0.1,
  "jsonReliability": 0.1
}
```

### 4.2 前端代码

```json
{
  "coding": 0.35,
  "reasoning": 0.15,
  "codeReview": 0.15,
  "speed": 0.1,
  "jsonReliability": 0.1,
  "longContext": 0.15
}
```

### 4.3 后端代码

```json
{
  "coding": 0.35,
  "reasoning": 0.25,
  "toolUse": 0.1,
  "codeReview": 0.1,
  "jsonReliability": 0.1,
  "longContext": 0.1
}
```

### 4.4 Debug

```json
{
  "reasoning": 0.35,
  "coding": 0.25,
  "codeReview": 0.15,
  "longContext": 0.1,
  "toolUse": 0.1,
  "jsonReliability": 0.05
}
```

### 4.5 文档生成

```json
{
  "chinese": 0.25,
  "speed": 0.2,
  "lowCost": 0.2,
  "longContext": 0.15,
  "reasoning": 0.1,
  "jsonReliability": 0.1
}
```

### 4.6 长上下文阅读

```json
{
  "longContext": 0.45,
  "reasoning": 0.2,
  "coding": 0.1,
  "jsonReliability": 0.1,
  "speed": 0.05,
  "lowCost": 0.1
}
```

### 4.7 多模态分析

```json
{
  "multimodal": 0.45,
  "reasoning": 0.2,
  "longContext": 0.15,
  "jsonReliability": 0.1,
  "speed": 0.05,
  "lowCost": 0.05
}
```

## 5. 分数计算

```ts
score = capabilityScore * capabilityWeight
      + userPreferenceBonus
      + historicalSuccessBonus
      - costPenalty
      - latencyPenalty
      - errorRatePenalty
```

## 6. 工作模式

### 6.1 快速模式

优先：

- speed
- lowCost
- 历史成功率

适合：

- 小修改
- 文档
- 测试
- 简单 Bug

### 6.2 深度模式

优先：

- reasoning
- coding
- codeReview
- longContext

适合：

- 架构设计
- 核心模块
- 复杂 Bug
- 安全审查

### 6.3 省钱模式

策略：

1. 先用低成本模型。
2. 失败后升级到中等模型。
3. 仍失败再用强模型。

示例：

```text
DeepSeek Flash → DeepSeek Pro → Minimax / Claude / OpenAI
```

### 6.4 自定义模式

用户直接指定：

- 某个 Agent 用哪个模型。
- 某个任务类型用哪个模型。
- 某个项目禁用哪个模型。

## 7. 降级与升级策略

### 7.1 自动升级

触发条件：

- 同一任务失败 2 次。
- JSON 输出连续不合法。
- 测试连续失败。
- 模型返回不支持工具调用。
- 上下文超限。

升级路径示例：

```text
低成本模型 → Pro 模型 → 强推理模型 → 请求用户确认
```

### 7.2 自动降级

触发条件：

- 当前任务复杂度低。
- 成本即将超限。
- 用户选择省钱模式。
- 当前模型 rate limit。

降级路径示例：

```text
强模型 → Flash / mini → 本地模型
```

## 8. 成本控制

### 8.1 预算层级

1. 全局每日预算
2. 项目预算
3. Agent 预算
4. 单任务预算
5. 单次调用预算

### 8.2 成本展示

任务开始前展示：

- 预计模型
- 预计调用次数
- 预计 Token
- 预计成本区间

任务结束后展示：

- 实际模型
- 实际 Token
- 实际成本
- 成功 / 失败
- 是否重试

## 9. 历史表现学习

系统需要记录：

- 模型在不同任务类型上的成功率
- 平均耗时
- 平均成本
- 重试率
- 用户接受率
- 测试通过率

Router 后续根据历史结果微调推荐。

## 10. 当前用户默认路由

在只有 Minimax + DeepSeek 时：

| 任务类型 | 主模型 | 备用模型 |
|---|---|---|
| requirement_analysis | Minimax M2.7 | DeepSeek Pro |
| architecture_design | DeepSeek Pro | Minimax M2.7 |
| frontend_coding | DeepSeek Pro | DeepSeek Flash |
| backend_coding | DeepSeek Pro | Minimax M2.7 |
| test_generation | DeepSeek Flash | DeepSeek Pro |
| debugging | DeepSeek Pro | Minimax M2.7 |
| documentation | DeepSeek Flash | Minimax M2.7 |
| integration | Minimax M2.7 | DeepSeek Pro |
| code_review | DeepSeek Pro | Minimax M2.7 |

---

## 9. 场景级 Agent 与模型推荐

### 9.1 Router 的新增职责

Router 不只负责“给已有任务选择模型”，还要支持“根据用户场景推荐 Agent 组织结构”。

新增职责：

1. 预估需要多少 Agent。
2. 预估需要哪些 Agent 角色。
3. 判断哪些 Agent 必须启用。
4. 给每个 Agent 推荐模型。
5. 给每个推荐生成解释。
6. 在用户调整 Agent 后重新计算风险和成本。

### 9.2 输入

```ts
export interface ScenarioRoutingInput {
  userScenario: string;
  productType?: string;
  targetPlatforms?: string[];
  complexityPreference?: 'demo' | 'mvp' | 'production';
  userModelProviderIds: string[];
  costPreference: 'cheap' | 'balanced' | 'quality';
  speedPreference: 'fast' | 'balanced' | 'careful';
  privacyPreference: 'normal' | 'sensitive' | 'local_first';
  existingProjectPath?: string;
}
```

### 9.3 输出

```ts
export interface ScenarioRoutingOutput {
  scenarioSummary: string;
  estimatedAgentCount: number;
  agents: PlannedAgent[];
  modelRecommendationSummary: string;
  estimatedCostLevel: 'low' | 'medium' | 'high';
  riskNotes: string[];
  userEditableFields: string[];
}
```

### 9.4 Agent 数量推荐规则

| 场景 | 推荐 Agent |
|---|---|
| 简单脚本 | Orchestrator、Coder、Test/Debug |
| 单页面 Web App | Orchestrator、PM、Frontend、Test、Doc |
| 标准桌面 App | Orchestrator、PM、Architect、Frontend、Backend、Test、Debug、Doc |
| 涉及敏感数据 | 额外增加 Security Agent |
| 涉及复杂数据处理 | 额外增加 Data Agent |
| 涉及 UI 质量 | 额外增加 UI Reviewer Agent |
| 涉及大代码库 | 额外增加 Codebase Analyst Agent |
| 涉及部署上线 | 额外增加 DevOps Agent |

### 9.5 推荐解释模板

每个 Agent 的模型推荐必须给出解释：

```text
推荐 {modelName} 给 {agentName}，因为该 Agent 需要 {capabilityList}。
当前已配置模型中，{modelName} 在 {reasoning/coding/longContext/cost/speed} 维度最匹配。
如果你更重视成本，可以改用 {cheapAlternative}；如果你更重视质量，可以改用 {qualityAlternative}。
```

### 9.6 用户改动后的重算

当用户修改 Agent Team Plan 时，Router 必须重新计算：

- 总成本等级。
- 风险等级。
- 执行顺序。
- 依赖关系。
- 模型并发压力。
- 是否缺少关键角色。

---

## 9. 自主执行下的路由策略

在自主交付模式下，Router 不只负责“选模型”，还要支持主控智能体自动推进项目。

### 9.1 自动路由允许范围

在用户预先设定的预算、权限和模型范围内，Router 可以自动：

1. 为任务选择最合适模型。
2. 为失败任务切换备用模型。
3. 将复杂任务升级给更强模型。
4. 将简单任务降级给低成本模型。
5. 根据上下文长度选择长上下文模型。
6. 根据任务风险选择是否加入 Reviewer Agent。

### 9.2 自动路由禁止范围

Router 不得自动：

1. 使用用户未授权的 Provider。
2. 超过项目预算。
3. 把敏感文件发送给未经允许的云端模型。
4. 让无权限 Agent 修改受保护文件。
5. 绕过安全策略执行命令。

### 9.3 路由决策记录

每次模型选择都必须记录：

```ts
export interface ModelRoutingLog {
  id: string;
  projectRunId: string;
  taskId: string;
  agentId: string;
  selectedProviderId: string;
  selectedModelProfileId: string;
  fallbackProviderIds: string[];
  fallbackModelProfileIds: string[];
  reason: string;
  capabilityScore: number;
  estimatedCost: number;
  actualCost?: number;
  createdAt: string;
}
```

### 9.4 模型升级策略

默认升级顺序：

```text
低成本模型失败
  ↓
同模型重试一次
  ↓
切换同级备用模型
  ↓
升级到强推理 / 强编码模型
  ↓
仍失败则通知 Orchestrator
  ↓
Orchestrator 判断是否请求用户决策
```

---

## 10. 用户微调后的模型重路由

用户在项目中途修改 Agent、模型或 Prompt 后，Router 必须重新计算路由策略。

### 10.1 重路由触发条件

1. 用户给某个 Agent 换模型。
2. 用户新增 Agent。
3. 用户删除 Agent。
4. 用户修改 Agent 职责。
5. 用户修改任务范围。
6. 用户修改预算。
7. 某个模型连续失败。
8. 某个模型成本超出预期。

### 10.2 重路由输出

Router 必须输出：

1. 受影响 Agent。
2. 受影响任务。
3. 新模型选择。
4. 质量变化预估。
5. 成本变化预估。
6. 速度变化预估。
7. 是否需要重跑任务。

### 10.3 展示给用户的话术

```text
你将 Debug Agent 从 Flash 模型改为 Pro 模型。

影响评估：
- 修复复杂 Bug 的成功率预计提升。
- 单次调用成本会增加。
- 已完成任务不需要重跑。
- 后续测试失败会优先交给新的 Debug Agent 处理。
```
