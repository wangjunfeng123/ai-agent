use async_openai::types::chat::{ChatCompletionTool, ChatCompletionTools, FunctionObjectArgs};
use serde_json::json;

pub fn web_search_tool_definition() -> ChatCompletionTools {
    ChatCompletionTools::Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name("web_search")
            .description("search the web for current information on a given query.")
            .parameters(json!({
                "type" : "object",
                "properties":{
                    "query": {
                        "type":"string",
                        "description": "the search query to execute with travily",
                    },
                    "max_results": {
                        "type":"number",
                        "description": "the maximum number of search results to return",
                        "default": 5,
                        "maximum": 20,
                        "minimum": 0,
                    },
                    "topic": {
                        "type":"string",
                        "description": "The category of the search.news is useful for retrieving real-time updates, particularly about politics, sports, and major current events covered by mainstream media sources. general is for broader, more general-purpose searches that may include a wide range of sources. ",
                        "enum": ["general","news","finance"],
                        "default": "general",
                    },
                    "time_range": {
                        "type":"string",
                        "description": "Useful when looking for sources that have published or updated data. By default, results with no detectable published date are not removed; set filter_by_published_date to true to remove them.",
                        "enum": ["day","week","month","year"],
                    },

                },
                "required":["query"]
            }
            ))
            .build()
            .expect("failed to build web_search tool definition"),
    })
}
