use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::json;

// 定义计算器工具调用
pub fn calculator_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name("calculator")
            .description("perform basic arithmetic operations")
            .parameters(json!({
                "type":"object",
                "properties": {
                    "operator": {
                        "type": "string",
                        "description":"arithmetic  operation to perform",
                        "enum": ["add","substract","multiply","divide"],
                    },
                    "first_number":{
                        "type":"number",
                        "description":"first number of the calculation"
                    },
                    "second_number":{
                        "type":"number",
                        "description":"second number of the calculation"
                    }
                },
                "required": ["operator","first_number","second_number"]
            }))
            .build()
            .expect("failed to build calculator tool definition"),
    })
}
