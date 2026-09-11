use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};

pub async fn chat_complete(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<String> {
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
    let resp = client.chat().create(request).await?;
    tracing::info!("Resp={:#?}", resp);

    anyhow::Ok(String::new())
}
