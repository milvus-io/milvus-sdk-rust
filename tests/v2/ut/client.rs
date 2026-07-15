// Licensed to the LF AI & Data foundation under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::common::MockServer;
use milvus::v2::error::Error;
use milvus::v2::{ClientV2, ConnectConfig, RetryConfig};
use std::net::TcpListener;
use std::time::{Duration, Instant};

#[tokio::test]
async fn client_connection_interfaces_work_with_mock_server() {
    let server = MockServer::start().await;
    let config = ConnectConfig::new().uri(&server.uri);
    let client = ClientV2::new(&config).await.unwrap();

    client.set_rpc_deadline(Duration::from_secs(1));
    client.set_retry_param(RetryConfig::new().max_attempts(2));
    let health = client
        .check_health(
            milvus::v2::request::utility::CheckHealthRequest::builder()
                .build()
                .expect("valid request"),
        )
        .await
        .unwrap();
    assert_eq!(health.is_healthy().to_owned(), true);
    assert!(health.reasons().is_empty());
    assert_eq!(health.quota_states().to_owned(), ["ReadLimited"]);

    server.assert_called("connect");
    server.assert_called("check_health");
    server.shutdown().await;
}

#[tokio::test]
async fn client_connection_waits_for_the_configured_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stalled connection endpoint");
    let address = listener.local_addr().expect("unused endpoint address");
    drop(listener);
    let uri = format!("http://{address}");
    let connect_timeout = Duration::from_millis(100);
    let config = ConnectConfig::new()
        .uri(uri)
        .connect_timeout(connect_timeout);

    let started = Instant::now();
    let error = match ClientV2::new(&config).await {
        Ok(_) => panic!("unavailable endpoint must time out"),
        Err(error) => error,
    };
    let elapsed = started.elapsed();

    assert!(elapsed >= connect_timeout);
    assert!(elapsed < Duration::from_secs(2));
    assert!(
        matches!(error, Error::Timeout(ref message) if message.contains("connecting to Milvus")),
        "unexpected connection error: {error:?}"
    );
}
