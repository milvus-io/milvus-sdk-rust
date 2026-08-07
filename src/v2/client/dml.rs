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

//! ClientV2 data-manipulation operations.

use super::{ClientV2, RetrySemantics};
use crate::proto::{common, milvus};
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Insert data into a collection.You can input column-based data or row-based data.
    pub async fn insert(
        &self,
        request: request::dml::InsertRequest,
    ) -> Result<response::dml::InsertResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection_name = request.collection_name.clone();
        for attempt in 0..2 {
            let resolved = self
                .resolve_data(
                    &database,
                    &collection_name,
                    &request.columns,
                    &request.rows,
                    false,
                    false,
                    &[],
                )
                .await?;
            let response = self
                .retry_rpc(
                    || {
                        let fields = resolved.to_proto_fields(
                            &request.columns,
                            &request.rows,
                            false,
                            false,
                        )?;
                        request.to_proto_with_fields(
                            fields,
                            resolved.row_count,
                            resolved.schema_timestamp,
                            &database,
                        )
                    },
                    RetrySemantics::NonIdempotent,
                    |mut service, request| async move { service.insert(request).await },
                    dml_retry_status,
                )
                .await?;
            if attempt == 0 && is_schema_mismatch(&response.status) {
                self.remove_collection_description(&database, &collection_name);
                continue;
            }
            status_to_result(&response.status)?;
            self.update_dml_timestamp(
                &database,
                &resolved.canonical_collection_name,
                response.timestamp,
            );
            return Ok(response::dml::DmlResponse::from_proto(response));
        }
        unreachable!("insert schema-mismatch retry loop always returns")
    }

    /// Upsert entities of a collection.You can input column-based data or row-based data.
    pub async fn upsert(
        &self,
        request: request::dml::UpsertRequest,
    ) -> Result<response::dml::UpsertResponse> {
        let partial_update = request.is_partial_update();
        let field_ops = request.field_ops;
        let request = request.insert;
        let database = self.effective_database(request.database_name.as_deref());
        let collection_name = request.collection_name.clone();
        for attempt in 0..2 {
            let resolved = self
                .resolve_data(
                    &database,
                    &collection_name,
                    &request.columns,
                    &request.rows,
                    true,
                    partial_update,
                    &field_ops,
                )
                .await?;
            let response = self
                .retry_rpc(
                    || {
                        let fields = resolved.to_proto_fields(
                            &request.columns,
                            &request.rows,
                            true,
                            partial_update,
                        )?;
                        let insert = request.to_proto_with_fields(
                            fields,
                            resolved.row_count,
                            resolved.schema_timestamp,
                            &database,
                        )?;
                        Ok(milvus::UpsertRequest {
                            base: insert.base,
                            db_name: insert.db_name,
                            collection_name: insert.collection_name,
                            partition_name: insert.partition_name,
                            fields_data: insert.fields_data,
                            hash_keys: insert.hash_keys,
                            num_rows: insert.num_rows,
                            schema_timestamp: insert.schema_timestamp,
                            partial_update,
                            namespace: None,
                            field_ops: field_ops
                                .iter()
                                .cloned()
                                .map(crate::v2::types::FieldPartialUpdateOp::into_proto)
                                .collect(),
                        })
                    },
                    RetrySemantics::NonIdempotent,
                    |mut service, request| async move { service.upsert(request).await },
                    dml_retry_status,
                )
                .await?;
            if attempt == 0 && is_schema_mismatch(&response.status) {
                self.remove_collection_description(&database, &collection_name);
                continue;
            }
            status_to_result(&response.status)?;
            self.update_dml_timestamp(
                &database,
                &resolved.canonical_collection_name,
                response.timestamp,
            );
            return Ok(response::dml::DmlResponse::from_proto(response));
        }
        unreachable!("upsert schema-mismatch retry loop always returns")
    }

    /// Delete entities by filtering expression or ID array.
    pub async fn delete(
        &self,
        request: request::dml::DeleteRequest,
    ) -> Result<response::dml::DeleteResponse> {
        let database = self.effective_database(request.database_name.as_deref());
        let collection_name = request.collection_name.clone();
        let description = self
            .get_collection_description(&database, &collection_name)
            .await?;
        let canonical_collection_name = if description.collection_name.is_empty() {
            collection_name.clone()
        } else {
            description.collection_name.clone()
        };
        let primary_field_name = if request.has_ids() {
            Some(
                description
                    .schema
                    .as_ref()
                    .and_then(|schema| schema.fields.iter().find(|field| field.is_primary_key))
                    .map(|field| field.name.clone())
                    .ok_or_else(|| {
                        crate::v2::error::Error::MalformedResponse(
                            "collection schema has no primary key".into(),
                        )
                    })?,
            )
        } else {
            None
        };
        let raw = request.into_proto(&database, primary_field_name.as_deref())?;
        let response = self
            .retry_rpc(
                || Ok(raw.clone()),
                RetrySemantics::NonIdempotent,
                |mut service, request| async move { service.delete(request).await },
                |response| response.status.clone(),
            )
            .await?;
        status_to_result(&response.status)?;
        self.update_dml_timestamp(&database, &canonical_collection_name, response.timestamp);
        Ok(response::dml::DmlResponse::from_proto(response))
    }
}

#[allow(deprecated)]
fn is_schema_mismatch(status: &Option<common::Status>) -> bool {
    status
        .as_ref()
        .is_some_and(|status| status.error_code == common::ErrorCode::SchemaMismatch as i32)
}

#[allow(deprecated)]
fn dml_retry_status(response: &milvus::MutationResult) -> Option<common::Status> {
    let mut status = response.status.clone()?;
    if status.error_code == common::ErrorCode::SchemaMismatch as i32 {
        status.error_code = common::ErrorCode::Success as i32;
        status.code = 0;
    }
    Some(status)
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::{dml_retry_status, is_schema_mismatch};
    use crate::proto::{common, milvus};

    #[test]
    #[allow(deprecated)]
    fn detects_legacy_schema_mismatch_status() {
        let status = Some(common::Status {
            error_code: common::ErrorCode::SchemaMismatch as i32,
            ..Default::default()
        });
        assert!(is_schema_mismatch(&status));
        assert!(!is_schema_mismatch(&None));

        let accepted = dml_retry_status(&milvus::MutationResult {
            status,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(accepted.error_code, common::ErrorCode::Success as i32);
        assert_eq!(accepted.code, 0);
    }
}
