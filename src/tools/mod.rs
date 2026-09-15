pub mod calculator;
pub mod tool;
pub mod web_search;

use std::collections::HashMap;

use crate::tools::{
    calculator::r#impl::CalculatorTool, tool::Tool, web_search::r#impl::WebSearchTool,
};

// pub fn get_tools() -> Vec<ChatCompletionTools> {
//     vec![calculator_tool_definition(), web_search_tool_definition()]
// }

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub fn build_toolbox() -> ToolBox {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool), Box::new(WebSearchTool)];

    tools
        .into_iter()
        .map(|t| (t.name().to_string(), t))
        .collect()
}
