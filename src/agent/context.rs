use crate::agent::event::Event;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// token使用量统计
#[derive(Debug, Clone, Copy, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    // 累计token用量
    pub fn add(&mut self, prompt_tokens: u32, completion_tokens: u32, total_tokens: u32) {
        self.prompt_tokens += prompt_tokens;
        self.completion_tokens += completion_tokens;
        self.total_tokens += total_tokens;
    }
}

// ExecutionContext是整个agent的核心组件；
//  1. 一个ExecutionContext包含多个Event
//  2. 执行步数，防止出现死循环
//  3. state:临时的数据
//  4. final_result 最终的结果
#[derive(Debug)]
pub struct ExecutionContext {
    // 唯一ID
    pub execution_id: String,
    // 事件列表
    pub events: Vec<Event>,
    // 当前步数
    pub current_step: u32,
    // 临时存储的草稿本
    pub state: HashMap<String, Value>,
    // 最终结果
    pub final_result: Option<String>,
    // token使用量
    pub usage: TokenUsage,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            events: Vec::new(),
            current_step: 0,
            state: HashMap::new(),
            final_result: None,
            usage: TokenUsage::default(),
        }
    }
    // 添加event
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }
    // 增加步数
    pub fn increment_step(&mut self) {
        self.current_step += 1
    }
}
impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
