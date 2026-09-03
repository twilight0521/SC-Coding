# SuperCompany Coding

> Everyone can code. Everyone is a super company.

SuperCompany Coding 是一个模型无关的多 Agent AI Coding 工作台。你可以接入不同的模型，为每个 Agent 分配职责，让 AI 团队共同完成需求拆解、代码生成、测试、调试和文档交付。

## 项目状态

当前仓库处于 MVP 原型阶段，重点是跑通“本地核心 + Provider Profile + 单项目开发闭环”。产品设计面向任意模型，不会把 Agent 逻辑绑定到某一家模型厂商。

## 核心能力

- **Provider 配置**：通过 OpenAI-Compatible API 接入 Minimax、DeepSeek、OpenRouter、Ollama、LM Studio 等服务。
- **Agent 团队**：创建多个 Agent，为每个 Agent 绑定主模型、备用模型、职责 Prompt 和权限。
- **智能路由**：根据任务类型、模型能力、成本和速度要求推荐模型，并支持失败降级。
- **项目工作区**：打开本地项目，查看文件树、代码 Diff 和 Git 状态。
- **开发闭环**：任务拆解 → 代码生成 → Diff 确认 → 测试 → 自动修复 → README 与交付报告。
- **自主交付方向**：后续支持由 Orchestrator 自动组建团队、推进阶段并向用户汇报。

## 技术栈

| 层级          | 技术                    |
| ------------- | ----------------------- |
| 桌面应用      | Tauri 2                 |
| 前端          | React 19 + TypeScript   |
| 样式          | Tailwind CSS            |
| 状态管理      | Zustand                 |
| 本地核心      | Rust + Tauri Commands   |
| 数据存储      | SQLite                  |
| 编辑器与 Diff | Monaco Editor（规划中） |

## 快速开始

### 环境要求

- Node.js `>= 20`
- pnpm `>= 9`
- Rust `>= 1.78`
- Tauri 运行所需的系统依赖，参见 [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### 安装与运行

```bash
git clone <your-repository-url>
cd SuperCompany_Coding_Docs_v5_Company_Style_Agent_Collaboration
pnpm install
pnpm dev
```

构建桌面应用：

```bash
pnpm build
```

常用检查命令：

```bash
pnpm typecheck
pnpm lint
```

## 仓库结构

```text
.
├── apps/desktop/       # Tauri + React 桌面端
├── packages/core/      # Project Run Core 与运行时能力
├── packages/shared/    # 前后端共享类型与工具
├── docs/               # 产品、架构和开发规范
├── package.json        # 根级脚本
└── pnpm-workspace.yaml # pnpm workspace 配置
```

## 文档导航

- [文档总览](docs/00_INDEX.md)
- [产品需求（PRD）](docs/PRD.md)
- [技术架构](docs/ARCHITECTURE.md)
- [Agent 规范](docs/AGENT_SPEC.md)
- [Provider 接入规范](docs/MODEL_PROVIDER_SPEC.md)
- [模型能力矩阵](docs/MODEL_CAPABILITY_MATRIX.md)
- [路由规范](docs/ROUTER_SPEC.md)
- [开发任务拆解](docs/TASKS.md)
- [测试计划](docs/TEST_PLAN.md)
- [安全规范](docs/SECURITY.md)
- [产品路线图](docs/ROADMAP.md)

建议先阅读 [文档总览](docs/00_INDEX.md)，再根据角色查看产品、研发或 AI Coding 执行规范。

## 设计原则

1. **模型无关**：Agent 只依赖统一的 Provider Adapter，不直接依赖厂商品牌。
2. **本地优先**：Project Run Core 负责状态、事件、工具调用和持久化，桌面 UI 作为客户端。
3. **配置与运行时分离**：Provider Preset、Profile 和 Model Profile 负责接入配置，Runtime 负责执行任务。
4. **渐进式交付**：优先完成单项目 MVP，再扩展 Agent 内部通信、人工控制回路和 Checkpoint。

## 参与贡献

欢迎提交 Issue 和 Pull Request。提交代码前请先阅读 [开发执行规范](docs/AGENTS.md)，并确保通过类型检查和 lint。

## License

License 尚未确定。
