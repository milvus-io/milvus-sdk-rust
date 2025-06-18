use crate::error::Result;
use crate::{
    client::Client,
    data::FieldColumn,
    error::Error,
    proto::{
        self,
        common::{MsgBase, MsgType},
        milvus::{InsertRequest, UpsertRequest},
        schema::DataType,
    },
    value::ValueVec,
};

#[derive(Debug, Clone)]
pub struct InsertOptions {
    pub(crate) partition_name: String,
}

impl Default for InsertOptions {
    fn default() -> Self {
        Self {
            partition_name: String::new(),
        }
    }
}

impl InsertOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_partition_name(partition_name: String) -> Self {
        Self::default().partition_name(partition_name)
    }

    pub fn partition_name(mut self, partition_name: String) -> Self {
        self.partition_name = partition_name.to_owned();
        self
    }
}

#[derive(Debug, Clone)]
pub struct DeleteOptions {
    pub(crate) ids: ValueVec,
    pub(crate) filter: String,
    pub(crate) partition_name: String,
}

impl DeleteOptions {
    fn new() -> Self {
        Self {
            ids: ValueVec::None,
            filter: String::new(),
            partition_name: String::new(),
        }
    }

    pub fn with_ids(ids: ValueVec) -> Self {
        let mut opt = Self::new();
        opt.ids = ids;
        opt
    }

    pub fn with_filter(filter: String) -> Self {
        let mut opt = Self::new();
        opt.filter = filter;
        opt
    }

    pub fn partition_name(mut self, partition_name: String) -> Self {
        self.partition_name = partition_name;
        self
    }
}

impl Client {
    pub async fn insert<S>(
        &self,
        collection_name: S,
        fields_data: Vec<FieldColumn>,
        options: Option<InsertOptions>,
    ) -> Result<crate::proto::milvus::MutationResult>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        let row_num = fields_data.first().map(|c| c.len()).unwrap_or(0);
        let collection_name = collection_name.into();

        let result = self
            .client
            .clone()
            .insert(InsertRequest {
                base: Some(MsgBase::new(MsgType::Insert)),
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                partition_name: options.partition_name,
                num_rows: row_num as u32,
                fields_data: fields_data.into_iter().map(|f| f.into()).collect(),
                hash_keys: Vec::new(),
                schema_timestamp:0,
            })
            .await?
            .into_inner();

        self.collection_cache
            .update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }

    pub async fn delete(
        &self,
        collection_name: impl Into<String>,
        options: &DeleteOptions,
    ) -> Result<crate::proto::milvus::MutationResult> {
        let collection_name = collection_name.into();

        let expr = self.compose_expr(&collection_name, options).await?;

        let result = self
            .client
            .clone()
            .delete(proto::milvus::DeleteRequest {
                base: Some(MsgBase::new(MsgType::Delete)),
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                expr: expr,
                partition_name: options.partition_name.clone(),
                hash_keys: Vec::new(),
                consistency_level:crate::proto::common::ConsistencyLevel::Strong.into(),
                expr_template_values:std::collections::HashMap::new(),
            })
            .await?
            .into_inner();

        self.collection_cache
            .update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }

    async fn compose_expr(&self, collection_name: &str, options: &DeleteOptions) -> Result<String> {
        let mut expr = String::new();
        match options.filter.len() {
            0 => {
                let collection = self.collection_cache.get(collection_name).await?;
                let pk = collection.fields.iter().find(|f| f.is_primary_key).unwrap();

                let mut expr = String::new();
                expr.push_str(&pk.name);
                expr.push_str(" in [");
                match (pk.dtype, options.ids.clone()) {
                    (DataType::Int64, ValueVec::Long(values)) => {
                        for (i, v) in values.iter().enumerate() {
                            if i > 0 {
                                expr.push_str(",");
                            }
                            expr.push_str(format!("{}", v).as_str());
                        }
                        expr
                    }

                    (DataType::VarChar, ValueVec::String(values)) => {
                        for (i, v) in values.iter().enumerate() {
                            if i > 0 {
                                expr.push_str(",");
                            }
                            expr.push_str(v.as_str());
                        }
                        expr
                    }

                    _ => {
                        return Err(Error::InvalidParameter(
                            "pk type".to_owned(),
                            pk.dtype.as_str_name().to_owned(),
                        ));
                    }
                }
            }

            _ => options.filter.clone(),
        };
        expr.push(')');

        Ok(expr)
    }

    pub async fn upsert<S>(
        &self,
        collection_name: S,
        fields_data: Vec<FieldColumn>,
        options: Option<InsertOptions>,
    ) -> Result<crate::proto::milvus::MutationResult>
    where
        S: Into<String>,
    {
        let options = options.unwrap_or_default();
        let row_num = fields_data.first().map(|c| c.len()).unwrap_or(0);
        let collection_name = collection_name.into();

        let result = self
            .client
            .clone()
            .upsert(UpsertRequest {
                base: Some(MsgBase::new(MsgType::Upsert)),
                db_name: "".to_string(),
                collection_name: collection_name.clone(),
                partition_name: options.partition_name,
                num_rows: row_num as u32,
                fields_data: fields_data.into_iter().map(|f| f.into()).collect(),
                hash_keys: Vec::new(),
                schema_timestamp:0,
            })
            .await?
            .into_inner();

        self.collection_cache
            .update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }
}
