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

//! Global-cluster topology types and discovery.
//!
//! A Milvus deployment may expose a single logical **global-cluster** endpoint that fronts
//! multiple physical clusters. The global endpoint serves a REST topology describing the member
//! clusters; the SDK connects to the writable primary and can fail over to a new primary when the
//! topology changes.

use crate::v2::error::{Error, Result};
use serde::Deserialize;

/// Substring that marks an endpoint URI as a global-cluster endpoint.
const GLOBAL_CLUSTER_MARKER: &str = "global-cluster";

/// Checks whether a URI points to a global-cluster endpoint.
pub(crate) fn is_global_endpoint(uri: &str) -> bool {
    !uri.is_empty() && uri.to_ascii_lowercase().contains(GLOBAL_CLUSTER_MARKER)
}

/// Capability bitset of a member cluster.
///////////////////////////////////////////////////////////////////////////////
// ClusterCapability
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClusterCapability(u8);

impl ClusterCapability {
    /// Bit for read-only access.
    #[cfg(test)]
    pub(crate) const READABLE: u8 = 0b01;
    /// Bit for write access.
    pub(crate) const WRITABLE: u8 = 0b10;

    /// Returns whether the cluster is readable.
    #[cfg(test)]
    pub(crate) fn is_readable(self) -> bool {
        self.0 & Self::READABLE != 0
    }

    /// Returns whether the cluster is writable.
    pub(crate) fn is_writable(self) -> bool {
        self.0 & Self::WRITABLE != 0
    }
}

/// A single member cluster in a global topology.
///////////////////////////////////////////////////////////////////////////////
// ClusterInfo
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClusterInfo {
    pub(crate) cluster_id: String,
    pub(crate) endpoint: String,
    pub(crate) capability: ClusterCapability,
}

impl ClusterInfo {
    /// Returns the cluster identifier.
    #[cfg(test)]
    pub(crate) fn cluster_id(&self) -> &str {
        &self.cluster_id
    }

    /// Returns the physical endpoint of this cluster.
    pub(crate) fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Returns the capability bitset of this cluster.
    #[cfg(test)]
    pub(crate) fn capability(&self) -> ClusterCapability {
        self.capability
    }

    /// Returns whether this cluster is the writable primary.
    pub(crate) fn is_primary(&self) -> bool {
        self.capability.is_writable()
    }
}

/// The global-cluster topology served by a global endpoint.
///////////////////////////////////////////////////////////////////////////////
// GlobalTopology
///////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GlobalTopology {
    pub(crate) version: i64,
    pub(crate) clusters: Vec<ClusterInfo>,
}

impl GlobalTopology {
    /// Returns the topology version.
    pub(crate) fn version(&self) -> i64 {
        self.version
    }

    /// Returns the member clusters.
    #[cfg(test)]
    pub(crate) fn clusters(&self) -> &[ClusterInfo] {
        &self.clusters
    }

    /// Returns the writable primary cluster.
    pub(crate) fn primary(&self) -> Result<&ClusterInfo> {
        self.clusters
            .iter()
            .find(|cluster| cluster.is_primary())
            .ok_or_else(|| Error::Unexpected("no primary cluster found in global topology".into()))
    }
}

/// Wire shape of a single cluster in the topology REST response.
///////////////////////////////////////////////////////////////////////////////
// ClusterInfoJson
///////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize)]
struct ClusterInfoJson {
    #[serde(rename = "clusterId")]
    cluster_id: String,
    endpoint: String,
    capability: u8,
}

/// Wire shape of the topology REST response data.
///////////////////////////////////////////////////////////////////////////////
// TopologyDataJson
///////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize)]
struct TopologyDataJson {
    version: i64,
    clusters: Vec<ClusterInfoJson>,
}

/// Wire shape of the topology REST response envelope.
///////////////////////////////////////////////////////////////////////////////
// TopologyResponseJson
///////////////////////////////////////////////////////////////////////////////
#[derive(Deserialize)]
struct TopologyResponseJson {
    code: i32,
    message: Option<String>,
    data: Option<TopologyDataJson>,
}

/// Parses a global-cluster topology REST response body.
pub(crate) fn parse_topology_response(body: &str) -> Result<GlobalTopology> {
    let response: TopologyResponseJson = serde_json::from_str(body).map_err(|error| {
        Error::MalformedResponse(format!(
            "global topology response is not valid JSON: {error}"
        ))
    })?;
    if response.code != 0 {
        return Err(Error::Unexpected(
            response
                .message
                .unwrap_or_else(|| "global topology request failed".to_owned()),
        ));
    }
    let data = response.data.ok_or_else(|| {
        Error::MalformedResponse("global topology response contains no data".into())
    })?;
    Ok(GlobalTopology {
        version: data.version,
        clusters: data
            .clusters
            .into_iter()
            .map(|cluster| ClusterInfo {
                cluster_id: cluster.cluster_id,
                endpoint: cluster.endpoint,
                capability: ClusterCapability(cluster.capability),
            })
            .collect(),
    })
}

/// Builds the global-cluster topology REST URL from a global endpoint URI.
///
/// A scheme-less endpoint defaults to `https` when TLS is enabled and `http` otherwise, matching
/// the scheme [`cluster_endpoint_uri`](crate::v2::client::global_cluster::cluster_endpoint_uri)
/// derives for member endpoints so a plaintext global endpoint is not sent a TLS handshake. An
/// explicit `http://` endpoint is upgraded to `https://` when TLS is enabled so the bearer token
/// is never sent over plaintext.
pub(crate) fn topology_url(global_endpoint: &str, tls: bool) -> String {
    let base = global_endpoint.trim();
    let base = if base.starts_with("http://") || base.starts_with("https://") {
        if tls && base.starts_with("http://") {
            base.replacen("http://", "https://", 1)
        } else {
            base.to_owned()
        }
    } else {
        let scheme = if tls { "https" } else { "http" };
        format!("{scheme}://{base}")
    };
    let base = base.trim_end_matches('/');
    format!("{base}/{GLOBAL_CLUSTER_MARKER}/topology")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_global_endpoints_case_insensitively() {
        assert!(is_global_endpoint(
            "https://my-global-cluster.example.com:443"
        ));
        assert!(is_global_endpoint("https://xxx.global-cluster.yyy.com:443"));
        assert!(!is_global_endpoint("http://localhost:19530"));
        assert!(!is_global_endpoint(""));
    }

    #[test]
    fn parses_a_topology_response() {
        let body = r#"{"code":0,"data":{"version":7,"clusters":[
            {"clusterId":"primary","endpoint":"host-a:19530","capability":3},
            {"clusterId":"replica","endpoint":"host-b:19530","capability":1}
        ]}}"#;
        let topology = parse_topology_response(body).expect("valid topology");
        assert_eq!(topology.version(), 7);
        assert_eq!(topology.clusters().len(), 2);
        let primary = topology.primary().expect("has a primary");
        assert_eq!(primary.cluster_id(), "primary");
        assert_eq!(primary.endpoint(), "host-a:19530");
        assert!(primary.capability().is_writable());
        assert!(primary.capability().is_readable());
        let replica = &topology.clusters()[1];
        assert!(!replica.is_primary());
        assert!(replica.capability().is_readable());
        assert!(!replica.capability().is_writable());
    }

    #[test]
    fn rejects_a_failed_topology_response() {
        let body = r#"{"code":1,"message":"boom","data":null}"#;
        let error = parse_topology_response(body).unwrap_err();
        assert!(matches!(
            error,
            Error::Unexpected(message) if message.contains("boom")
        ));
    }

    #[test]
    fn rejects_a_topology_without_a_primary() {
        let body = r#"{"code":0,"data":{"version":1,"clusters":[
            {"clusterId":"ro","endpoint":"host-c:19530","capability":1}
        ]}}"#;
        let topology = parse_topology_response(body).expect("valid topology");
        assert!(topology.primary().is_err());
    }

    #[test]
    fn builds_the_topology_url() {
        assert_eq!(
            topology_url("https://my.global-cluster.example.com:443", true),
            "https://my.global-cluster.example.com:443/global-cluster/topology"
        );
        assert_eq!(
            topology_url("my.global-cluster.example.com:443/", true),
            "https://my.global-cluster.example.com:443/global-cluster/topology"
        );
        assert_eq!(
            topology_url("my.global-cluster.example.com:19530/", false),
            "http://my.global-cluster.example.com:19530/global-cluster/topology",
            "a scheme-less plaintext endpoint must default to http"
        );
        assert_eq!(
            topology_url("http://my.global-cluster.example.com:443", true),
            "https://my.global-cluster.example.com:443/global-cluster/topology",
            "an explicit http endpoint must be upgraded to https when TLS is enabled"
        );
        assert_eq!(
            topology_url("http://my.global-cluster.example.com:19530", false),
            "http://my.global-cluster.example.com:19530/global-cluster/topology",
            "an explicit http endpoint stays http without TLS"
        );
    }
}
