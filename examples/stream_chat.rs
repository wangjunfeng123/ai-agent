use ai_agent::{
    content::KIMI_K27_CODE_MODEL,
    llm::{semaphore::get_semaphore, stream::chat_stream_with_retry},
};
use tokio::task::JoinSet;
use tracing::{Instrument, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let prompts = vec![
        "http和https的区别",
        "rust 中Arc和rc的区别",
        "什么是异步编程，和多线程有什么区别",
        "解释下TCP的三次握手",
        "什么是ai agent,做下大概的描述",
        "讲讲rust 包tracing怎么使用的",
    ];

    let mut set = JoinSet::new();
    for prompt in prompts {
        let span = tracing::info_span!("Chat", prompt = prompt);
        set.spawn(
            async move {
                tracing::info!("{prompt}");
                let permit = get_semaphore().acquire().await?;
                let output =
                    chat_stream_with_retry(KIMI_K27_CODE_MODEL, Some("你是一个全能助手"), prompt)
                        .await?;
                drop(permit);
                Ok::<_, anyhow::Error>((prompt, output))
            }
            .instrument(span),
        );
    }

    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((prompt, result))) => tracing::info!("{prompt}--{result}"),
            Ok(Err(err)) => tracing::error!("task panic :{err}"),
            Err(err) => tracing::error!("task panic ::{err}"),
        }
    }
    anyhow::Ok(())
}
