use ai_agent::{content::KIMI_K27_CODE_MODEL, llm::complete_struct::chat_complete_struct};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ret = chat_complete_struct(
        KIMI_K27_CODE_MODEL,
        Some("你是一个全能助手"),
        "我要去美加墨世界杯，如何安排？",
    )
    .await;
    tracing::info!("{:?}", ret);
    anyhow::Ok(())
}
