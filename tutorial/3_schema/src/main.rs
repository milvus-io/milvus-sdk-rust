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

use milvus::v2::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<()> {
    let uri = std::env::var("MILVUS_URI").unwrap_or_else(|_| "http://localhost:19530".to_owned());
    let token = std::env::var("MILVUS_TOKEN").unwrap_or_else(|_| "root:Milvus".to_owned());
    let prefix = tutorial_collection_prefix();
    let collections = [
        format!("{prefix}_SCALAR"),
        format!("{prefix}_VECTOR"),
        format!("{prefix}_STRUCT"),
    ];

    let config = ConnectConfig::new().uri(uri).token(token);
    // ClientV2::new connects to the configured endpoint and authenticates the schema operations.
    println!("Calling ClientV2::new: connect to Milvus");
    let client = ClientV2::new(&config).await?;
    println!("ClientV2::new completed");

    // Capture the tutorial result so cleanup still runs if a later operation fails.
    let tutorial_result = demonstrate_schemas(&client, &collections).await;
    let cleanup_result = cleanup_collections(&client, &collections).await;

    if let Err(error) = &cleanup_result {
        eprintln!("Failed to clean up the schema tutorial: {error}");
    }
    tutorial_result?;
    cleanup_result?;
    Ok(())
}

async fn demonstrate_schemas(client: &ClientV2, collections: &[String; 3]) -> Result<()> {
    create_and_describe(
        client,
        &collections[0],
        "Scalar and container data types",
        scalar_schema(),
    )
    .await?;
    create_and_describe(
        client,
        &collections[1],
        "Vector data types",
        vector_schema(),
    )
    .await?;
    create_and_describe(
        client,
        &collections[2],
        "Sparse, Int8, and struct data types",
        struct_schema(),
    )
    .await?;
    Ok(())
}

fn scalar_schema() -> CollectionSchema {
    CollectionSchema::new()
        .description("Scalar and container field examples")
        .enable_dynamic_field(false)
        .add_field(primary_key())
        .add_field(
            FieldSchema::new()
                .name("bool_value")
                .description("Boolean value; this field may be null")
                .data_type(DataType::Bool)
                .nullable(true),
        )
        .add_field(
            FieldSchema::new()
                .name("int8_value")
                .data_type(DataType::Int8)
                .default_value(DefaultValue::Int32(0)),
        )
        .add_field(
            FieldSchema::new()
                .name("int16_value")
                .data_type(DataType::Int16),
        )
        .add_field(
            FieldSchema::new()
                .name("int32_value")
                .data_type(DataType::Int32),
        )
        .add_field(
            FieldSchema::new()
                .name("int64_value")
                .data_type(DataType::Int64),
        )
        .add_field(
            FieldSchema::new()
                .name("float_value")
                .data_type(DataType::Float),
        )
        .add_field(
            FieldSchema::new()
                .name("double_value")
                .data_type(DataType::Double),
        )
        .add_field(
            FieldSchema::new()
                .name("varchar_value")
                .description("Variable-length UTF-8 text")
                .data_type(DataType::VarChar)
                .max_length(512),
        )
        .add_field(
            FieldSchema::new()
                .name("json_value")
                .data_type(DataType::Json),
        )
        .add_field(
            FieldSchema::new()
                .name("geometry_value")
                .description("Geometry encoded as Well-Known Text")
                .data_type(DataType::Geometry),
        )
        .add_field(
            FieldSchema::new()
                .name("timestamp_value")
                .description("Timestamp with time zone")
                .data_type(DataType::Timestamptz),
        )
        .add_field(
            FieldSchema::new()
                .name("int64_array")
                .description("At most 32 Int64 elements")
                .data_type(DataType::Array)
                .element_type(DataType::Int64)
                .max_capacity(32),
        )
}

fn vector_schema() -> CollectionSchema {
    CollectionSchema::new()
        .description("Dense, binary, and sparse vector examples")
        .enable_dynamic_field(false)
        .add_field(primary_key())
        .add_field(
            FieldSchema::new()
                .name("float_vector")
                .data_type(DataType::FloatVector)
                .dimension(8),
        )
        .add_field(
            FieldSchema::new()
                .name("binary_vector")
                .description("A 64-bit binary vector")
                .data_type(DataType::BinaryVector)
                .dimension(64),
        )
        .add_field(
            FieldSchema::new()
                .name("float16_vector")
                .data_type(DataType::Float16Vector)
                .dimension(8),
        )
        .add_field(
            FieldSchema::new()
                .name("bfloat16_vector")
                .data_type(DataType::BFloat16Vector)
                .dimension(8),
        )
}

fn struct_schema() -> CollectionSchema {
    let events = StructFieldSchema::new()
        .name("events")
        .description("An entity may contain up to 16 event records")
        .max_capacity(16)
        .add_field(
            FieldSchema::new()
                .name("label")
                .data_type(DataType::VarChar)
                .max_length(128),
        )
        .add_field(
            FieldSchema::new()
                .name("position")
                .data_type(DataType::Int32),
        )
        .add_field(
            FieldSchema::new()
                .name("embedding")
                .data_type(DataType::FloatVector)
                .dimension(8),
        );

    CollectionSchema::new()
        .description("Sparse, Int8 vector, and struct-array examples")
        .enable_dynamic_field(false)
        .add_field(primary_key())
        .add_field(
            FieldSchema::new()
                .name("sparse_vector")
                .description("Sparse vectors do not declare a fixed dimension")
                .data_type(DataType::SparseFloatVector),
        )
        .add_field(
            FieldSchema::new()
                .name("int8_vector")
                .data_type(DataType::Int8Vector)
                .dimension(8),
        )
        .add_struct_field(events)
}

fn primary_key() -> FieldSchema {
    FieldSchema::new()
        .name("id")
        .description("Every collection requires exactly one primary key")
        .data_type(DataType::Int64)
        .primary_key(true)
}

async fn create_and_describe(
    client: &ClientV2,
    collection: &str,
    heading: &str,
    schema: CollectionSchema,
) -> Result<()> {
    println!("\n{heading}\nCreating {collection:?}");
    // create_collection persists the supplied `schema` under `collection_name`; `description`
    // attaches human-readable collection metadata.
    println!("Calling create_collection: create {collection:?}");
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .description(heading)
                .schema(schema)
                .build()?,
        )
        .await?;
    println!("create_collection completed");

    // describe_collection reads the schema back from Milvus so the tutorial can inspect the
    // server's representation of every field and struct sub-field.
    println!("Calling describe_collection: read back {collection:?}");
    let response = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(collection)
                .build()?,
        )
        .await?;
    println!("describe_collection completed");
    let schema = response.description().get_schema();

    for field in schema.get_fields() {
        println!(
            "  field={:<18} type={:?}{}{}{}",
            field.get_name(),
            field.get_data_type(),
            dimension_description(field),
            element_description(field),
            if field.is_primary_key() {
                " primary_key"
            } else {
                ""
            }
        );
    }
    for field in schema.get_struct_fields() {
        println!(
            "  field={:<18} type=Struct max_capacity={}",
            field.get_name(),
            field.get_max_capacity()
        );
        for sub_field in field.get_fields() {
            println!(
                "    sub_field={:<14} type={:?}{}",
                sub_field.get_name(),
                sub_field.get_data_type(),
                dimension_description(sub_field)
            );
        }
    }
    Ok(())
}

fn dimension_description(field: &FieldSchema) -> String {
    let dimension = field.get_dimension();
    if dimension == 0 {
        String::new()
    } else {
        format!(" dim={dimension}")
    }
}

fn element_description(field: &FieldSchema) -> String {
    field
        .get_element_type()
        .map_or_else(String::new, |element| {
            format!(
                " element_type={element:?} max_capacity={}",
                field.get_max_capacity()
            )
        })
}

async fn cleanup_collections(client: &ClientV2, collections: &[String]) -> Result<()> {
    for collection in collections {
        // has_collection checks whether this tutorial resource exists before cleanup.
        println!("Calling has_collection: check {collection:?}");
        let exists = client
            .has_collection(
                HasCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?
            .exists();
        println!("has_collection completed");
        if !exists {
            continue;
        }
        println!("\nDropping tutorial collection {collection:?}");
        // drop_collection permanently removes this schema-only tutorial collection.
        println!("Calling drop_collection: remove {collection:?}");
        client
            .drop_collection(
                DropCollectionRequest::builder()
                    .collection_name(collection)
                    .build()?,
            )
            .await?;
        println!("drop_collection completed");
    }
    Ok(())
}

fn tutorial_collection_prefix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("RUST_V2_SCHEMA_{timestamp}_{}", std::process::id())
}
