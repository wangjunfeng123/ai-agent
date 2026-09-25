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
    let ret = agent.run("1加上3乘以5等于多少").await?;
    tracing::info!("{:#?}", ret);

    anyhow::Ok(())
}
