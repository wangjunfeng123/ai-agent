use crate::{
    agent::{ContentItem, ExecutionContext, callback::BeforeLlmCallback, llm_request::LlmRequest},
    callback::context_optimizer::count_token,
};

/**
 * 滑动窗口
 */
pub struct SlidingWindow {
    pub model: String,
    // token 请求token数最大值，超过这个阈值就会触发精简
    pub token_threshold: usize,
    // 滑动窗口的最大值
    pub window_size: usize,
}

#[async_trait::async_trait]
impl BeforeLlmCallback for SlidingWindow {
    async fn call(&self, _context: &ExecutionContext, request: &mut LlmRequest) {
        if count_token(&self.model, &request) < self.token_threshold {
            return;
        }
        // 去除用户内容之前的内容
        let usr_idx = request
            .constents
            .iter()
            .position(|item| matches!(item, ContentItem::Message { role ,.. } if role == "user"));
        let Some(usr_idx) = usr_idx else {
            return;
        };
        let header = request.constents[..=usr_idx].to_vec();

        let mut reminding = request.constents[usr_idx + 1..].to_vec();
        if reminding.len() > self.window_size {
            let cut = reminding.len() - self.window_size;
            reminding = reminding.split_off(cut);
        }
        request.constents = header.into_iter().chain(reminding).collect();
    }
}
