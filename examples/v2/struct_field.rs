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

mod utils;

use milvus::v2 as sdk;
use milvus::v2::error::Result;
use milvus::v2::prelude::*;
use rand::Rng;
use serde_json::{json, Value};
use utils::*;

const COLLECTION: &str = "RUST_V2_STRUCT_FIELD";
const ID: &str = "id";
const NAME: &str = "film_name";
const CLIPS: &str = "clips";
const SIMPLIFY: &str = "simplify_clips";
const FRAME: &str = "frame_number";
const DESC: &str = "clip_desc";
const FLOAT: &str = "clip_float_embedding";
const BINARY: &str = "clip_binary_embedding";
const INT8: &str = "clip_int8_embedding";
const SCORE: &str = "score";
const LANG: &str = "lang";
const TAGS: &str = "tags";
const NOTE: &str = "note";
const MARKS: &str = "marks";
const METADATA: &str = "metadata";
const RATING: &str = "rating";
const TAG: &str = "tag";
const FLOAT_DIM: usize = 128;
const BINARY_DIM: usize = 32;
const INT8_DIM: usize = 32;
const SIMPLIFY_DIM: usize = 32;
const EXTRA_ROW_ID: i64 = 5000;

fn combine(struct_field: &str, sub_field: &str) -> String {
    format!("{struct_field}[{sub_field}]")
}

fn as_float_vector(value: &Value) -> Option<Vec<f32>> {
    value.as_array().map(|vector| {
        vector
            .iter()
            .filter_map(Value::as_f64)
            .map(|v| v as f32)
            .collect()
    })
}

async fn describe_collection(client: &ClientV2) -> Result<()> {
    let described = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    let schema = described.description().get_schema();
    for field in schema.get_fields() {
        println!("  field: {}", field.get_name());
    }
    for struct_field in schema.get_struct_fields() {
        println!("  struct: {}", struct_field.get_name());
        for sub_field in struct_field.get_fields() {
            println!(
                "    {}: {:?}, nullable={}, element_nullable={}",
                sub_field.get_name(),
                sub_field.get_data_type(),
                sub_field.is_nullable(),
                sub_field.is_element_nullable()
            );
        }
    }
    Ok(())
}

async fn create_collection(client: &ClientV2) -> Result<()> {
    let clips = sdk::StructFieldSchema::new()
        .name(CLIPS)
        .description("clips of a film")
        .max_capacity(100)
        .nullable(true)
        .add_field(
            sdk::FieldSchema::new()
                .name(FRAME)
                .description("from which frame this clip begin")
                .data_type(sdk::DataType::Int32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(DESC)
                .description("description of a clip")
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(FLOAT)
                .description("float embedding of a clip")
                .data_type(sdk::DataType::FloatVector)
                .dimension(FLOAT_DIM as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(BINARY)
                .description("binary embedding of a clip")
                .data_type(sdk::DataType::BinaryVector)
                .dimension(BINARY_DIM as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(INT8)
                .description("int8 embedding of a clip")
                .data_type(sdk::DataType::Int8Vector)
                .dimension(INT8_DIM as u32),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(SCORE)
                .description("score of a clip, element-level null allowed")
                .data_type(sdk::DataType::Int64)
                .nullable(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(LANG)
                .description("language of a clip, element-level null allowed")
                .data_type(sdk::DataType::VarChar)
                .max_length(16)
                .nullable(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(TAGS)
                .description("tags of a clip, a nested array with null inner values allowed")
                .data_type(sdk::DataType::Array)
                .element_type(sdk::DataType::VarChar)
                .max_capacity(8)
                .max_length(16)
                .nullable(true)
                .element_nullable(true),
        );
    let simplify = sdk::StructFieldSchema::new()
        .name(SIMPLIFY)
        .description("simplify clips")
        .max_capacity(100)
        .add_field(
            sdk::FieldSchema::new()
                .name(FLOAT)
                .description("clip has been simplified")
                .data_type(sdk::DataType::FloatVector)
                .dimension(SIMPLIFY_DIM as u32),
        );
    let schema = sdk::CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            sdk::FieldSchema::new()
                .name(ID)
                .data_type(sdk::DataType::Int64)
                .primary_key(true),
        )
        .add_field(
            sdk::FieldSchema::new()
                .name(NAME)
                .data_type(sdk::DataType::VarChar)
                .max_length(1024),
        )
        .add_struct_field(clips)
        .add_struct_field(simplify);
    drop_collection(client, COLLECTION).await;
    client
        .create_collection(
            sdk::request::collection::CreateCollectionRequest::builder()
                .collection_name(COLLECTION)
                .schema(schema)
                .build()?,
        )
        .await?;

    // The vector sub-fields are non-element-nullable, so MAX_SIM metrics are allowed for the
    // embedding-list searches below; the simplify field uses an element-level L2 metric.
    client
        .create_index(
            sdk::request::index::CreateIndexRequest::builder()
                .collection_name(COLLECTION)
                .index_params(vec![
                    IndexParam::new()
                        .field_name(combine(CLIPS, FLOAT))
                        .index_name("index_float")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::MaxSimIp),
                    IndexParam::new()
                        .field_name(combine(CLIPS, BINARY))
                        .index_name("index_binary")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::MaxSimHamming),
                    IndexParam::new()
                        .field_name(combine(CLIPS, INT8))
                        .index_name("index_int8")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::MaxSimL2),
                    IndexParam::new()
                        .field_name(combine(SIMPLIFY, FLOAT))
                        .index_name("index_simplify")
                        .index_type(IndexType::Hnsw)
                        .metric_type(MetricType::L2),
                ])
                .build()?,
        )
        .await?;
    client
        .load_collection(
            sdk::request::collection::LoadCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Collection created: {COLLECTION}");
    describe_collection(client).await?;
    Ok(())
}

fn build_base_row(id: i64, with_clips: bool) -> Value {
    let mut rng = rand::thread_rng();
    let clips = if with_clips {
        Value::Array(
            (0..5)
                .map(|_| {
                    json!({
                        FRAME: rng.gen_range(0..10000),
                        DESC: format!("clip_description_{id}"),
                        FLOAT: float_vector(FLOAT_DIM),
                        BINARY: binary_vector(BINARY_DIM),
                        INT8: int8_vector(INT8_DIM),
                        SCORE: rng.gen_range(0..100),
                        LANG: "en",
                        TAGS: ["a", "b"],
                    })
                })
                .collect(),
        )
    } else {
        // the clips struct field is nullable; store a null row for it
        Value::Null
    };
    let simplify = Value::Array(
        (0..2)
            .map(|_| json!({FLOAT: float_vector(SIMPLIFY_DIM)}))
            .collect(),
    );
    json!({ID: id, NAME: format!("film_{id}"), CLIPS: clips, SIMPLIFY: simplify})
}

// Rows demonstrating element-level null: a missing or null sub-field key is stored as a null
// element for the element-nullable sub-fields (score/lang); nested arrays support null inner
// values (tags: ["c", null]) and empty arrays; the whole struct row can be null (clips: null).
// The ids stay outside the 0..1999 range used by insert_data so each primary key is inserted
// exactly once.
fn build_element_null_rows() -> Vec<Value> {
    let row2000 = json!({
        ID: 2000,
        NAME: "film_2000",
        CLIPS: [
            {
                FRAME: 0, DESC: "clip_description_2000",
                FLOAT: float_vector(FLOAT_DIM), BINARY: binary_vector(BINARY_DIM), INT8: int8_vector(INT8_DIM),
                SCORE: 1, LANG: "en", TAGS: ["a", "b"],
            },
            {
                FRAME: 1, DESC: "clip_description_2000",
                FLOAT: float_vector(FLOAT_DIM), BINARY: binary_vector(BINARY_DIM), INT8: int8_vector(INT8_DIM),
                SCORE: 2, LANG: null, TAGS: ["c", null],
            },
            {
                FRAME: 2, DESC: "clip_description_2000",
                FLOAT: float_vector(FLOAT_DIM), BINARY: binary_vector(BINARY_DIM), INT8: int8_vector(INT8_DIM),
                SCORE: 3, TAGS: [],
            },
        ],
        SIMPLIFY: [],
    });
    let row2001 = json!({ID: 2001, NAME: "film_2001", CLIPS: [], SIMPLIFY: []});
    let row2002 = json!({ID: 2002, NAME: "film_2002", CLIPS: Value::Null, SIMPLIFY: []});
    vec![row2000, row2001, row2002]
}

// Query rows that carry element-level nulls and verify null elements / null struct rows decode.
async fn verify_element_null(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    println!("Verify element-level null struct fields on rows 2000/2001/2002");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id in [2000, 2001, 2002]")
                .output_fields([ID, CLIPS])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    for row in query.results().rows()? {
        let id = row.get_i64(ID)?;
        let clips = row.get(CLIPS)?;
        println!("query result id={id}, clips={clips:?}");
        if id == 2002 && !matches!(clips, ResultValue::Null) {
            return Err(Error::Unexpected(format!(
                "expected null clips for id=2002, got: {clips:?}"
            )));
        }
    }
    println!("Element-level null verification passed");
    Ok(())
}

async fn insert_data(client: &ClientV2, row_count: i64) -> Result<()> {
    const BATCH_SIZE: usize = 300;
    let mut inserted_count = 0;
    while inserted_count < row_count {
        let next_batch = ((row_count - inserted_count) as usize).min(BATCH_SIZE);
        let mut rows = Vec::with_capacity(next_batch);
        for index in 0..next_batch {
            let id = inserted_count + index as i64;
            // insert null clips struct for every 100th row
            rows.push(build_base_row(id, id % 100 != 0));
        }
        let insert = client
            .insert(
                sdk::request::dml::InsertRequest::builder()
                    .collection_name(COLLECTION)
                    .rows(rows)
                    .build()?,
            )
            .await?;
        inserted_count += insert.insert_count() as i64;
        println!("Inserted row count: {}", insert.insert_count());
    }
    let count = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .output_fields(["count(*)"])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("{} rows persisted", query_count(count.results())?);
    Ok(())
}

async fn query(client: &ClientV2, filter: &str) -> Result<QueryResponse> {
    println!("===================================================");
    println!("Query with filter expression: {filter}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .limit(3)
                .output_fields([CLIPS, SIMPLIFY])
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    for row in query.results().rows()? {
        println!("\t{}", serde_json::Value::Object(row.to_entity_row()?));
    }
    Ok(query)
}

async fn search(
    client: &ClientV2,
    anns_field: &str,
    vectors: SearchVectors,
    output_fields: Vec<String>,
) -> Result<()> {
    println!("===================================================");
    println!("Search on field '{anns_field}' in struct '{CLIPS}'");
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(anns_field.to_owned())
                .vectors(vectors)
                .output_fields(output_fields)
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    for (index, result) in response.results().iter().enumerate() {
        println!("Results of No.{index} embedding list");
        for row in result.rows()? {
            println!("\t{}", serde_json::Value::Object(row.to_entity_row()?));
        }
    }
    Ok(())
}

// Embedding-list search builds one EmbeddingList per query row from the queried struct elements
// and searches the given struct vector sub-field with a MAX_SIM metric.
async fn embedding_list_search<T>(
    client: &ClientV2,
    anns_field: &str,
    filter: &str,
    extract: impl Fn(&serde_json::Map<String, Value>) -> Option<Vec<T>>,
    make_list: impl Fn(Vec<Vec<T>>) -> EmbeddingList,
) -> Result<()> {
    let query = query(client, filter).await?;
    let mut lists = Vec::new();
    for row in query.results().rows()? {
        let ResultValue::Struct(values) = row.get(CLIPS)? else {
            // this row holds a null (nullable) struct field, so it has no embeddings to search on
            continue;
        };
        let mut vectors = Vec::new();
        for value in values {
            if let Some(vector) = extract(value) {
                vectors.push(vector);
            }
        }
        lists.push(make_list(vectors));
    }
    search(
        client,
        anns_field,
        SearchVectors::EmbeddingLists(lists),
        vec![
            NAME.to_owned(),
            combine(CLIPS, FRAME),
            combine(CLIPS, DESC),
            combine(SIMPLIFY, FLOAT),
        ],
    )
    .await
}

async fn search_float_vector_field(client: &ClientV2, filter: &str) -> Result<()> {
    embedding_list_search(
        client,
        &combine(CLIPS, FLOAT),
        filter,
        |element| as_float_vector(element.get(FLOAT)?),
        |vectors| EmbeddingList::new().vectors(vectors),
    )
    .await
}

async fn search_binary_vector_field(client: &ClientV2, filter: &str) -> Result<()> {
    embedding_list_search(
        client,
        &combine(CLIPS, BINARY),
        filter,
        |element| {
            element.get(BINARY)?.as_array().map(|vector| {
                vector
                    .iter()
                    .filter_map(Value::as_u64)
                    .map(|v| v as u8)
                    .collect()
            })
        },
        |vectors| EmbeddingList::new().binary_vectors(vectors),
    )
    .await
}

async fn search_int8_vector_field(client: &ClientV2, filter: &str) -> Result<()> {
    embedding_list_search(
        client,
        &combine(CLIPS, INT8),
        filter,
        |element| {
            element.get(INT8)?.as_array().map(|vector| {
                vector
                    .iter()
                    .filter_map(Value::as_i64)
                    .map(|v| v as i8)
                    .collect()
            })
        },
        |vectors| EmbeddingList::new().int8_vectors(vectors),
    )
    .await
}

// Element-level search is to "find similar structs in the struct field", each item of result
// corresponds to one struct in the struct field, with an elementOffset value indicating which
// struct it is.
async fn element_level_search(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    println!("Element-level search on field '{FLOAT}' in struct '{SIMPLIFY}'");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id == 5")
                .limit(1)
                .output_fields([SIMPLIFY])
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    let row = query.results().rows()?.next().expect("id==5 must exist");
    let ResultValue::Struct(structs) = row.get(SIMPLIFY)? else {
        return Err(Error::Unexpected(
            "id==5 simplify_clips must not be null".into(),
        ));
    };
    // each row of SIMPLIFY_STRUCT_FIELD contains two structs, and each struct has a
    // FLOAT_VECTOR_FIELD, so we will have 2 vectors to search for each row
    let mut query_vectors = Vec::new();
    for struct_value in structs {
        query_vectors.push(
            struct_value
                .get(FLOAT)
                .and_then(as_float_vector)
                .ok_or_else(|| {
                    Error::Unexpected("simplify struct must carry a float vector".into())
                })?,
        );
    }
    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(combine(SIMPLIFY, FLOAT))
                .vectors(SearchVectors::Float(query_vectors))
                .metric_type(MetricType::L2)
                .output_fields([NAME])
                .limit(3)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    for (index, result) in response.results().iter().enumerate() {
        println!("Results of No.{index} query element");
        for row in result.rows()? {
            println!(
                "\t{}, elementOffset={:?}",
                serde_json::Value::Object(row.to_entity_row()?),
                row.element_offset()
            );
        }
    }
    Ok(())
}

async fn element_level_query(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    let filter = format!("element_filter({CLIPS}, $[{FRAME}] < 2000)");
    println!("Element-level query with filter expression: {filter}");
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(filter)
                .limit(5)
                .output_fields(vec![
                    ID.to_owned(),
                    NAME.to_owned(),
                    combine(CLIPS, FRAME),
                    combine(CLIPS, DESC),
                ])
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    // the server evaluates the filter at element granularity and returns element_indices;
    // the SDK expands one entity into one result per matched element, each carrying the
    // elementOffset of the matched element within the struct-array field
    for row in query.results().rows()? {
        println!(
            "\t{}, elementOffset={:?}",
            serde_json::Value::Object(row.to_entity_row()?),
            row.element_offset()
        );
    }
    Ok(())
}

fn build_metadata_struct_array() -> Value {
    json!([
        {RATING: 5, TAG: "favorite"},
        {RATING: 4, TAG: "classic"},
    ])
}

async fn add_collection_struct_field(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    println!("Add a new struct field to the existing collection");
    client
        .add_collection_struct_field(
            AddCollectionStructFieldRequest::builder()
                .collection_name(COLLECTION)
                .struct_field(
                    sdk::StructFieldSchema::new()
                        .name(METADATA)
                        .description("additional metadata for films")
                        .max_capacity(8)
                        .nullable(true)
                        .add_field(
                            sdk::FieldSchema::new()
                                .name(RATING)
                                .data_type(sdk::DataType::Int32),
                        )
                        .add_field(
                            sdk::FieldSchema::new()
                                .name(TAG)
                                .data_type(sdk::DataType::VarChar)
                                .max_length(128),
                        ),
                )
                .build()?,
        )
        .await?;
    println!("Added struct field: {METADATA}");
    describe_collection(client).await?;

    let existing = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id == 5")
                .output_fields([ID, METADATA])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Existing row after addCollectionStructField() - new field is null");
    for row in existing.results().rows()? {
        println!("\t{}", serde_json::Value::Object(row.to_entity_row()?));
    }

    let mut new_row = build_base_row(EXTRA_ROW_ID, true);
    new_row[METADATA] = build_metadata_struct_array();
    client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .row(new_row)
                .build()?,
        )
        .await?;

    let new_row_query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(format!("id == {EXTRA_ROW_ID}"))
                .output_fields([ID, NAME, METADATA])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("New row with the added struct field");
    for row in new_row_query.results().rows()? {
        println!("\t{}", serde_json::Value::Object(row.to_entity_row()?));
    }

    let projected = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter(format!("id == {EXTRA_ROW_ID}"))
                .output_fields([
                    ID,
                    NAME,
                    &combine(METADATA, RATING),
                    &combine(METADATA, TAG),
                ])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    println!("Projected subfields from the added struct field");
    for row in projected.results().rows()? {
        println!("\t{}", serde_json::Value::Object(row.to_entity_row()?));
    }
    Ok(())
}

// Evolve an existing struct field: add sub-fields (including a nested Array sub-field), insert
// rows carrying them, and drop one again. Sub-field add/drop use the bracketed `struct[sub]` path.
async fn evolve_struct_sub_field(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    println!("Evolve struct field '{CLIPS}' by adding sub-fields");

    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    sdk::FieldSchema::new()
                        .name(combine(CLIPS, NOTE))
                        .data_type(sdk::DataType::VarChar)
                        .max_length(64)
                        .nullable(true),
                )
                .build()?,
        )
        .await?;
    println!("Added struct sub-field {CLIPS}[{NOTE}]");

    client
        .add_collection_field(
            AddCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field(
                    sdk::FieldSchema::new()
                        .name(combine(CLIPS, MARKS))
                        .data_type(sdk::DataType::Array)
                        .element_type(sdk::DataType::Int32)
                        .max_capacity(4)
                        .nullable(true)
                        .element_nullable(true),
                )
                .build()?,
        )
        .await?;
    println!("Added nested Array struct sub-field {CLIPS}[{MARKS}]");

    let row = json!({
        ID: 6000,
        NAME: "film_6000",
        CLIPS: [{
            FRAME: 0,
            DESC: "clip_description_6000",
            FLOAT: float_vector(FLOAT_DIM),
            BINARY: binary_vector(BINARY_DIM),
            INT8: int8_vector(INT8_DIM),
            SCORE: 7,
            NOTE: "hello",
            MARKS: [1, 2],
        }],
        SIMPLIFY: [],
    });
    client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .row(row)
                .build()?,
        )
        .await?;

    let described = client
        .describe_collection(
            DescribeCollectionRequest::builder()
                .collection_name(COLLECTION)
                .build()?,
        )
        .await?;
    println!("Collection fields after sub-field evolution:");
    for struct_field in described.description().get_schema().get_struct_fields() {
        if struct_field.get_name() == CLIPS {
            for sub_field in struct_field.get_fields() {
                println!(
                    "  {}: {:?}, nullable={}, element_nullable={}",
                    sub_field.get_name(),
                    sub_field.get_data_type(),
                    sub_field.is_nullable(),
                    sub_field.is_element_nullable()
                );
            }
        }
    }

    client
        .drop_collection_field(
            DropCollectionFieldRequest::builder()
                .collection_name(COLLECTION)
                .field_name(combine(CLIPS, NOTE))
                .build()?,
        )
        .await?;
    println!("Dropped struct sub-field {CLIPS}[{NOTE}]");
    Ok(())
}

// Verify that a nullable struct field returns null data for the rows inserted with clips = null.
async fn verify_nullable_struct(client: &ClientV2) -> Result<()> {
    println!("===================================================");
    println!("Verify nullable struct field '{CLIPS}' returns null data");

    // rows with id % 100 == 0 were inserted with clips = null
    let query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id in [0, 100, 200, 300]")
                .output_fields([ID, NAME, CLIPS, SIMPLIFY])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    for row in query.results().rows()? {
        let id = row.get_i64(ID)?;
        let clips = row.get(CLIPS)?;
        let simplify_size = match row.get(SIMPLIFY)? {
            ResultValue::Struct(values) => values.len(),
            _ => 0,
        };
        println!("query result id={id}, clips={clips:?}, simplify_clips.size={simplify_size}");
        if id % 100 == 0 && !matches!(clips, ResultValue::Null) {
            return Err(Error::Unexpected(format!(
                "expected null clips for id={id}, got: {clips:?}"
            )));
        }
    }

    // take the simplify_clips vector of a null-clips row and search on it element-level;
    // the top-1 hit should be that row itself and its clips output must decode back to null
    let vector_query = client
        .query(
            QueryRequest::builder()
                .collection_name(COLLECTION)
                .filter("id == 100")
                .output_fields([SIMPLIFY])
                .consistency_level(sdk::ConsistencyLevel::Strong)
                .build()?,
        )
        .await?;
    let row = vector_query
        .results()
        .rows()?
        .next()
        .expect("id==100 must exist");
    let ResultValue::Struct(simplify_structs) = row.get(SIMPLIFY)? else {
        return Err(Error::Unexpected(
            "id==100 simplify_clips must not be null".into(),
        ));
    };
    let query_vector = simplify_structs[0]
        .get(FLOAT)
        .and_then(as_float_vector)
        .ok_or_else(|| Error::Unexpected("simplify struct must carry a float vector".into()))?;

    let response = client
        .search(
            SearchRequest::builder()
                .collection_name(COLLECTION)
                .vector_field(combine(SIMPLIFY, FLOAT))
                .vectors(SearchVectors::Float(vec![query_vector]))
                .metric_type(MetricType::L2)
                .output_fields([ID, CLIPS])
                .limit(5)
                .consistency_level(sdk::ConsistencyLevel::Bounded)
                .build()?,
        )
        .await?;
    let mut found_null_row = false;
    for result in response.results() {
        for row in result.rows()? {
            let id = row.get_i64(ID)?;
            let clips = row.get(CLIPS)?;
            println!("search result id={id}, clips={clips:?}");
            if id == 100 {
                if !matches!(clips, ResultValue::Null) {
                    return Err(Error::Unexpected(format!(
                        "expected null clips for id=100 in search result, got: {clips:?}"
                    )));
                }
                found_null_row = true;
            }
        }
    }
    if !found_null_row {
        return Err(Error::Unexpected(
            "the null-clips row id=100 was not returned by search".into(),
        ));
    }

    println!("Nullable struct verification passed");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Example start...");
    let client = client().await?;

    create_collection(&client).await?;
    insert_data(&client, 2000).await?;

    // insert rows demonstrating element-level null and null struct rows
    client
        .insert(
            sdk::request::dml::InsertRequest::builder()
                .collection_name(COLLECTION)
                .rows(build_element_null_rows())
                .build()?,
        )
        .await?;
    verify_element_null(&client).await?;
    verify_nullable_struct(&client).await?;

    search_float_vector_field(&client, "id in [5, 8]").await?;
    search_binary_vector_field(
        &client,
        &format!("MATCH_LEAST({CLIPS}, $[{FRAME}] < 2000, threshold=3)"),
    )
    .await?;
    search_int8_vector_field(&client, "id == 999").await?;
    element_level_search(&client).await?;
    element_level_query(&client).await?;

    add_collection_struct_field(&client).await?;
    evolve_struct_sub_field(&client).await?;

    drop_collection(&client, COLLECTION).await;
    Ok(())
}
