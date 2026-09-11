use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, ResponseFormat, ResponseFormatJsonSchema,
};

use crate::models::action_plan::ActionPlan;

pub async fn chat_complete_struct(
    model: &str,
    system: Option<&str>,
    prompt: &str,
) -> anyhow::Result<ActionPlan> {
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

    // structured 变化的地方
    let schema = schemars::schema_for!(ActionPlan);
    let schemars_json = schema.as_value().clone();
    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: Some(
                "a step by step agent action plan with difficulty and time estimate".into(),
            ),
            name: "action_plan".into(),
            schema: schemars_json,
            strict: Some(true),
        },
    };

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .max_tokens(204890u32)
        .response_format(response_format)
        .build()?;

    let resp = client.chat().create(request).await?;
    let action_pan: ActionPlan = resp
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| anyhow::anyhow!("no content to resp"))
        .and_then(|s| serde_json::from_str(&s).map_err(Into::into))?;

    anyhow::Ok(action_pan)
}
