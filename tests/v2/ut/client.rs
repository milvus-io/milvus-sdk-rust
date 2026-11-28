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

use super::common::{MockMilvus, MockServer};
use milvus::proto::milvus::milvus_service_server::MilvusServiceServer;
use milvus::proto::milvus::RunningRequestInfo;
use milvus::v2::error::Error;
use milvus::v2::request::rbac::UpdatePasswordRequest;
use milvus::v2::request::utility::{CheckHealthRequest, ListRunningRequestsRequest};
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

async fn start_zstd_server(
    mock: MockMilvus,
) -> (
    String,
    tokio::task::JoinHandle<Result<(), tonic::transport::Error>>,
) {
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::codec::CompressionEncoding::Zstd;
    use tonic::service::interceptor::InterceptedService;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let uri = format!("http://{}", listener.local_addr().unwrap());
    let service = MilvusServiceServer::new(mock)
        .accept_compressed(Zstd)
        .send_compressed(Zstd);
    let service = InterceptedService::new(service, |request: tonic::Request<()>| {
        assert_eq!(request.metadata().get("grpc-encoding").unwrap(), "zstd");
        let accepted = request
            .metadata()
            .get("grpc-accept-encoding")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(accepted.split(',').any(|encoding| encoding == "zstd"));
        Ok(request)
    });
    let server = tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(service)
            .serve_with_incoming(TcpListenerStream::new(listener)),
    );
    (uri, server)
}

#[tokio::test]
async fn zstd_handshake_and_rpc_round_trip_after_reset() {
    let mock = MockMilvus::default();
    let (uri, server) = start_zstd_server(mock.clone()).await;
    let config = ConnectConfig::new().uri(uri);
    assert!(!config.get_zstd_compression());
    let client = ClientV2::new(&config.zstd_compression(true)).await.unwrap();
    client
        .update_password(
            UpdatePasswordRequest::builder()
                .username("alice")
                .old_password("old-password")
                .new_password("new-password")
                .reset_connection(true)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

    let health = client
        .check_health(CheckHealthRequest::builder().build().unwrap())
        .await
        .unwrap();
    assert!(health.is_healthy());
    assert_eq!(health.quota_states(), ["ReadLimited"]);
    let credentials = mock.authorization_headers("connect");
    assert_eq!(credentials.len(), 2);
    assert!(credentials[0].is_none());
    assert!(credentials[1].is_some());
    assert_eq!(mock.authorization_headers("check_health"), credentials[1..]);
    server.abort();
}

#[tokio::test]
async fn zstd_handshake_reports_unsupported_peer() {
    let server = MockServer::start().await;
    let config = ConnectConfig::new().uri(&server.uri).zstd_compression(true);
    let error = ClientV2::new(&config)
        .await
        .err()
        .expect("the peer did not enable zstd");
    assert!(
        matches!(&error, Error::Grpc(status)
            if status.code() == tonic::Code::Unimplemented && status.message().contains("zstd")),
        "{error:?}"
    );
    // Only the fixture's original, uncompressed Connect reached the service.
    assert_eq!(server.service.call_count("connect"), 1);
    server.shutdown().await;
}

#[tokio::test]
async fn zstd_reply_respects_the_default_decoded_size_limit() {
    let mock = MockMilvus::default();
    let (uri, server) = start_zstd_server(mock.clone()).await;
    let client = ClientV2::new(&ConnectConfig::new().uri(uri).zstd_compression(true))
        .await
        .unwrap();
    client.set_retry_param(RetryConfig::new().max_attempts(1));
    let request = ListRunningRequestsRequest::builder().build().unwrap();
    let mut row = RunningRequestInfo {
        request_id: "7-to03zu-1".into(),
        expr: "id in [0]".into(),
        ..Default::default()
    };
    mock.set_running_requests(vec![row.clone()]);
    let response = client.list_running_requests(request.clone()).await.unwrap();
    assert_eq!(response.requests()[0].get_expr(), row.expr);

    // The expression is returned as metadata and is never evaluated.
    row.expr = "x".repeat(4 * 1024 * 1024 + 1);
    mock.set_running_requests(vec![row]);
    let error = client.list_running_requests(request).await.unwrap_err();
    assert!(
        matches!(&error, Error::Grpc(status)
            if status.code() == tonic::Code::ResourceExhausted
                && status.message().contains("decompress")),
        "{error:?}"
    );
    assert_eq!(mock.call_count("list_running_requests"), 2);
    server.abort();
}
