use schemars::schema_for;
use serde_json::Value;

use crate::{
    agent::ExecutionContext,
    tools::{
        calculator::execute::{CalculatorArgs, calculator},
        tool::Tool,
    },
};

pub struct CalculatorTool;

#[async_trait::async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "perform basic arithmetic operations"
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schema_for!(CalculatorArgs))
            .expect("failed to serialize CalculatorArgs schema")
    }

    async fn execute(
        &self,
        args_json: &str,
        _context: &ExecutionContext,
    ) -> anyhow::Result<String> {
        let arg: CalculatorArgs = serde_json::from_str(args_json)?;
        let ret = calculator(&arg.operator, arg.first_number, arg.second_number);
        match ret {
            Ok(val) => Ok(val.to_string()),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
