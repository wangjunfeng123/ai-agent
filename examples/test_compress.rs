use std::sync::Arc;

use ai_agent::{
    agent::Agent, callback::search_compressor::SearchCompressorCallback,
    content::DEEPSEEK_V4_FLASH, tools::build_toolbox,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let toolbox = Arc::new(build_toolbox().await?);
    let instructions = r#"
        你是一个网页搜索工具
        "#;
    let agent = Agent::new(DEEPSEEK_V4_FLASH, Some(instructions), toolbox)
        .with_max_step(5)
        .with_after_tool_callback(Arc::new(SearchCompressorCallback));
    let result = agent.run("2026年女篮世界杯决赛比分是？").await?;
    tracing::info!("result = {}", result.output);

    anyhow::Ok(())
}
