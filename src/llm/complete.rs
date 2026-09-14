use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionTools, CreateChatCompletionRequestArgs,
};

use crate::tools::{
    calculator::execute::{CalculatorArgs, calculator},
    web_search::execute::{WebSearchArgs, web_search},
};

pub async fn chat_complete(
    model: &str,
    system: Option<&str>,
    prompt: &str,
    tools: Vec<ChatCompletionTools>,
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

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages.clone())
        .tools(tools.clone())
        .max_tokens(204890u32)
        .build()?;

    let resp = client.chat().create(request).await?;

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
                    let function_args = function_call.function.arguments;

                    if function_name == "calculator" {
                        let args: CalculatorArgs = serde_json::from_str(&function_args)?;
                        let result =
                            calculator(&args.operator, args.first_number, args.second_number);
                        let tool_result = match result {
                            Ok(calc_result) => calc_result.to_string(),
                            Err(err) => err,
                        };
                        tracing::info!("tool call result={tool_result}");
                        messages.push(
                            ChatCompletionRequestToolMessageArgs::default()
                                .tool_call_id(function_call.id)
                                .content(tool_result)
                                .build()?
                                .into(),
                        );
                    } else if function_name == "web_search" {
                        let args: WebSearchArgs = serde_json::from_str(&function_args)?;
                        let result = web_search(args).await;
                        let tool_result = match result {
                            Ok(search_ret) => serde_json::to_string(&search_ret)?,
                            Err(error) => error.to_string(),
                        };

                        tracing::info!("web search result = {tool_result}");

                        messages.push(
                            ChatCompletionRequestToolMessageArgs::default()
                                .tool_call_id(function_call.id)
                                .content(tool_result)
                                .build()?
                                .into(),
                        );
                    }
                }
                _ => tracing::error!("not support calculation"),
            }
        }

        // 再次发送请求给llm
        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages.clone())
            .tools(tools.clone())
            .max_tokens(204890u32)
            .build()?;
        let resp = client.chat().create(request).await?;

        let content = resp
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("no content to resp"))?;
        return anyhow::Ok(content);
    }

    let content = resp
        .clone()
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .ok_or_else(|| anyhow::anyhow!("no content to resp"))?;

    anyhow::Ok(content)
}
