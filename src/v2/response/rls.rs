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

//! Response types returned by row-level security (RLS) operations.

use crate::proto::milvus;
use crate::v2::error::Result;
use crate::v2::types::RowPolicy;
use serde_json::Value;

///////////////////////////////////////////////////////////////////////////////
// ListRowPoliciesResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_row_policies operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ListRowPoliciesResponse {
    pub(crate) policies: Vec<RowPolicy>,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
}

impl ListRowPoliciesResponse {
    #[cfg(test)]
    fn empty() -> Self {
        Self {
            policies: Vec::new(),
            database_name: String::new(),
            collection_name: String::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListRowPoliciesResponseBuilder {
        ListRowPoliciesResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the row policies.
    pub fn policies(&self) -> &[RowPolicy] {
        &self.policies
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn from_proto(value: milvus::ListRowPoliciesResponse) -> Result<Self> {
        let mut policies = Vec::with_capacity(value.policies.len());
        for policy in value.policies {
            policies.push(RowPolicy::from_proto(policy)?);
        }
        Ok(Self {
            policies,
            database_name: value.db_name,
            collection_name: value.collection_name,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRowPoliciesResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRowPoliciesResponse.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct ListRowPoliciesResponseBuilder {
    value: ListRowPoliciesResponse,
}

#[cfg(test)]
impl ListRowPoliciesResponseBuilder {
    /// Sets the policies and returns this value for further mutation.
    pub fn policies(mut self, values: impl IntoIterator<Item = RowPolicy>) -> Self {
        self.value.policies = values.into_iter().collect();
        self
    }

    /// Sets the database name and returns this value for further mutation.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Builds the response.
    pub fn build(self) -> ListRowPoliciesResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRLSPrincipalTagsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 get_rls_principal_tags operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct GetRLSPrincipalTagsResponse {
    pub(crate) tags: Value,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
    pub(crate) principal_name: String,
}

impl GetRLSPrincipalTagsResponse {
    #[cfg(test)]
    fn empty() -> Self {
        Self {
            tags: Value::Object(Default::default()),
            database_name: String::new(),
            collection_name: String::new(),
            principal_name: String::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> GetRLSPrincipalTagsResponseBuilder {
        GetRLSPrincipalTagsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the decoded principal tags.
    pub fn tags(&self) -> &Value {
        &self.tags
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
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

    pub(crate) fn from_proto(value: milvus::GetRlsPrincipalTagsResponse) -> Result<Self> {
        let tags = if value.tags.is_empty() {
            Value::Object(Default::default())
        } else {
            let decoded: Value = serde_json::from_str(&value.tags)?;
            if !decoded.is_object() {
                return Err(crate::v2::error::Error::conversion(
                    "principal tags must be a JSON object",
                ));
            }
            decoded
        };
        Ok(Self {
            tags,
            database_name: value.db_name,
            collection_name: value.collection_name,
            principal_name: value.principal_name,
        })
    }
}

///////////////////////////////////////////////////////////////////////////////
// GetRLSPrincipalTagsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for GetRLSPrincipalTagsResponse.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct GetRLSPrincipalTagsResponseBuilder {
    value: GetRLSPrincipalTagsResponse,
}

#[cfg(test)]
impl GetRLSPrincipalTagsResponseBuilder {
    /// Sets the tags and returns this value for further mutation.
    pub fn tags(mut self, value: Value) -> Self {
        self.value.tags = value;
        self
    }

    /// Sets the database name and returns this value for further mutation.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Sets the principal name and returns this value for further mutation.
    pub fn principal_name(mut self, value: impl Into<String>) -> Self {
        self.value.principal_name = value.into();
        self
    }

    /// Builds the response.
    pub fn build(self) -> GetRLSPrincipalTagsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRLSPrincipalsResponse
///////////////////////////////////////////////////////////////////////////////
/// Response returned by the ClientV2 list_rls_principals operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListRLSPrincipalsResponse {
    pub(crate) principal_names: Vec<String>,
    pub(crate) database_name: String,
    pub(crate) collection_name: String,
}

impl ListRLSPrincipalsResponse {
    #[cfg(test)]
    fn empty() -> Self {
        Self {
            principal_names: Vec::new(),
            database_name: String::new(),
            collection_name: String::new(),
        }
    }

    ///////////////////////////////////////////////////////////////////////////////
    // Test Cases
    ///////////////////////////////////////////////////////////////////////////////

    #[cfg(test)]
    pub(crate) fn builder() -> ListRLSPrincipalsResponseBuilder {
        ListRLSPrincipalsResponseBuilder {
            value: Self::empty(),
        }
    }

    /// Returns the principal names.
    pub fn principal_names(&self) -> &[String] {
        &self.principal_names
    }

    /// Returns the database name.
    pub fn database_name(&self) -> &str {
        &self.database_name
    }

    /// Returns the collection name.
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub(crate) fn from_proto(value: milvus::ListRlsPrincipalsResponse) -> Self {
        Self {
            principal_names: value.principal_names,
            database_name: value.db_name,
            collection_name: value.collection_name,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ListRLSPrincipalsResponseBuilder
///////////////////////////////////////////////////////////////////////////////
/// Builder for ListRLSPrincipalsResponse.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct ListRLSPrincipalsResponseBuilder {
    value: ListRLSPrincipalsResponse,
}

#[cfg(test)]
impl ListRLSPrincipalsResponseBuilder {
    /// Sets the principal names and returns this value for further mutation.
    pub fn principal_names(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.value.principal_names = values.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the database name and returns this value for further mutation.
    pub fn database_name(mut self, value: impl Into<String>) -> Self {
        self.value.database_name = value.into();
        self
    }

    /// Sets the collection name and returns this value for further mutation.
    pub fn collection_name(mut self, value: impl Into<String>) -> Self {
        self.value.collection_name = value.into();
        self
    }

    /// Builds the response.
    pub fn build(self) -> ListRLSPrincipalsResponse {
        self.value
    }
}

///////////////////////////////////////////////////////////////////////////////
// Test Cases
///////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::types::{RowPolicyAction, RowPolicyType};

    #[test]
    #[allow(deprecated)]
    fn list_row_policies_response_decode() {
        let response = ListRowPoliciesResponse::from_proto(milvus::ListRowPoliciesResponse {
            status: None,
            db_name: "books_db".into(),
            collection_name: "books".into(),
            policies: vec![milvus::RowPolicy {
                policy_name: "region_filter".into(),
                actions: vec![
                    crate::proto::milvus::RowPolicyAction::Query as i32,
                    crate::proto::milvus::RowPolicyAction::Search as i32,
                ],
                roles: vec![],
                using_expr: "region == 'west'".into(),
                check_expr: "".into(),
                description: "west only".into(),
                created_at: 42,
                policy_type: crate::proto::milvus::RowPolicyType::Restrictive as i32,
            }],
        })
        .expect("decode policy");
        assert_eq!(response.database_name(), "books_db");
        assert_eq!(response.collection_name(), "books");
        assert_eq!(response.policies().len(), 1);
        let policy = &response.policies()[0];
        assert_eq!(policy.get_policy_name(), "region_filter");
        assert_eq!(
            policy.get_actions(),
            [RowPolicyAction::Query, RowPolicyAction::Search]
        );
        assert_eq!(policy.get_using_expr(), "region == 'west'");
        assert_eq!(policy.get_description(), "west only");
        assert_eq!(policy.get_created_at(), 42);
        assert_eq!(policy.get_policy_type(), RowPolicyType::Restrictive);
    }

    #[test]
    #[allow(deprecated)]
    fn list_row_policies_response_rejects_unknown_action() {
        let result = ListRowPoliciesResponse::from_proto(milvus::ListRowPoliciesResponse {
            status: None,
            db_name: "default".into(),
            collection_name: "books".into(),
            policies: vec![milvus::RowPolicy {
                policy_name: "p".into(),
                actions: vec![99],
                roles: vec![],
                using_expr: "".into(),
                check_expr: "".into(),
                description: "".into(),
                created_at: 0,
                policy_type: 0,
            }],
        });
        assert!(result.is_err());
    }

    #[test]
    fn get_rls_principal_tags_response_decode() {
        let response =
            GetRLSPrincipalTagsResponse::from_proto(milvus::GetRlsPrincipalTagsResponse {
                status: None,
                tags: r#"{"region":"west","level":[1,2]}"#.into(),
                db_name: "default".into(),
                collection_name: "books".into(),
                principal_name: "alice".into(),
            })
            .expect("decode tags");
        assert_eq!(
            response.tags(),
            &serde_json::json!({"region": "west", "level": [1, 2]})
        );
        assert_eq!(response.principal_name(), "alice");
        assert!(
            GetRLSPrincipalTagsResponse::from_proto(milvus::GetRlsPrincipalTagsResponse {
                status: None,
                tags: "not json".into(),
                db_name: "default".into(),
                collection_name: "books".into(),
                principal_name: "alice".into(),
            },)
            .is_err()
        );
        assert!(
            GetRLSPrincipalTagsResponse::from_proto(milvus::GetRlsPrincipalTagsResponse {
                status: None,
                tags: "[1, 2]".into(),
                db_name: "default".into(),
                collection_name: "books".into(),
                principal_name: "alice".into(),
            },)
            .is_err()
        );
    }

    #[test]
    fn list_rls_principals_response_decode() {
        let response = ListRLSPrincipalsResponse::from_proto(milvus::ListRlsPrincipalsResponse {
            status: None,
            principal_names: vec!["alice".into(), "bob".into()],
            db_name: "default".into(),
            collection_name: "books".into(),
        });
        assert_eq!(response.principal_names(), ["alice", "bob"]);
        assert_eq!(response.database_name(), "default");
        assert_eq!(response.collection_name(), "books");
    }

    #[test]
    fn response_builders_round_trip() {
        let policies = ListRowPoliciesResponse::builder()
            .policies([RowPolicy::new().policy_name("p")])
            .database_name("default")
            .collection_name("books")
            .build();
        assert_eq!(policies.database_name(), "default");
        assert_eq!(policies.collection_name(), "books");
        assert_eq!(policies.policies()[0].get_policy_name(), "p");

        let tags = GetRLSPrincipalTagsResponse::builder()
            .tags(serde_json::json!({"region": "west"}))
            .database_name("default")
            .collection_name("books")
            .principal_name("alice")
            .build();
        assert_eq!(tags.database_name(), "default");
        assert_eq!(tags.collection_name(), "books");
        assert_eq!(tags.principal_name(), "alice");
        assert_eq!(tags.tags(), &serde_json::json!({"region": "west"}));

        let principals = ListRLSPrincipalsResponse::builder()
            .principal_names(["alice"])
            .database_name("default")
            .collection_name("books")
            .build();
        assert_eq!(principals.database_name(), "default");
        assert_eq!(principals.collection_name(), "books");
        assert_eq!(principals.principal_names(), ["alice"]);
    }
}
