use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use async_stream::stream;
use futures::{Stream, StreamExt};

// 流式返回
pub fn chat_stream(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> impl Stream<Item = anyhow::Result<String>> {
    stream! {
        let client = async_openai::Client::new();

        let mut messages = Vec::new();
        if let Some(message) = system {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(message)
                    .build()?
                    .into(),
            );
        }
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .max_tokens(204890u32)
            .build()?;

        let mut stream = client.chat().create_stream(request).await?;
        while let Some(response_ret) = stream.next().await {
            match response_ret {
                Ok(chunk) => if let Some(choice) = chunk.choices.first() {
                        if let Some(new_text) = &choice.delta.content {
                    yield Ok(new_text.clone());
                }
                },
                Err(err) => yield Err(err.into()),
            }
        }
    }
}
