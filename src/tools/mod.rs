use async_openai::types::chat::ChatCompletionTools;

use crate::tools::calculator::definition::calculator_tool_definition;

pub mod calculator;

pub fn get_tools() -> Vec<ChatCompletionTools> {
    vec![calculator_tool_definition()]
}
