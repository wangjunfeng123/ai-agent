use anyhow::{Context, Ok};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct WebSearchArgs {
    pub query: String,
    #[schemars(range(min = 0, max = 20))]
    #[serde(default = "default_max_results")]
    pub max_results: u8,
    #[serde(default = "default_topic")]
    pub topic: String,
    #[serde(default)]
    pub time_range: Option<String>,
}

fn default_max_results() -> u8 {
    2
}

fn default_topic() -> String {
    "general".to_string()
}

// 调用trvily请求值
#[derive(Debug, Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    max_results: u8,
    topic: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_range: Option<&'a str>,

    search_depth: &'a str,
    include_answer: bool,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

// 调用trvily返回值
// Serialize序列化，Deserialize反序列化
#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<SearchResult>,
    answer: Option<String>,
}
//
// 返回给大语言模型的结构
#[derive(Debug, Serialize)]
pub struct WebSearchOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,

    pub results: Vec<SearchResult>,
}

pub async fn web_search(args: WebSearchArgs) -> anyhow::Result<WebSearchOutput> {
    let tavily_api_key = std::env::var("TAVILY_API_KEY").context("tavily api key not found")?;

    let tavily_req = TavilyRequest {
        api_key: &tavily_api_key,
        query: &args.query,
        max_results: args.max_results,
        topic: &args.topic,
        time_range: args.time_range.as_deref(),
        search_depth: "advanced",
        include_answer: true,
    };

    let response = reqwest::Client::new()
        .post("https://api.tavily.com/search")
        .json(&tavily_req)
        .send()
        .await
        .context("request tavily failed~")?;

    if !response.status().is_success() {
        anyhow::bail!("tavily returned status ={}", response.status());
    }

    let parsed: TavilyResponse = response
        .json()
        .await
        .context("parse tavily response failed")?;
    Ok(WebSearchOutput {
        answer: parsed.answer,
        results: parsed.results,
    })
}
