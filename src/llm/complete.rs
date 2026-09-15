use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionTools, CreateChatCompletionRequestArgs,
};

use crate::tools::ToolBox;

pub async fn chat_complete(
    model: &str,
    system: Option<&str>,
    prompt: &str,
    tools_box: &ToolBox,
) -> anyhow::Result<String> {
    let client = async_openai::Client::new();

    // 构建请求参数
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

    let tool_definitions: Vec<ChatCompletionTools> = tools_box
        .values()
        .filter_map(|p| match p.definition() {
            Ok(def) => Some(def),
            Err(e) => {
                tracing::error!("skip tool {} ,failed to get tool definition {e}", p.name());
                None
            }
        })
        .collect();

    loop {
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages.clone())
            .tools(tool_definitions.clone())
            .max_tokens(204890u32)
            .build()?;

        let resp = client.chat().create(request).await?;
        tracing::info!("llm response={:#?}", resp);
        let msg = resp
            .clone()
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("no message to resp"))?
            .message;

        if let Some(tool_calls) = msg.tool_calls {
            messages.push(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .tool_calls(tool_calls.clone())
                    .build()?
                    .into(),
            );
            for tool_call in tool_calls {
                match tool_call {
                    async_openai::types::chat::ChatCompletionMessageToolCalls::Function(
                        function_call,
                    ) => {
                        let function_name = function_call.function.name;
                        let args = function_call.function.arguments;
                        tracing::info!("tool call function_name={function_name},args={args}");

                        let tool_result = match tools_box.get(&function_name) {
                            Some(tool) => match tool.execute(&args).await {
                                Ok(result) => {
                                    tracing::info!("tool result = {result}");
                                    result
                                }
                                Err(error) => {
                                    let err_msg = format!("tool call error={error}");
                                    tracing::error!(err_msg);
                                    err_msg
                                }
                            },
                            None => {
                                let err_msg = format!("not support calculation");
                                tracing::error!(err_msg);
                                err_msg
                            }
                        };
                        messages.push(
                            ChatCompletionRequestToolMessageArgs::default()
                                .tool_call_id(function_call.id)
                                .content(tool_result)
                                .build()?
                                .into(),
                        );
                    }
                    _ => tracing::error!("not support calculation"),
                }
            }
        } else {
            let content = msg
                .content
                .ok_or_else(|| anyhow::anyhow!("no content to resp"))?;
            return anyhow::Ok(content);
        }
    }
}
