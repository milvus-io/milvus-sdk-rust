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

use milvus::v2::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug)]
struct CapturedRequest {
    path: String,
    headers: HashMap<String, String>,
    body: Value,
}

async fn start_server(
    status: u16,
    response_body: &'static str,
) -> (String, oneshot::Receiver<CapturedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind HTTP test server");
    let address = listener.local_addr().expect("HTTP test server address");
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept HTTP request");
        let mut buffer = Vec::new();
        let header_end = loop {
            let mut chunk = [0_u8; 4_096];
            let count = stream.read(&mut chunk).await.expect("read HTTP request");
            assert!(count > 0, "request ended before its headers");
            buffer.extend_from_slice(&chunk[..count]);
            if let Some(position) = find_bytes(&buffer, b"\r\n\r\n") {
                break position + 4;
            }
        };

        let headers_text = std::str::from_utf8(&buffer[..header_end])
            .expect("HTTP headers are UTF-8")
            .to_owned();
        let mut lines = headers_text.split("\r\n");
        let request_line = lines.next().expect("HTTP request line");
        let path = request_line
            .split_whitespace()
            .nth(1)
            .expect("HTTP request path")
            .to_owned();
        let headers = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
            .collect::<HashMap<_, _>>();
        let content_length = headers
            .get("content-length")
            .and_then(|value| value.parse::<usize>().ok())
            .expect("HTTP content length");
        while buffer.len() < header_end + content_length {
            let mut chunk = [0_u8; 4_096];
            let count = stream.read(&mut chunk).await.expect("read HTTP body");
            assert!(count > 0, "request ended before its body");
            buffer.extend_from_slice(&chunk[..count]);
        }
        let body = serde_json::from_slice(&buffer[header_end..header_end + content_length])
            .expect("JSON HTTP body");
        let _ = sender.send(CapturedRequest {
            path,
            headers,
            body,
        });

        let reason = if status == 200 { "OK" } else { "Error" };
        let response = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream
            .write_all(response.as_bytes())
            .await
            .expect("write HTTP response");
    });
    (format!("http://{address}"), receiver)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn client(url: String) -> BulkImport {
    BulkImport::new(
        &BulkImportConfig::new()
            .url(url)
            .api_key("root:Milvus")
            .timeout(Duration::from_secs(2)),
    )
    .expect("valid bulk-import client")
}

#[tokio::test]
async fn bulk_import_sends_rest_contract_and_decodes_response() {
    let (url, captured) = start_server(
        200,
        r#"{"code":0,"message":"success","data":{"jobId":"job-123"}}"#,
    )
    .await;
    let response = client(url)
        .bulk_import(
            BulkImportRequest::builder()
                .database_name("books_db")
                .collection_name("books")
                .partition_name("archive")
                .files(vec![vec!["one.parquet"], vec!["id.npy", "embedding.npy"]])
                .option("auto_commit", json!(false))
                .build()
                .expect("valid create import request"),
        )
        .await
        .expect("create import job");
    assert_eq!(response.job_id(), Some("job-123"));

    let captured = captured.await.expect("captured create request");
    assert_eq!(captured.path, "/v2/vectordb/jobs/import/create");
    assert_eq!(
        captured.headers.get("authorization").map(String::as_str),
        Some("Bearer root:Milvus")
    );
    assert_eq!(
        captured.headers.get("db-name").map(String::as_str),
        Some("books_db")
    );
    assert_eq!(captured.body["dbName"], "books_db");
    assert_eq!(captured.body["collectionName"], "books");
    assert_eq!(captured.body["files"].as_array().map(Vec::len), Some(2));
    assert_eq!(captured.body["options"]["auto_commit"], false);
    assert!(captured.body.get("apiKey").is_none());
}

#[tokio::test]
async fn list_and_describe_use_milvus_26_paths() {
    for path in [
        "/v2/vectordb/jobs/import/list",
        "/v2/vectordb/jobs/import/describe",
    ] {
        let (url, captured) = start_server(200, r#"{"code":0,"data":{}}"#).await;
        let client = client(url);
        if path.ends_with("/list") {
            client
                .list_import_jobs(
                    ListImportJobsRequest::builder()
                        .database_name("books_db")
                        .collection_name("books")
                        .page_size(20)
                        .current_page(2)
                        .build()
                        .expect("valid list request"),
                )
                .await
                .expect("list import jobs");
        } else {
            let request = GetImportProgressRequest::builder()
                .database_name("books_db")
                .job_id("job-123")
                .build()
                .expect("valid job request");
            client
                .get_import_progress(request)
                .await
                .expect("describe import job");
        }

        let captured = captured.await.expect("captured request");
        assert_eq!(captured.path, path);
        assert_eq!(
            captured.headers.get("db-name").map(String::as_str),
            Some("books_db")
        );
        if path.ends_with("/list") {
            assert_eq!(captured.body["pageSize"], 20);
            assert_eq!(captured.body["currentPage"], 2);
        } else {
            assert_eq!(captured.body["jobId"], "job-123");
            assert!(captured.body.get("dbName").is_none());
            assert!(captured.body.get("jobID").is_none());
        }
    }
}

#[tokio::test]
async fn rest_server_and_http_failures_are_typed() {
    let (url, _) = start_server(
        200,
        r#"{"code":1100,"message":"invalid import source","data":{}}"#,
    )
    .await;
    let error = client(url)
        .list_import_jobs(
            ListImportJobsRequest::builder()
                .build()
                .expect("valid list request"),
        )
        .await
        .expect_err("non-zero REST code must fail");
    assert!(matches!(
        error,
        Error::BulkImport(BulkImportError::Server { code: 1100, .. })
    ));

    let (url, _) = start_server(503, "service unavailable").await;
    let error = client(url)
        .list_import_jobs(
            ListImportJobsRequest::builder()
                .build()
                .expect("valid list request"),
        )
        .await
        .expect_err("non-200 HTTP status must fail");
    assert!(matches!(
        error,
        Error::BulkImport(BulkImportError::HttpStatus { status: 503, .. })
    ));
}
