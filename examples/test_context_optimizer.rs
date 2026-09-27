use tracing::Level;
use tracing_subscriber::FmtSubscriber;

// 这个测试用例可以用于测试context_optimizer这个中的2个上下文压缩方式
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
