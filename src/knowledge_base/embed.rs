use async_openai::types::embeddings::{
    CreateEmbeddingRequest, CreateEmbeddingRequestArgs, EmbeddingInput,
};

// 文字转向量
pub async fn embed_texts(text: &[String], model: &str) -> anyhow::Result<Vec<Vec<f32>>> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let client = async_openai::Client::new();
    let request: CreateEmbeddingRequest = CreateEmbeddingRequestArgs::default()
        .model(model)
        .input(EmbeddingInput::StringArray(text.to_vec()))
        .build()?;

    let response = client.embeddings().create(request).await?;
    let mut data = response.data;

    data.sort_by_key(|e| e.index);
    Ok(data.into_iter().map(|e| e.embedding).collect())
}

// 单个字符调用
pub async fn embed_text(text: &str, model: &str) -> anyhow::Result<Vec<f32>> {
    let owned = [text.to_string()];
    let mut list = embed_texts(&owned, model).await?;
    list.pop()
        .ok_or_else(|| anyhow::anyhow!("embeding api  return no vectors"))
}
