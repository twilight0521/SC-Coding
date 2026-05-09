# 模型能力矩阵与 Agent 岗位适配

## 1. 设计原则

SuperCompany Coding 不应该问：

> 这个产品支持 Minimax 还是 DeepSeek？

而应该问：

> 当前任务需要什么能力？哪个模型在这个能力上最合适？

所以系统需要把模型抽象为能力标签，而不是品牌标签。

## 2. 能力维度

每个模型配置都需要维护以下能力评分，范围 0-5：

| 能力 | 含义 |
|---|---|
| reasoning | 复杂推理、架构判断、任务规划 |
| coding | 代码生成能力 |
| codeReview | 代码审查、质量判断 |
| longContext | 读取大项目、大文件、长文档 |
| speed | 响应速度 |
| lowCost | 成本优势 |
| toolUse | 工具调用、函数调用、Agent Loop 稳定性 |
| jsonReliability | 稳定输出结构化 JSON |
| multimodal | 图片、PDF、音频、视频等多模态理解 |
| chinese | 中文理解与中文文档能力 |
| localDeploy | 本地部署 / 私有化部署可行性 |
| rag | 检索增强、知识库问答、企业文档理解 |
| realtime | 实时信息检索或外部信息能力 |

能力评分挂在 `ModelProfile` 上，而不是只挂在 Provider 上。同一个 Provider Profile 下的不同模型可以有不同能力、价格和上下文长度。TypeScript 使用 camelCase，例如 `codeReview`；SQLite 使用 snake_case，例如 `code_review`，必须在 Repository 层集中映射。

## 3. 主流模型家族适配建议

> 注意：以下是产品内置“默认推荐策略”，用户可以覆盖。不同模型版本会变化，因此系统必须允许用户手动调整评分。

### 3.1 OpenAI / GPT / Reasoning Models

适合岗位：

- CEO / Judge Agent
- Complex Reasoning Agent
- Architect Agent
- Code Reviewer Agent
- Security Reviewer Agent
- Integration Agent

适合任务：

- 复杂产品方案推理
- 跨模块架构判断
- 复杂 Bug 根因分析
- 安全审查
- 多方案评估
- 高质量代码审查
- 需要严格结构化输出的任务

不优先用于：

- 大量重复文档生成
- 低价值批量任务
- 简单格式化

路由建议：

```text
高复杂度 + 高风险 + 需要准确判断 → OpenAI 强推理模型
低复杂度 + 高频任务 → OpenAI mini / nano 或其他低成本模型
```

### 3.2 Anthropic / Claude

适合岗位：

- Architect Agent
- Senior Coder Agent
- Refactor Agent
- Frontend Quality Agent
- Code Reviewer Agent
- Long Session Agent

适合任务：

- 长时间代码协作
- 大型重构
- 复杂代码库理解
- 前端体验打磨
- 代码审查
- 工具调用 / Computer Use 类任务
- 高质量文档和解释

不优先用于：

- 极低成本批量任务
- 简单文件改名 / 格式化

路由建议：

```text
复杂代码库 + 长上下文 + 需要稳定重构 → Claude
UI 细节 / 代码质量 / 文档质量要求高 → Claude
```

### 3.3 Google / Gemini

适合岗位：

- Long Context Reader Agent
- Multimodal Analyst Agent
- Research Agent
- Repo Understanding Agent
- Requirement Extraction Agent

适合任务：

- 读取超长文档
- 读取完整代码仓库
- PDF / 图片 / 视频 / 音频理解
- 从复杂资料中提取需求
- 分析大量日志
- 多模态输入场景

不优先用于：

- 很小的低成本代码片段生成
- 对终端式 Agent Loop 有特殊要求的任务，除非适配器验证稳定

路由建议：

```text
上下文很长 / 输入有图片 PDF 视频 / 需要大范围理解 → Gemini
```

### 3.4 MiniMax / M 系列

适合岗位：

- CEO / Orchestrator Agent
- Agent Team Lead
- Integration Agent
- Product Agent
- Fullstack Agent
- Document Delivery Agent

适合任务：

- 多 Agent 编排
- 复杂 Agent Harness
- 工程任务统筹
- 代码理解与重构
- 多轮对话式开发
- 高质量办公文档交付
- 项目总结与交付报告

在当前用户配置中的建议：

- Minimax M2.7 作为默认主控模型。
- 负责拆任务、分配 Agent、合并结果、判断是否升级模型。

不优先用于：

- 极简单、极大量、低价值的批处理任务，除非使用 highspeed / 低成本配置。

路由建议：

```text
多 Agent 协调 / 任务统筹 / 交付总结 / 复杂协作 → MiniMax M2.7
```

### 3.5 DeepSeek

适合岗位：

- Backend Agent
- Algorithm Agent
- Debug Agent
- Architect Agent
- Test Agent
- Batch Coding Agent

适合任务：

- 后端模块实现
- 算法逻辑
- 复杂 Bug 修复
- 推理型代码任务
- 单模块实现
- 低成本批量开发
- 测试用例生成

当前用户配置建议：

- DeepSeek Pro：复杂代码、后端、架构、Debug。
- DeepSeek Flash：简单代码、文档、测试、批量修复。

不优先用于：

- 多模态资料理解
- 需要超长上下文读取的全仓库任务，除非模型版本支持并验证可用

路由建议：

```text
复杂代码 → DeepSeek Pro
简单代码 / 测试 / 文档 / 低成本批量 → DeepSeek Flash
```

### 3.6 Qwen

适合岗位：

- Terminal Coding Agent
- Local Coding Agent
- Backend Agent
- Chinese Product Agent
- Open-source Agent

适合任务：

- 终端式代码任务
- 中文产品需求理解
- 阿里云 / 国内生态项目
- 本地或私有化部署
- 代码库理解
- 中低成本模块开发

不优先用于：

- 对海外 API 生态深度依赖的任务，除非配置了对应工具链

路由建议：

```text
国内生态 + 中文需求 + 代码 Agent + 可私有化 → Qwen
```

### 3.7 Moonshot / Kimi

适合岗位：

- Long Context Code Reader Agent
- Repo Understanding Agent
- Refactor Agent
- Document Analysis Agent

适合任务：

- 长上下文代码理解
- 大文件阅读
- 长文档阅读
- 复杂项目迁移
- 代码库级别重构前分析
- 工具调用型开发任务

不优先用于：

- 高频、短小、极低成本任务

路由建议：

```text
长代码库 / 长文档 / 需要一次性读大量上下文 → Kimi
```

### 3.8 ByteDance / Doubao / Seed

适合岗位：

- Fast Coding Agent
- Consumer App Agent
- Multimodal Product Agent
- China Ecosystem Agent
- Cost-efficient Agent

适合任务：

- 面向国内应用生态的产品开发
- C 端应用需求理解
- 快速代码生成
- 多模态理解
- 中低成本多轮任务
- 移动端 / 前端 / 内容类工具

不优先用于：

- 需要严格私有化的场景，除非使用企业部署方案

路由建议：

```text
国内 C 端产品 / 快速迭代 / 成本敏感 / 多模态 → Doubao / Seed
```

### 3.9 Zhipu / GLM

适合岗位：

- Agentic Engineering Agent
- Architect Agent
- Local / Private Deployment Agent
- Code Reasoning Agent

适合任务：

- 复杂工程推理
- Agent 长流程任务
- 私有化部署
- 中文企业场景
- 代码与系统设计

不优先用于：

- 极致低延迟任务，具体取决于部署方式

路由建议：

```text
私有化 + 中文企业 + 工程推理 + Agent 工作流 → GLM
```

### 3.10 xAI / Grok

适合岗位：

- Realtime Research Agent
- Market / Trend Agent
- External Context Agent
- General Coding Agent

适合任务：

- 需要实时外部信息
- 需要结合 X / Web 趋势
- 技术选型调研
- 竞品分析
- 当前资料补充
- 一般代码任务

不优先用于：

- 不需要外部实时信息的低成本批量代码生成

路由建议：

```text
需要实时信息 / 市场趋势 / 技术动态 → Grok
```

### 3.11 Mistral / Codestral

适合岗位：

- Code Generation Agent
- Local Coding Agent
- Completion Agent
- European / Open-weight Deployment Agent

适合任务：

- 代码生成
- 代码补全
- 本地部署
- 自主可控场景
- 英文代码项目
- 低延迟代码助手

不优先用于：

- 超复杂跨模块架构判断，除非使用旗舰模型并经过验证

路由建议：

```text
开源 / 本地 / 代码补全 / 代码生成 → Mistral / Codestral
```

### 3.12 Cohere / Command

适合岗位：

- Enterprise RAG Agent
- Knowledge Base Agent
- Search Agent
- Tool Use Agent
- Multilingual Enterprise Agent

适合任务：

- 企业知识库问答
- 检索增强生成
- 多语言文档处理
- 工具调用
- 内部流程自动化

不优先用于：

- 纯代码生成主力，除非项目强依赖企业知识库

路由建议：

```text
企业文档 / RAG / 知识检索 / 多语言工作流 → Cohere
```

### 3.13 本地模型 / Ollama / LM Studio / vLLM

适合岗位：

- Private Code Agent
- Offline Agent
- Low-cost Batch Agent
- Sensitive File Reviewer
- Local Completion Agent

适合任务：

- 涉及敏感代码的本地分析
- 离线开发
- 低成本批量任务
- 小范围代码补全
- 私有化企业场景

不优先用于：

- 高复杂度架构推理，除非本地模型足够强

路由建议：

```text
敏感数据 / 离线 / 私有化 / 低成本 → Local Models
```

## 4. Agent 岗位与模型匹配总表

| Agent 岗位 | 最适合模型类型 | 可选模型举例 |
|---|---|---|
| Orchestrator | Agentic、强推理、工具调用稳定 | Minimax M2.7、OpenAI reasoning、Claude、GLM |
| Product Manager | 中文强、产品表达强、推理稳定 | Minimax、Claude、Gemini、Qwen、DeepSeek Pro |
| Architect | 强推理、代码理解、长上下文 | Claude、OpenAI reasoning、DeepSeek Pro、GLM、Kimi |
| Frontend Engineer | 代码生成、审美、组件理解 | Claude、OpenAI、DeepSeek Pro、Qwen、Doubao |
| Backend Engineer | 逻辑推理、代码生成、Debug | DeepSeek Pro、OpenAI、Claude、Qwen、GLM |
| Fullstack Engineer | 综合编码能力 | Claude、OpenAI、Minimax、DeepSeek Pro、Qwen |
| Test Engineer | 低成本、批量生成、结构化 | DeepSeek Flash、Qwen、Mistral、OpenAI mini |
| Debug Engineer | 推理、错误日志理解 | DeepSeek Pro、OpenAI reasoning、Claude、GLM |
| Code Reviewer | 质量判断、安全意识 | Claude、OpenAI reasoning、DeepSeek Pro、GLM |
| Security Reviewer | 强推理、安全规则、严谨性 | OpenAI reasoning、Claude、GLM |
| Document Writer | 低成本、表达、中文强 | DeepSeek Flash、Minimax、Claude、Qwen |
| Long Context Reader | 长上下文 | Gemini、Kimi、Claude、OpenAI long-context |
| Multimodal Analyst | 多模态 | Gemini、GPT 多模态、Claude 多模态、Doubao / Seed |
| Research Agent | 实时信息、检索 | Grok、Gemini、OpenAI with search、Cohere RAG |
| Cost Controller | 快速、低成本、结构化 | DeepSeek Flash、OpenAI mini、Qwen、小模型 |

## 5. 当前只有 Minimax + DeepSeek 时的最佳分工

| 任务 | 推荐模型 | 理由 |
|---|---|---|
| 项目总控 | Minimax M2.7 | 更适合 Agent Team 和复杂工作流统筹 |
| PRD / 任务拆解 | Minimax M2.7 | 适合产品与多轮上下文组织 |
| 架构设计 | DeepSeek Pro | 强推理、适合技术拆解 |
| 后端代码 | DeepSeek Pro | 逻辑与代码实现较强 |
| 前端代码 | DeepSeek Pro | 复杂页面用 Pro，简单组件可用 Flash |
| 测试生成 | DeepSeek Flash | 低成本批量生成 |
| 文档生成 | DeepSeek Flash | 低成本输出 README / 注释 |
| 复杂 Bug | DeepSeek Pro | 推理与代码修复 |
| 合并协调 | Minimax M2.7 | 跨 Agent 结果统筹 |
| 最终交付报告 | Minimax M2.7 | 综合总结和文档交付 |

## 6. 路由策略示例

### 6.1 复杂 Bug

```text
先 DeepSeek Flash 尝试快速修复
→ 失败后 DeepSeek Pro 深度分析
→ 再失败后 Minimax M2.7 结合上下文重新拆解问题
→ 若接入 Claude / OpenAI，升级给 Code Reviewer / Reasoning 模型
```

### 6.2 大代码库理解

```text
如果接入 Gemini / Kimi / Claude 长上下文模型：优先使用长上下文模型
如果只有 Minimax / DeepSeek：先做文件索引和摘要，再分块读取
```

### 6.3 文档任务

```text
低成本文档：Flash / mini / Qwen
高质量交付文档：Claude / Minimax / OpenAI
长资料总结：Gemini / Kimi / Claude
```

### 6.4 安全审查

```text
低风险代码：DeepSeek Pro / Claude
高风险代码：OpenAI reasoning / Claude / GLM
需要本地保密：本地模型 + 静态扫描工具
```

## 7. 产品内置默认能力评分模板

初始评分仅用于推荐，用户可以手动修改。

| 模型家族 | reasoning | coding | longContext | speed | lowCost | toolUse | multimodal | localDeploy |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| OpenAI reasoning | 5 | 5 | 5 | 3 | 2 | 5 | 4 | 1 |
| Claude | 5 | 5 | 5 | 3 | 2 | 5 | 4 | 1 |
| Gemini | 5 | 4 | 5 | 3 | 3 | 4 | 5 | 1 |
| MiniMax M2.7 | 5 | 5 | 4 | 4 | 3 | 5 | 3 | 2 |
| DeepSeek Pro | 5 | 5 | 3 | 4 | 4 | 4 | 2 | 3 |
| DeepSeek Flash | 3 | 4 | 3 | 5 | 5 | 3 | 1 | 3 |
| Qwen | 4 | 4 | 4 | 4 | 4 | 4 | 3 | 4 |
| Kimi | 4 | 4 | 5 | 3 | 3 | 4 | 4 | 2 |
| Doubao / Seed | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 2 |
| GLM | 5 | 4 | 4 | 3 | 4 | 5 | 3 | 4 |
| Grok | 4 | 4 | 4 | 4 | 3 | 4 | 3 | 1 |
| Mistral / Codestral | 3 | 4 | 3 | 4 | 4 | 3 | 2 | 5 |
| Cohere Command | 4 | 3 | 4 | 4 | 3 | 5 | 2 | 3 |
| Local Models | 2-4 | 2-4 | 2-4 | 3-5 | 5 | 2-4 | 0-3 | 5 |

## 8. 实现要求

1. 能力评分不能写死在代码里，必须存数据库。
2. 系统提供默认模板，但用户可以修改。
3. 每次模型调用后，记录成功率、耗时、错误率，用于动态调整推荐。
4. Router 需要输出推荐理由。
5. 用户永远可以手动覆盖模型选择。

## 9. 参考资料

- OpenAI API Models: https://developers.openai.com/api/docs/models
- Anthropic Claude Computer Use: https://platform.claude.com/docs/en/agents-and-tools/tool-use/computer-use-tool
- Google Gemini Models: https://ai.google.dev/gemini-api/docs/models
- Google Gemini Long Context: https://ai.google.dev/gemini-api/docs/long-context
- MiniMax M2.7 for AI Coding Tools: https://platform.minimax.io/docs/guides/text-ai-coding-tools
- MiniMax Models: https://platform.minimax.io/docs/guides/models-intro
- DeepSeek API Docs: https://api-docs.deepseek.com/
- DeepSeek V3.2 Release: https://api-docs.deepseek.com/news/news251201
- Qwen Code: https://qwen.ai/qwencode
- Kimi API Docs: https://platform.kimi.ai/docs/overview
- ByteDance Seed 2.0: https://seed.bytedance.com/blog/seed-2-0-official-launch
- xAI Models: https://docs.x.ai/developers/models
- Mistral Models: https://docs.mistral.ai/models
- Cohere Models: https://docs.cohere.com/docs/models
