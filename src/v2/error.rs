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

//! Error and result types returned by ClientV2 APIs.

use crate::proto::common::{ErrorCode, Status};
use std::result;
use std::sync::Arc;
use thiserror::Error;
use tonic::Status as GrpcError;

pub use crate::v2::bulk_import::BulkImportError;

///////////////////////////////////////////////////////////////////////////////
// ServerError
///////////////////////////////////////////////////////////////////////////////
/// Error returned by the Milvus server.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
#[error("Milvus server error (code={code}, message={reason})")]
pub struct ServerError {
    code: i32,
    legacy_code: i32,
    reason: String,
}

impl ServerError {
    /// Returns the modern Milvus server error code.
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Returns the deprecated error code supplied by older Milvus servers.
    pub fn legacy_code(&self) -> i32 {
        self.legacy_code
    }

    /// Returns the server error message.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    #[allow(deprecated)]
    fn from_status(status: Status) -> Self {
        Self {
            code: status.code,
            legacy_code: status.error_code,
            reason: status.reason,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ConversionError
///////////////////////////////////////////////////////////////////////////////
/// Error produced while converting SDK, JSON, or protobuf values.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum ConversionError {
    #[error("failed to encode protobuf data: {0}")]
    /// Represents the ProtobufEncode case.
    ProtobufEncode(#[source] prost::EncodeError),

    #[error("failed to decode protobuf data: {0}")]
    /// Represents the ProtobufDecode case.
    ProtobufDecode(#[source] prost::DecodeError),

    #[error("JSON conversion failed: {0}")]
    /// Represents the Json case.
    Json(#[source] Arc<serde_json::Error>),

    #[error("{0}")]
    /// Represents the Value case.
    Value(String),
}

///////////////////////////////////////////////////////////////////////////////
// ValidationError
///////////////////////////////////////////////////////////////////////////////
/// Error produced when a request or SDK value fails local validation.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
#[error("invalid parameter {parameter}: {reason}")]
pub struct ValidationError {
    parameter: String,
    reason: String,
}

impl ValidationError {
    pub(crate) fn new(parameter: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            parameter: parameter.into(),
            reason: reason.into(),
        }
    }

    /// Returns the name of the invalid parameter or field.
    pub fn parameter(&self) -> &str {
        &self.parameter
    }

    /// Returns why the value was rejected.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

///////////////////////////////////////////////////////////////////////////////
// Error
///////////////////////////////////////////////////////////////////////////////
/// Error returned by a ClientV2 operation.
#[derive(Debug, Clone, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("gRPC error: {0}")]
    /// Represents the Grpc case.
    Grpc(#[from] GrpcError),

    #[error(transparent)]
    /// Represents the Server case.
    Server(#[from] ServerError),

    #[error(transparent)]
    /// Represents the Conversion case.
    Conversion(#[from] ConversionError),

    #[error(transparent)]
    /// Represents the Validation case.
    Validation(#[from] ValidationError),

    #[error(transparent)]
    /// Represents the BulkImport case.
    BulkImport(#[from] BulkImportError),

    #[error("operation timed out: {0}")]
    /// Represents the Timeout case.
    Timeout(String),

    #[error("malformed server response: {0}")]
    /// Represents the MalformedResponse case.
    MalformedResponse(String),

    #[error("operation cancelled: {0}")]
    /// Represents the Cancelled case.
    Cancelled(String),

    #[error("RPC retry exhausted after {attempts} attempts: {source}")]
    /// Represents the RetryExhausted case.
    RetryExhausted {
        /// Number of transport or server attempts that were made.
        attempts: u32,
        #[source]
        /// Final error that prevented another retry.
        source: Box<Error>,
    },

    #[error("{0}")]
    /// Represents the Unexpected case.
    Unexpected(String),
}

impl Error {
    pub(crate) fn conversion(message: impl Into<String>) -> Self {
        ConversionError::Value(message.into()).into()
    }

    pub(crate) fn validation(parameter: String, reason: String) -> Self {
        ValidationError::new(parameter, reason).into()
    }
}

impl From<prost::EncodeError> for Error {
    fn from(error: prost::EncodeError) -> Self {
        ConversionError::ProtobufEncode(error).into()
    }
}

impl From<prost::DecodeError> for Error {
    fn from(error: prost::DecodeError) -> Self {
        ConversionError::ProtobufDecode(error).into()
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        ConversionError::Json(Arc::new(error)).into()
    }
}

#[allow(deprecated)]
impl From<Status> for Error {
    fn from(status: Status) -> Self {
        Error::Server(ServerError::from_status(status))
    }
}

/// Public type alias for Result.
pub type Result<T> = result::Result<T, Error>;

#[allow(deprecated)]
pub(crate) fn status_to_result(status: &Option<Status>) -> Result<()> {
    let status = status
        .clone()
        .ok_or_else(|| Error::MalformedResponse("response contains no status".to_owned()))?;

    if status.code == 0 && status.error_code == ErrorCode::Success as i32 {
        Ok(())
    } else {
        Err(Error::from(status))
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use prost::Message;
    use std::collections::HashMap;

    #[test]
    fn status_to_result_accepts_complete_success() {
        let status = Some(Status {
            code: 0,
            error_code: ErrorCode::Success as i32,
            ..Default::default()
        });

        assert!(status_to_result(&status).is_ok());
    }

    #[test]
    fn status_to_result_preserves_modern_server_error() {
        let status = Some(Status {
            code: 1100,
            error_code: ErrorCode::Success as i32,
            reason: "modern failure".into(),
            retriable: true,
            detail: "internal detail".into(),
            extra_info: HashMap::from([("trace".into(), "hidden".into())]),
        });

        let error = status_to_result(&status).unwrap_err();
        let Error::Server(error) = error else {
            panic!("expected server error");
        };
        assert_eq!(error.code(), 1100);
        assert_eq!(error.legacy_code(), ErrorCode::Success as i32);
        assert_eq!(error.reason(), "modern failure");
    }

    #[test]
    fn status_to_result_preserves_legacy_server_error() {
        let status = Some(Status {
            code: 0,
            error_code: ErrorCode::CollectionNotExists as i32,
            reason: "legacy failure".into(),
            ..Default::default()
        });

        let error = status_to_result(&status).unwrap_err();
        let Error::Server(error) = error else {
            panic!("expected server error");
        };
        assert_eq!(error.code(), 0);
        assert_eq!(error.legacy_code(), ErrorCode::CollectionNotExists as i32);
        assert_eq!(error.reason(), "legacy failure");
    }

    #[test]
    fn status_to_result_preserves_unknown_legacy_code() {
        let status = Some(Status {
            code: 0,
            error_code: 123_456,
            reason: "unknown legacy failure".into(),
            ..Default::default()
        });

        let error = status_to_result(&status).unwrap_err();
        let Error::Server(error) = error else {
            panic!("expected server error");
        };
        assert_eq!(error.code(), 0);
        assert_eq!(error.legacy_code(), 123_456);
        assert_eq!(error.reason(), "unknown legacy failure");
    }

    #[test]
    fn serde_json_errors_are_reported_as_conversion_errors() {
        let error: Error = serde_json::from_str::<serde_json::Value>("{")
            .unwrap_err()
            .into();

        assert!(matches!(error, Error::Conversion(ConversionError::Json(_))));
        assert!(error.to_string().starts_with("JSON conversion failed:"));
    }

    #[test]
    fn value_conversion_errors_preserve_their_reason() {
        let error = Error::conversion("unknown protobuf data type 999");

        assert!(matches!(
            &error,
            Error::Conversion(ConversionError::Value(reason))
                if reason == "unknown protobuf data type 999"
        ));
        assert_eq!(error.to_string(), "unknown protobuf data type 999");
    }

    #[test]
    fn protobuf_encode_errors_are_reported_as_conversion_errors() {
        let status = Status {
            reason: "does not fit".into(),
            ..Default::default()
        };
        let mut storage = [0u8; 0];
        let mut output = storage.as_mut_slice();
        let error: Error = status.encode(&mut output).unwrap_err().into();

        assert!(matches!(
            error,
            Error::Conversion(ConversionError::ProtobufEncode(_))
        ));
    }

    #[test]
    fn protobuf_decode_errors_are_reported_as_conversion_errors() {
        let error: Error = Status::decode([0xff].as_slice()).unwrap_err().into();

        assert!(matches!(
            error,
            Error::Conversion(ConversionError::ProtobufDecode(_))
        ));
    }

    #[test]
    fn grpc_errors_use_user_facing_display_formatting() {
        let error = Error::Grpc(tonic::Status::unavailable("server unavailable"));

        let message = error.to_string();
        assert!(message.starts_with("gRPC error: "));
        assert!(message.contains("Unavailable"));
        assert!(message.contains("server unavailable"));
    }

    #[test]
    fn validation_errors_expose_parameter_and_reason() {
        let error = Error::validation("limit".into(), "must be positive".into());

        let Error::Validation(error) = error else {
            panic!("expected validation error");
        };
        assert_eq!(error.parameter(), "limit");
        assert_eq!(error.reason(), "must be positive");
        assert_eq!(
            error.to_string(),
            "invalid parameter limit: must be positive"
        );
    }
}
