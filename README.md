# ai-agent

一个基于 Rust 构建的 AI Agent 最小实现，封装了大模型对话、流式输出、结构化输出与工具调用（Function Calling）能力，并内置了 **计算器** 与 **联网搜索** 两个工具。

底层通过 [`async-openai`](https://crates.io/crates/async-openai) 调用 OpenAI 兼容的 Chat Completions API，默认对接 **Moonshot / Kimi**，联网搜索使用 **Tavily**。

## 功能特性

- **基础对话**：`src/llm/complete.rs` — 非流式对话与工具调用（Function Calling）循环。
- **流式输出**：`src/llm/stream.rs` — 逐 token 流式返回，并内置指数退避重试（`backon`）。
- **结构化输出**：`src/llm/complete_struct.rs`、`src/llm/struct_ds.rs` — 通过 JSON-Schema 约束模型输出，直接反序列化为强类型结构体。
- **并发限流**：`src/llm/semaphore.rs` — 基于 `tokio::sync::Semaphore` 的全局信号量，控制并发请求数。
- **工具系统**：`src/tools/` — 提供统一的 `Tool` trait 抽象与工具箱（`ToolBox`）。
  - `calculator`：基础四则运算。
  - `web_search`：调用 Tavily API 联网搜索实时信息。

## 项目结构

```
src/
├── lib.rs                    # 库入口，导出 content / llm / models / tools
├── main.rs                   # 可执行入口（结构化输出演示）
├── content.rs                # 模型常量（KIMI_K27_CODE_MODEL）
├── models/
│   └── action_plan.rs        # ActionPlan / ActionStep / Difficulty 结构
├── llm/
│   ├── complete.rs           # 对话 + 工具调用
│   ├── complete_struct.rs    # 结构化输出（ResponseFormat::JsonSchema）
│   ├── stream.rs             # 流式输出 + 重试
│   ├── semaphore.rs          # 并发限流
│   └── struct_ds.rs          # 结构化输出（ResponseFormat::JsonObject + Prompt 注入 Schema）
└── tools/
    ├── tool.rs               # Tool trait（统一抽象）
    ├── mod.rs                # build_toolbox() 组装工具
    ├── calculator/           # 计算器工具（定义 / 执行 / 实现）
    └── web_search/           # 联网搜索工具（定义 / 执行 / 实现）
```

## 环境配置

复制 `Cargo.toml` 同级的 `.env` 示例，填写以下环境变量：

```env
OPENAI_BASE_URL=https://api.moonshot.cn/v1
OPENAI_API_KEY=your_moonshot_api_key

# One secure API for real-time web access.
TAVILY_API_KEY=your_tavily_api_key
```

## 运行示例

| 示例 | 说明 | 命令 |
| ---- | ---- | ---- |
| `stream_chat` | 多任务并发流式对话并限流 | `cargo run --example stream_chat` |
| `stream_structured` | 结构化输出（返回 `ActionPlan`） | `cargo run --example stream_structured` |
| `tool_call_complete` | 工具调用对话（联网搜索） | `cargo run --example tool_call_complete` |
| `simple_agent_loop` | 简单 Agent 循环（搜索 + 计算） | `cargo run --example simple_agent_loop` |

> 注意：Moonshot 账号对请求频率有限制（默认约 3 RPM），命中原生限流时会返回 `429 Too Many Requests`，稍后重试即可。

## 技术要点

- 基于 **2024 edition** 的 Rust 工程，异步运行时使用 `tokio`。
- `Tool` trait 通过 `definition()` 自动生成 Function Calling 的 JSON Schema，支持扩展现有工具集。
- 结构化输出严格使用 JSON-Schema 约束，保证可反序列化为强类型结构体（`ActionPlan`）。
