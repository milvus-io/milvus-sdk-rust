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

//! Row-level security (RLS) request objects.
//!
//! Row policies attach predicate filters to a collection for specific operations, and principal
//! tags carry the per-user attribute values those predicates reference.

use super::validation::{non_empty_strings, required, required_slice};
use crate::proto::milvus;
use crate::v2::error::{Error, Result};
use crate::v2::types::{RowPolicyAction, RowPolicyType};
use serde_json::Value;
use std::collections::HashSet;

fn validate_policy_type(value: RowPolicyType) -> Result<()> {
    if matches!(value, RowPolicyType::Unknown) {
        return Err(Error::validation(
            "policy_type".into(),
            "must be permissive or restrictive".into(),
        ));
    }
    Ok(())
}

fn validate_policy_definition(
    using_expr: &str,
    check_expr: &str,
    actions: &[RowPolicyAction],
) -> Result<()> {
    if using_expr.is_empty() && check_expr.is_empty() {
        return Err(Error::validation(
            "using_expr".into(),
            "at least one of using_expr or check_expr is required".into(),
        ));
    }
    let mut seen = HashSet::new();
    for action in actions {
        if !seen.insert(action) {
            return Err(Error::validation(
                "actions".into(),
                "must not contain duplicate actions".into(),
            ));
        }
    }
    Ok(())
}

fn validate_rls_tags(tags: &Value) -> Result<()> {
    let Some(object) = tags.as_object() else {
        return Err(Error::validation(
            "tags".into(),
            "must be a JSON object".into(),
        ));
    };
    if object.is_empty() {
        return Err(Error::validation("tags".into(), "must not be empty".into()));
    }
    for (key, value) in object {
        let Some(values) = (match value {
            Value::Array(values) if !values.is_empty() => Some(values.as_slice()),
            Value::Array(_) => None,
            other => Some(std::slice::from_ref(other)),
        }) else {
            continue;
        };
        for item in values {
            if item.is_boolean() || !(item.is_string() || item.is_number()) {
                return Err(Error::validation(
                    format!("tags[{key}]"),
                    "values must be strings, numbers, or arrays of them".into(),
                ));
            }
            if let Some(number) = item.as_f64() {
                if !number.is_finite() {
                    return Err(Error::validation(
                        format!("tags[{key}]"),
                        "values must be finite numbers".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
// CreateRowPolicyRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 create_row_policy operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct CreateRowPolicyRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) policy_name: String,
    pub(crate) policy_type: RowPolicyType,
    pub(crate) actions: Vec<RowPolicyAction>,
    pub(crate) using_expr: String,
    pub(crate) check_expr: String,
    pub(crate) description: String,
}

impl CreateRowPolicyRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            policy_name: Default::default(),
            policy_type: RowPolicyType::Permissive,
            actions: Default::default(),
            using_expr: Default::default(),
            check_expr: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> CreateRowPolicyRequestBuilder {
        CreateRowPolicyRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> CreateRowPolicyRequestBuilder {
        CreateRowPolicyRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the policy name.
    pub fn policy_name(&self) -> &str {
        &self.policy_name
    }

    /// Returns the policy type.
    pub fn policy_type(&self) -> RowPolicyType {
        self.policy_type
    }

    /// Returns the actions.
    pub fn actions(&self) -> &[RowPolicyAction] {
        &self.actions
    }

    /// Returns the using expression.
    pub fn using_expr(&self) -> &str {
        &self.using_expr
    }

    /// Returns the check expression.
    pub fn check_expr(&self) -> &str {
        &self.check_expr
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    #[allow(deprecated)]
    pub(crate) fn into_proto(self, default_db: &str) -> milvus::CreateRowPolicyRequest {
        milvus::CreateRowPolicyRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            policy_name: self.policy_name,
            actions: self
                .actions
                .into_iter()
                .map(RowPolicyAction::into_proto)
                .collect(),
            roles: Vec::new(),
            using_expr: self.using_expr,
            check_expr: self.check_expr,
            description: self.description,
            policy_type: Some(self.policy_type.into_proto()),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// CreateRowPolicyRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for CreateRowPolicyRequest.
#[derive(Debug, Clone)]
pub struct CreateRowPolicyRequestBuilder {
    value: CreateRowPolicyRequest,
}

impl CreateRowPolicyRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the policy name and returns the updated value.
    pub fn policy_name(mut self, value: impl Into<String>) -> Self {
        self.value.policy_name = value.into();
        self
    }

    /// Sets the policy type and returns the updated value.
    pub fn policy_type(mut self, value: RowPolicyType) -> Self {
        self.value.policy_type = value;
        self
    }

    /// Sets the actions and returns the updated value.
    pub fn actions(mut self, values: impl IntoIterator<Item = RowPolicyAction>) -> Self {
        self.value.actions = values.into_iter().collect();
        self
    }

    /// Sets the using expression and returns the updated value.
    pub fn using_expr(mut self, value: impl Into<String>) -> Self {
        self.value.using_expr = value.into();
        self
    }

    /// Sets the check expression and returns the updated value.
    pub fn check_expr(mut self, value: impl Into<String>) -> Self {
        self.value.check_expr = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `policy_name` is empty
    /// - `actions` is empty
    /// - `policy_type` is [`RowPolicyType::Unknown`]
    /// - both `using_expr` and `check_expr` are empty
    /// - `actions` contains duplicates
    pub fn build(self) -> Result<CreateRowPolicyRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("policy_name", &self.value.policy_name)?;
        required_slice("actions", &self.value.actions)?;
        validate_policy_type(self.value.policy_type)?;
        validate_policy_definition(
            &self.value.using_expr,
            &self.value.check_expr,
            &self.value.actions,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateRowPolicyRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 update_row_policy operation.
///
/// Replace the named policy definition, preserving its server-side identity. The policy must
/// exist. All definition fields replace their old values; omitted expressions and description
/// become empty strings. `policy_type` is required: because the server replaces every definition
/// field, an omitted type would silently reset the policy to permissive and could weaken a
/// restrictive policy.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct UpdateRowPolicyRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) policy_name: String,
    pub(crate) policy_type: RowPolicyType,
    pub(crate) actions: Vec<RowPolicyAction>,
    pub(crate) using_expr: String,
    pub(crate) check_expr: String,
    pub(crate) description: String,
}

impl UpdateRowPolicyRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            policy_name: Default::default(),
            policy_type: RowPolicyType::Unknown,
            actions: Default::default(),
            using_expr: Default::default(),
            check_expr: Default::default(),
            description: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> UpdateRowPolicyRequestBuilder {
        UpdateRowPolicyRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> UpdateRowPolicyRequestBuilder {
        UpdateRowPolicyRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the policy name.
    pub fn policy_name(&self) -> &str {
        &self.policy_name
    }

    /// Returns the policy type.
    pub fn policy_type(&self) -> RowPolicyType {
        self.policy_type
    }

    /// Returns the actions.
    pub fn actions(&self) -> &[RowPolicyAction] {
        &self.actions
    }

    /// Returns the using expression.
    pub fn using_expr(&self) -> &str {
        &self.using_expr
    }

    /// Returns the check expression.
    pub fn check_expr(&self) -> &str {
        &self.check_expr
    }

    /// Returns the description.
    pub fn description(&self) -> &str {
        &self.description
    }

    #[allow(deprecated)]
    pub(crate) fn into_proto(self, default_db: &str) -> milvus::UpdateRowPolicyRequest {
        milvus::UpdateRowPolicyRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            policy_name: self.policy_name,
            actions: self
                .actions
                .into_iter()
                .map(RowPolicyAction::into_proto)
                .collect(),
            roles: Vec::new(),
            using_expr: self.using_expr,
            check_expr: self.check_expr,
            description: self.description,
            policy_type: self.policy_type.into_proto(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UpdateRowPolicyRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for UpdateRowPolicyRequest.
#[derive(Debug, Clone)]
pub struct UpdateRowPolicyRequestBuilder {
    value: UpdateRowPolicyRequest,
}

impl UpdateRowPolicyRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the policy name and returns the updated value.
    pub fn policy_name(mut self, value: impl Into<String>) -> Self {
        self.value.policy_name = value.into();
        self
    }

    /// Sets the policy type and returns the updated value.
    pub fn policy_type(mut self, value: RowPolicyType) -> Self {
        self.value.policy_type = value;
        self
    }

    /// Sets the actions and returns the updated value.
    pub fn actions(mut self, values: impl IntoIterator<Item = RowPolicyAction>) -> Self {
        self.value.actions = values.into_iter().collect();
        self
    }

    /// Sets the using expression and returns the updated value.
    pub fn using_expr(mut self, value: impl Into<String>) -> Self {
        self.value.using_expr = value.into();
        self
    }

    /// Sets the check expression and returns the updated value.
    pub fn check_expr(mut self, value: impl Into<String>) -> Self {
        self.value.check_expr = value.into();
        self
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.value.description = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `policy_name` is empty
    /// - `actions` is empty
    /// - `policy_type` is [`RowPolicyType::Unknown`]
    /// - both `using_expr` and `check_expr` are empty
    /// - `actions` contains duplicates
    pub fn build(self) -> Result<UpdateRowPolicyRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("policy_name", &self.value.policy_name)?;
        required_slice("actions", &self.value.actions)?;
        validate_policy_type(self.value.policy_type)?;
        validate_policy_definition(
            &self.value.using_expr,
            &self.value.check_expr,
            &self.value.actions,
        )?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropRowPolicyRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 drop_row_policy operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DropRowPolicyRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) policy_name: String,
}

impl DropRowPolicyRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            policy_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DropRowPolicyRequestBuilder {
        DropRowPolicyRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DropRowPolicyRequestBuilder {
        DropRowPolicyRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the policy name.
    pub fn policy_name(&self) -> &str {
        &self.policy_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DropRowPolicyRequest {
        milvus::DropRowPolicyRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            policy_name: self.policy_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DropRowPolicyRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DropRowPolicyRequest.
#[derive(Debug, Clone)]
pub struct DropRowPolicyRequestBuilder {
    value: DropRowPolicyRequest,
}

impl DropRowPolicyRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the policy name and returns the updated value.
    pub fn policy_name(mut self, value: impl Into<String>) -> Self {
        self.value.policy_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `policy_name` is empty
    pub fn build(self) -> Result<DropRowPolicyRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("policy_name", &self.value.policy_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRowPoliciesRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_row_policies operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ListRowPoliciesRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListRowPoliciesRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListRowPoliciesRequestBuilder {
        ListRowPoliciesRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListRowPoliciesRequestBuilder {
        ListRowPoliciesRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ListRowPoliciesRequest {
        milvus::ListRowPoliciesRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRowPoliciesRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRowPoliciesRequest.
#[derive(Debug, Clone)]
pub struct ListRowPoliciesRequestBuilder {
    value: ListRowPoliciesRequest,
}

impl ListRowPoliciesRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    pub fn build(self) -> Result<ListRowPoliciesRequest> {
        required("collection_name", &self.value.collection_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// SetRLSPrincipalTagsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 set_rls_principal_tags operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct SetRLSPrincipalTagsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) principal_name: String,
    pub(crate) tags: Value,
}

impl SetRLSPrincipalTagsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            principal_name: Default::default(),
            tags: Value::Object(Default::default()),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> SetRLSPrincipalTagsRequestBuilder {
        SetRLSPrincipalTagsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> SetRLSPrincipalTagsRequestBuilder {
        SetRLSPrincipalTagsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the principal name.
    pub fn principal_name(&self) -> &str {
        &self.principal_name
    }

    /// Returns the tags.
    pub fn tags(&self) -> &Value {
        &self.tags
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::SetRlsPrincipalTagsRequest {
        milvus::SetRlsPrincipalTagsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            principal_name: self.principal_name,
            tags: self.tags.to_string(),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SetRLSPrincipalTagsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for SetRLSPrincipalTagsRequest.
#[derive(Debug, Clone)]
pub struct SetRLSPrincipalTagsRequestBuilder {
    value: SetRLSPrincipalTagsRequest,
}

impl SetRLSPrincipalTagsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the principal name and returns the updated value.
    pub fn principal_name(mut self, value: impl Into<String>) -> Self {
        self.value.principal_name = value.into();
        self
    }

    /// Sets the tags and returns the updated value.
    pub fn tags(mut self, value: Value) -> Self {
        self.value.tags = value;
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `principal_name` is empty
    /// - `tags` is not a non-empty JSON object, or a tag value is not a string, number, or
    ///   array of them
    pub fn build(self) -> Result<SetRLSPrincipalTagsRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("principal_name", &self.value.principal_name)?;
        validate_rls_tags(&self.value.tags)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRLSPrincipalTagsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 get_rls_principal_tags operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GetRLSPrincipalTagsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) principal_name: String,
}

impl GetRLSPrincipalTagsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            principal_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> GetRLSPrincipalTagsRequestBuilder {
        GetRLSPrincipalTagsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> GetRLSPrincipalTagsRequestBuilder {
        GetRLSPrincipalTagsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the principal name.
    pub fn principal_name(&self) -> &str {
        &self.principal_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::GetRlsPrincipalTagsRequest {
        milvus::GetRlsPrincipalTagsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            principal_name: self.principal_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRLSPrincipalTagsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRLSPrincipalTagsRequest.
#[derive(Debug, Clone)]
pub struct GetRLSPrincipalTagsRequestBuilder {
    value: GetRLSPrincipalTagsRequest,
}

impl GetRLSPrincipalTagsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the principal name and returns the updated value.
    pub fn principal_name(mut self, value: impl Into<String>) -> Self {
        self.value.principal_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `principal_name` is empty
    pub fn build(self) -> Result<GetRLSPrincipalTagsRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("principal_name", &self.value.principal_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRLSPrincipalsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 list_rls_principals operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ListRLSPrincipalsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
}

impl ListRLSPrincipalsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> ListRLSPrincipalsRequestBuilder {
        ListRLSPrincipalsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> ListRLSPrincipalsRequestBuilder {
        ListRLSPrincipalsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::ListRlsPrincipalsRequest {
        milvus::ListRlsPrincipalsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRLSPrincipalsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRLSPrincipalsRequest.
#[derive(Debug, Clone)]
pub struct ListRLSPrincipalsRequestBuilder {
    value: ListRLSPrincipalsRequest,
}

impl ListRLSPrincipalsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    pub fn build(self) -> Result<ListRLSPrincipalsRequest> {
        required("collection_name", &self.value.collection_name)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// DeleteRLSPrincipalTagsRequest
///////////////////////////////////////////////////////////////////////////////
/// Parameters for the ClientV2 delete_rls_principal_tags operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DeleteRLSPrincipalTagsRequest {
    pub(crate) database_name: Option<String>,
    pub(crate) collection_name: String,
    pub(crate) principal_name: String,
    pub(crate) tag_keys: Vec<String>,
}

impl DeleteRLSPrincipalTagsRequest {
    fn empty() -> Self {
        Self {
            database_name: Default::default(),
            collection_name: Default::default(),
            principal_name: Default::default(),
            tag_keys: Default::default(),
        }
    }

    /// Creates a builder for this request.
    pub fn builder() -> DeleteRLSPrincipalTagsRequestBuilder {
        DeleteRLSPrincipalTagsRequestBuilder {
            value: Self::empty(),
        }
    }

    /// Converts this request back into a builder while preserving its current values.
    pub fn into_builder(self) -> DeleteRLSPrincipalTagsRequestBuilder {
        DeleteRLSPrincipalTagsRequestBuilder { value: self }
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &Option<String> {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Returns the principal name.
    pub fn principal_name(&self) -> &str {
        &self.principal_name
    }

    /// Returns the tag keys; an empty list deletes all tags.
    pub fn tag_keys(&self) -> &[String] {
        &self.tag_keys
    }

    pub(crate) fn into_proto(self, default_db: &str) -> milvus::DeleteRlsPrincipalTagsRequest {
        milvus::DeleteRlsPrincipalTagsRequest {
            base: None,
            db_name: self.database_name.unwrap_or_else(|| default_db.to_owned()),
            collection_name: self.collection_name,
            principal_name: self.principal_name,
            tag_keys: self.tag_keys,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// DeleteRLSPrincipalTagsRequestBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for DeleteRLSPrincipalTagsRequest.
#[derive(Debug, Clone)]
pub struct DeleteRLSPrincipalTagsRequestBuilder {
    value: DeleteRLSPrincipalTagsRequest,
}

impl DeleteRLSPrincipalTagsRequestBuilder {
    /// Sets the database name and returns the updated value.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = Some(value.into());
        self
    }

    /// Sets the collection name and returns the updated value.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the principal name and returns the updated value.
    pub fn principal_name(mut self, value: impl Into<String>) -> Self {
        self.value.principal_name = value.into();
        self
    }

    /// Sets the tag keys and returns the updated value.
    pub fn tag_keys(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.tag_keys = values.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the configured values and builds the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::v2::error::Error::Validation`] when:
    /// - `collection_name` is empty
    /// - `principal_name` is empty
    /// - `tag_keys` contains an empty entry
    pub fn build(self) -> Result<DeleteRLSPrincipalTagsRequest> {
        required("collection_name", &self.value.collection_name)?;
        required("principal_name", &self.value.principal_name)?;
        non_empty_strings("tag_keys", &self.value.tag_keys)?;
        Ok(self.value)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::types::RowPolicyAction;

    #[test]
    fn create_row_policy_request_default_values() {
        let value = CreateRowPolicyRequest::empty();
        assert_eq!(value.database_name(), &None);
        assert!(value.collection_name().is_empty());
        assert!(value.policy_name().is_empty());
        assert_eq!(value.policy_type(), RowPolicyType::Permissive);
        assert!(value.actions().is_empty());
        assert!(value.using_expr().is_empty());
        assert!(value.check_expr().is_empty());
        assert!(value.description().is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn create_row_policy_request_populated_values() {
        let value = CreateRowPolicyRequest::builder()
            .database_name("books_db")
            .collection_name("books")
            .policy_name("region_filter")
            .policy_type(RowPolicyType::Restrictive)
            .actions([RowPolicyAction::Query, RowPolicyAction::Search])
            .using_expr("region == 'west'")
            .check_expr("region == 'west'")
            .description("restrict reads to west")
            .build()
            .expect("valid request");
        assert_eq!(value.database_name().as_deref(), Some("books_db"));
        assert_eq!(value.collection_name(), "books");
        assert_eq!(value.policy_name(), "region_filter");
        assert_eq!(value.policy_type(), RowPolicyType::Restrictive);
        assert_eq!(
            value.actions(),
            [RowPolicyAction::Query, RowPolicyAction::Search]
        );
        assert_eq!(value.using_expr(), "region == 'west'");
        assert_eq!(value.check_expr(), "region == 'west'");
        assert_eq!(value.description(), "restrict reads to west");

        let proto = value.into_proto("default");
        assert_eq!(proto.db_name, "books_db");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.policy_name, "region_filter");
        assert_eq!(
            proto.policy_type,
            Some(RowPolicyType::Restrictive.into_proto())
        );
        assert_eq!(
            proto.actions,
            vec![
                RowPolicyAction::Query.into_proto(),
                RowPolicyAction::Search.into_proto()
            ]
        );
        assert!(proto.roles.is_empty());
    }

    #[test]
    fn create_row_policy_request_build_validation() {
        assert!(CreateRowPolicyRequest::builder().build().is_err());
        assert!(CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .build()
            .is_err());
        assert!(CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .actions([RowPolicyAction::Query])
            .policy_type(RowPolicyType::Unknown)
            .build()
            .is_err());
        assert!(CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .actions([RowPolicyAction::Query])
            .build()
            .is_err());
        assert!(CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .actions([RowPolicyAction::Query, RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .is_err());
        let accepted = CreateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .actions([RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .expect("defaults to permissive");
        assert_eq!(accepted.policy_type(), RowPolicyType::Permissive);
    }

    #[test]
    fn update_row_policy_request_requires_policy_type() {
        assert!(UpdateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .actions([RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .is_err());
        assert!(UpdateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("p")
            .policy_type(RowPolicyType::Restrictive)
            .actions([RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .is_ok());
    }

    #[test]
    fn update_row_policy_request_populated_values() {
        let value = UpdateRowPolicyRequest::builder()
            .collection_name("books")
            .policy_name("region_filter")
            .policy_type(RowPolicyType::Permissive)
            .actions([RowPolicyAction::Query])
            .using_expr("region == 'west'")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.db_name, "default");
        assert_eq!(proto.policy_name, "region_filter");
        assert_eq!(proto.policy_type, RowPolicyType::Permissive.into_proto());
    }

    #[test]
    fn drop_row_policy_request_populated_values() {
        let value = DropRowPolicyRequest::builder()
            .database_name("books_db")
            .collection_name("books")
            .policy_name("region_filter")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.db_name, "books_db");
        assert_eq!(proto.collection_name, "books");
        assert_eq!(proto.policy_name, "region_filter");
        assert!(DropRowPolicyRequest::builder().build().is_err());
    }

    #[test]
    fn list_row_policies_request_populated_values() {
        let value = ListRowPoliciesRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.db_name, "default");
        assert_eq!(proto.collection_name, "books");
        assert!(ListRowPoliciesRequest::builder().build().is_err());
    }

    #[test]
    fn set_rls_principal_tags_request_populated_values() {
        let value = SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({"region": "west", "level": [1, 2]}))
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.principal_name, "alice");
        assert_eq!(proto.tags, r#"{"level":[1,2],"region":"west"}"#);
        assert!(SetRLSPrincipalTagsRequest::builder().build().is_err());
    }

    #[test]
    fn set_rls_principal_tags_request_rejects_invalid_tags() {
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!([1, 2]))
            .build()
            .is_err());
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({}))
            .build()
            .is_err());
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({"admin": true}))
            .build()
            .is_err());
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({"region": "west", "ok": true}))
            .build()
            .is_err());
    }

    #[test]
    fn set_rls_principal_tags_request_accepts_empty_array_tags() {
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({"group": []}))
            .build()
            .is_ok());
        assert!(SetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tags(serde_json::json!({"region": "west", "group": []}))
            .build()
            .is_ok());
    }

    #[test]
    fn get_rls_principal_tags_request_populated_values() {
        let value = GetRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.principal_name, "alice");
        assert!(GetRLSPrincipalTagsRequest::builder().build().is_err());
    }

    #[test]
    fn list_rls_principals_request_populated_values() {
        let value = ListRLSPrincipalsRequest::builder()
            .collection_name("books")
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.collection_name, "books");
        assert!(ListRLSPrincipalsRequest::builder().build().is_err());
    }

    #[test]
    fn delete_rls_principal_tags_request_populated_values() {
        let value = DeleteRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tag_keys(["region"])
            .build()
            .expect("valid request");
        let proto = value.into_proto("default");
        assert_eq!(proto.principal_name, "alice");
        assert_eq!(proto.tag_keys, ["region"]);
        let value = DeleteRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .build()
            .expect("empty tag_keys deletes all tags");
        let proto = value.into_proto("default");
        assert!(proto.tag_keys.is_empty());
        assert!(DeleteRLSPrincipalTagsRequest::builder().build().is_err());
    }

    #[test]
    fn delete_rls_principal_tags_request_rejects_empty_tag_key() {
        assert!(DeleteRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tag_keys(["", "region"])
            .build()
            .is_err());
        assert!(DeleteRLSPrincipalTagsRequest::builder()
            .collection_name("books")
            .principal_name("alice")
            .tag_keys(["region"])
            .build()
            .is_ok());
    }
}
