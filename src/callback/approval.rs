use std::{collections::HashSet, io::Write};

use crate::agent::{
    ExecutionContext,
    callback::{BeforeToolCallBack, ToolCallView},
};

pub struct ApprovalCallback {
    dangerous_tools: HashSet<String>,
}

impl ApprovalCallback {
    pub fn new(dangerous_tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            dangerous_tools: dangerous_tools.into_iter().map(Into::into).collect(),
        }
    }
}

#[async_trait::async_trait]
impl BeforeToolCallBack for ApprovalCallback {
    // None 表示不拦截
    // Some 表示拦截
    async fn call(
        &self,
        _context: &ExecutionContext,
        tool_call: ToolCallView<'_>,
    ) -> Option<String> {
        // 执行不包含危险操作
        if !self.dangerous_tools.contains(tool_call.name) {
            return None;
        }
        println!("⚠️ 即将执行高危操作");
        println!(
            "工具名称={};工具参数={}",
            tool_call.name, tool_call.arguments
        );
        let approved = tokio::task::spawn_blocking(|| {
            print!("是否执行？");
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            input.trim().eq_ignore_ascii_case("y")
        })
        .await
        .unwrap_or(false);

        // 同意执行
        if approved {
            tracing::info!("✅已批准，继续执行");
            None
        } else {
            tracing::info!("❎已拒绝，终止执行！");
            Some(format!("User denied execute  of {}", tool_call.name))
        }
    }
}
