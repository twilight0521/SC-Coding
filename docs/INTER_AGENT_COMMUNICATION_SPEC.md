# INTER_AGENT_COMMUNICATION_SPEC.md

# SuperCompany Coding 智能体协作通信规范

## 1. 模块定位

本模块用于把 SuperCompany Coding 从“黑盒自动执行工具”升级为“可观察、可暂停、可干预的 AI 软件公司”。

用户不需要逐个点击 Agent 推进任务，但必须能看到 Agent 之间如何讨论、分歧、评审、决策和交付。

核心原则：

> 用户是 CEO，主控 Agent 是项目负责人，其他 Agent 是研发团队成员。  
> 用户只听主控汇报，但可以随时打开公司内部会议室，查看和干预 Agent 之间的协作。

---

## 2. 产品目标

### 2.1 解决的问题

传统多 Agent 工具常见问题：

1. Agent 之间协作过程不可见，用户不知道为什么这么做。
2. 系统自动执行时像黑盒，用户只能看到最终结果。
3. 出错后不知道是谁的责任，也不知道在哪个环节偏了。
4. 用户想调整时，只能停止整个项目，无法局部修改。
5. Agent 之间没有“公司式协作感”，缺少会议、评审、争议、决策记录。

### 2.2 本模块要做到

1. **展示 Agent 内部协作过程**：谁向谁提问、谁提出方案、谁反驳、谁评审、谁最终拍板。
2. **主控统一汇报**：默认只给用户展示主控 Agent 总结，不让用户被子 Agent 消息刷屏。
3. **可展开查看细节**：用户可点开某个阶段，查看完整 Agent 对话和决策链。
4. **可随时暂停微调**：暂停后可改 Agent 数量、职责、模型、Prompt、任务范围和预算。
5. **可恢复执行**：调整后由主控 Agent 重新评估影响，再继续推进项目。

---

## 3. Agent 通信结构

### 3.1 通信层级

Agent 通信分为 4 层：

```text
Level 1：CEO View
  用户只看到主控 Agent 的阶段汇报、风险汇报和决策请求。

Level 2：Project Room View
  用户看到不同 Agent 在当前阶段的讨论摘要。

Level 3：Agent Thread View
  用户查看某个任务下的完整 Agent 对话。

Level 4：Raw Message View
  用户查看原始模型输入、输出、上下文和工具调用日志。
```

默认展示 Level 1，用户需要时再展开 Level 2 / Level 3 / Level 4。

---

## 4. Agent 对话类型

### 4.1 需求澄清会议

触发时机：用户提交项目场景后。

参与 Agent：

- Orchestrator Agent
- PM Agent
- Architect Agent
- Product Risk Agent，可选
- UX Agent，可选

目标：

1. 理解用户真实目标。
2. 判断需求是否过大。
3. 识别缺失信息。
4. 形成 MVP 范围。
5. 输出主控汇报。

示例：

```text
PM Agent：当前需求同时包含模型配置、多 Agent 编排、代码执行、桌面端打包，建议 MVP 聚焦模型配置 + Agent 编排 + 本地项目执行。

Architect Agent：如果第一版就做插件市场和云同步，架构复杂度会明显上升，建议后置。

Orchestrator Agent：我建议将第一版拆为 6 个核心模块，并暂不做云端同步。是否按该范围继续？
```

---

### 4.2 架构评审会议

触发时机：技术方案生成后、正式编码前。

参与 Agent：

- Architect Agent
- Backend Agent
- Frontend Agent
- Security Agent
- Test Agent
- Orchestrator Agent

目标：

1. 确认技术栈是否合理。
2. 确认模块边界。
3. 找出安全风险。
4. 找出测试难点。
5. 形成最终架构决策。

输出内容：

- 架构方案
- 争议点
- 取舍理由
- 被否决方案
- 最终决策

---

### 4.3 开发同步会议

触发时机：每个开发阶段开始或结束时。

参与 Agent：

- 当前阶段相关 Agent
- Orchestrator Agent

目标：

1. 对齐当前阶段目标。
2. 分配文件和模块。
3. 避免多个 Agent 修改同一文件冲突。
4. 设定阶段验收标准。

---

### 4.4 代码评审会议

触发时机：重要模块完成后。

参与 Agent：

- Coder Agent
- Reviewer Agent
- Test Agent
- Security Agent，可选
- Orchestrator Agent

目标：

1. 审查代码质量。
2. 判断是否符合需求。
3. 找出潜在 Bug。
4. 检查是否引入安全问题。
5. 决定是否进入测试。

---

### 4.5 Bug 复盘会议

触发时机：测试失败、构建失败或运行报错。

参与 Agent：

- Debug Agent
- Test Agent
- 原开发 Agent
- Architect Agent，可选
- Orchestrator Agent

目标：

1. 判断错误来源。
2. 分配修复责任。
3. 避免重复修复。
4. 决定是否升级到更强模型。
5. 形成修复计划。

---

### 4.6 冲突仲裁会议

触发时机：Agent 之间出现方案分歧。

典型分歧：

1. 技术栈选择分歧。
2. 模块边界分歧。
3. 是否重构分歧。
4. 是否增加依赖分歧。
5. 是否扩大范围分歧。
6. 是否需要用户确认分歧。

仲裁规则：

1. Orchestrator Agent 先总结分歧。
2. 各 Agent 给出理由和风险。
3. 如属于低风险工程选择，Orchestrator Agent 直接决策。
4. 如涉及产品范围、成本、隐私、安全、删除文件、上线发布，必须请求用户确认。

---

## 5. 消息类型设计

### 5.1 Message Type

```ts
type AgentMessageType =
  | 'proposal'       // 提案
  | 'question'       // 提问
  | 'answer'         // 回答
  | 'objection'      // 反对意见
  | 'review'         // 评审意见
  | 'decision'       // 决策
  | 'risk'           // 风险提示
  | 'handoff'        // 任务交接
  | 'status'         // 状态更新
  | 'test_result'    // 测试结果
  | 'debug_report'   // 调试报告
  | 'user_request'   // 用户干预
  | 'system_event';  // 系统事件
```

### 5.2 消息结构

```json
{
  "id": "msg_001",
  "projectRunId": "project_run_001",
  "threadId": "thread_arch_review_001",
  "fromAgentId": "agent_architect",
  "toAgentId": "agent_backend",
  "type": "question",
  "title": "确认本地数据库方案",
  "content": "模型配置和 Agent 配置是否统一存 SQLite？API Key 是否只存 Keychain 引用？",
  "relatedTaskId": "task_database_design",
  "relatedFiles": ["src/db/schema.ts"],
  "riskLevel": "medium",
  "createdAt": "2026-05-07T10:00:00Z"
}
```

---

## 6. 会议室模型

### 6.1 Project Room

Project Room 是项目级会议室，展示当前项目的所有关键讨论。

内容包括：

1. 当前阶段。
2. 正在讨论的问题。
3. 参与 Agent。
4. 当前结论。
5. 待用户确认事项。
6. 历史会议记录。

### 6.2 Task Thread

Task Thread 是任务级讨论线程。

每个任务都应有自己的线程，记录：

1. 任务目标。
2. 分配给哪个 Agent。
3. Agent 如何理解任务。
4. 和其他 Agent 的接口对齐。
5. 代码修改说明。
6. 评审意见。
7. 测试结果。
8. 是否完成。

### 6.3 Decision Log

所有重要决策必须进入 Decision Log。

记录字段：

1. 决策内容。
2. 决策人：Orchestrator / User。
3. 参与 Agent。
4. 备选方案。
5. 选择理由。
6. 风险。
7. 后续影响。

---

## 7. 用户看到的协作展示

### 7.1 默认 CEO 视图

用户看到主控智能体汇报：

```text
阶段：架构设计完成

我已经组织 PM、Architect、Security、Test 四个智能体完成架构评审。

结论：
1. 第一版建议使用 Tauri + React + SQLite。
2. API Key 不进入 SQLite，只保存 Keychain 引用。
3. 多 Agent 通信先使用本地事件流，不做云端同步。
4. 高风险命令默认禁止执行。

分歧：
Architect 建议引入插件系统预留接口，但 PM 认为 MVP 暂不开放插件市场。我已决定只保留扩展接口，不做插件市场 UI。

下一步：进入项目初始化和数据库结构开发。
```

### 7.2 展开后看到公司内部讨论

```text
Architect Agent：建议将 Agent Runtime 与 UI 解耦，否则后续接插件会困难。

PM Agent：MVP 不应过度设计插件系统，用户第一阶段只关心能否跑通项目。

Security Agent：无论是否做插件，都需要提前约束命令执行权限。

Orchestrator Agent：采纳 Security 意见；插件市场后置，但 Runtime 层保留 adapter 接口。
```

---

## 8. 中断与恢复机制

Agent 通信必须支持项目随时暂停。

暂停后必须保存：

1. 当前阶段。
2. 当前任务状态。
3. 所有 Agent 消息。
4. 当前上下文摘要。
5. 未完成文件变更。
6. 用户已确认和未确认决策。
7. 当前模型调用预算。
8. 当前 Prompt 版本。

恢复时，Orchestrator Agent 先读取快照，并向用户汇报：

```text
项目已暂停在“前端模块开发”阶段。

当前状态：
- 12 个任务已完成 7 个
- 2 个任务正在进行
- 3 个任务待执行
- 当前有 1 个风险：Frontend Agent 和 Integration Agent 都需要修改 routes.ts

你刚刚修改了 Frontend Agent 的 Prompt，我会重新评估相关任务，并只重跑受影响的部分。
```

---

## 9. 微调后影响评估

用户修改以下内容后，Orchestrator Agent 必须做影响评估：

| 用户操作 | 系统必须评估 |
|---|---|
| 删除 Agent | 哪些任务失去负责人，是否需要重新分配 |
| 新增 Agent | 哪些任务适合转交给新 Agent |
| 修改 Agent 职责 | 当前任务是否仍然匹配该 Agent |
| 修改模型 | 成本、速度、质量、上下文能力是否变化 |
| 修改 Prompt | 哪些任务需要重新执行或重新评审 |
| 修改项目范围 | PRD、任务树、架构和测试是否需要更新 |
| 修改预算 | 是否需要降级模型或减少评审轮次 |

---

## 10. 不能打扰用户的情况

以下情况不应打扰用户，只在阶段汇报中总结：

1. Agent 之间的一般接口确认。
2. 普通代码风格分歧。
3. 普通测试失败后的自动修复。
4. 低风险依赖版本调整。
5. 文档措辞调整。
6. 非核心文件的重命名。

---

## 11. 必须打扰用户的情况

以下情况必须暂停并请求用户确认：

1. 项目范围明显扩大。
2. 预计成本超过用户预算。
3. 需要执行高风险命令。
4. 需要删除或覆盖大量文件。
5. 涉及 API Key、隐私数据、简历数据、合同数据等敏感信息。
6. 需要联网下载未知脚本。
7. 需要推送远程仓库或部署上线。
8. Agent 之间出现无法由主控决策的产品方向分歧。

---

## 12. 验收标准

MVP 阶段至少实现：

1. 每个任务有独立 Agent Thread。
2. 主控 Agent 能生成阶段汇报。
3. 用户可展开查看 Agent 协作摘要。
4. 重要决策进入 Decision Log。
5. 用户可暂停项目。
6. 暂停后可修改 Agent、模型、Prompt、任务范围。
7. 修改后 Orchestrator 能生成影响评估。
8. 用户确认后可继续执行。
