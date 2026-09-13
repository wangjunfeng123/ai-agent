use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs, ResponseFormat,
};

use crate::models::action_plan::ActionPlan;

pub async fn chat_complete_struct_ds(model: &str, prompt: &str) -> anyhow::Result<ActionPlan> {
    let client = async_openai::Client::new();

    let mut messages = Vec::new();
    messages.push(
        ChatCompletionRequestSystemMessageArgs::default()
            .content(build_system_prompt())
            .build()?
            .into(),
    );
    messages.push(
        ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into(),
    );

    // structured 变化的地方
    let resp_fmt = ResponseFormat::JsonObject;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .max_tokens(204890u32)
        .response_format(resp_fmt)
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

fn build_system_prompt() -> String {
    let schema = schemars::schema_for!(ActionPlan);
    let schema_str = serde_json::to_string(&schema).unwrap();

    format!(
        r#"You are a planning assistant. Analyze the user's request and respond with a JSON object.

    The output must be valid JSON that strictly conforms to this JSON Schema:

    {schema_str}

    Rules:
    - Output ONLY the raw JSON object, no markdown fences, no explanation
    - All required fields must be present
    - `difficulty` must be exactly one of: "Easy", "Medium", "Hard"
    - `steps` must be a non-empty array
    - Respond with JSON only"#
    )
}
