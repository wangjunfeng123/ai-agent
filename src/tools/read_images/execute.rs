use std::{fs, path::Path};

use async_openai::types::chat::{
    ChatCompletionRequestMessageContentPartImageArgs,
    ChatCompletionRequestMessageContentPartTextArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequestArgs, ImageUrl,
};
use base64::Engine;

use crate::tools::read_images::r#impl::ReadImagesArgs;

// 根据关键字查询images图中的信息
pub async fn analyze_images(arg: &ReadImagesArgs, model: &String) -> anyhow::Result<String> {
    let path = Path::new(&arg.file_path);
    if !path.exists() {
        anyhow::bail!("file not found path={}", path.display());
    }
    let bytes = fs::read(path)?;

    // 拼装扩展名
    let ext = match path.extension().and_then(|ext| ext.to_str()) {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    // image转base64
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    let data = format!("data:{ext};base64,{encoded}");

    let msgs = ChatCompletionRequestUserMessageArgs::default()
        .content(vec![
            ChatCompletionRequestUserMessageContentPart::Text(
                ChatCompletionRequestMessageContentPartTextArgs::default()
                    .text(&arg.query)
                    .build()?,
            ),
            ChatCompletionRequestUserMessageContentPart::ImageUrl(
                ChatCompletionRequestMessageContentPartImageArgs::default()
                    .image_url(ImageUrl {
                        url: data,
                        detail: None,
                    })
                    .build()?,
            ),
        ])
        .build()?;

    // 发起请求
    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(vec![msgs.into()])
        .max_tokens(10241u32)
        .build()?;
    let client = async_openai::Client::new();
    let response = client.chat().create(request).await?;
    response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| anyhow::anyhow!("no content in version response"))
}
