use crate::{
    agent::{ContentItem, ExecutionContext, callback::BeforeLlmCallback, llm_request::LlmRequest},
    callback::context_optimizer::count_token,
};

/// 滑动窗口上下文裁剪。
///
/// 当消息总 token 数超过 `token_threshold` 时触发精简：
/// - 保留头部（system 提示 + 第一条 user 消息）与尾部最近的消息；
/// - 以「条」为单位从中间的旧消息开始丢弃，直到剩余 token 不超过 `window_size`；
/// - `ToolCall` / `ToolResult` 严格成对丢弃，保证消息流合法。
pub struct SlidingWindow {
    pub model: String,
    /// 触发精简的 token 阈值
    pub token_threshold: usize,
    /// 精简后希望达到的 token 上限
    pub window_size: usize,
}

#[async_trait::async_trait]
impl BeforeLlmCallback for SlidingWindow {
    async fn call(&self, _context: &mut ExecutionContext, request: &mut LlmRequest) {
        // 未超阈值：无需处理
        if count_token(&self.model, request) < self.token_threshold {
            return;
        }

        // 找到头部边界（第一条 user 消息，含其之前的内容：system 等）
        let Some(user_idx) = request
            .constents
            .iter()
            .position(|item| matches!(item, ContentItem::Message { role, .. } if role == "user"))
        else {
            return;
        };

        // 头部长度；裁剪只会发生在其后
        let header = user_idx + 1;
        if request.constents.len() <= header {
            return;
        }

        // 需要丢弃的token量
        let over: isize = count_token(&self.model, request) as isize - self.window_size as isize;
        if over <= 0 {
            return;
        }

        // 标记要删除的位置；ToolCall 丢时其配对 ToolResult 一并丢
        let mut to_drop = vec![false; request.constents.len()];
        let mut dropped_tokens = 0usize;
        let len = request.constents.len();
        let mut idx = header;
        while idx < len && (over - dropped_tokens as isize) > 0 {
            to_drop[idx] = true;
            dropped_tokens += token_of(&self.model, &request.constents[idx]);
            // 若是 ToolCall，同步标记配对的 ToolResult
            if let ContentItem::ToolCall { tool_call_id, .. } = &request.constents[idx] {
                if let Some(pos) = request.constents[idx + 1..].iter().position(|item| {
                    matches!(item, ContentItem::ToolResult { tool_call_id: id, .. }
                            if id == tool_call_id)
                }) {
                    let pos = idx + 1 + pos;
                    if !to_drop[pos] {
                        to_drop[pos] = true;
                        dropped_tokens += token_of(&self.model, &request.constents[pos]);
                    }
                }
            }
            idx += 1;
        }

        // 倒序删除，避免索引错位
        for i in (header..len).rev() {
            if to_drop[i] {
                request.constents.remove(i);
            }
        }
    }
}

fn token_of(model: &str, item: &ContentItem) -> usize {
    let mut wrapper = LlmRequest::default();
    wrapper.constents.push(item.clone());
    count_token(model, &wrapper)
}
