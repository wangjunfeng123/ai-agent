use std::collections::HashMap;

use serde_json::Value;

use crate::agent::{ContentItem, llm_request::LlmRequest};

//
pub struct Compaction {
    // 保证MessageContents保留几条不被压缩
    pub keep_recent: usize,
}

impl Compaction {
    pub fn apply(&self, request: &mut LlmRequest) {
        // 受保护的content，即不压缩
        // 压缩的是压缩工具调用的结果
        let protect_from = request.constents.len().saturating_sub(self.keep_recent);
        let mut call_arg: HashMap<String, Value> = HashMap::new();

        for (idx, item) in request.constents.iter_mut().enumerate() {
            match item {
                ContentItem::ToolCall {
                    tool_call_id,
                    arguments,
                    ..
                } => {
                    call_arg.insert(tool_call_id.clone(), arguments.clone());
                }
                ContentItem::ToolResult {
                    tool_call_id,
                    name,
                    content,
                    ..
                } => {
                    if idx >= protect_from {
                        //受保护的content不压缩
                        continue;
                    } else {
                        // 执行压缩的相关逻辑，把执行结果修改掉
                        let val = call_arg.get(&tool_call_id.to_string());
                        let replacement = match name.as_str() {
                            "read_file" => {
                                let path = val
                                    .and_then(|a| a.get("path"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("unknow");
                                Some(format!(
                                    "read_file {path} was already read.call read_file again if you need",
                                ))
                            }
                            "web_search" => {
                                let query = val
                                    .and_then(|a| a.get("query"))
                                    .and_then(Value::as_str)
                                    .unwrap_or("unkown query");
                                Some(format!(
                                    "web_search {query} was already processed. call web search again if you need it."
                                ))
                            }
                            _ => None,
                        };
                        if let Some(new_replace) = replacement {
                            *content = new_replace;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
