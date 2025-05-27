use std::collections::HashMap;

use crate::{
    client::Client,
    error::{Error, Result},
    proto::{
        common::{KeyValuePair, MsgBase, MsgType},
        milvus::{
            AlterDatabaseRequest, CreateDatabaseRequest, DescribeDatabaseRequest,
            DropDatabaseRequest, ListDatabasesRequest,
        },
    },
    utils::{hashmap_to_vec, status_to_result, vec_to_hashmap},
};

pub struct DatabaseDescription {
    pub db_id: i64,
    pub db_name: String,
    pub properties: HashMap<String, String>,
}

impl Client {
    pub async fn create_database(
        &self,
        db_name: &str,
        properties: HashMap<String, String>,
    ) -> Result<()> {
        let status = self
            .client
            .clone()
            .create_database(CreateDatabaseRequest {
                base: Some(MsgBase::new(MsgType::CreateDatabase)),
                db_name: db_name.to_owned(),
                properties: properties
                    .iter()
                    .map(|(k, v)| KeyValuePair {
                        key: k.clone(),
                        value: v.clone(),
                    })
                    .collect(),
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }

    pub async fn list_databases(&self) -> Result<Vec<String>> {
        let response = self
            .client
            .clone()
            .list_databases(ListDatabasesRequest {
                base: Some(MsgBase::new(MsgType::ListDatabases)),
            })
            .await?
            .into_inner();

        status_to_result(&response.status)?;
        Ok(response.db_names)
    }

    pub async fn describe_database(&self, db_name: &str) -> Result<DatabaseDescription> {
        let response = self
            .client
            .clone()
            .describe_database(DescribeDatabaseRequest {
                base: Some(MsgBase::new(MsgType::DescribeDatabase)),
                db_name: db_name.to_owned(),
            })
            .await?
            .into_inner();

        status_to_result(&response.status)?;
        Ok(DatabaseDescription {
            db_id: response.db_id,
            db_name: response.db_name,
            properties: vec_to_hashmap(&response.properties),
        })
    }

    pub async fn alter_database_properties(
        &self,
        db_name: &str,
        properties: HashMap<String, String>,
    ) -> Result<()> {
        let status = self
            .client
            .clone()
            .alter_database(AlterDatabaseRequest {
                base: Some(MsgBase::new(MsgType::AlterDatabase)),
                db_name: db_name.to_owned(),
                properties: hashmap_to_vec(&properties),
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }

    pub async fn drop_database_properties(
        &self,
        db_name: &str,
        property_keys: Vec<String>,
    ) -> Result<()> {
        let status = self
            .client
            .clone()
            .alter_database(AlterDatabaseRequest {
                base: Some(MsgBase::new(MsgType::AlterDatabase)),
                db_name: db_name.to_owned(),
                delete_keys: property_keys,
                ..Default::default()
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }

    pub async fn use_database(&mut self, db_name: &str) -> Result<()> {
        if self.db_name == db_name {
            return Result::Ok(());
        }

        let db = db_name.to_string();
        if !db_name.is_empty() {
            let dbs = self.list_databases().await?;

            if !dbs.contains(&db) {
                return Result::Err(Error::Unexpected(format!(
                    "Database with the name '{db_name}' does not exist."
                )));
            }
        }

        self.db_name = db.clone();
        self.collection_cache.db_name = db;
        Result::Ok(())
    }

    pub async fn drop_database(&self, db_name: &str) -> Result<()> {
        let status = self
            .client
            .clone()
            .drop_database(DropDatabaseRequest {
                base: Some(MsgBase::new(MsgType::DropDatabase)),
                db_name: db_name.to_owned(),
            })
            .await?
            .into_inner();

        status_to_result(&Some(status))
    }
}
