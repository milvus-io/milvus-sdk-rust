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

use crate::client::{Client, ServerVersion};
use crate::collection::{CompactionInfo, CompactionState};
use crate::error::{Error, Result};
use crate::proto::common::{MsgBase, MsgType};
use crate::proto::milvus::{
    ConnectRequest, FlushRequest, GetCompactionStateRequest, ManualCompactionRequest,
};
use crate::proto::{self};
use crate::utils::status_to_result;
use std::collections::HashMap;

impl Client {
    pub async fn flush_collections<C>(&self, collections: C) -> Result<HashMap<String, Vec<i64>>>
    where
        C: IntoIterator,
        C::Item: ToString,
    {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(MsgBase::new(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: collections.into_iter().map(|x| x.to_string()).collect(),
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(res
            .coll_seg_i_ds
            .into_iter()
            .map(|(k, v)| (k, v.data))
            .collect())
    }

    pub async fn get_server_version(&self) -> Result<ServerVersion> {
        let res = self
            .client
            .clone()
            .connect(ConnectRequest {
                base: Some(MsgBase::new(MsgType::Connect)),
                client_info: None,
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;
        let info = res.server_info.ok_or_else(|| {
            Error::Unexpected("server version response missing server info".to_string())
        })?;

        Ok(ServerVersion {
            version: info.build_tags,
            build_time: info.build_time,
            git_commit: info.git_commit,
            go_version: info.go_version,
            deploy_mode: info.deploy_mode,
        })
    }

    pub async fn flush<S>(&self, collection_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let res = self
            .client
            .clone()
            .flush(FlushRequest {
                base: Some(MsgBase::new(MsgType::Flush)),
                db_name: "".to_string(),
                collection_names: vec![collection_name.into()],
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(())
    }

    /// manual compaction
    ///
    /// # Arguments
    ///
    /// * `collection_name` - The name of the collection
    /// * `is_clustering` - Whether to perform clustering compaction
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `CompactionInfo` if successful, or an error if the compaction fails.
    pub async fn manual_compaction<S>(
        &self,
        collection_name: S,
        is_clustering: Option<bool>,
    ) -> Result<CompactionInfo>
    where
        S: Into<String>,
    {
        let collection_name = collection_name.into();
        let collection = self.collection_cache.get(&collection_name).await?;
        let major_compaction = is_clustering.unwrap_or(false);

        let resp = self
            .client
            .clone()
            .manual_compaction(ManualCompactionRequest {
                collection_id: collection.id,
                timetravel: 0,
                major_compaction,
                collection_name: collection.name,
                db_name: "".to_string(),
                partition_id: 0,
                segment_ids: vec![],
                channel: "".to_string(),
                l0_compaction: false,
                target_size: 0,
            })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok(resp.into())
    }

    pub async fn get_compaction_state(&self, compaction_id: i64) -> Result<CompactionState> {
        let resp = self
            .client
            .clone()
            .get_compaction_state(GetCompactionStateRequest { compaction_id })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok(resp.into())
    }

    /// Test text analyzers. Returns tokenized results.
    /// Requires Milvus 2.6+.
    pub async fn run_analyzer(
        &self,
        texts: Vec<String>,
        analyzer_params: &str,
    ) -> Result<Vec<proto::milvus::AnalyzerResult>> {
        let resp = self
            .client
            .clone()
            .run_analyzer(proto::milvus::RunAnalyzerRequest {
                base: Some(MsgBase::new(MsgType::RunAnalyzer)),
                analyzer_params: analyzer_params.to_string(),
                placeholder: texts.into_iter().map(|t| t.into_bytes()).collect(),
                with_detail: true,
                with_hash: false,
                db_name: "".to_string(),
                collection_name: "".to_string(),
                field_name: "".to_string(),
                analyzer_names: vec![],
            })
            .await?
            .into_inner();
        status_to_result(&resp.status)?;
        Ok(resp.results)
    }
}
