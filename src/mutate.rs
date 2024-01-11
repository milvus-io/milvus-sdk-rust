use crate::{client::Client, data::FieldColumn, proto::{milvus::InsertRequest, common::{MsgBase, MsgType}, self}, error::Error, utils::status_to_result, schema::FieldData};
use crate::error::Result;

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
    pub(crate) ids: proto::schema::FieldData,
    pub(crate) filter: String,
    pub(crate) partition_names: Vec<String>,
}

impl DeleteOptions {
    fn new() -> Self {
        Self {
            ids: FieldData::default(),
            filter: String::new(),
            partition_names: Vec::new(),
        }
    }

    pub fn with_ids(ids: FieldColumn) -> Self {
        Self::new().ids(ids.into())
    }

    pub fn with_partition_names(partition_names: Vec<String>) -> Self {
        Self::new().partition_names(partition_names)
    }

    pub fn with_filter(filter: String) -> Self {
        Self::new().filter(filter)
    }

    pub fn ids(mut self, ids: proto::schema::FieldData) -> Self {
        self.ids = ids;
        self
    }

    pub fn partition_names(mut self, partition_names: Vec<String>) -> Self {
        self.partition_names = partition_names;
        self
    }

    pub fn filter(mut self, filter: String) -> Self {
        self.filter = filter;
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
            })
            .await?
            .into_inner();

        self.collection_cache.update_timestamp(&collection_name, result.timestamp);

        Ok(result)
    }
}