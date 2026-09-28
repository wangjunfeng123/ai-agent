use std::sync::Arc;

use ai_agent::agent::Agent;
use ai_agent::content::KIMI_K27_CODE_MODEL;
use ai_agent::tools::build_toolbox;
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

    let tools = build_toolbox().await?;
    let instructions = r#"你是一个全能的助手。。
        当你需要当前信息时，请使用搜索工具，
        当你使用计算时，请使用计算工具，
        你可以查看pool2moon web服务的一些信息。
        工具返回结果后，请直接回答这些结果，不要说我不知道。"#;
    let agent =
        Agent::new(KIMI_K27_CODE_MODEL, Some(instructions), Arc::new(tools)).with_max_step(15);

    let ret = agent
        .run(
            "我要去美加墨世界杯，如何安排？",
            &Uuid::new_v4().to_string(),
        )
        .await?;
    tracing::info!("{:?}", ret);

    anyhow::Ok(())
}
