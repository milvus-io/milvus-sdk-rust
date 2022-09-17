#[cfg(test)]
mod tests {
    use milvus::value::Value;

    use milvus::schema::FieldSchema;

    struct Test {
        pub id: i64,
        pub hash: Vec<u8>,
        pub listing_id: i32,
        pub provider: i8,
    }

    #[test]
    fn test_const_schema() {}
}
