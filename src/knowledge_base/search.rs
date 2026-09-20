use std::collections::BinaryHeap;

use anyhow::Ok;

use crate::{
    content::BAAI_BGE_M3,
    knowledge_base::embed::{embed_text, embed_texts},
};

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub text: String,
    pub similarity: f32,
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    } else {
        dot / (norm_a * norm_b)
    }
}

struct ScoredIndex {
    similarity: f32,
    index: usize,
}

impl PartialEq for ScoredIndex {
    fn eq(&self, other: &Self) -> bool {
        self.similarity == other.similarity
    }
}

impl Eq for ScoredIndex {}
impl Ord for ScoredIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.similarity.total_cmp(&self.similarity)
    }
}
impl PartialOrd for ScoredIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// query: 查询的数据
// chunks 块列表
// top_k: 取相似度高的前k个
pub async fn vector_search(
    query: &str,
    chunks: &[String],
    top_k: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    if chunks.is_empty() || top_k == 0 {
        return Ok(Vec::new());
    }
    let query_embedding = embed_text(query, BAAI_BGE_M3).await?;
    let chunks_embedding = embed_texts(chunks, BAAI_BGE_M3).await?;

    let mut heap: BinaryHeap<ScoredIndex> = BinaryHeap::with_capacity(top_k + 1);
    for (index, embedding) in chunks_embedding.iter().enumerate() {
        let similarity = cosine_similarity(&query_embedding, embedding);
        heap.push(ScoredIndex { similarity, index });
        if heap.len() > top_k {
            heap.pop();
        }
    }
    let mut ranked: Vec<ScoredIndex> = heap.into_vec();
    ranked.sort_by(|a, b| b.similarity.total_cmp(&a.similarity));

    Ok(ranked
        .into_iter()
        .map(|scored| SearchHit {
            text: chunks[scored.index].clone(),
            similarity: scored.similarity,
        })
        .collect())
}
