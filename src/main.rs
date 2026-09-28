use std::sync::Arc;

use ai_agent::{agent::Agent, content::KIMI_K27_CODE_MODEL, tools::build_toolbox};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let toolbox = build_toolbox().await?;
    let instructions = "你是一个全能助手";
    let agent =
        Agent::new(KIMI_K27_CODE_MODEL, Some(instructions), Arc::new(toolbox)).with_max_step(20);

    let ret = agent
        .run(
            "我要去美加墨世界杯，如何安排？",
            &Uuid::new_v4 ().to_string(),
        )
        .await?;
    tracing::info!("{:?}", ret);
    anyhow::Ok(())
}
