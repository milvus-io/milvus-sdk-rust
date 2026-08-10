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

mod utils;

use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use serde_json::{json, Value};
use utils::client;

async fn run_analyzer(client: &ClientV2, params: Value, text: &str) -> Result<()> {
    println!("\nRun analyzer params: {params}");
    println!("Text: {text}");
    let response = client
        .run_analyzer(
            RunAnalyzerRequest::builder()
                .analyzer_params(params.to_string())
                .texts([text])
                .with_detail(true)
                .with_hash(true)
                .build()?,
        )
        .await?;
    for result in response.results() {
        for token in result.get_tokens() {
            println!(
                "\t{{token: {}, start: {}, end: {}, position: {}, position_len: {}, hash: {}}}",
                token.get_text(),
                token.get_start_offset(),
                token.get_end_offset(),
                token.get_position(),
                token.get_position_length(),
                token.get_hash()
            );
        }
        println!("\t------------------------------");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = client().await?;
    let cases = [
        (
            json!({"tokenizer":"standard","filter":[{"type":"stop","stop_words":["and","for"]}]}),
            "Milvus supports L2 distance and IP similarity for float vector.",
        ),
        (
            json!({"tokenizer":"jieba","filter":["cnalphanumonly"]}),
            "Milvus 是 LF AI & Data Foundation 下的一个开源项目，以 Apache 2.0 许可发布。",
        ),
        (
            json!({"tokenizer":{"type":"lindera","dict_kind":"ipadic"}}),
            "東京スカイツリーの最寄り駅はとうきょうスカイツリー駅で",
        ),
        (json!({"tokenizer":"icu"}), "Привет! Как дела?"),
        (
            json!({"tokenizer":"standard","filter":[{"type":"length","max":6}]}),
            "The length filter allows control over token length requirements for text processing.",
        ),
        (
            json!({"tokenizer":"standard","filter":[{"type":"decompounder","word_list":["dampf","schiff","fahrt","brot","backen","automat"]}]}),
            "dampfschifffahrt brotbackautomat",
        ),
        (
            json!({"tokenizer":"standard","filter":[{"type":"stemmer","language":"english"}]}),
            "running runs looked ran runner",
        ),
        (
            json!({"tokenizer":"standard","filter":[{"type":"regex","expr":"^(?!test)"}]}),
            "testItem apple testCase banana",
        ),
    ];
    for (params, text) in cases {
        run_analyzer(&client, params, text).await?;
    }
    Ok(())
}
