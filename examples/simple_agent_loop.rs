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

    let tools = build_toolbox();
    let ret = chat_complete(
        KIMI_K27_CODE_MODEL,
        Some(
            r#"你是一个全能的助手。。
            当你需要当前信息时，请使用搜索工具，
            当你使用计算时，请使用计算工具，
            工具返回结果后，请直接回答这些结果，不要说我不知道。"#,
        ),
        "2026年女篮世界杯决赛比分？",
        &tools,
    )
    .await;
    tracing::info!("答案={:?}", ret);

    let ret = chat_complete(
        KIMI_K27_CODE_MODEL,
        Some(
            r#"你是一个全能的助手。。
            当你需要当前信息时，请使用搜索工具，
            当你使用计算时，请使用计算工具，
            工具返回结果后，请直接回答这些结果，不要说我不知道。"#,
        ),
        "123加上3乘以4等于多少？",
        &tools,
    )
    .await;
    tracing::info!("答案={:?}", ret);

    anyhow::Ok(())
}
