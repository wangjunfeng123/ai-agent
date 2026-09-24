use crate::agent::{ExecutionContext, ToolResultStatus};

#[derive(Debug, Clone, Copy)]
pub struct ToolCallView<'a> {
    pub tool_call_id: &'a str,
    pub name: &'a str,
    pub arguments: &'a str,
}

#[async_trait::async_trait]
pub trait BeforeToolCallBack: Sync + Send {
    // Some 说明需要拦截
    // None 放任执行当前任务
    async fn call(&self, context: &ExecutionContext, tool_call: ToolCallView<'_>)
    -> Option<String>;
}

#[async_trait::async_trait]
pub trait AfterToolCallBack: Sync + Send {
    async fn call(
        &self,
        context: &ExecutionContext,
        tool_call_id: &str,
        name: &str,
        status: ToolResultStatus,
        content: &str,
    ) -> Option<(ToolResultStatus, String)>;
}
