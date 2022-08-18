use milvus::{
    client::Client,
    collection::Collection,
    data::{FieldColumn, FromField},
    error::Error,
    schema::Collection as _,
    schema::{self, Entity, FieldSchema},
    value::Value,
};

use rand::prelude::*;

// #[derive(Debug, Clone, milvus::Entity)]
#[derive(Debug, Clone, Default)]
struct ImageEntity {
    // #[milvus(primary, auto_id = false)]
    id: i64,

    // #[milvus(dim = 1024)]
    hash: Vec<u8>, //BitVec
    listing_id: i32,
    provider: i8,
}

impl schema::Entity for ImageEntity {
    const NAME: &'static str = "images_test";
    const SCHEMA: &'static [schema::FieldSchema<'static>] = &[
        FieldSchema::new_primary_int64("id", None, false),
        FieldSchema::new_binary_vector("hash", None, 256),
        FieldSchema::new_int32("listing_id", None),
        FieldSchema::new_int8("provider", None),
    ];

    type ColumnIntoIter = std::array::IntoIter<
        (&'static FieldSchema<'static>, Value<'static>),
        { Self::SCHEMA.len() },
    >;

    fn iter(&self) -> Self::ColumnIntoIter {
        [
            (&Self::SCHEMA[0], self.id.into()),
            (&Self::SCHEMA[1], self.hash.clone().into()),
            (&Self::SCHEMA[2], self.listing_id.into()),
            (&Self::SCHEMA[3], self.provider.into()),
        ]
        .into_iter()
    }

    fn into_iter(self) -> Self::ColumnIntoIter {
        [
            (&Self::SCHEMA[0], self.id.into()),
            (&Self::SCHEMA[1], self.hash.into()),
            (&Self::SCHEMA[2], self.listing_id.into()),
            (&Self::SCHEMA[3], self.provider.into()),
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone)]
struct ImageBatch {
    id: Vec<i64>,
    hash: Vec<u8>,
    listing_id: Vec<i32>,
    provider: Vec<i32>,
}

impl schema::IntoDataFields for ImageBatch {
    fn into_data_fields(self) -> Vec<schema::FieldData> {
        let scm = <Self as schema::Collection>::Entity::SCHEMA;
        vec![
            milvus::data::make_field_data(&scm[0], self.id),
            milvus::data::make_field_data(&scm[1], self.hash),
            milvus::data::make_field_data(&scm[2], self.listing_id),
            milvus::data::make_field_data(&scm[3], self.provider),
        ]
    }
}
impl schema::FromDataFields for ImageBatch {
    fn from_data_fields(mut fileds: Vec<schema::FieldData>) -> Option<Self> {
        let mut this = ImageBatch::with_capacity(0);

        while let Some(fld) = fileds.pop() {
            let field = if let Some(f) = fld.field {
                f
            } else {
                continue;
            };

            match fld.field_name.as_str() {
                "id" => this.id = FromField::from_field(field)?,
                "hash" => this.hash = FromField::from_field(field)?,
                "listing_id" => this.listing_id = FromField::from_field(field)?,
                "provider" => this.provider = FromField::from_field(field)?,
                _ => continue,
            }
        }

        Some(this)
    }
}

impl<'a> schema::Collection<'a> for ImageBatch {
    type Entity = ImageEntity;
    type IterRows = Box<dyn Iterator<Item = Self::Entity> + 'a>;
    type IterColumns = Box<dyn Iterator<Item = FieldColumn<'static>>>;

    fn with_capacity(cap: usize) -> Self {
        let scm = <Self as schema::Collection>::Entity::SCHEMA;

        Self {
            id: Vec::with_capacity(cap * scm[0].dim as usize),
            hash: Vec::with_capacity(cap * scm[1].dim as usize),
            listing_id: Vec::with_capacity(cap * scm[2].dim as usize),
            provider: Vec::with_capacity(cap * scm[3].dim as usize),
        }
    }

    fn add(&mut self, mut entity: Self::Entity) {
        self.id.push(entity.id);
        self.hash.append(&mut entity.hash);
        self.listing_id.push(entity.listing_id);
        self.provider.push(entity.provider as i32);
    }

    fn index(&self, idx: usize) -> Option<Self::Entity> {
        let schm = <Self::Entity as Entity>::SCHEMA;
        let hash_size = schm[1].dim as usize / 8;
        let offset = idx * hash_size;

        Some(ImageEntity {
            id: *self.id.get(idx)? as _,
            hash: self.hash[offset..offset + hash_size].to_vec() as _,
            listing_id: *self.listing_id.get(idx)? as _,
            provider: *self.provider.get(idx)? as _,
            ..Default::default()
        })
    }

    fn iter_columns(&self) -> Self::IterColumns {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.id.len()
    }
}

#[derive(Debug, Clone)]
struct ImageQueryResult {
    id: Vec<i64>,
    hash: Vec<u8>,
    listing_id: Vec<i32>,
}

impl schema::IntoDataFields for ImageQueryResult {
    fn into_data_fields(self) -> Vec<schema::FieldData> {
        let scm = <Self as schema::Collection>::Entity::SCHEMA;
        vec![
            milvus::data::make_field_data(&scm[0], self.id),
            milvus::data::make_field_data(&scm[1], self.hash),
            milvus::data::make_field_data(&scm[2], self.listing_id),
        ]
    }
}
impl schema::FromDataFields for ImageQueryResult {
    fn from_data_fields(mut fileds: Vec<schema::FieldData>) -> Option<Self> {
        let mut this = ImageQueryResult::with_capacity(0);

        while let Some(fld) = fileds.pop() {
            let field = if let Some(f) = fld.field {
                f
            } else {
                continue;
            };

            match fld.field_name.as_str() {
                "id" => this.id = FromField::from_field(field)?,
                "hash" => this.hash = FromField::from_field(field)?,
                "listing_id" => this.listing_id = FromField::from_field(field)?,
                _ => continue,
            }
        }

        Some(this)
    }
}

impl<'a> schema::Collection<'a> for ImageQueryResult {
    type Entity = ImageEntity;
    type IterRows = Box<dyn Iterator<Item = Self::Entity> + 'a>;
    type IterColumns = Box<dyn Iterator<Item = FieldColumn<'static>> + 'a>;

    fn with_capacity(cap: usize) -> Self {
        let scm = <Self as schema::Collection>::Entity::SCHEMA;

        Self {
            id: Vec::with_capacity(cap * scm[0].dim as usize),
            hash: Vec::with_capacity(cap * scm[1].dim as usize),
            listing_id: Vec::with_capacity(cap * scm[2].dim as usize),
        }
    }

    fn add(&mut self, mut entity: Self::Entity) {
        self.id.push(entity.id);
        self.hash.append(&mut entity.hash);
        self.listing_id.push(entity.listing_id);
    }

    fn index(&self, idx: usize) -> Option<Self::Entity> {
        let schm = <Self::Entity as Entity>::SCHEMA;
        let hash_size = schm[1].dim as usize / 8;
        let offset = idx * hash_size;

        Some(ImageEntity {
            id: *self.id.get(idx)? as _,
            hash: self.hash[offset..offset + hash_size].to_vec() as _,
            listing_id: *self.listing_id.get(idx)? as _,
            ..Default::default()
        })
    }

    fn iter_columns(&self) -> Self::IterColumns {
        unimplemented!()
    }

    fn len(&self) -> usize {
        self.id.len()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";

    let client = Client::new(URL).await?;
    let images: Collection<ImageEntity> = client.get_collection().await?;

    if !images.exists().await? {
        images.create(None, None).await?;
    }

    let mut batch = ImageBatch::with_capacity(1000);

    for i in 1..=1000 {
        let mut rng = rand::thread_rng();
        let hash: [u8; 32] = rng.gen();

        batch.add(ImageEntity {
            id: i,
            hash: hash.to_vec(),
            listing_id: (i % 100) as i32,
            provider: 0,
        });
    }

    images.insert(batch, Option::<&str>::None).await?;
    images.flush().await?;
    images.load_blocked(1).await?;

    let x: ImageQueryResult = images.query::<_, _, [&str; 0]>("id < 200", []).await?;

    for v in x.iter_rows() {
        println!("{:?}", v);
    }

    Ok(())
}
