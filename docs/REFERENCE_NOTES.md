# 参考资料与设计依据

本项目的模型能力矩阵只作为默认推荐，不代表固定结论。模型变化很快，因此系统必须允许用户自行调整能力评分。

## 官方资料方向

- OpenAI API Models：复杂推理、Coding、工具调用、长上下文。
- Anthropic Claude：Coding、长上下文、Computer Use、工具调用。
- Google Gemini：多模态、长上下文、复杂资料理解。
- MiniMax M2.7：AI Coding、Agent Teams、复杂 Agent Harness。
- DeepSeek：OpenAI / Anthropic 兼容 API、Reasoning-first models for agents。
- Qwen Code：终端式 Coding Agent、代码库理解。
- Kimi：长上下文 Coding、Tool Calling。
- ByteDance Seed：Agent era、Seed Code、复杂任务执行。
- xAI Grok：实时搜索、工具调用、通用 Coding。
- Mistral / Codestral：代码生成、开源 / 开放权重场景。
- Cohere Command：企业 RAG、工具调用、多语言企业任务。

## 架构参考方向

- opencode：https://github.com/sst/opencode 和 https://opencode.ai/docs/providers/ 。采用 provider-agnostic 与 client/server 思路，客户端只是交互外壳。SuperCompany Coding 参考这一点，将 ProjectRun、TaskRunner、ToolExecutor、事件流放在 Local Core，而不是绑死在 React UI。
- Claude Code Router：https://musistudio.github.io/claude-code-router/ 。采用 Providers + Router + transformers 管理模型 endpoint、model、proxy、fallback。SuperCompany Coding 参考这一点，将 Minimax / DeepSeek 等优先建成 ProviderPreset + ProviderProfile，而不是在 Agent Runtime 里写厂商分支。
- CC Switch：https://ccswitch.ai/ 。将多个 AI Coding CLI 的 Provider、MCP、Prompt、Skills 配置集中管理。SuperCompany Coding 参考其“配置集中管理与一键切换”方向，但不直接复刻跨 CLI 管理能力。
- models.dev / provider registry 类思路：模型能力、价格、上下文和工具支持要按模型维护，不应只按 provider 维护。

## 产品化处理方式

不要在代码里假设某个模型永远最强。正确方式是：

1. 初始化默认能力模板。
2. 允许用户手动调整。
3. 记录历史任务表现。
4. Router 根据真实表现动态推荐。
5. 用户永远可以覆盖推荐。
