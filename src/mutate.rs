use crate::error::Result;
use crate::utils::status_to_result;
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
    pub(crate) namespace: Option<String>,
}

impl Default for InsertOptions {
    fn default() -> Self {
        Self {
            partition_name: String::new(),
            namespace: None,
        }
    }
}

impl InsertOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_partition_name(partition_name: impl Into<String>) -> Self {
        Self::default().partition_name(partition_name)
    }

    pub fn partition_name(mut self, partition_name: impl Into<String>) -> Self {
        self.partition_name = partition_name.into();
        self
    }

    /// Set namespace for multi-tenancy (Milvus 2.6+)
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct UpsertOptions {
    pub(crate) partition_name: String,
    pub(crate) namespace: Option<String>,
    pub(crate) partial_update: bool,
}

impl Default for UpsertOptions {
    fn default() -> Self {
        Self {
            partition_name: String::new(),
            namespace: None,
            partial_update: false,
        }
    }
}

impl UpsertOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn partition_name(mut self, partition_name: impl Into<String>) -> Self {
        self.partition_name = partition_name.into();
        self
    }

    /// Set namespace for multi-tenancy (Milvus 2.6+)
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Enable partial update: only update specified fields (Milvus 2.6+)
    pub fn partial_update(mut self, enabled: bool) -> Self {
        self.partial_update = enabled;
        self
    }
}

impl From<InsertOptions> for UpsertOptions {
    fn from(options: InsertOptions) -> Self {
        Self {
            partition_name: options.partition_name,
            namespace: options.namespace,
            partial_update: false,
        }
    }
}

pub trait IntoUpsertOptions {
    fn into_upsert_options(self) -> UpsertOptions;
}

impl IntoUpsertOptions for UpsertOptions {
    fn into_upsert_options(self) -> UpsertOptions {
        self
    }
}

impl IntoUpsertOptions for InsertOptions {
    fn into_upsert_options(self) -> UpsertOptions {
        self.into()
    }
}

impl IntoUpsertOptions for Option<UpsertOptions> {
    fn into_upsert_options(self) -> UpsertOptions {
        self.unwrap_or_default()
    }
}

impl IntoUpsertOptions for Option<InsertOptions> {
    fn into_upsert_options(self) -> UpsertOptions {
        self.unwrap_or_default().into()
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

#[cfg(test)]
mod test {
    use super::{InsertOptions, IntoUpsertOptions};

    #[test]
    fn option_insert_options_convert_to_upsert_options() {
        let options = Some(
            InsertOptions::new()
                .partition_name("p0")
                .namespace("tenant-a"),
        )
        .into_upsert_options();

        assert_eq!(options.partition_name, "p0");
        assert_eq!(options.namespace.as_deref(), Some("tenant-a"));
        assert!(!options.partial_update);
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
                schema_timestamp: 0,
                namespace: options.namespace,
            })
            .await?
            .into_inner();

        status_to_result(&result.status)?;

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
                consistency_level: crate::proto::common::ConsistencyLevel::Strong.into(),
                expr_template_values: std::collections::HashMap::new(),
            })
            .await?
            .into_inner();

        status_to_result(&result.status)?;

        self.collection_cache
            .update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }

    async fn compose_expr(&self, collection_name: &str, options: &DeleteOptions) -> Result<String> {
        let expr = match options.filter.len() {
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
                        expr.push_str("]");
                        expr
                    }

                    (DataType::VarChar, ValueVec::String(values)) => {
                        for (i, v) in values.iter().enumerate() {
                            if i > 0 {
                                expr.push_str(",");
                            }
                            expr.push_str(v.as_str());
                        }
                        expr.push_str("]");
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

        Ok(expr)
    }

    pub async fn upsert<S, O>(
        &self,
        collection_name: S,
        fields_data: Vec<FieldColumn>,
        options: O,
    ) -> Result<crate::proto::milvus::MutationResult>
    where
        S: Into<String>,
        O: IntoUpsertOptions,
    {
        let options = options.into_upsert_options();
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
                schema_timestamp: 0,
                partial_update: options.partial_update,
                namespace: options.namespace,
            })
            .await?
            .into_inner();

        status_to_result(&result.status)?;

        self.collection_cache
            .update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::proto::common::{ErrorCode, Status};
    use crate::proto::milvus::MutationResult;
    use crate::utils::status_to_result;

    #[test]
    fn status_to_result_success() {
        let result = MutationResult {
            status: Some(Status {
                error_code: ErrorCode::Success as i32,
                reason: "ok".to_string(),
                code: 0,
                retriable: false,
                detail: "".to_string(),
                extra_info: Default::default(),
            }),
            i_ds: None,
            succ_index: vec![],
            err_index: vec![],
            acknowledged: true,
            insert_cnt: 10,
            delete_cnt: 0,
            upsert_cnt: 0,
            timestamp: 0,
        };

        assert!(status_to_result(&result.status).is_ok());
    }

    #[test]
    fn status_to_result_error() {
        let result = MutationResult {
            status: Some(Status {
                error_code: ErrorCode::IllegalArgument as i32,
                reason: "varchar length exceeds limit".to_string(),
                code: 5,
                retriable: false,
                detail: "".to_string(),
                extra_info: Default::default(),
            }),
            i_ds: None,
            succ_index: vec![],
            err_index: vec![1, 2, 3],
            acknowledged: false,
            insert_cnt: 0,
            delete_cnt: 0,
            upsert_cnt: 0,
            timestamp: 0,
        };

        let err = status_to_result(&result.status).unwrap_err();
        assert!(format!("{err}").contains("varchar length exceeds limit"));
    }
}
