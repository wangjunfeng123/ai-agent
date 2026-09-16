use ai_agent::{content::KIMI_K27_CODE_MODEL, llm::complete::chat_complete, tools::build_toolbox};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let tools = build_toolbox().await?;
    let ret = chat_complete(
        KIMI_K27_CODE_MODEL,
        Some(
            r#"你是一个全能的助手。。
            当你需要当前信息时，请使用搜索工具，
            当你使用计算时，请使用计算工具，
            你可以查看pool2moon web服务的一些信息。
            工具返回结果后，请直接回答这些结果，不要说我不知道。"#,
        ),
        "帮我看下pool2moon 服务的健康状况？",
        &tools,
    )
    .await;
    tracing::info!("答案={:?}", ret);

    anyhow::Ok(())
}
