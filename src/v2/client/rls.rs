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

//! ClientV2 row-level security (RLS) operations.

use super::ClientV2;
use crate::v2::error::status_to_result;
use crate::v2::error::Result;
use crate::v2::{request, response};

impl ClientV2 {
    /// Creates a row-level security policy on a collection.
    ///
    /// Creating a policy with an existing name is an error. The caller must hold the
    /// `ManageRLS` privilege on the collection.
    pub async fn create_row_policy(
        &self,
        request: request::rls::CreateRowPolicyRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            create_row_policy,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Replaces the named policy definition, preserving its server-side identity.
    ///
    /// The policy must exist. All definition fields replace their old values; omitted
    /// expressions and description become empty strings. The caller must hold the `ManageRLS`
    /// privilege on the collection.
    pub async fn update_row_policy(
        &self,
        request: request::rls::UpdateRowPolicyRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            update_row_policy,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Drops a row-level security policy by name.
    ///
    /// Dropping a missing policy succeeds. The caller must hold the `ManageRLS` privilege on
    /// the collection.
    pub async fn drop_row_policy(&self, request: request::rls::DropRowPolicyRequest) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            drop_row_policy,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Lists the row-level security policies attached to a collection.
    ///
    /// The caller must hold the `ViewRLS` privilege on the collection.
    pub async fn list_row_policies(
        &self,
        request: request::rls::ListRowPoliciesRequest,
    ) -> Result<response::rls::ListRowPoliciesResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, list_row_policies, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        response::rls::ListRowPoliciesResponse::from_proto(response)
    }

    /// Incrementally upserts a non-empty mapping of principal tags on a collection.
    ///
    /// Values may be strings, finite numbers, or one-dimensional arrays of strings or numbers.
    /// Keys omitted from this call retain their values. The caller must hold the `ManageRLS`
    /// privilege on the collection.
    pub async fn set_rls_principal_tags(
        &self,
        request: request::rls::SetRLSPrincipalTagsRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            set_rls_principal_tags,
            request.into_proto(&database)
        )?;
        self.status(status)
    }

    /// Returns the stored tags of a principal as decoded JSON.
    ///
    /// The caller must hold the `ViewRLS` privilege on the collection.
    pub async fn get_rls_principal_tags(
        &self,
        request: request::rls::GetRLSPrincipalTagsRequest,
    ) -> Result<response::rls::GetRLSPrincipalTagsResponse> {
        let database = self.current_database();
        let response =
            rpc_with_retry!(self, get_rls_principal_tags, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        response::rls::GetRLSPrincipalTagsResponse::from_proto(response)
    }

    /// Lists the application principal names with stored tags in a collection.
    ///
    /// The caller must hold the `ViewRLS` privilege on the collection.
    pub async fn list_rls_principals(
        &self,
        request: request::rls::ListRLSPrincipalsRequest,
    ) -> Result<response::rls::ListRLSPrincipalsResponse> {
        let database = self.current_database();
        let response = rpc_with_retry!(self, list_rls_principals, request.into_proto(&database))?;
        status_to_result(&response.status)?;
        Ok(response::rls::ListRLSPrincipalsResponse::from_proto(
            response,
        ))
    }

    /// Deletes selected tag keys of a principal, or all tags when `tag_keys` is empty.
    ///
    /// The caller must hold the `ManageRLS` privilege on the collection.
    pub async fn delete_rls_principal_tags(
        &self,
        request: request::rls::DeleteRLSPrincipalTagsRequest,
    ) -> Result<()> {
        let database = self.current_database();
        let status = status_rpc_with_retry!(
            NonIdempotent,
            self,
            delete_rls_principal_tags,
            request.into_proto(&database)
        )?;
        self.status(status)
    }
}
