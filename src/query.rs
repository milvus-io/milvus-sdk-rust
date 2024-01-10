use std::borrow::Borrow;
use std::collections::HashMap;

use prost::Message;
use prost::bytes::BytesMut;

use crate::client::{Client, ConsistencyLevel};
use crate::collection::{SearchResult, ParamValue};
use crate::data::FieldColumn;
use crate::index::MetricType;
use crate::proto::common::{MsgBase, MsgType, KeyValuePair, PlaceholderGroup, PlaceholderValue, PlaceholderType, DslType};
use crate::proto::milvus::SearchRequest;
use crate::proto::schema::DataType;
use crate::utils::status_to_result;
use crate::value::Value;
use crate::{error::*, proto};
use crate::error::Error as SuperError;

const STRONG_TIMESTAMP: u64 = 0;
const BOUNDED_TIMESTAMP: u64 = 2;
const EVENTUALLY_TIMESTAMP: u64 = 1;

#[derive(Debug, Clone)]
pub struct QueryOptions {
    output_fields: Vec<String>,
    partition_names: Vec<String>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            output_fields: Vec::new(),
            partition_names: Vec::new(),
        }
    }
}

impl QueryOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_output_fields(output_fields: Vec<String>) -> Self {
        Self::default().output_fields(output_fields)
    }

    pub fn with_partition_names(partition_names: Vec<String>) -> Self {
        Self::default().partition_names(partition_names)
    }

    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }
}

pub struct SearchOptions {
    pub(crate) expr: String,
    pub(crate) limit: usize,
    pub(crate) output_fields: Vec<String>,
    pub(crate) partitions: Vec<String>,
    pub(crate) params:      serde_json::Value,
    pub(crate) metric_type: MetricType,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            expr: String::new(),
            limit: 10,
            output_fields: Vec::new(),
            partitions: Vec::new(),
            params: serde_json::Value::default(),
            metric_type: MetricType::L2,
        }
    }
}

impl SearchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_expr(expr: String) -> Self {
        Self::default().expr(expr)
    }

    pub fn with_limit(limit: usize) -> Self {
        Self::default().limit(limit)
    }

    pub fn with_output_fields(output_fields: Vec<String>) -> Self {
        Self::default().output_fields(output_fields)
    }

    pub fn with_partitions(partitions: Vec<String>) -> Self {
        Self::default().partitions(partitions)
    }

    pub fn with_params(params: HashMap<String, ParamValue>) -> Self {
        let mut options = Self::default();
        for (k, v) in params {
            options = options.add_param(k, v);
        }
        options
    }

    pub fn with_metric_type(metric_type: MetricType) -> Self {
        Self::default().metric_type(metric_type)
    }

    pub fn expr(mut self, expr: String) -> Self {
        self.expr = expr;
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn output_fields(mut self, output_fields: Vec<String>) -> Self {
        self.output_fields = output_fields;
        self
    }

    pub fn partitions(mut self, partitions: Vec<String>) -> Self {
        self.partitions = partitions;
        self
    }

    pub fn add_param(mut self, key: impl Into<String>, value: ParamValue) -> Self {
        if self.params.is_null() {
            self.params = ParamValue!({});
        }
        self.params
            .as_object_mut()
            .unwrap()
            .insert(key.into(), value);
        self
    }

    pub fn metric_type(mut self, metric_type: MetricType) -> Self {
        self.metric_type = metric_type;
        self
    }
}

impl Client {
pub(crate) async fn get_gts_from_consistency(&self,collection_name:&str, consistency_level: ConsistencyLevel) -> u64 {
    match consistency_level {
        ConsistencyLevel::Strong => STRONG_TIMESTAMP,
        ConsistencyLevel::Bounded => BOUNDED_TIMESTAMP,
        ConsistencyLevel::Eventually => EVENTUALLY_TIMESTAMP,
        ConsistencyLevel::Session => self.collection_cache.get_timestamp(collection_name).unwrap_or(EVENTUALLY_TIMESTAMP),

        // This level not works for now
        ConsistencyLevel::Customized => 0,
    }
}

pub async fn query<S,Exp>(&self,collection_name:S  , expr: Exp, options: &QueryOptions) -> Result<Vec<FieldColumn>>
where
    S: AsRef<str>,
    Exp: AsRef<str>,
{
    let collection_name = collection_name.as_ref();
    let collection = self
        .collection_cache
        .get(collection_name).await?;

    let consistency_level = collection.consistency_level;
    let mut output_fields = options.output_fields.clone();
    if output_fields.is_empty() {
       output_fields = collection.fields.iter().map(|f| f.name.clone()).collect();
    }

    let res = self
        .client
        .clone()
        .query(proto::milvus::QueryRequest {
            base: Some(MsgBase::new(MsgType::Retrieve)),
            db_name: "".to_owned(),
            collection_name: collection_name.to_owned(),
            expr: expr.as_ref().to_owned(),
            output_fields: output_fields,
            partition_names: options.partition_names.clone(),
            travel_timestamp: 0,
            guarantee_timestamp: self.get_gts_from_consistency(collection_name, consistency_level).await,
            query_params: Vec::new(),
            not_return_all_meta: false,
            consistency_level: ConsistencyLevel::default() as _,
            use_default_consistency: false,
        })
        .await?
        .into_inner();

    status_to_result(&res.status)?;

    Ok(res
        .fields_data
        .into_iter()
        .map(|f| FieldColumn::from(f))
        .collect())
}

pub async fn search<S>(
    &self,
    collection_name:S,
    data: Vec<Value<'_>>,
    vec_field: S,
    option: &SearchOptions,
) -> Result<Vec<SearchResult<'_>>>
where
    S: Into<String>,
{
    // check and prepare params
    let search_params: Vec<KeyValuePair> = vec![
        KeyValuePair {
            key: "anns_field".to_owned(),
            value: vec_field.into(),
        },
        KeyValuePair {
            key: "topk".to_owned(),
            value: option.limit.to_string(),
        },
        KeyValuePair {
            key: "params".to_owned(),
            value: serde_json::to_string(&option.params)?,
        },
        KeyValuePair {
            key: "metric_type".to_owned(),
            value: option.metric_type.to_string(),
        },
        KeyValuePair {
            key: "round_decimal".to_owned(),
            value: "-1".to_owned(),
        },
    ];

    let collection_name = collection_name.into();
    let collection = self.collection_cache.get(&collection_name).await?;

    let res = self
        .client
        .clone()
        .search(SearchRequest {
            base: Some(MsgBase::new(MsgType::Search)),
            db_name: "".to_string(),
            collection_name: collection_name.clone(),
            partition_names: option.partitions.clone(),
            dsl: option.expr.clone(),
            nq: data.len() as _,
            placeholder_group: get_place_holder_group(data)?,
            dsl_type: DslType::BoolExprV1 as _,
            output_fields: option.output_fields.clone().into_iter().map(|f| f.into()).collect(),
            search_params,
            travel_timestamp: 0,
            guarantee_timestamp: self.get_gts_from_consistency(&collection_name, collection.consistency_level).await,
            not_return_all_meta: false,
            consistency_level: ConsistencyLevel::default() as _,
            use_default_consistency: false,
            search_by_primary_keys: false,
        })
        .await?
        .into_inner();
    status_to_result(&res.status)?;
    let raw_data = res
        .results
        .ok_or(SuperError::Unexpected("no result for search".to_owned()))?;
    let mut result = Vec::new();
    let mut offset = 0;
    let fields_data = raw_data
        .fields_data
        .into_iter()
        .map(Into::into)
        .collect::<Vec<FieldColumn>>();
    let raw_id = raw_data.ids.unwrap().id_field.unwrap();

    for k in raw_data.topks {
        let k = k as usize;
        let mut score = Vec::new();
        score.extend_from_slice(&raw_data.scores[offset..offset + k]);
        let mut result_data = fields_data
            .iter()
            .map(FieldColumn::copy_with_metadata)
            .collect::<Vec<FieldColumn>>();
        for j in 0..fields_data.len() {
            for i in offset..offset + k {
                result_data[j].push(fields_data[j].get(i).ok_or(SuperError::Unexpected(
                    "out of range while indexing field data".to_owned(),
                ))?);
            }
        }

        let id = match raw_id {
            proto::schema::i_ds::IdField::IntId(ref d) => {
                Vec::<Value>::from_iter(d.data[offset..offset + k].iter().map(|&x| x.into()))
            }
            proto::schema::i_ds::IdField::StrId(ref d) => Vec::<Value>::from_iter(
                d.data[offset..offset + k].iter().map(|x| x.clone().into()),
            ),
        };

        result.push(SearchResult {
            size: k as i64,
            score,
            field: result_data,
            id,
        });

        offset += k;
    }

    Ok(result)
}

}

fn get_place_holder_group(vectors: Vec<Value>) -> Result<Vec<u8>> {
    let group = PlaceholderGroup {
        placeholders: vec![get_place_holder_value(vectors)?],
    };
    let mut buf = BytesMut::new();
    group.encode(&mut buf).unwrap();
    return Ok(buf.to_vec());
}

fn get_place_holder_value(vectors: Vec<Value>) -> Result<PlaceholderValue> {
    let mut place_holder = PlaceholderValue {
        tag: "$0".to_string(),
        r#type: PlaceholderType::None as _,
        values: Vec::new(),
    };
    // if no vectors, return an empty one
    if vectors.len() == 0 {
        return Ok(place_holder);
    };

    match vectors[0] {
        Value::FloatArray(_) => place_holder.r#type = PlaceholderType::FloatVector as _,
        Value::Binary(_) => place_holder.r#type = PlaceholderType::BinaryVector as _,
        _ => {
            return Err(SuperError::from(crate::collection::Error::IllegalType(
                "place holder".to_string(),
                vec![DataType::BinaryVector, DataType::FloatVector],
            )))
        }
    };

    for v in &vectors {
        match (v, &vectors[0]) {
            (Value::FloatArray(d), Value::FloatArray(_)) => {
                let mut bytes = Vec::<u8>::with_capacity(d.len() * 4);
                for f in d.iter() {
                    bytes.extend_from_slice(&f.to_le_bytes());
                }
                place_holder.values.push(bytes)
            }
            (Value::Binary(d), Value::Binary(_)) => place_holder.values.push(d.to_vec()),
            _ => {
                return Err(SuperError::from(crate::collection::Error::IllegalType(
                    "place holder".to_string(),
                    vec![DataType::BinaryVector, DataType::FloatVector],
                )))
            }
        };
    }
    return Ok(place_holder);
}
