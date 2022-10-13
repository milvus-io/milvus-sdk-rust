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
// For custom error handling
// TODO

use crate::collection::Error as CollectionError;
use crate::proto::common::{ErrorCode, Status};
use crate::schema::Error as SchemaError;
use std::result;
use thiserror::Error;
use tonic::transport::Error as CommError;
use tonic::Status as GrpcError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0:?}")]
    Communication(#[from] CommError),

    #[error("{0:?}")]
    Collection(#[from] CollectionError),

    #[error("{0:?}")]
    Grpc(#[from] GrpcError),

    #[error("{0:?}")]
    Schema(#[from] SchemaError),

    #[error("{0:?} {1:?}")]
    Server(ErrorCode, String),

    #[error("{0:?}")]
    ProstEncode(#[from] prost::EncodeError),

    #[error("{0:?}")]
    ProstDecode(#[from] prost::DecodeError),
    
    #[error("Conversion error")]
    Conversion,
    #[error("{0:?}")]
    SerdeJsonErr(#[from] serde_json::Error),

    #[error("parameter {0:?} with invalid value {1:?}")]
    InvalidParameter(String, String),

    #[error("{0:?}")]
    Other(#[from] anyhow::Error),

    #[error("{0}")]
    Unexpected(String),
}

impl From<Status> for Error {
    fn from(s: Status) -> Self {
        Error::Server(ErrorCode::from_i32(s.error_code).unwrap(), s.reason)
    }
}

pub type Result<T> = result::Result<T, Error>;
