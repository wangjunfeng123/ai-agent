use ai_agent::{
    knowledge_base::{chunk::fixed_length_chunking, search::vector_search},
    tools::web_search::execute::{WebSearchArgs, web_search},
};
use tiktoken_rs::cl100k_base;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let args = WebSearchArgs {
        query: "2026年美加墨世界杯最佳射手是？".to_string(),
        max_results: 10,
        topic: "general".to_string(),
        time_range: Some("year".to_string()),
    };
    // 1.根据关键字query通过web工具搜索10篇文章。
    let output = web_search(args).await?;
    let full_text = output
        .results
        .iter()
        .map(|r| format!("title :{}\n{}", r.title, r.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    // 2.这个工具是根据文本计算token的使用量
    let enc = cl100k_base()?;
    let total_token = enc.encode_with_special_tokens(&full_text).len();

    tracing::info!("total full text ={}", full_text.len());
    tracing::info!("total token ={}", total_token);

    let mut all_chunks = Vec::new();

    // 3.把这些文章转成向量
    for result in &output.results {
        let text = format!("title :{}\n{}", result.title, result.content);
        for chunk in fixed_length_chunking(&text, 500, 50) {
            all_chunks.push(chunk);
        }
    }

    // 4.根据关键字去搜索向量关联度高的chunk
    let hits = vector_search("2026年美加墨世界杯最佳射手是?", &all_chunks, 3).await?;
    println!("{}", "=".repeat(50));
    for (i, hit) in hits.iter().enumerate() {
        let preview: String = hit.text.chars().take(300).collect();
        print!("\n[{}] similarity {:.3}", i + 1, hit.similarity);
        println!("{preview}");
    }
    // 5. 只留下项目的的3段，拼起来，看token数降到了多少
    let selected_text = hits
        .iter()
        .map(|hit| hit.text.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    let selected_tokens = enc.encode_with_special_tokens(&selected_text).len();

    println!("total token {total_token}");
    println!("selected token {selected_tokens}");

    // 6.计算选中的块，占总token的比例，从而计算节省了多少token
    println!("\n{}", "=".repeat(50));
    println!(
        "saving token precent ={}",
        (1.0 - selected_tokens as f64 / total_token as f64) * 100.0
    );

    anyhow::Ok(())
}
