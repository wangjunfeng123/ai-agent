use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::ExecutionContext,
    tools::{file_read::execute::read_file, tool::Tool},
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadFileArgs {
    pub path: String,
    #[serde(default = "default_start")]
    pub start_line: usize,
    #[serde(default = "default_end")]
    pub end_line: i16,
}

fn default_start() -> usize {
    1
}
fn default_end() -> i16 {
    -1
}

pub struct ReadFileTool;

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "read a text file (return with line numbers.optionally a range )or a csv file "
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ReadFileArgs))
            .expect("ReadFileArgs schema is not serialized")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let arg: ReadFileArgs = serde_json::from_str(args_json)?;
        let ret = read_file(&arg);
        match ret {
            Ok(val) => Ok(val),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
