use std::sync::Arc;

use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionTools, CreateChatCompletionRequestArgs,
    FunctionCall,
};

use crate::{
    agent::{ContentItem, Event, ExecutionContext, ToolResultStatus},
    tools::ToolBox,
};

#[derive(Debug)]
pub struct AgentResult {
    pub output: String,
    pub context: ExecutionContext,
}

pub struct Agent {
    model: String,
    instructions: Option<String>,
    toolbox: Arc<ToolBox>,
    max_steps: u32,
}

impl Agent {
    pub fn new(
        model: impl Into<String>,
        instructions: Option<impl Into<String>>,
        toolbox: Arc<ToolBox>,
    ) -> Self {
        Self {
            model: model.into(),
            instructions: instructions.map(Into::into),
            toolbox,
            max_steps: 10,
        }
    }

    pub fn with_max_step(mut self, max_steps: u32) -> Self {
        self.max_steps = max_steps;
        self
    }

    pub async fn run(&self, user_input: &str) -> anyhow::Result<AgentResult> {
        let mut context = ExecutionContext::new();

        context.add_event(Event::new(
            context.execution_id.clone(),
            "user".to_string(),
            vec![ContentItem::Message {
                role: "user".to_string(),
                content: user_input.to_string(),
            }],
        ));

        let client = async_openai::Client::new();

        let tool_definition: Vec<ChatCompletionTools> = self
            .toolbox
            .values()
            .filter_map(|t| match t.definition() {
                Ok(def) => Some(def),
                Err(e) => {
                    tracing::error!("skip tool {} ,failed to get tool definition {e}", t.name());
                    None
                }
            })
            .collect();

        loop {
            if context.current_step >= self.max_steps {
                anyhow::bail!(
                    "Agent executeded the maximum of {} steps without a final answer",
                    self.max_steps
                );
            }

            let messages = self.build_message(&context)?;

            let request = CreateChatCompletionRequestArgs::default()
                .model(self.model.clone())
                .messages(messages)
                .tools(tool_definition.clone())
                .max_tokens(204890u32)
                .build()?;

            let resp = client.chat().create(request).await?;
            tracing::info!("llm response={:#?}", resp);

            let msg = resp
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("no message to resp"))?
                .message;

            if let Some(tool_calls) = msg.tool_calls {
                self.record_tool_calls(&mut context, &tool_calls);
                self.execute_tool_calls(&mut context, &tool_calls).await;
            } else {
                // 程序进入这个分支，收尾
                let content = msg
                    .content
                    .ok_or_else(|| anyhow::anyhow!("no content to resp"))?;
                context.add_event(Event::new(
                    context.execution_id.clone(),
                    "agent",
                    vec![ContentItem::Message {
                        role: "assiast".to_string(),
                        content: content.clone(),
                    }],
                ));
                context.final_result = Some(content.clone());

                return Ok(AgentResult {
                    output: content,
                    context,
                });
            }
            context.increment_step();
        }
    }

    /// 构建上下文中的request
    /// 第一次发起调用的时候，context.events.ContentItem 是没有工具类型的调用的
    fn build_message(
        &self,
        context: &ExecutionContext,
    ) -> anyhow::Result<Vec<ChatCompletionRequestMessage>> {
        let mut messages = Vec::new();
        // 系统提示词
        if let Some(system) = &self.instructions {
            messages.push(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system.as_str())
                    .build()?
                    .into(),
            );
        };
        // 提取Event中的
        for event in &context.events {
            for item in &event.content {
                match item {
                    ContentItem::Message { role, content } => {
                        let message: ChatCompletionRequestMessage = if role == "user" {
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(content.clone())
                                .build()?
                                .into()
                        } else {
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .content(content.clone())
                                .build()?
                                .into()
                        };
                        messages.push(message);
                    }
                    ContentItem::ToolCall {
                        tool_call_id,
                        name,
                        arguments,
                    } => {
                        let tool_call = ChatCompletionMessageToolCalls::Function(
                            ChatCompletionMessageToolCall {
                                id: tool_call_id.clone(),
                                function: FunctionCall {
                                    name: name.clone(),
                                    arguments: arguments.to_string(),
                                },
                            },
                        );
                        // 同一轮模型中，可能产生多次工具调用
                        // tool call要合并进一条assistant,
                        // 否则llm认为是不同的助理消息
                        if let Some(ChatCompletionRequestMessage::Assistant(last)) =
                            messages.last_mut()
                        {
                            last.tool_calls.get_or_insert_with(Vec::new).push(tool_call);
                        } else {
                            messages.push(
                                ChatCompletionRequestAssistantMessageArgs::default()
                                    .tool_calls(vec![tool_call])
                                    .build()?
                                    .into(),
                            );
                        }
                    }
                    ContentItem::ToolResult {
                        tool_call_id,
                        content,
                        ..
                    } => {
                        messages.push(
                            ChatCompletionRequestToolMessageArgs::default()
                                .tool_call_id(tool_call_id.clone())
                                .content(content.clone())
                                .build()?
                                .into(),
                        );
                    }
                }
            }
        }
        Ok(messages)
    }

    // 记录工具的调用
    fn record_tool_calls(
        &self,
        context: &mut ExecutionContext,
        tool_calls: &[ChatCompletionMessageToolCalls],
    ) {
        let mut call_items = Vec::new();
        for tool_call in tool_calls {
            if let ChatCompletionMessageToolCalls::Function(fun) = tool_call {
                let arguments: serde_json::Value = serde_json::from_str(&fun.function.arguments)
                    .unwrap_or(serde_json::Value::Null);
                call_items.push(ContentItem::ToolCall {
                    tool_call_id: fun.id.clone(),
                    name: fun.function.name.clone(),
                    arguments,
                });
            }
        }
        context.add_event(Event::new(
            context.execution_id.clone(),
            "agent",
            call_items,
        ));
    }

    // 执行工具调用
    async fn execute_tool_calls(
        &self,
        context: &mut ExecutionContext,
        tool_calls: &[ChatCompletionMessageToolCalls],
    ) {
        let mut result_items = Vec::new();
        for tool_call in tool_calls {
            // 是方法调用的执行相关逻辑
            if let ChatCompletionMessageToolCalls::Function(fun) = tool_call {
                let name = &fun.function.name;
                let arg = &fun.function.arguments;

                tracing::info!("tool call function_name={name},arg={arg}");

                let (status, content) = match self.toolbox.get(name) {
                    Some(tool) => match tool.execute(arg, context).await {
                        Ok(result) => {
                            tracing::info!("tool execute success and result={result}");
                            (ToolResultStatus::Success, result)
                        }
                        Err(err) => {
                            let err_msg = format!("tool execute failed and err_msg={err}");
                            tracing::info!(err_msg);
                            (ToolResultStatus::Error, err_msg)
                        }
                    },
                    None => {
                        tracing::info!("tool not found");
                        (ToolResultStatus::Error, "tool not found".to_string())
                    }
                };
                result_items.push(ContentItem::ToolResult {
                    tool_call_id: fun.id.clone(),
                    name: fun.function.name.clone(),
                    status,
                    content,
                });
            }
        }
        context.add_event(Event::new(
            context.execution_id.clone(),
            "tool",
            result_items,
        ));
    }
}
