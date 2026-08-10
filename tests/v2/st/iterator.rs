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

use super::common;

const TOTAL_COUNT: usize = 1_000;
const ID_FIELD: &str = "id";
const AGE_FIELD: &str = "age";
const NAME_FIELD: &str = "name";
const VECTOR_FIELD: &str = "vec";

#[tokio::test]
async fn query_iterator_batches_and_limits() {
    let client = common::client().await;
    let collection = common::unique_collection_name("query_iterator");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    prepare_iterator_collection(&client, &collection).await;

    for reduce_stop_for_best in [true, false] {
        let rows = collect_query_rows(
            &client,
            &collection,
            "",
            128,
            Some(1_000),
            100,
            &["*"],
            reduce_stop_for_best,
        )
        .await;
        assert_eq!(rows.len(), 900);
        for row in &rows {
            assert_iterator_row(row);
        }
        let mut ids = rows
            .iter()
            .map(|row| row[ID_FIELD].as_i64().expect("primary key is an integer"))
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids, (100_i64..1_000).collect::<Vec<_>>());
    }

    common::drop_collection(&client, &collection)
        .await
        .expect("drop query-iterator collection");
}

#[tokio::test]
async fn query_iterator_applies_filter() {
    let client = common::client().await;
    let collection = common::unique_collection_name("query_iterator_filter");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    prepare_iterator_collection(&client, &collection).await;

    let rows = collect_query_rows(
        &client,
        &collection,
        "age > 30",
        29,
        None,
        0,
        &[AGE_FIELD, NAME_FIELD],
        true,
    )
    .await;
    assert_eq!(rows.len(), expected_age_count(|age| age > 30));
    assert!(rows.len() < TOTAL_COUNT);
    for row in &rows {
        assert_iterator_row(row);
        assert!(row[AGE_FIELD].as_i64().expect("age is an integer") > 30);
    }

    common::drop_collection(&client, &collection)
        .await
        .expect("drop filtered query-iterator collection");
}

#[tokio::test]
async fn search_iterator_batches_and_limits() {
    let client = common::client().await;
    let collection = common::unique_collection_name("search_iterator");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    prepare_iterator_collection(&client, &collection).await;

    for (batch_size, limit, expected_count) in [
        (37, None, TOTAL_COUNT),
        (1_000, None, TOTAL_COUNT),
        (37, Some(0), 0),
        (37, Some(5), 5),
        (1_000, Some(TOTAL_COUNT + 50), TOTAL_COUNT),
    ] {
        let rows = collect_search_rows(&client, &collection, "", batch_size, limit).await;
        assert_eq!(rows.len(), expected_count);
        for row in &rows {
            assert_iterator_row(row);
        }
    }

    common::drop_collection(&client, &collection)
        .await
        .expect("drop search-iterator collection");
}

#[tokio::test]
async fn search_iterator_applies_filter() {
    let client = common::client().await;
    let collection = common::unique_collection_name("search_iterator_filter");
    let _cleanup = common::CollectionCleanup::new([&collection]);
    prepare_iterator_collection(&client, &collection).await;

    let rows = collect_search_rows(&client, &collection, "age <= 30", 29, None).await;
    assert_eq!(rows.len(), expected_age_count(|age| age <= 30));
    assert!(rows.len() < TOTAL_COUNT);
    for row in &rows {
        assert_iterator_row(row);
        assert!(row[AGE_FIELD].as_i64().expect("age is an integer") <= 30);
    }

    common::drop_collection(&client, &collection)
        .await
        .expect("drop filtered search-iterator collection");
}

async fn prepare_iterator_collection(client: &ClientV2, collection: &str) {
    let schema = CollectionSchema::new()
        .enable_dynamic_field(false)
        .add_field(
            FieldSchema::new()
                .name(ID_FIELD)
                .data_type(DataType::Int64)
                .primary_key(true)
                .auto_id(false),
        )
        .add_field(
            FieldSchema::new()
                .name(AGE_FIELD)
                .data_type(DataType::Int16),
        )
        .add_field(
            FieldSchema::new()
                .name(NAME_FIELD)
                .data_type(DataType::VarChar)
                .max_length(64),
        )
        .add_field(
            FieldSchema::new()
                .name(VECTOR_FIELD)
                .data_type(DataType::FloatVector)
                .dimension(4),
        );
    client
        .create_collection(
            CreateCollectionRequest::builder()
                .collection_name(collection)
                .schema(schema)
                .consistency_level(ConsistencyLevel::Strong)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create iterator collection");

    let ages = (0..TOTAL_COUNT).map(age_for_index).collect::<Vec<_>>();
    let ids = (0..TOTAL_COUNT as i64).collect::<Vec<_>>();
    let names = (0..TOTAL_COUNT)
        .map(|index| format!("name_{index}"))
        .collect::<Vec<_>>();
    let vectors = (0..TOTAL_COUNT)
        .map(|index| {
            let value = index as f32 / TOTAL_COUNT as f32;
            [value, 1.0 - value, (index % 7) as f32 / 7.0, 0.5].to_vec()
        })
        .collect::<Vec<_>>();
    client
        .insert(
            InsertRequest::builder()
                .collection_name(collection)
                .columns(vec![
                    FieldData::Int64 {
                        name: ID_FIELD.into(),
                        values: ids,
                    },
                    FieldData::Int16 {
                        name: AGE_FIELD.into(),
                        values: ages,
                    },
                    FieldData::VarChar {
                        name: NAME_FIELD.into(),
                        values: names,
                    },
                    FieldData::FloatVector {
                        name: VECTOR_FIELD.into(),
                        values: vectors,
                    },
                ])
                .build()
                .expect("build iterator insert"),
        )
        .await
        .expect("insert iterator data");
    client
        .flush(
            FlushRequest::builder()
                .collection_names([collection])
                .wait_flushed_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("flush iterator data");
    client
        .create_index(
            CreateIndexRequest::builder()
                .collection_name(collection)
                .index_param(
                    IndexParam::new()
                        .field_name(VECTOR_FIELD)
                        .index_type(IndexType::Flat)
                        .metric_type(MetricType::L2),
                )
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("create iterator vector index");
    client
        .load_collection(
            LoadCollectionRequest::builder()
                .collection_name(collection)
                .load_fields([ID_FIELD, AGE_FIELD, NAME_FIELD, VECTOR_FIELD])
                .sync(true)
                .timeout_ms(60_000)
                .build()
                .expect("valid request"),
        )
        .await
        .expect("load iterator collection");
}

async fn collect_query_rows(
    client: &ClientV2,
    collection: &str,
    filter: &str,
    batch_size: usize,
    limit: Option<i64>,
    offset: i64,
    output_fields: &[&str],
    reduce_stop_for_best: bool,
) -> Vec<EntityRow> {
    let query = QueryRequest::builder()
        .collection_name(collection)
        .filter(filter)
        .output_fields(output_fields.iter().copied())
        .offset(offset)
        .consistency_level(ConsistencyLevel::Strong);
    let query = query.build().expect("valid query request");
    let mut request = QueryIteratorRequest::builder()
        .query(query)
        .batch_size(batch_size)
        .reduce_stop_for_best(reduce_stop_for_best);
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    let mut iterator = client
        .query_iterator(request.build().expect("valid request"))
        .await
        .expect("create query iterator");

    let mut rows = Vec::new();
    while let Some(response) = iterator.next().await.expect("fetch query iterator batch") {
        let batch = response
            .results()
            .get_output_rows()
            .expect("materialize query iterator rows");
        assert!(!batch.is_empty());
        assert!(batch.len() <= batch_size);
        assert_eq!(response.results().get_row_count() as usize, batch.len());
        rows.extend(batch);
    }
    assert!(iterator
        .next()
        .await
        .expect("query iterator remains finished")
        .is_none());
    rows
}

async fn collect_search_rows(
    client: &ClientV2,
    collection: &str,
    filter: &str,
    batch_size: usize,
    limit: Option<usize>,
) -> Vec<EntityRow> {
    let search = SearchRequest::builder()
        .collection_name(collection)
        .vectors(SearchVectors::Float(vec![vec![0.5, 0.5, 0.5, 0.5]]))
        .filter(filter)
        .output_fields([AGE_FIELD, NAME_FIELD])
        .consistency_level(ConsistencyLevel::Strong)
        .build()
        .expect("valid request");
    let mut request = SearchIteratorRequest::builder()
        .search(search)
        .batch_size(batch_size);
    if let Some(limit) = limit {
        request = request.limit(limit);
    }
    let mut iterator = client
        .search_iterator(request.build().expect("valid search iterator request"))
        .await
        .expect("create search iterator");

    let mut rows = Vec::new();
    let mut previous_score = f32::NEG_INFINITY;
    while let Some(response) = iterator.next().await.expect("fetch search iterator batch") {
        assert_eq!(response.results().len().to_owned(), 1);
        let result = &response.results().get_results()[0];
        assert!(!result.is_empty());
        assert!(result.len() <= batch_size);
        for score in result.get_scores() {
            assert!(
                *score >= previous_score,
                "L2 iterator scores must be non-decreasing: {score} < {previous_score}"
            );
            previous_score = *score;
        }
        let batch = result
            .get_output_rows()
            .expect("materialize search iterator rows");
        assert_eq!(batch.len(), result.len());
        rows.extend(batch);
    }
    assert!(iterator
        .next()
        .await
        .expect("search iterator remains finished")
        .is_none());
    rows
}

fn assert_iterator_row(row: &EntityRow) {
    let age = row[AGE_FIELD].as_i64().expect("age is an integer");
    let name = row[NAME_FIELD].as_str().expect("name is a string");
    let index = name
        .strip_prefix("name_")
        .expect("name has the inserted prefix")
        .parse::<usize>()
        .expect("name suffix is the inserted index");
    assert!(index < TOTAL_COUNT);
    assert_eq!(age, i64::from(age_for_index(index)));
}

fn age_for_index(index: usize) -> i16 {
    10 + (index % 51) as i16
}

fn expected_age_count(predicate: impl Fn(i16) -> bool) -> usize {
    (0..TOTAL_COUNT)
        .map(age_for_index)
        .filter(|age| predicate(*age))
        .count()
}
