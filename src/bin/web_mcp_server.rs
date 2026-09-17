use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, ServiceExt, schemars, tool, tool_router, transport::stdio};
use schemars::JsonSchema;
use serde::Deserialize;

// ---------- 参数 / 请求体类型 ----------
// 说明：这是MCP请求的参数，
// 当我们请求web-server的时候，name会拼接到URL路径中；
#[derive(Debug, Deserialize, JsonSchema)]
struct HelloName {
    name: String,
}

// ---------- MCP Server 本体 ----------
// 这个 Server 不连数据库，它只是 expense-tracker-api 这个
// axum web 服务的一个"翻译层"：每个 MCP 工具方法内部
// 其实就是发一次 HTTP 请求过去，把结果包装成 MCP 要求的格式返回
#[derive(Clone)]
struct ExpenseServer {
    http: reqwest::Client,
    base_url: String,
}

impl ExpenseServer {
    // 优先读环境变量，读不到就用本地默认值，
    // 这样以后部署到别的地方也不用改代码
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: std::env::var("EXPENSE_API_URL")
                .unwrap_or_else(|_| "http://localhost:9999".to_string()),
        }
    }

    /// 公用的相应逻辑，所有的调用都调用它
    /// 同样的判断。核心思路：
    /// - 请求本身失败（网络错误等）→ 包装成 McpError
    /// - 请求发出去了，但 API 返回非 2xx（比如 404 / 401 / 400）→ 也算 McpError
    /// - 只有 2xx 才算成功，把响应体原样包成 CallToolResult 返回
    async fn respond(
        result: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<CallToolResult, McpError> {
        match result {
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .await
                    .unwrap_or_else(|e| format!("读取响应内容失败: {e}"));

                if status.is_success() {
                    Ok(CallToolResult::success(vec![ContentBlock::text(body)]))
                } else {
                    Err(McpError::internal_error(
                        format!("expense-tracker-api 返回了 {status}: {body}"),
                        None,
                    ))
                }
            }
            Err(e) => Err(McpError::internal_error(
                format!("请求 expense-tracker-api 失败: {e}"),
                None,
            )),
        }
    }
}

// #[tool_router(server_handler)] 是"单 impl 块"写法：
// 不需要额外再写一个 impl ServerHandler for ExpenseServer，
// 宏会把这里标了 #[tool] 的方法自动收集成一份工具清单
#[tool_router(server_handler)]
impl ExpenseServer {
    #[tool(description = "pool2moon get result by name")]
    async fn hello_name(
        &self,
        Parameters(p): Parameters<HelloName>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .get(format!("{}/expenses/{}", self.base_url, p.name))
            .send()
            .await;
        Self::respond(result).await
    }

    #[tool(description = "pool2moon health check")]
    async fn health_check(&self) -> Result<CallToolResult, McpError> {
        let result = self
            .http
            .get(format!("{}/health_check", self.base_url))
            .send()
            .await;

        Self::respond(result).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 日志要写到 stderr，不能写到 stdout —— 因为 stdio 传输方式下，
    // stdout 是留给 MCP 协议本身通信用的，混进普通日志会把协议搞坏
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let server = ExpenseServer::new();
    // 用 stdio 传输方式启动：这个进程会被 Client 当作子进程拉起，
    // 通过标准输入输出跟 Client 交换消息，跟我们幻灯片里讲的一致
    let service = server.serve(stdio()).await?;
    service.waiting().await?;

    Ok(())
}
