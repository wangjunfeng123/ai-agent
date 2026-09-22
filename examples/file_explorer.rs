use std::sync::Arc;

use ai_agent::{
    agent::Agent,
    content::{KIMI_K27_CODE_MODEL, VISION_MODEL},
    tools::build_file_explorer_toolbox,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let tools = Arc::new(build_file_explorer_toolbox(VISION_MODEL));
    let instructions = r#"
        你是一个一个善于探索的文件助手
        拿到一个压缩包先解压unzip_file，然后利用list_files看目录结构，
        靠文件名判断文件是否相关，用read_file和read_images逐一确认，
        最后再给出结论，不要跳过步骤直接给出答案。
        "#;

    let agent = Agent::new(KIMI_K27_CODE_MODEL, Some(instructions), tools).with_max_step(20);
    let result = agent.run("分析这个压缩包/Users/wangjunfeng/workspace/github/ai-agent/resources/payment.zip，看看小精灵支付平台支持支付宝吗").await?;

    tracing::info!("答案={:?}", result.output);

    tracing::info!(
        "\n本次执行了{}步骤，记录了{}个event",
        result.context.current_step,
        result.context.events.len()
    );
    anyhow::Ok(())
}
