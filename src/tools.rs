pub mod calculator;
pub mod file_delete;
pub mod file_list;
pub mod file_read;
pub mod file_unzip;
pub mod mcp;
pub mod tool;
pub mod web_search;

use std::{collections::HashMap, sync::Arc};

use anyhow::Ok;

use crate::tools::{
    calculator::r#impl::CalculatorTool,
    mcp::{client::McpClient, tool::McpTool},
    tool::Tool,
    web_search::r#impl::WebSearchTool,
};

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub async fn build_toolbox() -> anyhow::Result<ToolBox> {
    // 读取Function calling
    let mut tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool), Box::new(WebSearchTool)];

    // 加载MCP的tools
    let mcp_client = Arc::new(McpClient::connect().await?);

    for tool in mcp_client.list_tools().await? {
        tools.push(Box::new(McpTool::new(mcp_client.clone(), tool)));
    }
    Ok(tools
        .into_iter()
        .map(|t| (t.name().to_string(), t))
        .collect())
}
