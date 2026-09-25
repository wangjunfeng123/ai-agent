use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    agent::ExecutionContext,
    tools::{calculator::execute::calculator, tool::Tool},
};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculatorArgs {
    pub operator: Operator,
    pub first_number: f64,
    pub second_number: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

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
        let operator = match arg.operator {
            Operator::Add => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
        };
        let ret = calculator(operator, arg.first_number, arg.second_number);
        match ret {
            Ok(val) => Ok(val.to_string()),
            Err(err) => Ok(format!("Error: {err}")),
        }
    }
}
