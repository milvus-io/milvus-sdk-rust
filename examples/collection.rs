use milvus::{client::Client, error::Error, schema::AsFieldDataValue};

// #[derive(Debug, Clone, milvus::Entity)]
// struct ImageEntity {
//     #[milvus(primary, auto_id = false)]
//     id: i64,
//     #[milvus(dim = 1024)]
//     hash: BitVec,
//     listing_id: i32,
//     provider: i8,
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    const URL: &str = "http://localhost:19530";
    const NAME: &str = "images";

    let client = Client::new(URL).await?;
    let images = client.get_collection(NAME).await?.unwrap();

    // let mut batch = InsertBatch::new();
    // batch.push(ImageEntity {
    //     id: 12,
    //     hash: vec![1; 128],
    //     listing_id: 17,
    //     provider: 0,
    // });

    let mut fields = images.create_insert_frame().await?;
    fields
        .add([
            ("id", &12i64 as &dyn AsFieldDataValue),
            ("hash", &vec![1; 128]),
            ("listing_id", &17i32),
            ("provider", &0i8),
        ])
        .unwrap();

    images.insert(fields, Option::<String>::None).await?;

    let x = client.flush_collections(["images"]).await?;

    println!("{:?}", x);

    let fields = images
        .query(
            "id < 100",
            ["id", "hash", "listing_id", "provider"],
            [""; 0],
        )
        .await?;

    println!("{:?}", fields);

    Ok(())
}
