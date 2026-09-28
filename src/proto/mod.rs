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

pub mod common {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.common.rs"));
}

pub mod feder {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.feder.rs"));
}

pub mod milvus {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.milvus.rs"));
}

pub mod msg {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.msg.rs"));
}

pub mod rg {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.rg.rs"));
}

pub mod schema {
    #![allow(rustdoc::broken_intra_doc_links, rustdoc::invalid_html_tags)]
    include!(concat!(env!("OUT_DIR"), "/milvus.proto.schema.rs"));
}

use self::common::{MsgBase, MsgType};

impl MsgBase {
    pub fn new(msg_type: MsgType) -> Self {
        Self {
            msg_type: msg_type.into(),
            ..Default::default()
        }
    }
}
