# 版本路线图

路线图按“先闭环、再扩展、最后公司式协作”的顺序推进。不要把 V1/V2 能力提前塞进 MVP。

## V0.1 内部原型

目标：验证本地 Project Run Core、Provider Profile 和 Agent 绑定。

- Tauri 基础 App
- Local Core API 边界
- SQLite
- ProviderPreset / ProviderProfile / ModelProfile
- OpenAI-Compatible Protocol Adapter
- Minimax preset
- DeepSeek preset
- Ollama / LM Studio preset
- API Key 安全存储
- Agent CRUD
- Agent 绑定模型
- 简单任务拆解

## V0.2 MVP

目标：用户可以完成一个小项目的 AI 开发闭环。

- ProjectRunService
- TaskRunner
- Router 基础版
- 默认 Agent：Orchestrator、Coder、Tester、Debugger、Doc
- 项目工作区
- 文件树
- Diff Viewer
- Terminal
- Test / Debug Loop，最多 3 轮
- README 生成
- 交付报告
- 决策日志
- Windows / macOS 打包

## V0.3 Provider 生态

目标：扩展模型接入，但仍保持 preset/profile 优先。

- Anthropic Protocol Adapter
- Gemini Protocol Adapter
- Custom HTTP Adapter
- OpenAI / Anthropic / Gemini preset
- Qwen / Kimi / GLM / Mistral / Cohere preset
- OpenRouter / LiteLLM 类聚合服务 preset
- Provider Profile 导入 / 导出
- 代理 / 镜像 endpoint
- Provider 级 rate limit
- 成本统计

## V0.4 Project Command Center

目标：把主界面从任务列表升级为项目运行中心。

- 阶段时间线
- 主控汇报流
- 活跃 Agent 展示
- 模型使用和成本展示
- 待审批卡片
- 风险卡片
- 可折叠 Diff / Terminal
- 最终交付物面板

## V0.5 场景驱动 Agent 团队生成

目标：用户输入场景，系统推荐团队。

- ScenarioPlan
- AgentTeamPlan
- PlannedAgent
- PromptVersion
- 规则版 Agent 数量推荐
- LLM 生成职责和 Prompt
- Router 推荐模型
- 用户编辑 Agent / Prompt / 模型
- 确认后创建真实 AgentConfig 和 ProjectRun

## V0.6 公司式 Agent 协作可视化

目标：让用户可展开查看内部协作，但默认只看主控汇报。

- Agent Thread
- Agent Message
- Agent Meeting
- 需求澄清会议
- 架构评审会议
- 代码评审会议
- Bug 复盘会议
- 冲突仲裁
- 会议摘要
- Raw Message debug 模式

## V0.7 暂停、微调与轻量恢复

目标：用户运行中可以接管方向。

- 暂停 ProjectRun
- 保存任务队列和未应用 patch
- 修改 Agent / Prompt / 模型 / 任务范围
- ImpactAssessment
- 确认后继续执行
- 基于 Git diff / patch 队列的轻量回滚

## V1.0 稳定版

目标：个人 AI 软件公司稳定体验。

- 完整自主项目交付流
- 高风险审批机制
- 预算控制
- Project Command Center
- 可追溯决策日志
- 多模型 Provider Profile 生态
- 可运行项目交付成功率达到 MVP 指标

## V2.0 SuperCompany Platform

目标：从 Coding 扩展为多岗位 AI 公司。

- Agent 市场
- 模板市场
- 插件系统
- 可选云同步
- 团队版
- SuperCompany Design / Research / Recruit / Sales / Ops / Data
