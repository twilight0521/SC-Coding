# SuperCompany Coding 文档总览

版本：v5 Company-Style Agent Collaboration

## 版本定位

本版本将 SuperCompany Coding 的长期愿景定义为“公司式 AI 研发团队”，但工程实现必须先收敛为“本地 Project Run Core + Provider Profile + 单项目闭环 MVP”。

核心变化：

1. 参考 opencode：核心执行、事件、工具、会话状态放在本地 Core，桌面 UI 是客户端。
2. 参考 CC Switch / Claude Code Router：模型接入优先用 Provider Preset / Profile / Model Profile，不把厂商逻辑写进 Agent。
3. MVP 先跑通单项目开发闭环。
4. 公司式内部会议、暂停微调、Checkpoint 回滚是后续增强，不阻塞 MVP。

---

## 文档列表

| 文件 | 说明 |
|---|---|
| `PRD.md` | 产品需求文档，定义产品定位、核心流程和功能范围 |
| `CLAUDE.md` | Claude Code / AI Coding 执行规范 |
| `../README.md` | GitHub 项目首页、启动指引与仓库说明 |
| `ARCHITECTURE.md` | 技术架构设计 |
| `AGENT_SPEC.md` | 多 Agent 角色、职责、通信与协作规范 |
| `SCENE_AGENT_PLANNER_SPEC.md` | 场景驱动 Agent 团队生成器 |
| `AUTONOMOUS_DELIVERY_SPEC.md` | 自主项目交付流 |
| `INTER_AGENT_COMMUNICATION_SPEC.md` | Agent 之间的公司式通信、会议、评审、决策记录 |
| `HUMAN_CONTROL_LOOP_SPEC.md` | 用户暂停、微调、恢复、回滚和影响评估规范 |
| `ORCHESTRATOR_REPORTING_SPEC.md` | 主控 Agent 汇报规范 |
| `MODEL_CAPABILITY_MATRIX.md` | 不同模型能力矩阵和任务适配 |
| `MODEL_PROVIDER_SPEC.md` | 模型 Provider 接入规范 |
| `ROUTER_SPEC.md` | 模型路由、成本控制、失败升级和重路由 |
| `DATABASE_SCHEMA.md` | SQLite 数据库设计 |
| `UI_SPEC.md` | 页面结构与交互设计 |
| `SECURITY.md` | 安全、隐私、命令执行和 API Key 管理规范 |
| `TASKS.md` | MVP 到后续版本的开发任务拆解 |
| `TEST_PLAN.md` | 测试计划与验收标准 |
| `ROADMAP.md` | 产品版本规划 |
| `REFERENCE_NOTES.md` | 参考说明 |
| `.env.example` | 环境变量示例 |
| `package.json.example` | 推荐脚本约定 |

---

## 建议阅读顺序

### 给产品 / 创始人

1. `PRD.md`
2. `SCENE_AGENT_PLANNER_SPEC.md`
3. `AUTONOMOUS_DELIVERY_SPEC.md`
4. `INTER_AGENT_COMMUNICATION_SPEC.md`
5. `HUMAN_CONTROL_LOOP_SPEC.md`
6. `ROADMAP.md`

### 给 Claude Code / Codex / AI Coding 工具

1. `CLAUDE.md`
2. `TASKS.md`
3. `ARCHITECTURE.md`
4. `DATABASE_SCHEMA.md`
5. `MODEL_PROVIDER_SPEC.md`
6. `ROUTER_SPEC.md`
7. `UI_SPEC.md`
8. `AGENT_SPEC.md`

### 给研发

1. `ARCHITECTURE.md`
2. `DATABASE_SCHEMA.md`
3. `MODEL_PROVIDER_SPEC.md`
4. `ROUTER_SPEC.md`
5. `AGENT_SPEC.md`
6. `INTER_AGENT_COMMUNICATION_SPEC.md`
7. `HUMAN_CONTROL_LOOP_SPEC.md`
8. `SECURITY.md`

---

## MVP 核心产品流程

```text
用户配置 Provider Profile
  ↓
用户创建 Agent 并绑定模型
  ↓
用户创建项目并输入需求
  ↓
Orchestrator 生成任务清单
  ↓
用户确认任务、预算和权限
  ↓
TaskRunner 调度 Agent 生成 patch
  ↓
Diff 展示修改
  ↓
Tester 运行测试
  ↓
Debugger 最多自动修复 3 轮
  ↓
Doc 生成 README 与交付报告
```

---

## 后续增强重点

### 1. Agent 内部通信

新增：`INTER_AGENT_COMMUNICATION_SPEC.md`

包括：

- Agent Message
- Agent Thread
- Agent Meeting
- Decision Log
- 需求澄清会议
- 架构评审会议
- 开发同步会议
- 代码评审会议
- Bug 复盘会议
- 冲突仲裁会议

### 2. 人类控制回路

新增：`HUMAN_CONTROL_LOOP_SPEC.md`

包括：

- 暂停项目
- 继续执行
- 修改 Agent 数量
- 修改 Agent 模型
- 修改 Agent Prompt
- 修改任务范围
- 修改预算
- 影响评估
- Checkpoint 回滚

### 3. Project Command Center

更新：`UI_SPEC.md`

产品主界面不再只是任务列表，而是 AI 公司作战室：

- 主控汇报
- Agent 团队状态
- 内部讨论
- 决策日志
- 文件 Diff
- Terminal
- 测试结果
- 暂停 / 恢复 / 微调入口

---

## 一句话定义

SuperCompany Coding 是一个公司式多 Agent AI Coding 工作台。用户只需要提出项目目标，AI 会自动组建团队、讨论方案、分工开发、测试修复并交付项目；用户主要听主控智能体汇报，并可随时暂停、查看内部讨论和调整团队方向。
