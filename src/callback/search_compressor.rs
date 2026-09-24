use serde_json::Value;

use crate::{
    agent::{ExecutionContext, ToolResultStatus, callback::AfterToolCallBack},
    knowledge_base::{chunk::fixed_length_chunking, search::vector_search},
};

pub struct SearchCompressorCallback;

const COMPRESS_THREHOLD: usize = 2000;
const CHUNK_SIZE: usize = 500;
const CHUNK_OVERLAP: usize = 50;
const TOP_K: usize = 2;

#[async_trait::async_trait]
impl AfterToolCallBack for SearchCompressorCallback {
    // return None ;表示不需要拦截
    // return Some ;表示需要拦截处理
    async fn call(
        &self,
        context: &ExecutionContext,
        tool_call_id: &str,
        tool_name: &str,
        status: ToolResultStatus,
        content: &str,
    ) -> Option<(ToolResultStatus, String)> {
        if tool_name != "web_search" || status != ToolResultStatus::Success {
            return None;
        }
        if content.len() < COMPRESS_THREHOLD {
            return None;
        }
        let query = extract_query(context, tool_call_id)?;
        let chunks = fixed_length_chunking(content, CHUNK_SIZE, CHUNK_OVERLAP);

        if chunks.is_empty() {
            return None;
        }
        tracing::info!(
            "○ compressing web_search result={} chars -> chunking into {} peice.",
            content.len(),
            chunks.len()
        );
        match vector_search(&query, &chunks, TOP_K).await {
            Ok(hits) => {
                tracing::info!("");
                let compressor = hits
                    .into_iter()
                    .map(|hit| hit.text)
                    .collect::<Vec<_>>()
                    .join("\n\n");
                tracing::info!(
                    "✅ compressing complete {} chars -> {} chars top {} of {} chunks",
                    content.len(),
                    compressor.len(),
                    TOP_K,
                    chunks.len()
                );
                Some((status, compressor))
            }
            Err(err) => {
                tracing::warn!("search compression skipped :{err}");
                None
            }
        }
    }
}

fn extract_query(context: &ExecutionContext, tool_call_id: &str) -> Option<String> {
    context
        .events
        .iter()
        .flat_map(|event| &event.content)
        .find_map(|item| match item {
            crate::agent::ContentItem::ToolCall {
                tool_call_id: id,
                name,
                arguments,
            } if id == tool_call_id && name == "web_search" => arguments
                .get("query")
                .and_then(Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
}
