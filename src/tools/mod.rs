pub mod calculator;
pub mod web_search;

use async_openai::types::chat::ChatCompletionTools;

use crate::tools::{
    calculator::definition::calculator_tool_definition,
    web_search::definition::web_search_tool_definition,
};

pub fn get_tools() -> Vec<ChatCompletionTools> {
    vec![calculator_tool_definition(), web_search_tool_definition()]
}
