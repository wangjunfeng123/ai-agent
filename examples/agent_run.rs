use ai_agent::{agent::Agent, content::KIMI_K27_CODE_MODEL, tools::build_toolbox};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let now = chrono::Utc::now().format("yyyy-mm-dd HH:MM:ss").to_string();

    let instructions = format!(
        r#"你是一个全能的助手。

        当前本地时间 {}

    当你需要当前信息时，请使用搜索工具，
    当你使用计算时，请使用计算工具，
    你可以查看pool2moon web服务的一些信息。
    工具返回结果后，请直接回答这些结果，不要说我不知道。"#,
        now
    );

    let tools = build_toolbox().await?;
    let agent = Agent::new(KIMI_K27_CODE_MODEL, Some(&instructions), &tools).with_max_step(8);
    let ret = agent.run("2026年女篮世界杯决赛比分").await?;
    tracing::info!("答案={:#?}", ret);

    anyhow::Ok(())
}
