// 压缩的三种方式
// 1.滑动窗口
// 2.压缩:主要是针对工具调用做压缩处理
// 3.总结
pub mod compaction;
pub mod sliding_window;
pub mod summarization;

use crate::{
    agent::{ExecutionContext, callback::BeforeLlmCallback, llm_request::LlmRequest},
    callback::context_optimizer::{compaction::Compaction, summarization::Summarization},
};

pub struct ContextOptimizer {
    pub model: String,
    pub token_threshold: usize,
    pub enable_compaction: bool,
    pub compaction_keep_recent: usize,
    pub enable_summarization: bool,
    pub summarization_keep_recent: usize,
}

impl ContextOptimizer {
    pub fn new(model: impl Into<String>) -> Self {
        ContextOptimizer {
            model: model.into(),
            token_threshold: 20_000,
            enable_compaction: true,
            compaction_keep_recent: 4,
            enable_summarization: true,
            summarization_keep_recent: 5,
        }
    }
}

// 上下文优化策略采用压缩+摘要的方式
#[async_trait::async_trait]
impl BeforeLlmCallback for ContextOptimizer {
    async fn call(&self, context: &mut ExecutionContext, request: &mut LlmRequest) {
        let before: usize = count_token(&self.model, request);
        if before <= self.token_threshold {
            return;
        }
        tracing::info!(
            "开始执行上下文优化策略before={},token_threshold={}",
            before,
            self.token_threshold
        );

        if self.enable_compaction {
            let comp = Compaction {
                keep_recent: self.compaction_keep_recent,
            };
            comp.apply(request);
            let after_comp = count_token(&self.model.clone(), request);
            tracing::info!("compaction before={},after={}", before, after_comp);
            if after_comp < self.token_threshold {
                return;
            }
        }
        if self.enable_summarization {
            let summary = Summarization {
                model: self.model.clone(),
                keep_recent: self.summarization_keep_recent,
            };
            match summary.apply(context, request).await {
                Ok(()) => {
                    let after_summary = count_token(&self.model, request);
                    tracing::info!("summarization before={},after={}", before, after_summary);
                }
                Err(e) => {
                    tracing::error!("summization error={}", e);
                }
            }
        }
    }
}

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
