use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::ExecutionContext,
    tools::{read_images::execute::analyze_images, tool::Tool},
};

// 读取图片的参数
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadImagesArgs {
    pub file_path: String,
    pub query: String,
}

pub struct ReadImagesTool {
    model: String,
}

impl ReadImagesTool {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }
}

#[async_trait::async_trait]
impl Tool for ReadImagesTool {
    fn name(&self) -> &str {
        "read-images"
    }

    fn description(&self) -> &str {
        "analyze an imag file with a version model to answer a question about what it shows"
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(ReadImagesArgs))
            .expect("schema is alway serialized")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let arg: ReadImagesArgs = serde_json::from_str(args_json)?;
        let ret = analyze_images(&arg, &self.model);
        match ret {
            Ok(val) => Ok(val),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
