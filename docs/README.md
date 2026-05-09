# SuperCompany Coding

**SuperCompany Coding** 是一款模型无关的多 Agent 协同开发桌面 App。

它的目标不是再做一个单模型 AI Chat，而是让用户用不同模型组建自己的 AI 软件公司。

> Everyone can code. Everyone is a super company.

## 核心能力

- 配置任意模型 Provider
- 支持 OpenAI-Compatible API
- 支持 Provider Preset / Profile 方式接入 Minimax、DeepSeek、Claude、Gemini、本地模型等
- 创建多个 Agent
- 为每个 Agent 指定不同模型
- 根据任务自动推荐最合适模型
- 多 Agent 协同完成开发
- 自动生成代码、展示 Diff、运行测试、修复错误、生成 README

## 推荐技术栈

- Tauri
- React
- TypeScript
- Tailwind CSS
- SQLite
- Monaco Editor
- xterm.js

## 快速开始

```bash
pnpm install
pnpm dev
```

## 推荐环境

```bash
node >= 20
pnpm >= 9
rust >= 1.78
```

## 文档

开发前请阅读：

1. `PRD.md`
2. `CLAUDE.md`
3. `ARCHITECTURE.md`
4. `MODEL_CAPABILITY_MATRIX.md`
5. `MODEL_PROVIDER_SPEC.md`
6. `AGENT_SPEC.md`
7. `ROUTER_SPEC.md`
8. `TASKS.md`

## MVP 功能

- Provider 配置中心
- Provider Preset / Profile / Model Profile
- Agent 管理中心
- Agent 与模型绑定
- 项目工作区
- 多 Agent 任务拆解
- 代码 Diff
- 终端执行
- 测试修复闭环

## 当前默认模型策略

如果用户只有 Minimax 和 DeepSeek：

- Minimax M2.7：主控、统筹、集成、复杂项目推进
- DeepSeek Pro：复杂代码、架构、Debug、后端
- DeepSeek Flash：简单模块、测试、文档、批量修复

但产品设计不局限于它们。

## 架构收敛原则

当前实现优先级参考 opencode 的本地核心思路：先做可复用的 Project Run Core，再让桌面 UI 调用它。模型接入参考 CC Switch / Claude Code Router 类工具的 profile/preset 思路：Minimax、DeepSeek 等优先作为 OpenAI-Compatible preset，不在 Agent Runtime 里写厂商品牌分支。

MVP 暂不做：

- 全量 Agent 内部会议记录；
- 完整 Checkpoint 文件系统回滚；
- 所有厂商的一方 Adapter；
- 历史成功率学习驱动的复杂 Router。

## 长期支持模型

- OpenAI
- Anthropic Claude
- Google Gemini
- Minimax
- DeepSeek
- Qwen
- Kimi
- Doubao / Seed
- GLM
- Grok
- Mistral
- Cohere
- Ollama / LM Studio / vLLM / LocalAI
- Custom HTTP Provider

---

## 场景驱动 Agent 团队生成

SuperCompany Coding 后续版本将支持：用户先输入产品场景，系统自动预估需要多少 Agent、每个 Agent 的职责、推荐模型和 Prompt。

示例：

```text
用户：我要做一个 Windows/macOS 桌面端简历匹配工具。

系统：建议使用 8 个 Agent：
- Orchestrator：统筹任务，推荐 Minimax M2.7 / Claude / OpenAI reasoning
- PM：需求拆解，推荐 Minimax / Claude / Qwen
- Architect：架构设计，推荐 Claude / DeepSeek Pro / OpenAI reasoning
- Frontend：页面开发，推荐 Claude / DeepSeek Pro / Qwen
- Backend：本地文件和匹配逻辑，推荐 DeepSeek Pro / Claude
- Test：测试，推荐 DeepSeek Flash / Qwen
- Debug：错误修复，推荐 DeepSeek Pro / Claude
- Doc：文档，推荐 DeepSeek Flash / Qwen
```

用户可以在执行前自由增删 Agent、修改职责、替换模型、编辑 Prompt。

## 新主流程：自主项目交付

SuperCompany Coding 不应该要求用户一个个点击 Agent 执行任务。

正确流程是：

```text
输入场景
  ↓
AI 生成 Agent 团队、模型推荐、Prompt、执行计划
  ↓
用户一次性调整并确认
  ↓
主控智能体自动推进项目
  ↓
用户只听阶段汇报并处理关键审批
  ↓
最终验收可运行项目
```

默认主界面是 `Project Command Center`，不是手动 Agent 控制台。

相关文档：

- `AUTONOMOUS_DELIVERY_SPEC.md`
- `ORCHESTRATOR_REPORTING_SPEC.md`
- `SCENE_AGENT_PLANNER_SPEC.md`

---

## 公司式 Agent 协作

SuperCompany Coding 的目标不是让用户手动点击每个 Agent，而是让用户拥有一支可以自动协作的 AI 研发团队。

用户只需要：

1. 输入项目场景。
2. 确认 AI 推荐的 Agent 团队。
3. 听 Orchestrator Agent 阶段汇报。
4. 必要时暂停并微调团队。
5. 最终验收项目。

系统会自动记录：

- Agent 之间的讨论；
- 架构评审会议；
- 代码评审会议；
- Bug 复盘会议；
- 冲突仲裁；
- 主控决策；
- 用户干预；
- 检查点和恢复记录。

相关文档：

- `INTER_AGENT_COMMUNICATION_SPEC.md`
- `HUMAN_CONTROL_LOOP_SPEC.md`
- `AUTONOMOUS_DELIVERY_SPEC.md`
- `ORCHESTRATOR_REPORTING_SPEC.md`
