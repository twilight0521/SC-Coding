# 开发任务拆解

本文件按“可交付版本”组织任务，避免把 MVP、公司式协作、Checkpoint 回滚等长期能力混在同一阶段。每次开发只领取一个小任务。

## V0.1 内部原型：本地核心 + Provider Profile

目标：参考 opencode 的核心/客户端分离方式，先做一个可运行的本地核心，不急着做完整公司式协作。

- [ ] 初始化 Tauri + React + TypeScript
- [ ] 配置 Tailwind CSS
- [ ] 配置 shadcn/ui
- [ ] 配置 pnpm workspace
- [ ] 配置 ESLint / Prettier
- [ ] 配置 SQLite
- [ ] 建立基础目录结构
- [ ] 实现主窗口布局
- [ ] 定义 Local Core API 边界：ProjectRun / Provider / Agent / Task / File / Tool / Log
- [ ] 定义 ProviderProtocol：`openai_chat_completions` / `anthropic_messages` / `gemini_generate_content` / `custom_http`
- [ ] 定义 ProviderPreset / ProviderProfile / ModelProfile
- [ ] 建立 `provider_configs`、`model_profiles`、`model_capabilities` 表
- [ ] 实现 API Key 安全存储，数据库只保存 `api_key_ref`
- [ ] 实现 OpenAI-Compatible Protocol Adapter
- [ ] 增加 Minimax preset
- [ ] 增加 DeepSeek preset
- [ ] 增加 Ollama / LM Studio preset
- [ ] 实现 Provider Profile CRUD
- [ ] 实现连接测试
- [ ] 实现基础能力评分 UI

验收标准：

- [ ] 用户可以用 Base URL + API Key + Model ID 添加 OpenAI-Compatible Profile
- [ ] 用户可以通过 Minimax / DeepSeek preset 快速创建 Profile
- [ ] API Key 不明文落盘
- [ ] 连接测试能返回成功、失败原因、延迟

## V0.2 MVP：单项目开发闭环

目标：用户可以创建一个小项目，完成“任务拆解 → 代码生成 → Diff → 测试 → 修复 → README”的闭环。

### Agent 与 Router

- [ ] 定义 AgentRole
- [ ] 定义 AgentConfig
- [ ] 建立 `agents`、`agent_permissions`、`agent_fallback_providers` 表
- [ ] 实现 Agent CRUD
- [ ] 实现 Agent 绑定主 Provider Profile / ModelProfile
- [ ] 实现 Agent 绑定备用 Provider Profile
- [ ] 实现默认 Agent：Orchestrator、Coder、Tester、Debugger、Doc
- [ ] 定义 TaskType
- [ ] 定义 RoutingInput / RoutingDecision
- [ ] 实现基础任务权重表
- [ ] 实现规则版模型评分算法
- [ ] 实现手动覆盖
- [ ] 实现失败降级
- [ ] 实现成本预估
- [ ] 建立 `routing_history` 表
- [ ] 将路由选择写入 `decision_logs`

### 项目工作区

- [ ] 创建 `projects` 表
- [ ] 实现新建项目
- [ ] 实现打开本地文件夹
- [ ] 实现敏感文件过滤
- [ ] 实现文件树
- [ ] 实现文件读取
- [ ] 集成 Monaco Editor
- [ ] 实现 Diff Viewer
- [ ] 实现 Git 状态读取
- [ ] 实现文件修改确认

### ProjectRun 与 TaskRunner

- [ ] 创建 `project_runs` 表
- [ ] 创建 `tasks`、`task_dependencies` 表
- [ ] 创建 `run_events`、`decision_logs` 表
- [ ] 实现 ProjectRunService：create / start / pause / cancel
- [ ] 实现 TaskRunner：串行执行、依赖判断、失败重试
- [ ] 实现 ContextBuilder：按任务收集必要文件片段
- [ ] 实现 Agent Prompt 组装
- [ ] 实现结构化输出解析
- [ ] 实现输出格式修复
- [ ] 实现 Agent 执行日志
- [ ] 实现模型调用日志 `model_call_logs`

### 开发闭环

- [ ] 实现需求输入
- [ ] Orchestrator 生成任务列表
- [ ] 用户确认任务列表和预算
- [ ] Coder 生成 patch
- [ ] Integration 应用普通 patch
- [ ] Tester 运行测试命令
- [ ] 捕获 stdout / stderr
- [ ] Debugger 最多自动修复 3 轮
- [ ] Doc 生成 README
- [ ] Orchestrator 生成交付报告

验收标准：

- [ ] 用户可以配置一个 OpenAI-Compatible Provider Profile
- [ ] 用户可以使用 Minimax / DeepSeek preset
- [ ] 用户可以创建 Agent 并绑定模型
- [ ] 用户可以创建项目并输入需求
- [ ] Orchestrator 可以拆解任务
- [ ] Coder 可以生成 patch
- [ ] Diff 可以展示修改
- [ ] Tester 可以运行命令并捕获错误
- [ ] Debugger 可以根据错误修复，最多 3 轮
- [ ] Doc Agent 可以生成 README
- [ ] 模型失败时可以切换备用模型
- [ ] API Key 不会明文落盘

## V0.3 Provider 生态与可观测性

目标：扩展模型接入能力，但仍保持 profile/preset 优先。

- [ ] 实现 Anthropic Protocol Adapter
- [ ] 实现 Gemini Protocol Adapter
- [ ] 实现 Custom HTTP Adapter
- [ ] 增加 OpenAI preset
- [ ] 增加 Anthropic preset
- [ ] 增加 Gemini preset
- [ ] 增加 Qwen / Kimi / GLM / Mistral / Cohere preset
- [ ] 增加 OpenRouter / LiteLLM 类聚合服务 preset
- [ ] 实现 Provider Profile 导入 / 导出，默认不导出 API Key
- [ ] 实现代理 / 镜像 endpoint 配置
- [ ] 实现 provider 级并发和 rate limit
- [ ] 实现成本统计页
- [ ] 实现日志分页和索引
- [ ] 记录历史成功率，但暂不让复杂学习策略影响 MVP 路由

## V0.4 Project Command Center

目标：把主界面从手动 Agent 控制台收敛为项目运行中心。

- [ ] 实现项目阶段时间线
- [ ] 实现主控智能体汇报流
- [ ] 实现当前活跃 Agent 展示
- [ ] 实现模型使用和成本展示
- [ ] 实现待审批卡片
- [ ] 实现风险卡片
- [ ] 实现自动运行状态展示
- [ ] 实现可折叠 Diff / Terminal 面板
- [ ] 实现最终交付物面板

## V0.5 场景驱动 Agent 组织生成器

目标：用户输入场景，系统推荐 Agent Team Plan。第一版用规则 + LLM 混合策略，不追求完全智能。

- [ ] 定义 ScenarioPlan、AgentTeamPlan、PlannedAgent、PromptVersion
- [ ] 新增 Scenario Input 页面
- [ ] 支持选择交付级别：Demo / MVP / Production
- [ ] 支持选择偏好：速度 / 成本 / 质量 / 隐私
- [ ] 实现场景规则分析
- [ ] 根据场景生成推荐 Agent 数量
- [ ] LLM 生成每个 Agent 的职责和 Prompt
- [ ] Router 根据 ModelProfile 能力推荐模型
- [ ] 支持新增 / 删除 / 合并 / 拆分 Agent
- [ ] 支持修改职责、权限、绑定模型
- [ ] 支持 Prompt 编辑和版本保存
- [ ] 用户调整后重新计算风险和成本
- [ ] 确认后转成真实 AgentConfig 和 ProjectRun

## V0.6 公司式 Agent 协作可视化

目标：让用户可以展开查看内部讨论，但默认仍只看主控汇报。

- [ ] 新增 `agent_threads` 表
- [ ] 新增 `agent_messages` 表
- [ ] 新增 `agent_meetings` 表
- [ ] 实现 AgentMessageRepository
- [ ] 实现 Agent Thread 创建逻辑
- [ ] 实现会议类型：需求澄清、架构评审、代码评审、Bug 复盘、冲突仲裁
- [ ] Orchestrator 能从 Agent 消息中生成会议摘要
- [ ] Orchestrator 能将重要结论写入 `decision_logs`
- [ ] 在 Project Command Center 中新增“内部讨论”面板
- [ ] 支持按会议类型 / 任务查看
- [ ] Raw Message 默认关闭，仅 debug 模式保存

## V0.7 暂停、微调与恢复

目标：支持运行中暂停和局部调整，但先做轻量恢复，不直接做完整文件系统回滚。

- [ ] 实现 `pauseProjectRun()`
- [ ] 实现 `resumeProjectRun()`
- [ ] 新增 `human_interventions` 表
- [ ] 新增 `impact_assessments` 表
- [ ] 保存当前任务队列和未应用 patch 引用
- [ ] 用户可修改 Agent、Prompt、模型、任务范围
- [ ] 修改后生成 ImpactAssessment
- [ ] 用户确认后继续执行
- [ ] 基于 Git diff / patch 队列实现轻量回滚

## V1.0 稳定版

目标：稳定交付个人 AI 软件公司体验。

- [ ] 完整 Project Command Center
- [ ] 自主项目交付流
- [ ] 高风险审批机制
- [ ] 成本与预算控制
- [ ] 交付报告
- [ ] 可追溯决策日志
- [ ] Windows / macOS 打包
- [ ] MVP 验收测试完整通过
