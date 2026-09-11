use ai_agent::{content::KIMI_K27_CODE, llm::complete::chat_complete};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ret = chat_complete(
        KIMI_K27_CODE,
        Some("你是一个全能助手"),
        "法国的首都是哪里？",
    )
    .await?;
    tracing::info!("{:?}", ret);
    anyhow::Ok(())
}
