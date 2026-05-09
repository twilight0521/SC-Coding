# 主控智能体汇报规范

## 1. 汇报定位

Orchestrator Agent 是用户唯一默认沟通对象。

用户不需要同时看 PM Agent、Architect Agent、Frontend Agent、Backend Agent、Test Agent、Debug Agent 的所有输出。默认情况下，用户只看主控智能体的项目级汇报。

子 Agent 日志可以查看，但不作为主交互。

---

## 2. 汇报目标

主控智能体的汇报必须解决四个问题：

1. 项目现在到哪一步了。
2. 已经完成了什么。
3. 有没有风险或阻塞。
4. 是否需要用户做决策。

汇报必须避免：

- 复制大量模型原始输出。
- 展示无意义思考过程。
- 把内部任务日志当成用户汇报。
- 每完成一个小文件就打断用户。

---

## 3. 汇报类型

### 3.1 启动前汇报

生成执行计划后输出。

内容：

- 需求理解
- 推荐 Agent 团队
- 每个 Agent 推荐模型
- 关键 Prompt 摘要
- 技术栈建议
- 风险点
- 预计阶段
- 预算区间
- 需要用户确认的选项

### 3.2 阶段汇报

每个主要阶段完成后输出。

内容：

- 当前阶段结果
- 完成项
- 产出物
- 当前风险
- 下一阶段计划
- 是否需要用户决策

### 3.3 异常汇报

遇到阻塞或高风险时输出。

内容：

- 发生了什么
- 影响范围
- 已尝试的自动修复
- 可选方案
- 主控建议

### 3.4 成本汇报

达到预算阈值时输出。

默认阈值：

- 已用 50%
- 已用 80%
- 已用 100%

内容：

- 当前消耗
- 消耗最高的 Agent
- 是否需要切换省钱模式
- 是否需要暂停

### 3.5 最终交付汇报

项目完成后输出。

内容：

- 交付状态
- 如何运行
- 已完成功能
- 测试结果
- 修改文件清单
- 已知问题
- 后续建议

---

## 4. 汇报频率设置

```ts
export type ReportCadence =
  | 'milestone_only'
  | 'balanced'
  | 'verbose'
  | 'exception_only';
```

| 模式 | 说明 | 适合场景 |
|---|---|---|
| milestone_only | 只在关键阶段汇报 | 用户不想被打扰 |
| balanced | 阶段完成、风险、预算节点汇报 | 默认 |
| verbose | 更多过程汇报 | 用户想观察 AI 工作过程 |
| exception_only | 只在异常或需决策时汇报 | Autopilot 模式 |

---

## 5. 汇报数据结构

```ts
export interface OrchestratorReport {
  id: string;
  projectRunId: string;
  type: 'startup' | 'milestone' | 'risk' | 'cost' | 'approval' | 'final';
  title: string;
  summary: string;
  completedItems: string[];
  currentRisks: string[];
  nextActions: string[];
  progressPercent: number;
  usedAgents: string[];
  usedModels: string[];
  estimatedCost?: number;
  actualCost?: number;
  requiresUserDecision: boolean;
  approvalRequestId?: string;
  createdAt: string;
}
```

---

## 6. 审批请求数据结构

```ts
export interface ApprovalRequest {
  id: string;
  projectRunId: string;
  title: string;
  reason: string;
  riskLevel: 'medium' | 'high' | 'critical';
  options: ApprovalOption[];
  recommendedOptionId: string;
  deadlinePolicy?: 'pause_until_user_response' | 'use_safe_default';
  createdAt: string;
}

export interface ApprovalOption {
  id: string;
  label: string;
  description: string;
  pros: string[];
  cons: string[];
  estimatedCostImpact?: number;
  estimatedTimeImpact?: string;
}
```

---

## 7. 默认汇报示例

```text
# 阶段汇报：架构设计完成

## 当前结论
项目架构已确定，可以进入代码生成阶段。

## 已完成
- 确定使用 Tauri + React + TypeScript + SQLite。
- 拆分出模型配置、Agent 管理、项目工作区、主控调度四个核心模块。
- 确定 API Key 使用本地加密存储。

## 当前风险
- Tauri 文件系统权限需要单独配置。
- 模型 Provider 适配器必须保持模型无关，不能写死厂商逻辑。

## 成本与进度
- 已用 Agent：Orchestrator、Architect、Security
- 当前进度：22%
- 预算状态：正常

## 下一步
开始生成项目骨架和数据库表结构。

## 是否需要你决策
不需要。下一步属于低风险工程实现，将自动继续。
```

---

## 8. 主控汇报中的内部协作摘要

主控 Agent 汇报时必须包含“团队内部协作摘要”，让用户知道项目不是单模型黑盒执行。

### 8.1 汇报模板

```text
阶段：[当前阶段]

我组织了以下 Agent 完成本阶段工作：
- PM Agent：负责需求边界
- Architect Agent：负责架构取舍
- Security Agent：负责安全审查
- Test Agent：负责测试策略

内部讨论结论：
1. [结论 1]
2. [结论 2]
3. [结论 3]

主要分歧：
- [Agent A] 认为 ...
- [Agent B] 认为 ...
- 我的决策是 ...，原因是 ...

需要你确认：
- [如无，则写：本阶段无须你确认，我会继续推进。]
```

### 8.2 什么时候展示分歧

以下情况必须在汇报中展示：

1. 涉及技术栈选择。
2. 涉及项目范围变化。
3. 涉及安全和隐私。
4. 涉及成本明显上升。
5. 涉及删除、重构、大范围文件修改。
6. 涉及用户之前明确表达的偏好。
