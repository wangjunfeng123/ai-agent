use std::sync::Arc;

use ai_agent::{content::KIMI_K27_CODE_MODEL, tools::build_toolbox};
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
    let instructions = Some(r#"我是一个可以计算数据的助手"#);
    let agent = ai_agent::agent::Agent::new(KIMI_K27_CODE_MODEL, instructions, Arc::new(tools))
        .with_max_step(10);
    let ret = agent
        .run("我叫李明，我是一名rust工程师", "turn_over1")
        .await?;
    tracing::info!("回答1111{:#?}", ret);

    let tools = build_toolbox().await?;
    let agent = ai_agent::agent::Agent::new(KIMI_K27_CODE_MODEL, instructions, Arc::new(tools))
        .with_max_step(10);
    let ret = agent
        .run("我叫的名字叫什么，我的职业是什么", "turn_over1")
        .await?;
    tracing::info!("回答2222{:#?}", ret);

    anyhow::Ok(())
}
