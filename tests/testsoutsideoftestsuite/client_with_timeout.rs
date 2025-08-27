use milvus::client::*;
use std::time::Duration;

const URL: &str = "http://localhost:19530";

#[tokio::test]
async fn test_client_with_timeout() {
    let client = Client::with_timeout(URL, Duration::from_secs(10), None, None).await;

    assert!(client.is_ok());
}
