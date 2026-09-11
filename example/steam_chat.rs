use std::result;

use ai_agent::{
    content::KIMI_K27_CODE,
    llm::{complete_struct::chat_complete_struct, steam::chat_stream},
};
use futures::StreamExt;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let ret = chat_stream(
        KIMI_K27_CODE,
        Some("你是一个全能助手"),
        "道德经第四章什么内容？",
    );
    futures::pin_mut!(ret);
    let mut output = String::new();

    while let Some(result) = ret.next().await {
        match result {
            Ok(txt) => {
                output.push_str(&txt);
                tracing::info!("{txt}")
            }
            Err(err) => {
                tracing::error!("error while stream :{}", err);
                return Err(err);
            }
        }
    }
    tracing::info!("{:?}", ret);
    anyhow::Ok(())
}
