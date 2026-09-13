use ai_agent::{
    content::KIMI_K27_CODE_MODEL,
    tools::{self, calculator::definition::calculator_tool_definition, get_tools},
};
use tokio::task::JoinSet;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let tools = get_tools();
    let ret = chat_complete(
        KIMI_K27_CODE_MODEL,
        Some("你是一个全能助手"),
        "我要去美加墨世界杯，如何安排？",
        tools,
    )
    .await;
    tracing::info!("{:?}", ret);

    anyhow::Ok(())
}
