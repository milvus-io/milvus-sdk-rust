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

//! Row-level security (RLS) domain types: row policies and principal tags.

use crate::proto::milvus;
use crate::v2::error::{Error, Result};

///////////////////////////////////////////////////////////////////////////////
// RowPolicyAction
///////////////////////////////////////////////////////////////////////////////
/// An operation that a row policy applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RowPolicyAction {
    /// Applies to query operations.
    Query,
    /// Applies to search operations.
    Search,
    /// Applies to insert operations.
    Insert,
    /// Applies to delete operations.
    Delete,
    /// Applies to upsert operations.
    Upsert,
    /// Applies to query-iterator operations.
    QueryIterator,
    /// Applies to search-iterator operations.
    SearchIterator,
    /// Applies to hybrid-search operations.
    HybridSearch,
}

impl RowPolicyAction {
    pub(crate) fn from_proto(value: i32) -> Result<Self> {
        Ok(match milvus::RowPolicyAction::try_from(value) {
            Ok(milvus::RowPolicyAction::Query) => Self::Query,
            Ok(milvus::RowPolicyAction::Search) => Self::Search,
            Ok(milvus::RowPolicyAction::Insert) => Self::Insert,
            Ok(milvus::RowPolicyAction::Delete) => Self::Delete,
            Ok(milvus::RowPolicyAction::Upsert) => Self::Upsert,
            Ok(milvus::RowPolicyAction::QueryIterator) => Self::QueryIterator,
            Ok(milvus::RowPolicyAction::SearchIterator) => Self::SearchIterator,
            Ok(milvus::RowPolicyAction::HybridSearch) => Self::HybridSearch,
            _ => {
                return Err(Error::conversion(format!(
                    "unknown row policy action {value}"
                )))
            }
        })
    }

    pub(crate) fn into_proto(self) -> i32 {
        match self {
            Self::Query => milvus::RowPolicyAction::Query as i32,
            Self::Search => milvus::RowPolicyAction::Search as i32,
            Self::Insert => milvus::RowPolicyAction::Insert as i32,
            Self::Delete => milvus::RowPolicyAction::Delete as i32,
            Self::Upsert => milvus::RowPolicyAction::Upsert as i32,
            Self::QueryIterator => milvus::RowPolicyAction::QueryIterator as i32,
            Self::SearchIterator => milvus::RowPolicyAction::SearchIterator as i32,
            Self::HybridSearch => milvus::RowPolicyAction::HybridSearch as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RowPolicyType
///////////////////////////////////////////////////////////////////////////////
/// How a row policy combines with the collection's other policies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RowPolicyType {
    /// Unspecified or unrecognized policy type.
    #[default]
    Unknown,
    /// A policy whose predicates are combined with OR across policies.
    Permissive,
    /// A policy whose predicates are combined with AND across policies.
    Restrictive,
}

impl RowPolicyType {
    pub(crate) fn from_proto(value: i32) -> Self {
        match milvus::RowPolicyType::try_from(value) {
            Ok(milvus::RowPolicyType::Permissive) => Self::Permissive,
            Ok(milvus::RowPolicyType::Restrictive) => Self::Restrictive,
            _ => Self::Unknown,
        }
    }

    pub(crate) fn into_proto(self) -> i32 {
        match self {
            Self::Unknown => milvus::RowPolicyType::Unknown as i32,
            Self::Permissive => milvus::RowPolicyType::Permissive as i32,
            Self::Restrictive => milvus::RowPolicyType::Restrictive as i32,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RowPolicy
///////////////////////////////////////////////////////////////////////////////
/// A row-level security policy attached to a collection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RowPolicy {
    pub(crate) policy_name: String,
    pub(crate) actions: Vec<RowPolicyAction>,
    pub(crate) using_expr: String,
    pub(crate) check_expr: String,
    pub(crate) description: String,
    pub(crate) created_at: i64,
    pub(crate) policy_type: RowPolicyType,
}

impl RowPolicy {
    /// Creates a value initialized with its SDK defaults.
    pub fn new() -> Self {
        Self {
            policy_name: String::new(),
            actions: Vec::new(),
            using_expr: String::new(),
            check_expr: String::new(),
            description: String::new(),
            created_at: 0,
            policy_type: RowPolicyType::Unknown,
        }
    }

    /// Sets the policy name and returns the updated value.
    pub fn policy_name(mut self, value: impl Into<String>) -> Self {
        self.policy_name = value.into();
        self
    }

    /// Sets the policy name and returns this value for further mutation.
    pub fn set_policy_name(&mut self, value: impl Into<String>) -> &mut Self {
        self.policy_name = value.into();
        self
    }

    /// Returns the configured policy name.
    pub fn get_policy_name(&self) -> &str {
        &self.policy_name
    }

    /// Sets the actions and returns the updated value.
    pub fn actions(mut self, values: impl IntoIterator<Item = RowPolicyAction>) -> Self {
        self.actions = values.into_iter().collect();
        self
    }

    /// Sets the actions and returns this value for further mutation.
    pub fn set_actions(&mut self, values: impl IntoIterator<Item = RowPolicyAction>) -> &mut Self {
        self.actions = values.into_iter().collect();
        self
    }

    /// Returns the configured actions.
    pub fn get_actions(&self) -> &[RowPolicyAction] {
        &self.actions
    }

    /// Adds one action to the existing values.
    pub fn add_action(mut self, value: RowPolicyAction) -> Self {
        self.actions.push(value);
        self
    }

    /// Sets the using expression and returns the updated value.
    pub fn using_expr(mut self, value: impl Into<String>) -> Self {
        self.using_expr = value.into();
        self
    }

    /// Sets the using expression and returns this value for further mutation.
    pub fn set_using_expr(&mut self, value: impl Into<String>) -> &mut Self {
        self.using_expr = value.into();
        self
    }

    /// Returns the configured using expression.
    pub fn get_using_expr(&self) -> &str {
        &self.using_expr
    }

    /// Sets the check expression and returns the updated value.
    pub fn check_expr(mut self, value: impl Into<String>) -> Self {
        self.check_expr = value.into();
        self
    }

    /// Sets the check expression and returns this value for further mutation.
    pub fn set_check_expr(&mut self, value: impl Into<String>) -> &mut Self {
        self.check_expr = value.into();
        self
    }

    /// Returns the configured check expression.
    pub fn get_check_expr(&self) -> &str {
        &self.check_expr
    }

    /// Sets the description and returns the updated value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = value.into();
        self
    }

    /// Sets the description and returns this value for further mutation.
    pub fn set_description(&mut self, value: impl Into<String>) -> &mut Self {
        self.description = value.into();
        self
    }

    /// Returns the configured description.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Sets the creation timestamp and returns the updated value.
    ///
    /// Zero means the server did not report a creation time.
    pub fn created_at(mut self, value: i64) -> Self {
        self.created_at = value;
        self
    }

    /// Sets the creation timestamp and returns this value for further mutation.
    pub fn set_created_at(&mut self, value: i64) -> &mut Self {
        self.created_at = value;
        self
    }

    /// Returns the creation timestamp; zero means unavailable.
    pub fn get_created_at(&self) -> i64 {
        self.created_at
    }

    /// Sets the policy type and returns the updated value.
    pub fn policy_type(mut self, value: RowPolicyType) -> Self {
        self.policy_type = value;
        self
    }

    /// Sets the policy type and returns this value for further mutation.
    pub fn set_policy_type(&mut self, value: RowPolicyType) -> &mut Self {
        self.policy_type = value;
        self
    }

    /// Returns the configured policy type.
    pub fn get_policy_type(&self) -> RowPolicyType {
        self.policy_type
    }

    pub(crate) fn from_proto(value: milvus::RowPolicy) -> Result<Self> {
        let mut actions = Vec::with_capacity(value.actions.len());
        for action in value.actions {
            actions.push(RowPolicyAction::from_proto(action)?);
        }
        Ok(Self {
            policy_name: value.policy_name,
            actions,
            using_expr: value.using_expr,
            check_expr: value.check_expr,
            description: value.description,
            created_at: value.created_at,
            policy_type: RowPolicyType::from_proto(value.policy_type),
        })
    }
}
