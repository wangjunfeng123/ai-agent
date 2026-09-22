use std::{sync::Arc, time::Duration};

use async_openai::types::chat::{
    ChatCompletionMessageToolCall, ChatCompletionMessageToolCalls,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
    ChatCompletionRequestUserMessageArgs, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ChatCompletionTools, CreateChatCompletionRequestArgs, FunctionCall, FunctionObjectArgs,
    ToolChoiceOptions,
};
use backon::{ExponentialBuilder, Retryable};
use schemars::JsonSchema;

use crate::{
    agent::{ContentItem, Event, ExecutionContext, ToolResultStatus},
    content::FINAL_ANSWER,
    tools::ToolBox,
};

#[derive(Debug)]
pub struct AgentResult {
    pub output: String,
    pub context: ExecutionContext,
}

// 结构化输出，方便让程序能解析
// 把结构化输出参数，包装成一次工具调用；当llm再次工具调用的时候，我们就知道是
// 泛型T
//  不同的任务，接受数据的格式是不同的；所以使用泛型
#[derive(Debug)]
pub struct StructuredAgentResult<T> {
    pub output: T,
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

            // 增加重试机制，延迟50ms在执行
            let resp = (|| async { client.chat().create(request.clone()).await })
                .retry(
                    ExponentialBuilder::default()
                        .with_max_times(3)
                        .with_min_delay(Duration::from_millis(5050)),
                )
                .await?;
            tracing::info!("llm response={:#?}", resp);

            // token 用量统计
            if let Some(usage) = &resp.usage {
                context.usage.add(
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens,
                );
            }

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

    // run_structured数据结构化结果
    // 这个方法必须调用工具
    pub async fn run_structured<T>(
        &self,
        user_input: &str,
    ) -> anyhow::Result<StructuredAgentResult<T>>
    where
        T: schemars::JsonSchema + serde::de::DeserializeOwned,
    {
        let mut context = ExecutionContext::new();

        // 保存用户输入到Event
        context.add_event(Event::new(
            context.execution_id.clone(),
            "user".to_string(),
            vec![ContentItem::Message {
                role: "user".to_string(),
                content: user_input.to_string(),
            }],
        ));

        let client = async_openai::Client::new();

        let mut tool_definition: Vec<ChatCompletionTools> = self
            .toolbox
            .values()
            .filter_map(|t| match t.definition() {
                Ok(def) => Some(def),
                Err(e) => {
                    tracing::error!("skip tool {}, failed to get tool definition {e}", t.name());
                    None
                }
            })
            .collect();
        // 加入final_answer工具到工具列表
        tool_definition.push(final_answer_too_definition::<T>()?);

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
                // 工具调用真的能让大模型的调用变得更好吗
                .tool_choice(ChatCompletionToolChoiceOption::Mode(
                    ToolChoiceOptions::Required,
                ))
                .max_tokens(204890u32)
                .build()?;

            let resp = (|| async { client.chat().create(request.clone()).await })
                .retry(
                    ExponentialBuilder::default()
                        .with_max_times(3)
                        .with_min_delay(Duration::from_millis(50)),
                )
                .await?;
            tracing::info!("llm response={:#?}", resp);

            // token 用量统计
            if let Some(usage) = &resp.usage {
                context.usage.add(
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens,
                );
            }

            let msg = resp
                .choices
                .into_iter()
                .next()
                .ok_or_else(|| anyhow::anyhow!("no message to resp"))?
                .message;

            let tool_calls = msg.tool_calls.ok_or_else(|| {
                anyhow::anyhow!("model return no tool call despite tool_choice =required")
            })?;

            self.record_tool_calls(&mut context, &tool_calls);
            let final_call = tool_calls.iter().find_map(|tool_call| match tool_call {
                ChatCompletionMessageToolCalls::Function(f) if f.function.name == FINAL_ANSWER => {
                    Some(f)
                }
                _ => None,
            });

            // 如果工具调用时final_answer,说明大模型要给出答案了
            if let Some(final_call) = final_call {
                let args = final_call.function.arguments.clone();
                let parsed: T = serde_json::from_str(&args)?;

                context.add_event(Event::new(
                    context.execution_id.clone(),
                    "tool",
                    vec![ContentItem::ToolResult {
                        tool_call_id: final_call.id.clone(),
                        name: FINAL_ANSWER.to_string(),
                        status: ToolResultStatus::Success,
                        content: args.clone(),
                    }],
                ));

                context.final_result = Some(args);
                return Ok(StructuredAgentResult {
                    output: parsed,
                    context,
                });
            } else {
                self.execute_tool_calls(&mut context, &tool_calls).await;
                context.increment_step();
            }
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

// JSON Schema 本质上是描述和验证 JSON 数据结构的规范，相当于数据的“蓝图”或“契约”。
// JSON Schema 描述类型，参数有，是否必填等等
// 这个方法就等于工具中定义了definiton,返回结构体的结构
fn final_answer_too_definition<T: JsonSchema>() -> anyhow::Result<ChatCompletionTools> {
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_value(schema)?;
    let function = FunctionObjectArgs::default()
        .name(FINAL_ANSWER)
        .description("return the final structured answer matching the required schema")
        .parameters(schema_json)
        .build()
        .map_err(|p| anyhow::anyhow!("failed to build final answer tool definiton :{}", p))?;
    Ok(ChatCompletionTools::Function(ChatCompletionTool {
        function,
    }))
}
