use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{agent::ExecutionContext, tools::tool::Tool};

#[derive(Debug, JsonSchema, Deserialize)]
pub struct DeleteFileArgs {
    pub file_path: String,
}

pub struct DeleteFileTool;

#[async_trait::async_trait]
impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "delete the file .this action cannot be undone."
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(DeleteFileArgs))
            .expect("schema is always serialized")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let arg: DeleteFileArgs = serde_json::from_str(args_json)?;
        tracing::info!("🗑️ attempting to delete:{}", arg.file_path);
        match std::fs::remove_file(&arg.file_path) {
            Ok(()) => {
                tracing::info!("✅ delete file succss path:{}", arg.file_path);
                Ok(format!("file {} has been deleted", arg.file_path))
            }
            Err(e) => {
                tracing::info!("❎ delete failed path:{}.{e}", arg.file_path);
                Err(anyhow::anyhow!("delete failed path:{}.{e}", arg.file_path))
            }
        }
    }
}
