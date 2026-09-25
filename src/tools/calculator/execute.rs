pub fn calculator(
    operator: &str,
    first_number: f64,
    second_number: f64,
) -> anyhow::Result<f64, String> {
    match operator {
        "+" => Ok(first_number + second_number),
        "-" => Ok(first_number - second_number),
        "*" => Ok(first_number * second_number),
        "/" => {
            if second_number == 0.0f64 {
                Err("second number is 0".to_string())
            } else {
                Ok(first_number / second_number)
            }
        }
        other => Err(format!("not support operation {other}")),
    }
}
