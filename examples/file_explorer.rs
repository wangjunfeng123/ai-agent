use std::sync::Arc;

use ai_agent::{
    agent::Agent,
    content::{DEEPSEEK_V4_FLASH, KIMI_K27_CODE_MODEL, VISION_MODEL},
    tools::build_file_explorer_toolbox,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let tools = Arc::new(build_file_explorer_toolbox(VISION_MODEL));
    let instructions = r#"
你是一个善于探索和审阅文件的助手。你必须严格遵守以下工作流程，不要跳过任何步骤。

可用工具及用途：
- unzip_file：解压 zip 压缩包，便于后续查看内容
- list_files：列出指定路径下的文件/目录（目录会排在前面）
- read_file：读取文本文件或 csv 文件（可指定行号范围）
- read_images：用视觉模型分析图片内容，判断图里显示的信息
- delete_file：删除文件（不可恢复，谨慎使用）

工作流程（按顺序执行，不可省略）：
1. 【解压】拿到压缩包后，第一步先调用 unzip_file 解压，并明确解压目标路径。
2. 【概览】用 list_files 查看解压后的目录结构，摸清文件布局，不要先列 `src` 等与问题无关的目录。
3. 【筛选】依赖文件名和路径判断哪些文件与用户问题相关，只对它们深入阅读。
4. 【细读】用 read_file 读取关键文本文件；遇到图片、截图、界面图时用 read_images 分析内容。
5. 【结论】基于以上证据给出明确、有依据的结论，并引用来源，不要凭空猜测。

纪律要求：
- 每一步都要在动手前说明你要做什么。
- 不相关的文件不要浪费时间阅读。
- 严禁在解压与目录概览之前就直接给出或臆测答案。
        "#;

    let agent = Agent::new(DEEPSEEK_V4_FLASH, Some(instructions), tools).with_max_step(20);
    let result = agent.run("分析这个压缩包/Users/wangjunfeng/workspace/github/ai-agent/resources/payment.zip，看看小精灵支付平台支持支付宝吗").await?;

    tracing::info!("答案={:?}", result.output);

    tracing::info!(
        "\n本次执行了{}步骤，记录了{}个event",
        result.context.current_step,
        result.context.events.len()
    );
    anyhow::Ok(())
}
