# ai-agent

一个基于 Rust 构建的开源 AI Agent 框架，覆盖 **LLM 对话、流式输出、结构化输出、工具调用（Function Calling）、可扩展回调机制、知识库向量检索** 与 **MCP（Model Context Protocol）** 集成。

底层通过 [`async-openai`](https://crates.io/crates/async-openai) 调用 OpenAI 兼容的 Chat Completions / Embeddings API，支持切换任意 OpenAI 兼容的模型服务商（默认对接 **硅基流动 SiliconFlow**）；联网搜索使用 **Tavily**，MCP 基于 [`rmcp`](https://crates.io/crates/rmcp) 实现。

## 核心特性

- **Agent 运行时**（`src/agent/`）：完整的 Agent loop，包含执行上下文、Event 事件流、token 用量统计、最大步数保护与内置指数退避重试。
  - `run()`：通用问答循环，自动编排工具直至给出最终答案。
  - `run_structured<T>()`：通过注入 `final_answer` 工具，强制模型输出符合 JSON-Schema 的强类型结构化结果。
- **回调机制**（`src/callback/`）：通过 `BeforeToolCallBack` / `AfterToolCallBack` 可在工具执行前后拦截、放行或改写结果。
  - `ApprovalCallback`：对高危险工具实现**人工审批**（确认后执行 / 拒绝并返回提示）。
  - `SearchCompressorCallback`：对超长的 `web_search` 结果进行**知识库向量压缩**，切块→向量化→语义 Top-K 抽取，显著节省 token。
- **LLM 客户端**（`src/llm/`）：非流式对话 + 工具调用、JSON-Schema 结构化输出、逐 token 流式输出（含重试）、基于信号量的并发限流。
- **知识库**（`src/knowledge_base/`）：固定长度文本分块（支持重叠窗口）、Embedding 向量化、基于余弦相似度的向量检索（`BinaryHeap` 维护 top_k）。
- **工具系统**（`src/tools/`）：统一 `Tool` trait 抽象 + 内置工具箱 `ToolBox`。
  - 计算器、联网搜索（Tavily）、MCP 动态发现工具。
  - **文件探索套件**：`file_list` / `file_read`（支持行号、区间、CSV）/ `file_delete` / `file_unzip` / `read_images`（视觉模型理解图片）。

## 项目结构

```
src/
├── lib.rs                 # 库入口：agent / callback / content / knowledge_base / llm / models / tools
├── main.rs                # 可执行入口（结构化输出演示）
├── content.rs             # 常量：模型名、Embedding 模型、final_answer 工具名
├── agent/
│   ├── runtime.rs         # Agent 运行时（run / run_structured、工具循环、重试）
│   ├── context.rs         # ExecutionContext（事件、步数、token 用量、临时状态）
│   ├── event.rs           # Event / ContentItem / ToolResultStatus（消息、工具调用、工具结果）
│   ├── callback.rs        # BeforeToolCallBack / AfterToolCallBack / ToolCallView trait
│   └── llm_request.rs     # （预留）
├── callback/
│   ├── approval.rs        # 高风险工具的人工审批回调
│   └── search_compressor.rs # 基于向量检索的搜索压缩回调
├── knowledge_base/
│   ├── chunk.rs           # 固定长度文本分块（fixed_length_chunking，支持重叠）
│   ├── embed.rs           # 文本向量化（embed_text / embed_texts）
│   └── search.rs          # 余弦相似度 + Top-K 向量检索（vector_search）
├── llm/
│   ├── complete.rs        # 对话 + 工具调用
│   ├── complete_struct.rs # 结构化输出（ResponseFormat::JsonSchema）
│   ├── stream.rs          # 流式输出 + 重试
│   ├── semaphore.rs       # 并发限流
│   └── struct_ds.rs       # 结构化输出（JsonObject + Prompt 注入 Schema）
├── models/
│   └── action_plan.rs     # ActionPlan / ActionStep / Difficulty 结构
├── tools/
│   ├── tool.rs            # Tool trait（name / desc / parameters / execute / definition）
│   ├── tools.rs           # ToolBox、build_toolbox()、build_file_explorer_toolbox()
│   ├── calculator/        # 计算器
│   ├── web_search/        # Tavily 联网搜索
│   ├── mcp/               # MCP 客户端与工具包装
│   ├── file_list/         # 文件列表
│   ├── file_read/         # 读取文件（行号 / 区间 / CSV）
│   ├── file_delete/       # 删除文件
│   ├── file_unzip/        # 解压文件
│   └── read_images/       # 视觉模型理解图片
└── bin/
    └── web_mcp_server.rs  # 示例 MCP Server（stdio 传输，转发至 expense-tracker-api）
```

## 环境配置

复制 `.env.example` 为 `.env`，填写以下环境变量：

```env
# 硅基流动 SiliconFlow（默认）
OPENAI_BASE_URL=https://api.siliconflow.cn/v1
OPENAI_API_KEY=your_siliconflow_api_key

# One secure API for real-time web access.
TAVILY_API_KEY=your_tavily_api_key

# web_mcp_server 转发目标（可选，默认 http://localhost:9999）
EXPENSE_API_URL=http://localhost:9999
```

## 使用示例

```rust
let tools = Arc::new(build_toolbox().await?);
let agent = Agent::new(model, Some(instructions), tools)
    .with_max_step(8)
    .with_before_tool_callback(Arc::new(ApprovalCallback::new(["file_delete"])))
    .with_after_tool_callback(Arc::new(SearchCompressorCallback));

let result = agent.run("2026年女篮世界杯决赛比分").await?;
tracing::info!("答案={:#?}", result.output);
tracing::info!("token 用量：{}", result.context.usage.total_tokens);
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
| `file_explorer` | 文件探索（列表 / 读取 / 图片理解 / 解压 / 删除） | `cargo run --example file_explorer` |
| `web_mcp_server` | 启动 MCP Server（stdio，供 Client 子进程拉起） | `cargo run --bin web_mcp_server` |

> 注意：部分模型服务商（如 Moonshot）对请求频率有限制，命中原生限流时会返回 `429 Too Many Requests`，稍后重试即可。

## 技术要点

- 基于 **2024 edition** 的 Rust 工程，异步运行时使用 `tokio`，日志使用 `tracing`。
- `Tool` trait 通过 `parameters()` / `definition()` 自动生成 Function Calling 的 JSON-Schema，支持无缝扩展现有工具集。
- Agent 运行时将「消息、工具调用、工具结果」统一建模为可序列化的 `Event` / `ContentItem`，可追溯、可审计。
- `run_structured<T>` 把结构化输出参数包装成 `final_answer` 工具调用，严格按 JSON-Schema 反序列化为强类型结构体。
- 回调机制支撑**人工审批**与**结果改写**，让敏感操作可控、长文本上下文成本可控。
- 知识库检索链路：`分块（chunk）→ 向量化（embed）→ Top-K 检索（search）`，默认使用 `BAAI/bge-m3`。
- MCP 集成：Agent 作为 MCP Client 以子进程拉起 Server，通过 stdio 通信，工具可动态发现并自动注入工具箱。

## License

MIT
