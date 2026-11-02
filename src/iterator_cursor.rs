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

//! Private wire negotiation shared by the two public search-iterator APIs.

use crate::proto::{common, milvus, schema};

pub(crate) const CURSOR_VERSION: &str = "search_iter_cursor_version";
pub(crate) const LAST_PK_TYPE: &str = "search_iter_last_pk_type";
pub(crate) const LAST_PK: &str = "search_iter_last_pk";

///////////////////////////////////////////////////////////////////////////////
// CursorMode
///////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorMode {
    Distance,
    PrimaryKey,
}

pub(crate) fn param<'a>(request: &'a milvus::SearchRequest, key: &str) -> Option<&'a str> {
    request
        .search_params
        .iter()
        .find(|pair| pair.key == key)
        .map(|pair| pair.value.as_str())
}

pub(crate) fn set_param(request: &mut milvus::SearchRequest, key: &str, value: String) {
    request.search_params.retain(|pair| pair.key != key);
    request.search_params.push(common::KeyValuePair {
        key: key.into(),
        value,
    });
}

fn remove_param(request: &mut milvus::SearchRequest, key: &str) {
    request.search_params.retain(|pair| pair.key != key);
    if let Some(params) = request
        .search_params
        .iter_mut()
        .find(|pair| pair.key == "params")
    {
        if let Ok(serde_json::Value::Object(mut nested)) = serde_json::from_str(&params.value) {
            nested.remove(key);
            params.value = serde_json::Value::Object(nested).to_string();
        }
    }
}

pub(crate) fn legacy_continuation(request: &milvus::SearchRequest) -> bool {
    ["search_iter_id", "search_iter_last_bound"]
        .iter()
        .any(|key| param(request, key).is_some_and(|value| !value.is_empty()))
}

pub(crate) fn clear_pk_controls(request: &mut milvus::SearchRequest) {
    for key in [CURSOR_VERSION, LAST_PK_TYPE, LAST_PK] {
        remove_param(request, key);
    }
}

pub(crate) fn configure(
    request: &mut milvus::SearchRequest,
    known_mode: Option<CursorMode>,
) -> Result<Option<CursorMode>, String> {
    let requested = param(request, CURSOR_VERSION).unwrap_or("").to_owned();
    if !requested.is_empty() && requested != "2" {
        return Err(format!(
            "unsupported search iterator cursor version {requested:?}"
        ));
    }
    if requested == "2" && known_mode.is_none() && legacy_continuation(request) {
        return Err("PK cursor mode requires a complete typed cursor; legacy token/bound continuation must omit cursor version 2".into());
    }
    if known_mode == Some(CursorMode::PrimaryKey) {
        set_param(request, CURSOR_VERSION, "2".into());
        return Ok(known_mode);
    }
    if known_mode == Some(CursorMode::Distance) || legacy_continuation(request) || requested != "2"
    {
        clear_pk_controls(request);
        return Ok(Some(CursorMode::Distance));
    }
    clear_pk_controls(request);
    set_param(request, CURSOR_VERSION, "2".into());
    Ok(None)
}

// Build a candidate next request without changing any iterator state. Callers
// commit it only after decoding and filtering the corresponding page succeeds.
pub(crate) fn advance(
    request: &milvus::SearchRequest,
    response: &milvus::SearchResults,
    previous_mode: Option<CursorMode>,
    primary_type: &str,
    batch_size: usize,
) -> Result<(milvus::SearchRequest, CursorMode), String> {
    let data = response
        .results
        .as_ref()
        .ok_or("search iterator response has no results")?;
    let iterator = data
        .search_iterator_v2_results
        .as_ref()
        .filter(|info| !info.token.is_empty())
        .ok_or("server does not support Search Iterator V2")?;
    let extra = response.status.as_ref().map(|status| &status.extra_info);
    let version = extra
        .and_then(|extra| extra.get(CURSOR_VERSION))
        .map(String::as_str)
        .unwrap_or("");
    if version == "2" && param(request, CURSOR_VERSION) != Some("2") {
        return Err("server activated PK cursor mode without client opt-in".into());
    }
    let mode = match version {
        "" => CursorMode::Distance,
        "2" => CursorMode::PrimaryKey,
        _ => {
            return Err(format!(
                "unsupported search iterator cursor version {version:?}"
            ))
        }
    };
    if previous_mode.is_some_and(|previous| previous != mode) {
        return Err("search iterator cursor mode changed between pages".into());
    }
    if param(request, "search_iter_id").is_some_and(|token| token != iterator.token) {
        return Err("search iterator token changed between pages".into());
    }
    let mut next = request.clone();
    if mode == CursorMode::PrimaryKey {
        if data.num_queries != 1 || data.topks.len() != 1 {
            return Err("search iterator requires exactly one result set".into());
        }
        let count =
            usize::try_from(data.topks[0]).map_err(|_| "negative search iterator row count")?;
        if count > batch_size
            || data.scores.len() != count
            || !iterator.last_bound.is_finite()
            || data.scores.iter().any(|score| !score.is_finite())
        {
            return Err("invalid search iterator scores or row count".into());
        }
        let (actual_type, last_pk, id_count) =
            match data.ids.as_ref().and_then(|ids| ids.id_field.as_ref()) {
                Some(schema::i_ds::IdField::IntId(ids)) => (
                    "int64",
                    ids.data.last().map(ToString::to_string),
                    ids.data.len(),
                ),
                Some(schema::i_ds::IdField::StrId(ids)) => {
                    ("varchar", ids.data.last().cloned(), ids.data.len())
                }
                None if count == 0 => (primary_type, None, 0),
                _ => return Err("unsupported search iterator primary keys".into()),
            };
        if actual_type != primary_type || id_count != count {
            return Err("search iterator primary keys do not match schema or row count".into());
        }
        if count > 0 {
            let extra = extra.ok_or("search iterator cursor metadata is missing")?;
            if extra.get(LAST_PK_TYPE).map(String::as_str) != Some(primary_type)
                || extra.get(LAST_PK) != last_pk.as_ref()
                || iterator.last_bound != data.scores[count - 1]
            {
                return Err("search iterator cursor does not match last result".into());
            }
            set_param(&mut next, LAST_PK_TYPE, primary_type.into());
            set_param(
                &mut next,
                LAST_PK,
                last_pk.expect("nonempty result has last PK"),
            );
        }
        set_param(&mut next, CURSOR_VERSION, "2".into());
    } else {
        for key in [CURSOR_VERSION, LAST_PK_TYPE, LAST_PK] {
            remove_param(&mut next, key);
        }
    }
    if next.guarantee_timestamp == 0 {
        if mode == CursorMode::PrimaryKey && response.session_ts == 0 {
            return Err("search iterator PK cursor response has no snapshot timestamp".into());
        }
        next.guarantee_timestamp = response.session_ts;
    }
    set_param(&mut next, "search_iter_id", iterator.token.clone());
    // f32 Display is the shortest exact round-trip representation, including tiny distances.
    set_param(
        &mut next,
        "search_iter_last_bound",
        iterator.last_bound.to_string(),
    );
    Ok((next, mode))
}
