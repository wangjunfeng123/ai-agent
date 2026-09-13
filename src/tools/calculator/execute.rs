use serde::Deserialize;

pub fn calculator(
    operator: &str,
    first_number: f64,
    second_number: f64,
) -> anyhow::Result<f64, String> {
    match operator {
        "add" => Ok(first_number + second_number),
        "subtract" => Ok(first_number - second_number),
        "multiply" => Ok(first_number * second_number),
        "divide" => {
            if second_number == 0.0f64 {
                Err("second number is 0".to_string())
            } else {
                Ok(first_number / second_number)
            }
        }
        other => Err(format!("not support operation {other}")),
    }
}

#[derive(Debug, Deserialize)]
pub struct CalculatorArgs {
    pub operator: String,
    pub first_number: f64,
    pub second_number: f64,
}
