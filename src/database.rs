//! Database management functionality for Milvus Rust SDK.
//!
//! This module provides comprehensive database operations including creation, deletion,
//! modification, and querying of databases in Milvus. It supports database-level
//! properties configuration and database switching functionality.
//!
//! # Examples
//!
//! ```rust
//! use milvus::client::Client;
//! use milvus::database::CreateDbOptions;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("localhost:19530").await?;
//!     
//!     // Create a database with custom properties
//!     let options = CreateDbOptions::new()
//!         .replica_number(3)
//!         .diskquota_mb(1024)
//!         .max_collections(100);
//!     
//!     client.create_database("my_database", Some(options)).await?;
//!     
//!     // Switch to the database
//!     client.using_database("my_database").await?;
//!     
//!     // List all databases
//!     let databases = client.list_databases().await?;
//!     println!("Available databases: {:?}", databases);
//!     
//!     Ok(())
//! }
//! ```

use crate::client::{Client, CombinedInterceptor, DbInterceptor};
use crate::collection::CollectionCache;
use crate::proto::common::{KeyValuePair, MsgBase, MsgType};
use crate::proto::milvus::milvus_service_client::MilvusServiceClient;
use crate::utils::status_to_result;
use crate::{error::*, proto};

/// Configuration options for creating a new database.
///
/// This struct provides a builder pattern for configuring database properties
/// such as replica count, resource groups, disk quota, and access controls.
///
/// # Examples
///
/// ```rust
/// use milvus::database::CreateDbOptions;
///
/// let options = CreateDbOptions::new()
///     .replica_number(3)
///     .diskquota_mb(1024)
///     .max_collections(100)
///     .force_deny_writing(false)
///     .force_deny_reading(false);
/// ```
#[derive(Debug, Clone)]
pub struct CreateDbOptions {
    /// Number of replicas for the database
    replica_number: Option<i32>,
    /// Resource groups dedicated to the database
    resource_groups: Option<Vec<String>>,
    /// Disk quota allocated to the database in megabytes (MB)
    diskquota_mb: Option<i32>,
    /// Maximum number of collections allowed in the database
    max_collections: Option<i32>,
    /// Whether to deny all write operations in the database
    force_deny_writing: Option<bool>,
    /// Whether to deny all read operations in the database
    force_deny_reading: Option<bool>,
}

/// Type alias for database properties, currently equivalent to CreateDbOptions
type DbProperties = CreateDbOptions;

impl Default for CreateDbOptions {
    fn default() -> Self {
        Self {
            replica_number: None,
            resource_groups: None,
            diskquota_mb: None,
            max_collections: None,
            force_deny_writing: None,
            force_deny_reading: None,
        }
    }
}

impl CreateDbOptions {
    /// Creates a new `CreateDbOptions` instance with default values.
    ///
    /// All properties are set to `None` by default, meaning they will use
    /// the server's default values.
    ///
    /// # Returns
    ///
    /// A new `CreateDbOptions` instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of replicas for the database.
    ///
    /// # Arguments
    ///
    /// * `replica_number` - The number of replicas to maintain
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn replica_number(mut self, replica_number: i32) -> Self {
        self.replica_number = Some(replica_number);
        self
    }

    /// Sets the resource groups dedicated to the database.
    ///
    /// # Arguments
    ///
    /// * `resource_groups` - Vector of resource group names
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn resource_groups(mut self, resource_groups: Vec<String>) -> Self {
        self.resource_groups = Some(resource_groups);
        self
    }

    /// Sets the disk quota allocated to the database in megabytes.
    ///
    /// # Arguments
    ///
    /// * `diskquota_mb` - Disk quota in megabytes
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn diskquota_mb(mut self, diskquota_mb: i32) -> Self {
        self.diskquota_mb = Some(diskquota_mb);
        self
    }

    /// Sets the maximum number of collections allowed in the database.
    ///
    /// # Arguments
    ///
    /// * `max_collections` - Maximum number of collections
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn max_collections(mut self, max_collections: i32) -> Self {
        self.max_collections = Some(max_collections);
        self
    }

    /// Sets whether to deny all write operations in the database.
    ///
    /// # Arguments
    ///
    /// * `force_deny_writing` - Whether to deny write operations
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn force_deny_writing(mut self, force_deny_writing: bool) -> Self {
        self.force_deny_writing = Some(force_deny_writing);
        self
    }

    /// Sets whether to deny all read operations in the database.
    ///
    /// # Arguments
    ///
    /// * `force_deny_reading` - Whether to deny read operations
    ///
    /// # Returns
    ///
    /// Returns `self` for method chaining.
    pub fn force_deny_reading(mut self, force_deny_reading: bool) -> Self {
        self.force_deny_reading = Some(force_deny_reading);
        self
    }
}

/// Response structure for database description operations.
///
/// Contains detailed information about a database including its name, ID,
/// creation timestamp, and properties.
#[derive(Debug)]
pub struct DescribeDbResponse {
    /// The name of the database
    pub db_name: String,
    /// The unique identifier of the database
    pub db_id: i64,
    /// The timestamp when the database was created
    pub created_timestamp: u64,
    /// The properties/configuration of the database
    pub properties: Vec<KeyValuePair>,
}

impl Client {
    /// Alters the properties of an existing database.
    ///
    /// This method allows you to modify database properties such as replica count,
    /// disk quota, and access controls. The operation will update the database
    /// with the new properties while preserving existing ones not specified.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to alter
    /// * `options` - The new properties to apply to the database
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use milvus::database::CreateDbOptions;
    ///
    /// let options = CreateDbOptions::new()
    ///     .replica_number(5)
    ///     .diskquota_mb(2048);
    ///     
    /// client.alter_database_properties("my_database", options).await?;
    /// ```
    pub async fn alter_database_properties<S: Into<String>>(
        &self,
        db_name: S,
        options: DbProperties,
    ) -> Result<()> {
        let db_name = db_name.into();
        let properties = prepare_properties(options);
        let db_id = self
            .describe_database(db_name.clone())
            .await?
            .db_id
            .to_string();
        let res = self
            .client
            .clone()
            .alter_database(proto::milvus::AlterDatabaseRequest {
                base: Some(MsgBase::new(MsgType::AlterDatabase)),
                db_name,
                db_id,
                properties,
                delete_keys: Vec::new(),
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))
    }

    /// Drops specific properties from a database.
    ///
    /// This method removes specified properties from the database configuration,
    /// reverting them to their default values.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database
    /// * `property_keys` - Vector of property keys to remove
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let property_keys = vec![
    ///     "database.replica.number".to_string(),
    ///     "database.diskQuota.mb".to_string(),
    /// ];
    ///
    /// client.drop_database_properties("my_database", property_keys).await?;
    /// ```
    pub async fn drop_database_properties<S: Into<String>>(
        &self,
        db_name: S,
        property_keys: Vec<String>,
    ) -> Result<()> {
        let db_name = db_name.into();
        let db_info = self.describe_database(db_name.clone()).await?;
        let db_id = db_info.db_id.to_string();
        let properties = db_info.properties;

        let res = self
            .client
            .clone()
            .alter_database(proto::milvus::AlterDatabaseRequest {
                base: Some(MsgBase::new(MsgType::AlterDatabase)),
                db_name,
                db_id,
                properties,
                delete_keys: property_keys,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))
    }

    /// Creates a new database with specified properties.
    ///
    /// This method creates a new database in Milvus with the given name and
    /// optional configuration properties. If no options are provided, the
    /// database will be created with default settings.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to create
    /// * `options` - Optional configuration properties for the database
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use milvus::database::CreateDbOptions;
    ///
    /// // Create database with default settings
    /// client.create_database("my_database", None).await?;
    ///
    /// // Create database with custom properties
    /// let options = CreateDbOptions::new()
    ///     .replica_number(3)
    ///     .diskquota_mb(1024)
    ///     .max_collections(100);
    ///     
    /// client.create_database("my_database", Some(options)).await?;
    /// ```
    pub async fn create_database<S: Into<String>>(
        &self,
        db_name: S,
        options: Option<CreateDbOptions>,
    ) -> Result<()> {
        let db_name: String = db_name.into();

        let properties: Vec<KeyValuePair> = prepare_properties(options.unwrap_or_default());

        let res = self
            .client
            .clone()
            .create_database(proto::milvus::CreateDatabaseRequest {
                base: Some(MsgBase::new(MsgType::CreateDatabase)),
                db_name,
                properties,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;

        Ok(())
    }

    /// Describes a database and returns its detailed information.
    ///
    /// This method retrieves comprehensive information about a database including
    /// its name, ID, creation timestamp, and all configured properties.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to describe
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing `DescribeDbResponse` with database details.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db_info = client.describe_database("my_database").await?;
    /// println!("Database ID: {}", db_info.db_id);
    /// println!("Created at: {}", db_info.created_timestamp);
    /// println!("Properties: {:?}", db_info.properties);
    /// ```
    pub async fn describe_database<S: Into<String>>(
        &self,
        db_name: S,
    ) -> Result<DescribeDbResponse> {
        let db_name = db_name.into();
        let res = self
            .client
            .clone()
            .describe_database(proto::milvus::DescribeDatabaseRequest {
                base: Some(MsgBase::new(MsgType::DescribeDatabase)),
                db_name,
            })
            .await?
            .into_inner();

        status_to_result(&res.status)?;

        Ok(DescribeDbResponse {
            db_name: res.db_name,
            db_id: res.db_id,
            created_timestamp: res.created_timestamp,
            properties: res.properties,
        })
    }

    /// Drops (deletes) a database and all its contents.
    ///
    /// This method permanently removes a database and all its collections,
    /// indexes, and data. This operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to drop
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// client.drop_database("my_database").await?;
    /// ```
    ///
    /// # Warning
    ///
    /// This operation is irreversible and will permanently delete all data
    /// in the database.
    pub async fn drop_database<S: Into<String>>(&self, db_name: S) -> Result<()> {
        let db_name = db_name.into();

        let res = self
            .client
            .clone()
            .drop_database(proto::milvus::DropDatabaseRequest {
                base: Some(MsgBase::new(MsgType::DropDatabase)),
                db_name,
            })
            .await?
            .into_inner();

        status_to_result(&Some(res))?;

        Ok(())
    }

    /// Lists all databases in the Milvus instance.
    ///
    /// This method retrieves a list of all database names that exist in
    /// the current Milvus instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of database names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let databases = client.list_databases().await?;
    /// println!("Available databases: {:?}", databases);
    /// ```
    pub async fn list_databases(&self) -> Result<Vec<String>> {
        let res = self
            .client
            .clone()
            .list_databases(proto::milvus::ListDatabasesRequest {
                base: Some(MsgBase::new(MsgType::ListDatabases)),
            })
            .await?
            .into_inner();
        status_to_result(&res.status)?;
        Ok(res.db_names)
    }

    /// Switches the client to work with a different database.
    ///
    /// This method changes the current database context for the client.
    /// After switching, all subsequent operations (collections, queries, etc.)
    /// will be performed in the specified database. The method also clears
    /// the collection cache to ensure fresh data from the new database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - The name of the database to switch to
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Switch to a different database
    /// client.using_database("my_database").await?;
    ///
    /// // Now all operations will be performed in "my_database"
    /// let collections = client.list_collections().await?;
    /// ```
    ///
    /// # Note
    ///
    /// This method clears the collection cache to ensure that subsequent
    /// operations reflect the current state of the new database.
    pub async fn using_database<S: Into<String>>(&mut self, db_name: S) -> Result<()> {
        let db_name = db_name.into();

        // Clear schema cache
        self.collection_cache.clear();

        // Update database name
        self.db_name = Some(db_name.clone());

        // Create new client with database interceptor
        let db_interceptor = DbInterceptor {
            db_name: Some(db_name),
        };
        let combined_interceptor = CombinedInterceptor {
            auth: self.auth_interceptor.clone(),
            db: db_interceptor,
        };

        let new_client =
            MilvusServiceClient::with_interceptor(self.channel.clone(), combined_interceptor);
        self.client = new_client.clone();
        self.collection_cache = CollectionCache::new(new_client);

        Ok(())
    }
}

/// Converts `CreateDbOptions` to a vector of `KeyValuePair` for gRPC communication.
///
/// This function transforms the builder-style options into the format expected
/// by the Milvus server API.
///
/// # Arguments
///
/// * `options` - The database options to convert
///
/// # Returns
///
/// A vector of `KeyValuePair` representing the database properties
fn prepare_properties(options: CreateDbOptions) -> Vec<KeyValuePair> {
    let mut properties: Vec<KeyValuePair> = Vec::new();

    if let Some(replica_number) = options.replica_number {
        properties.push(KeyValuePair {
            key: "database.replica.number".to_string(),
            value: replica_number.to_string(),
        });
    }

    if let Some(resource_groups) = options.resource_groups {
        properties.push(KeyValuePair {
            key: "database.resource_groups".to_string(),
            value: resource_groups.join(","),
        });
    }

    if let Some(diskquota_mb) = options.diskquota_mb {
        properties.push(KeyValuePair {
            key: "database.diskQuota.mb".to_string(),
            value: diskquota_mb.to_string(),
        });
    }

    if let Some(max_collections) = options.max_collections {
        properties.push(KeyValuePair {
            key: "database.max.collections".to_string(),
            value: max_collections.to_string(),
        });
    }

    if let Some(force_deny_writing) = options.force_deny_writing {
        properties.push(KeyValuePair {
            key: "database.force.deny.writing".to_string(),
            value: force_deny_writing.to_string(),
        });
    }

    if let Some(force_deny_reading) = options.force_deny_reading {
        properties.push(KeyValuePair {
            key: "database.force.deny.reading".to_string(),
            value: force_deny_reading.to_string(),
        });
    }

    properties
}
