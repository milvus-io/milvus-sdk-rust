use milvus::client::Client;
use milvus::error::Result;

mod common;
use common::*;

#[tokio::test]
async fn run_analyzer_returns_tokens() -> Result<()> {
    let client = Client::new(URL).await?;
    let results = client
        .run_analyzer(
            vec![
                "Milvus is a vector database".to_string(),
                "Rust SDK integration tests".to_string(),
            ],
            "{\"type\": \"standard\"}",
        )
        .await?;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|result| !result.tokens.is_empty()));

    Ok(())
}
