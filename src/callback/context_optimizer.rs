// 压缩的三种方式
// 1.滑动窗口
// 2.压缩
// 3.总结
pub mod sliding_window;

use crate::agent::llm_request::LlmRequest;

// 做压缩之前，需要计算上下文的大小
// 没有超过，可以不做处理
// 超过：在做压缩的逻辑
pub fn count_token(_model: &str, request: &LlmRequest) -> usize {
    let bpe = tiktoken_rs::cl100k_base()
        .expect("tiktoken should always resolve to the cl100k_base encoding");

    let mut total = 0usize;
    // 读取
    for instruction in &request.instructions {
        total += 4 + bpe.encode_ordinary(instruction).len();
    }

    for item in &request.constents {
        total += 4;
        total += match item {
            crate::agent::ContentItem::Message { content, .. } => {
                bpe.encode_ordinary(content).len()
            }
            crate::agent::ContentItem::ToolCall {
                name, arguments, ..
            } => {
                bpe.encode_ordinary(name).len() + bpe.encode_ordinary(&arguments.to_string()).len()
            }
            crate::agent::ContentItem::ToolResult { content, .. } => {
                bpe.encode_ordinary(content).len()
            }
        };
    }
    total
}
