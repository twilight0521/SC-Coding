# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**SuperCompany Coding** — 公司式多 Agent AI Coding 工作台。用户输入场景，AI 自动组建团队、讨论方案、分工开发、测试修复并交付项目。

核心架构：**Local Project Run Core** + **桌面 UI 客户端**，参考 opencode 的核心/客户端分离模式。

## 开发命令

```bash
# 环境要求: node >= 20, pnpm >= 9, rust >= 1.78

# 安装依赖
pnpm install

# 启动开发服务器 (前端)
pnpm dev

# 启动 Tauri 应用
cd apps/desktop && pnpm tauri dev

# 类型检查
pnpm typecheck

# 代码格式化
cd apps/desktop && pnpm format

# 代码检查
cd apps/desktop && pnpm lint

# 构建桌面应用
pnpm build
```

## 项目结构

```
apps/desktop/
├── src/
│   ├── components/
│   │   ├── ui/           # UI 组件 (Button, Card, Input)
│   │   ├── FileTree.tsx  # 文件树组件
│   │   └── TaskList.tsx  # 任务列表组件
│   ├── pages/
│   │   ├── ProvidersPage.tsx   # Provider 配置页
│   │   ├── AgentsPage.tsx       # Agent 管理页
│   │   ├── ProjectsPage.tsx     # 项目列表页
│   │   ├── ProjectDetailPage.tsx # 项目详情页
│   │   └── WelcomePage.tsx     # 欢迎页
│   ├── stores/
│   │   └── appStore.ts   # Zustand 状态管理
│   └── lib/utils.ts      # 工具函数
├── src-tauri/
│   └── src/
│       ├── commands/
│       │   ├── providers.rs  # Provider/Agent CRUD
│       │   ├── router.rs    # TaskType/Routing
│       │   └── projects.rs   # Project/Task/File
│       ├── db/
│       │   └── schema.rs    # SQLite schema
│       ├── models/          # 数据模型
│       └── security/        # API Key 安全存储
packages/shared/           # 共享类型 (简化处理)
packages/core/             # 核心逻辑 (简化处理)
docs/                      # 产品文档
```

## 架构要点

### 1. 本地核心 + 客户端分离

```
Local Core (Rust + SQLite)
  ├─ ProjectRunService, TaskRunner, OrchestratorRuntime
  ├─ AgentRuntime, ContextBuilder, CapabilityRouter
  └─ Provider Service (Adapter 模式)

Desktop UI (Tauri + React + Zustand)
  └─ 通过 Tauri Commands 与 Core 通信
```

### 2. Provider Adapter 模式

所有模型接入通过统一接口，新增厂商只需添加 Preset：
- Presets: Minimax, DeepSeek, Ollama, LM Studio

### 3. Agent 与模型解耦

Agent 定义职责，模型提供能力。

### 4. 数据层

- **SQLite** 本地存储
- **API Key 安全存储**：数据库只存 `api_key_ref`

## 已实现功能

### Provider 系统
- Provider CRUD
- Provider 连接测试
- 默认 Preset

### Agent 系统
- Agent CRUD
- Agent 权限管理
- 默认 Agent 模板 (Orchestrator, Coder, Tester, Debugger, Doc)

### Router 系统
- TaskType 定义 (13 种任务类型)
- 规则版模型评分算法
- 基于能力权重的路由选择

### Project 系统
- Project CRUD
- 文件树浏览
- 文件读取 (敏感文件除外)
- Task CRUD
- ProjectRun 创建和管理

## 开发约束

1. **禁止硬编码厂商**：代码中不得出现 `if (model === 'minimax')` 分支
2. **API Key 不落盘**：不得存在 localStorage 或日志明文
3. **禁止高风险命令默认执行**：需用户显式确认
4. **敏感文件不发送模型**：`.env`、`*.pem`、`id_rsa`、`node_modules/` 等

## 当前进度

### V0.1 完成 ✓
- [x] Tauri + React + Tailwind 初始化
- [x] ESLint / Prettier 配置
- [x] SQLite 数据库初始化
- [x] ProviderProfile CRUD
- [x] API Key 安全存储
- [x] Agent CRUD
- [x] 默认 Agent 模板

### V0.2 进行中 ✓ (核心功能完成)
- [x] TaskType 和基础 Router
- [x] 项目工作区 (文件树、代码编辑器)
- [x] ProjectRun 和 TaskRunner
- [ ] 开发闭环 (前端完整 UI)

### V0.3 规划
- Anthropic/Gemini Protocol Adapter
- 更多 Provider Presets
- 成本统计页

## Tauri 命令

**Provider:**
- `list_providers`, `get_provider_presets`, `create_provider`, `update_provider`, `delete_provider`, `test_provider_connection`

**Agent:**
- `list_agents`, `get_agent`, `create_agent`, `update_agent`, `delete_agent`, `get_default_agent_templates`, `create_default_agents`, `update_agent_permissions`, `get_agent_permissions`

**Router:**
- `get_task_types`, `route_task`, `save_routing_history`, `get_routing_history`, `get_available_models_for_routing`

**Project:**
- `list_projects`, `get_project`, `create_project`, `update_project`, `delete_project`
- `list_directory`, `read_file`, `get_file_info`
- `list_tasks`, `create_task`, `update_task_status`, `delete_task`
- `create_project_run`, `get_project_runs`, `update_project_run_status`

## 参考文档

- 详细架构：`docs/ARCHITECTURE.md`
- 数据库设计：`docs/DATABASE_SCHEMA.md`
- 开发任务：`docs/TASKS.md`
- 产品需求：`docs/PRD.md`