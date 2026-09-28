use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, CreateChatCompletionRequestArgs,
};
use serde_json::Value;

use crate::agent::{ContentItem, ExecutionContext, llm_request::LlmRequest};

/// 上下文优化；第三种方式汇总
///
/// 纯增量式摘要：每次只对 "上次摘要点到本次 summary_end" 之间**新产生**的内容做摘要，
/// 不拼接 / 重放之前的旧摘要。保留用户最初的提问（header），
/// 并保留最近的 `keep_recent` 条对话。摘要进度通过 `context.state` 持久化。
const SUMMARIZATION_PROMPT: &str = "you are summarizing an ai agent's work-in-process history.\
given the execution history below, write a short, structured summary covering:\
1) key findings so far; 2) tools that were called; 3) what remains to be done;\
Be concise - a few sentences is enough.\n\n Execution history:\n{history}";

pub struct Summarization {
    pub model: String,
    /// 保留最近的几次对话（消息条数）
    pub keep_recent: usize,
}

impl Summarization {
    pub async fn apply(
        &self,
        context: &mut ExecutionContext,
        request: &mut LlmRequest,
    ) -> anyhow::Result<()> {
        let len = request.constents.len();
        if len < self.keep_recent {
            return Ok(());
        }

        // 用户最初的提问位置（header 就是其之前的 system + 该 user 消息）
        let Some(user_idx) = request
            .constents
            .iter()
            .position(|item| matches!(item, ContentItem::Message { role, .. } if role == "user"))
        else {
            return Ok(());
        };
        let header = user_idx + 1;

        // 摘要区域的上界：其后的 keep_recent 条原样保留
        let summary_end = len.saturating_sub(self.keep_recent);
        // 上次摘要处理到的位置；首次就是第一条 user 消息
        let last_summary_idx = context
            .state_mut()
            .get("last_summary_idx")
            .and_then(Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(user_idx);
        // 本次新增待摘要的起始位置
        let summary_start = last_summary_idx + 1;

        // 新增内容不足：还不需要做摘要
        if summary_start >= summary_end || summary_start < header {
            return Ok(());
        }

        // 提取本次新增（[last_summary_idx, summary_end)）的执行历史文本
        let new_history = request.constents[summary_start..summary_end]
            .iter()
            .map(content_item_to_text)
            .collect::<Vec<_>>()
            .join("\n\n");

        // 仅对本次新增段做增量摘要，不拼接旧摘要
        let summary = self.summarize(&new_history).await?;

        // 持久化增量摘要的进度（下次从本次 summary_end 接着摘要）
        context
            .state_mut()
            .insert("last_summary_idx".into(), Value::from(summary_end as u64));
        // 记录最近一次增量摘要的结果，便于外部查看
        context
            .state_mut()
            .insert("summary".into(), Value::from(summary.clone()));

        // 重组消息：header + 摘要占位 + 最近的 keep_recent 条
        let mut new_constents: Vec<ContentItem> = request.constents[..header].to_vec();
        new_constents.push(ContentItem::Message {
            role: "assistant".to_string(),
            content: format!("[Summary of previous turns]\n{summary}"),
        });
        new_constents.extend_from_slice(&request.constents[summary_end..]);
        request.constents = new_constents;

        Ok(())
    }

    async fn summarize(&self, new_history: &str) -> anyhow::Result<String> {
        let prompt = SUMMARIZATION_PROMPT.replace("{history}", new_history);

        let client = async_openai::Client::new();
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(prompt.as_str())
                .build()?
                .into(),
        ];
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(messages)
            .build()?;
        let resp = client.chat().create(request).await?;
        resp.choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("summarization returned no content"))
    }
}

/// 把单条 ContentItem 转成可供摘要模型读取的文本
fn content_item_to_text(item: &ContentItem) -> String {
    match item {
        ContentItem::Message { role, content } => format!("[{role}]: {content}"),
        ContentItem::ToolCall {
            name, arguments, ..
        } => format!("[tool_call] {name}({arguments})"),
        ContentItem::ToolResult {
            name,
            status,
            content,
            ..
        } => format!("[tool_result {status:?}] {name}: {content}"),
    }
}
