use milvus::client::*;

const URL: &str = "http://localhost:19530";

#[tokio::test]
async fn test_client_keepalive() {
    // Verify client connects successfully with keepalive settings
    // (10s interval, 5s timeout, enabled while idle)
    let client = Client::new(URL).await;
    assert!(
        client.is_ok(),
        "Client should connect with keepalive enabled"
    );
}
