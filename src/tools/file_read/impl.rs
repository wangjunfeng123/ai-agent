use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    pub path: String,
    #[serde(default = "default_start")]
    pub start_line: u32,
    #[serde(default = "default_end")]
    pub end_line: i16,
}

fn default_start() -> u32 {
    1
}
fn default_end() -> i16 {
    -1
}
