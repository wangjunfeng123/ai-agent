use anyhow::Result;
use rmcp::model::{CallToolRequestParams, Tool};
use rmcp::service::{RoleClient, RunningService};
use rmcp::{
    ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

// mcp client 负责启动mcp server
impl McpClient {
    /// 把 web_mcp_server 当子进程拉起来，并建立连接
    pub async fn connect() -> Result<Self> {
        let service = ()
            .serve(TokioChildProcess::new(Command::new("cargo").configure(
                |cmd| {
                    cmd.args(["run", "--quiet", "--bin", "web_mcp_server"]);
                },
            ))?)
            .await?;

        Ok(Self { service })
    }

    /// 拿到 server 暴露的所有工具
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let result = self.service.list_tools(Default::default()).await?;
        Ok(result.tools)
    }

    /// 按名字调用某个工具，arguments 是一个 JSON 对象
    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<String> {
        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(arguments.as_object().cloned().unwrap_or_default());

        let result = self.service.call_tool(params).await?;

        let text = result
            .content
            .iter()
            .filter_map(|block| block.as_text().map(|t| t.text.clone()))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }
}
