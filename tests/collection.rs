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

use milvus::collection::ParamValue;
use milvus::data::FieldColumn;
use milvus::error::Result;
use milvus::index::{FunctionType, IndexParams, IndexType, MetricType};
use milvus::options::LoadOptions;
use milvus::proto::schema::DataType;
use milvus::query::{QueryOptions, SearchOptions};
use milvus::schema::{
    CollectionSchema, CollectionSchemaBuilder, FieldSchemaBuilder, FunctionSchemaBuilder,
};
use std::collections::HashMap;

mod common;
use common::*;

use milvus::value::{Value, ValueVec};

const METRIC_TYPE_LIST: &[MetricType; 2] = &[MetricType::L2, MetricType::COSINE];

#[tokio::test]
async fn manual_compaction_empty_collection() -> Result<()> {
    let (client, schema) = create_test_collection(true, None).await?;
    let resp = client.manual_compaction(schema.name()).await?;
    assert_eq!(0, resp.plan_count);

    client.drop_collection(schema.name()).await?;
    Ok(())
}

#[tokio::test]
async fn collection_upsert() -> Result<()> {
    for metric_type in METRIC_TYPE_LIST {
        let (client, schema) = create_test_collection(false, None).await?;
        let pk_data = gen_random_int64_vector(2000);
        let vec_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
        let pk_col = FieldColumn::new(schema.get_field("id").unwrap(), pk_data);
        let vec_col = FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), vec_data);
        client
            .upsert(schema.name(), vec![pk_col, vec_col], None)
            .await?;
        let index_params = IndexParams::new(
            DEFAULT_INDEX_NAME.to_owned(),
            IndexType::IvfFlat,
            *metric_type,
            HashMap::from([("nlist".to_owned(), "32".to_owned())]),
        );
        client
            .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
            .await?;
        client
            .load_collection(schema.name(), Some(LoadOptions::default()))
            .await?;

        let options = QueryOptions::default();
        let options = options.output_fields(vec![String::from("count(*)")]);
        let result = client.query(schema.name(), "", &options).await?;
        if let ValueVec::Long(vec) = &result[0].value {
            assert_eq!(2000, vec[0]);
        } else {
            panic!("invalid result");
        }

        client.drop_collection(schema.name()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn collection_basic() -> Result<()> {
    for metric_type in METRIC_TYPE_LIST {
        let (client, schema) = create_test_collection(true, None).await?;

        let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);

        let embed_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

        client
            .insert(schema.name(), vec![embed_column], None)
            .await?;
        client.flush(schema.name()).await?;
        let index_params = IndexParams::new(
            DEFAULT_INDEX_NAME.to_owned(),
            IndexType::IvfFlat,
            *metric_type,
            HashMap::from([("nlist".to_owned(), "32".to_owned())]),
        );
        client
            .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
            .await?;
        client
            .load_collection(schema.name(), Some(LoadOptions::default()))
            .await?;

        let options = QueryOptions::default();
        let result = client.query(schema.name(), "id > 0", &options).await?;

        println!(
            "result num: {}",
            result.first().map(|c| c.len()).unwrap_or(0),
        );

        client.drop_collection(schema.name()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn collection_index() -> Result<()> {
    for metric_type in METRIC_TYPE_LIST {
        let (client, schema) = create_test_collection(true, None).await?;

        let feature = gen_random_f32_vector(DEFAULT_DIM * 2000);

        let feature_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), feature);

        client
            .insert(schema.name(), vec![feature_column], None)
            .await?;
        client.flush(schema.name()).await?;

        let index_params = IndexParams::new(
            DEFAULT_INDEX_NAME.to_owned(),
            IndexType::IvfFlat,
            *metric_type,
            HashMap::from([("nlist".to_owned(), "32".to_owned())]),
        );
        client
            .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params.clone())
            .await?;
        let index_list = client
            .describe_index(schema.name(), DEFAULT_VEC_FIELD)
            .await?;
        assert!(index_list.len() == 1, "{}", index_list.len());
        let index = &index_list[0];

        assert_eq!(index.params().name(), index_params.name());
        assert_eq!(index.params().extra_params(), index_params.extra_params());

        client.drop_index(schema.name(), DEFAULT_VEC_FIELD).await?;
        client.drop_collection(schema.name()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn collection_search() -> Result<()> {
    for metric_type in METRIC_TYPE_LIST {
        let (client, schema) = create_test_collection(true, None).await?;

        let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
        let embed_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

        client
            .insert(schema.name(), vec![embed_column], None)
            .await?;

        let index_params = IndexParams::new(
            "ivf_flat".to_owned(),
            IndexType::IvfFlat,
            *metric_type,
            HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
        );
        client
            .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
            .await?;
        client.flush(schema.name()).await?;
        client
            .load_collection(schema.name(), Some(LoadOptions::default()))
            .await?;

        let mut option = SearchOptions::with_limit(10)
            .metric_type(*metric_type)
            .output_fields(vec!["id".to_owned()]);
        option = option.add_param("nprobe", ParamValue!(16));
        let query_vec = gen_random_f32_vector(DEFAULT_DIM);

        let result = client
            .search(
                schema.name(),
                vec![query_vec.into()],
                DEFAULT_VEC_FIELD,
                &option,
            )
            .await?;

        assert_eq!(result[0].size, 10);

        client.drop_collection(schema.name()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn collection_range_search() -> Result<()> {
    for metric_type in METRIC_TYPE_LIST {
        let (client, schema) = create_test_collection(true, None).await?;

        let embed_data = gen_random_f32_vector(DEFAULT_DIM * 2000);
        let embed_column =
            FieldColumn::new(schema.get_field(DEFAULT_VEC_FIELD).unwrap(), embed_data);

        client
            .insert(schema.name(), vec![embed_column], None)
            .await?;
        client.flush(schema.name()).await?;
        let index_params = IndexParams::new(
            "ivf_flat".to_owned(),
            IndexType::IvfFlat,
            *metric_type,
            HashMap::from_iter([("nlist".to_owned(), 32.to_string())]),
        );
        client
            .create_index(schema.name(), DEFAULT_VEC_FIELD, index_params)
            .await?;

        client
            .load_collection(schema.name(), Some(LoadOptions::default()))
            .await?;

        let radius_limit: f32 = match metric_type {
            MetricType::L2 => 20.0,
            MetricType::COSINE => 0.2,
            _ => unimplemented!(),
        };
        let range_filter_limit: f32 = match metric_type {
            MetricType::L2 => 10.0,
            MetricType::COSINE => 0.9,
            _ => unimplemented!(),
        };

        let mut option = SearchOptions::with_limit(5)
            .metric_type(*metric_type)
            .output_fields(vec!["id".to_owned()]);
        option = option.add_param("nprobe", ParamValue!(16));
        option = option.radius(radius_limit).range_filter(range_filter_limit);
        let query_vec = gen_random_f32_vector(DEFAULT_DIM);

        let result = client
            .search(
                schema.name(),
                vec![query_vec.into()],
                DEFAULT_VEC_FIELD,
                &option,
            )
            .await?;
        for record in &result {
            for value in &record.score {
                match metric_type {
                    MetricType::L2 => {
                        assert!(*value >= range_filter_limit && *value <= radius_limit)
                    }
                    MetricType::COSINE => {
                        assert!(*value >= radius_limit && *value <= range_filter_limit)
                    }
                    _ => unimplemented!(),
                }
            }
        }

        client.drop_collection(schema.name()).await?;
    }
    Ok(())
}

#[tokio::test]
async fn test_text_search() -> Result<()> {
    let schema_fn: CollectionSchemaFn = |name: &str| -> Result<CollectionSchema> {
        Ok(CollectionSchemaBuilder::new(name, "")
            .add_field(
                FieldSchemaBuilder::new()
                    .with_name("id")
                    .with_dtype(DataType::Int64)
                    .with_primary(true)
                    .with_auto_id(true)
                    .build(),
            )
            .add_field(
                FieldSchemaBuilder::new()
                    .with_name("text")
                    .with_dtype(DataType::VarChar)
                    .with_max_length(1024)
                    .enable_analyzer(true)
                    .build(),
            )
            .add_field(
                FieldSchemaBuilder::new()
                    .with_name("sparse")
                    .with_dtype(DataType::SparseFloatVector)
                    .build(),
            )
            .add_function(
                FunctionSchemaBuilder::new()
                    .with_name("text2vec")
                    .with_typ(FunctionType::BM25)
                    .with_input_field_names(vec!["text".to_owned()])
                    .with_output_field_names(vec!["sparse".to_owned()])
                    .build(),
            )
            .build()?)
    };
    let (client, schema) = create_test_collection(true, Some(schema_fn)).await?;

    let index_params = IndexParams::new(
        "sparse".to_owned(),
        IndexType::SparseInvertedIndex,
        MetricType::BM25,
        HashMap::new(),
    );
    client
        .create_index(schema.name(), "sparse", index_params)
        .await?;

    let text_column = FieldColumn::new(
        schema.get_field("text").unwrap(),
        vec![
            "information retrieval is a field of study.".to_owned(),
            "information retrieval focuses on finding relevant information in large datasets."
                .to_owned(),
            "data mining and information retrieval overlap in research.".to_owned(),
        ],
    );
    client
        .insert(schema.name(), vec![text_column], None)
        .await?;

    client.flush(schema.name()).await?;
    client
        .load_collection(schema.name(), Some(LoadOptions::default()))
        .await?;

    let mut options = SearchOptions::default();
    options = options.limit(3);
    options = options.output_fields(vec!["text".into()]);
    options = options.add_param("drop_ratio_search", ParamValue!(0.2));
    options = options.metric_type(MetricType::BM25);
    let result = client
        .search(
            schema.name(),
            vec!["whats the focus of information retrieval?".into()],
            "sparse",
            &options,
        )
        .await?;

    assert!(result.len() == 1);
    for record in &result {
        for value in &record.score {
            assert!(*value >= 0.2);
        }
    }

    for col in &result[0].field {
        assert!(col.name == "text");

        if let ValueVec::String(vec) = &col.value {
            assert_eq!(&vec[0], "information retrieval is a field of study.");
        }
        break;
    }

    client.drop_collection(schema.name()).await?;
    Ok(())
}
