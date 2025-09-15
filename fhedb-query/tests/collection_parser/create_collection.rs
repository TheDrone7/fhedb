use fhedb_core::prelude::FieldType;
use fhedb_query::prelude::*;

#[test]
fn basic() {
    let input = "CREATE COLLECTION users {id: id_int, name: string, age: int}";
    let result = parse_collection_query(input).unwrap();

    match result {
        CollectionQuery::Create {
            name,
            drop_if_exists,
            schema,
        } => {
            assert_eq!(name, "users");
            assert_eq!(drop_if_exists, false);
            assert_eq!(schema.fields.len(), 3);
            assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
            assert_eq!(schema.fields["name"].field_type, FieldType::String);
            assert_eq!(schema.fields["age"].field_type, FieldType::Int);
        }
    }
}
