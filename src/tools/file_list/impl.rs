use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::ExecutionContext,
    tools::{file_list::execute::list_file, tool::Tool},
};

#[derive(Debug, JsonSchema, Deserialize)]
pub struct ListFileArgs {
    #[serde(default = "default_path")]
    pub path: String,
}

fn default_path() -> String {
    ".".to_string()
}

pub struct ListFileTool;

#[async_trait::async_trait]
impl Tool for ListFileTool {
    fn name(&self) -> &str {
        "list_files"
    }

    fn description(&self) -> &str {
        "list files and directories at a given path,directories listed first"
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ListFileArgs))
            .expect("schema is always serialized")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let arg: ListFileArgs = serde_json::from_str(args_json)?;
        let ret = list_file(&arg.path);
        match ret {
            Ok(val) => Ok(val),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
