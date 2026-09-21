use std::{fs, path::Path};

use crate::tools::file_read::r#impl::ReadFileArgs;

// 根据路径读取文件，从第几行读取到第几行
pub fn read_file(arg: &ReadFileArgs) -> anyhow::Result<String> {
    let path = Path::new(&arg.path);
    if !path.exists() {
        anyhow::bail!("file not found path={}", arg.path);
    }
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("csv") => read_csv_as_markdown(path),
        _ => read_text_with_line_number(path, arg.start_line, arg.end_line),
    }
}

// 通过行号读取文件
fn read_text_with_line_number(
    path: &Path,
    start_line: usize,
    end_line: i16,
) -> anyhow::Result<String> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    // 起始行索引 = start_line-1
    let start_index = start_line.saturating_sub(1);
    // 结束索引 = 2者中最小的一个
    let end_index = (end_line as usize).min(lines.len());

    let mut result = String::new();
    for (offset, line) in lines
        .get(start_index..end_index)
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        result.push_str(&format!("{:>4} | {}\n", start_index + offset + 1, line));
    }

    Ok(result)
}

// 读取csv文件,转为markdown格式
fn read_csv_as_markdown(path: &Path) -> anyhow::Result<String> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.clone();

    let mut table = String::new();
    table.push_str(&format!(
        "| {} |\n",
        headers.into_iter().collect::<Vec<_>>().join(" | "),
    ));
    table.push_str(&format!("| {}\n", "---|".repeat(headers.len())));

    for record in reader.records() {
        let record = record?;
        table.push_str(&format!(
            "| {} |\n",
            record.iter().collect::<Vec<_>>().join(" | ")
        ));
    }
    Ok(table)
}
