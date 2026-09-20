# ai-agent

一个基于 Rust 构建的 AI Agent 最小实现，覆盖了大模型对话、流式输出、结构化输出、工具调用（Function Calling）、**Agent 运行时**、**知识库向量检索** 与 **MCP（Model Context Protocol）** 集成能力。

底层通过 [`async-openai`](https://crates.io/crates/async-openai) 调用 OpenAI 兼容的 Chat Completions / Embeddings API，默认对接 **Moonshot / Kimi**；联网搜索使用 **Tavily**，MCP 基于 [`rmcp`](https://crates.io/crates/rmcp) 实现。

## 功能特性

- **基础对话与工具调用**：`src/llm/complete.rs` — 非流式对话 + Function Calling 循环。
- **流式输出**：`src/llm/stream.rs` — 逐 token 流式返回，内置指数退避重试（`backon`）。
- **结构化输出**：`src/llm/complete_struct.rs`、`src/llm/struct_ds.rs` — 通过 JSON-Schema / ResponseFormat 约束模型输出，直接反序列化为强类型结构体（如 `ActionPlan`）。
- **并发限流**：`src/llm/semaphore.rs` — 基于 `tokio::sync::Semaphore` 的全局信号量，控制并发请求数。
- **Agent 运行时**：`src/agent/` — 完整的 Agent loop，含执行上下文、事件流、token 用量统计、最大步数保护与内置重试。
  - `run()`：通用问答循环，自动调用工具直至给出最终答案。
  - `run_structured<T>()`:通过注入 `final_answer` 工具强制模型输出符合 JSON-Schema 的结构化结果。
- **知识库**：`src/knowledge_base/` — 文本分块、向量化与语义检索。
  - 固定长度分块（支持重叠区域）。
  - 调用 Embedding API（默认 `BAAI/bge-m3`）将文本转为向量。
  - 基于余弦相似度的 Top-K 向量检索（`BinaryHeap` 维护 top_k）。
- **工具系统**：`src/tools/` — 统一的 `Tool` trait 抽象与工具箱（`ToolBox`）。
  - `calculator`：基础四则运算。
  - `web_search`：调用 Tavily API 联网搜索实时信息。
  - `mcp`：将 MCP Server 暴露的工具封装为普通 `Tool`，无缝融入 Agent loop。

## 项目结构

```
src/
├── lib.rs                    # 库入口，导出 agent / content / knowledge_base / llm / models / tools
├── main.rs                   # 可执行入口（结构化输出演示）
├── content.rs                # 常量：模型名、Embedding 模型、final_answer 工具名
├── agent/
│   ├── runtime.rs            # Agent 运行时（run / run_structured、工具调用循环、重试）
│   ├── context.rs            # ExecutionContext（事件、步数、token 用量、临时状态）
│   └── event.rs              # Event / ContentItem / ToolResultStatus（消息、工具调用、工具结果）
├── knowledge_base/
│   ├── chunk.rs              # 固定长度文本分块（fixed_length_chunking）
│   ├── embed.rs              # 文本向量化（embed_text / embed_texts）
│   └── search.rs             # 余弦相似度 + Top-K 向量检索（vector_search）
├── llm/
│   ├── complete.rs           # 对话 + 工具调用
│   ├── complete_struct.rs    # 结构化输出（ResponseFormat::JsonSchema）
│   ├── stream.rs             # 流式输出 + 重试
│   ├── semaphore.rs          # 并发限流
│   └── struct_ds.rs          # 结构化输出（ResponseFormat::JsonObject + Prompt 注入 Schema）
├── models/
│   └── action_plan.rs        # ActionPlan / ActionStep / Difficulty 结构
├── tools/
│   ├── tool.rs               # Tool trait（统一抽象）
│   ├── mod.rs (tools.rs)     # build_toolbox() 组装工具（内置工具 + MCP 工具）
│   ├── calculator/           # 计算器工具（定义 / 执行 / 实现）
│   ├── web_search/           # 联网搜索工具（定义 / 执行 / 实现）
│   └── mcp/
│       ├── client.rs         # MCP Client（以子进程方式拉起 Server 并通信）
│       └── tool.rs           # McpTool（把 MCP 工具包装为 Tool trait）
└── bin/
    └── web_mcp_server.rs     # 示例 MCP Server（stdio 传输，转发至 expense-tracker-api）
```

## 环境配置

复制 `.env.example` 为 `.env`，填写以下环境变量：

```env
OPENAI_BASE_URL=https://api.moonshot.cn/v1
OPENAI_API_KEY=your_moonshot_api_key

# One secure API for real-time web access.
TAVILY_API_KEY=your_tavily_api_key

# web_mcp_server 转发目标（可选，默认 http://localhost:9999）
EXPENSE_API_URL=http://localhost:9999
```

## 运行示例

| 示例 | 说明 | 命令 |
| ---- | ---- | ---- |
| `stream_chat` | 多任务并发流式对话并限流 | `cargo run --example stream_chat` |
| `stream_structured` | 结构化输出（返回 `ActionPlan`） | `cargo run --example stream_structured` |
| `tool_call_complete` | 工具调用对话（联网搜索） | `cargo run --example tool_call_complete` |
| `simple_agent_loop` | 简单 Agent 循环（搜索 + 计算） | `cargo run --example simple_agent_loop` |
| `agent_run` | 完整 Agent 运行时（含 token 用量统计） | `cargo run --example agent_run` |
| `web_search` | 直接调用 Tavily 搜索，再基于 MCP 查询服务 | `cargo run --example web_search` |
| `web_mcp_server` | 启动 MCP Server（stdio，供 Client 子进程拉起） | `cargo run --bin web_mcp_server` |

> 注意：Moonshot 账号对请求频率有限制（默认约 3 RPM），命中原生限流时会返回 `429 Too Many Requests`，稍后重试即可。

## 技术要点

- 基于 **2024 edition** 的 Rust 工程，异步运行时使用 `tokio`。
- `Tool` trait 通过 `definition()` 自动生成 Function Calling 的 JSON Schema，支持扩展现有工具集。
- Agent 运行时将「消息、工具调用、工具结果」统一建模为 `Event` / `ContentItem`，可序列化、可追溯。
- 结构化输出严格使用 JSON-Schema 约束，保证可反序列化为强类型结构体（`ActionPlan`）。
- 知识库检索链路：`分块（chunk）→ 向量化（embed）→ Top-K 检索（search）`。
- MCP 集成：Agent 作为 MCP Client 以子进程拉起 Server，通过 stdio 通信，工具可动态发现并自动注入工具箱。
