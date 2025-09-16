use bson::Bson;
use fhedb_core::db::schema::FieldType;
use fhedb_query::prelude::{FieldModification, parse_field_modifications, parse_schema};

#[test]
fn field_types() {
    let schema = parse_schema(
        "id: id_int, name: string, age: int, height: float, active: boolean, email_id: id_string",
    )
    .unwrap();
    assert_eq!(schema.fields.len(), 6);

    assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema.fields["id"].default_value, None);

    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["name"].default_value, None);

    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
    assert_eq!(schema.fields["age"].default_value, None);

    assert_eq!(schema.fields["height"].field_type, FieldType::Float);
    assert_eq!(schema.fields["height"].default_value, None);

    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(schema.fields["active"].default_value, None);

    assert_eq!(schema.fields["email_id"].field_type, FieldType::IdString);
    assert_eq!(schema.fields["email_id"].default_value, None);

    let schema = parse_schema("numbers: array<int>, names: array<string>").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["numbers"].field_type,
        FieldType::Array(Box::new(FieldType::Int))
    );
    assert_eq!(schema.fields["numbers"].default_value, None);

    assert_eq!(
        schema.fields["names"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["names"].default_value, None);

    let schema = parse_schema("matrix: array<array<int>>").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["matrix"].field_type,
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int))))
    );

    let schema = parse_schema("user_ref: ref<users>, company_ref: ref<companies>").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Reference("users".to_string())
    );
    assert_eq!(schema.fields["user_ref"].default_value, None);

    assert_eq!(
        schema.fields["company_ref"].field_type,
        FieldType::Reference("companies".to_string())
    );
    assert_eq!(schema.fields["company_ref"].default_value, None);

    let schema =
        parse_schema("tags: array<ref<tag_collection>>, metadata: array<array<string>>").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["tags"].field_type,
        FieldType::Array(Box::new(FieldType::Reference("tag_collection".to_string())))
    );

    assert_eq!(
        schema.fields["metadata"].field_type,
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::String))))
    );

    let schema = parse_schema(
        "name: STRING, age: INT, height: FLOAT, active: BOOLEAN, id: ID_INT, email: ID_STRING",
    )
    .unwrap();
    assert_eq!(schema.fields.len(), 6);

    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
    assert_eq!(schema.fields["height"].field_type, FieldType::Float);
    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(schema.fields["id"].field_type, FieldType::IdInt);
    assert_eq!(schema.fields["email"].field_type, FieldType::IdString);

    let schema = parse_schema("items: ARRAY<STRING>, refs: Array<Ref<users>>").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["items"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["refs"].field_type,
        FieldType::Array(Box::new(FieldType::Reference("users".to_string())))
    );
}

#[test]
fn nullable_constraint() {
    let schema = parse_schema("name: string(nullable), age: int ( nullable )").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["name"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["name"].default_value, None);

    assert_eq!(
        schema.fields["age"].field_type,
        FieldType::Nullable(Box::new(FieldType::Int))
    );
    assert_eq!(schema.fields["age"].default_value, None);

    let schema = parse_schema("items: array<string>(nullable)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["items"].field_type,
        FieldType::Nullable(Box::new(FieldType::Array(Box::new(FieldType::String))))
    );

    let schema = parse_schema("user_ref: ref<users>(nullable)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())))
    );

    let schema = parse_schema("name: String(NULLABLE), active: Boolean(Default = true)").unwrap();
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["name"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["active"].default_value,
        Some(Bson::Boolean(true))
    );
}

#[test]
fn default_values_valid() {
    let schema = parse_schema("age: int(default = 25), active: boolean(default = true)").unwrap();
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
    assert_eq!(schema.fields["age"].default_value, Some(Bson::Int64(25)));
    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["active"].default_value,
        Some(Bson::Boolean(true))
    );

    let schema = parse_schema("name: string(default = \"John Doe\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("John Doe".to_string()))
    );

    let schema = parse_schema("score: float(default = 95.5)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["score"].field_type, FieldType::Float);
    assert_eq!(
        schema.fields["score"].default_value,
        Some(Bson::Double(95.5))
    );

    let schema = parse_schema("description: string(nullable)(default = null)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["description"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["description"].default_value, Some(Bson::Null));

    let schema = parse_schema("tags: array<string>(default = [\"dev\", \"test\"])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["tags"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["tags"].default_value,
        Some(Bson::Array(vec![
            Bson::String("dev".to_string()),
            Bson::String("test".to_string())
        ]))
    );

    let schema = parse_schema("owner: ref<users>(default = \"admin\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["owner"].field_type,
        FieldType::Reference("users".to_string())
    );
    assert_eq!(
        schema.fields["owner"].default_value,
        Some(Bson::String("admin".to_string()))
    );

    let schema =
        parse_schema("id: id_int, name: string(default = \"Anonymous\"), age: int").unwrap();
    assert_eq!(schema.fields.len(), 3);
    assert_eq!(schema.fields["id"].default_value, None);
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("Anonymous".to_string()))
    );
    assert_eq!(schema.fields["age"].default_value, None);
}

#[test]
fn default_values_invalid() {
    let res = parse_schema("id: id_int(default = 1)");
    assert!(res.is_err());
    assert!(parse_schema("user_id: id_string(default = abc123)").is_err());
    assert!(parse_schema("uuid: id_string(default = \"uuid-123\")").is_err());

    assert!(parse_schema("age: int(default = abc)").is_err());
    assert!(parse_schema("age: int(default = 3.14)").is_err());
    assert!(parse_schema("height: float(default = not_a_number)").is_err());
    assert!(parse_schema("active: boolean(default = maybe)").is_err());
    assert!(parse_schema("active: boolean(default = 1)").is_err());

    assert!(parse_schema("age: int(default = null)").is_err());
    assert!(parse_schema("score: float(default = null)").is_err());
    assert!(parse_schema("active: boolean(default = null)").is_err());

    assert!(parse_schema("tags: array<string>(default = [1, 2, 3])").is_err());
    assert!(parse_schema("numbers: array<int>(default = [\"a\", \"b\"])").is_err());
    assert!(parse_schema("items: array<string>(default = 1, 2, 3)").is_err());
}

#[test]
fn empty() {
    let schema = parse_schema("").unwrap();
    assert_eq!(schema.fields.len(), 0);

    let schema = parse_schema("   ").unwrap();
    assert_eq!(schema.fields.len(), 0);

    let schema = parse_schema("\t\n  \r\n").unwrap();
    assert_eq!(schema.fields.len(), 0);
}

#[test]
fn extra_whitespace() {
    let schema = parse_schema("  name:string,age:int  ").unwrap();
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);

    let schema = parse_schema("\tname\t:\tstring\t,\tage\t:\tint\t").unwrap();
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);

    let schema = parse_schema("name : string ( nullable ) ( default = \"John\" )").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["name"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("John".to_string()))
    );

    let schema = parse_schema(" items : array < string > ").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["items"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );

    let schema = parse_schema(" user_ref : ref < users > ").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Reference("users".to_string())
    );
}

#[test]
fn invalid_syntax() {
    assert!(parse_schema("name string").is_err());
    assert!(parse_schema("name:").is_err());
    assert!(parse_schema(":string").is_err());
    assert!(parse_schema("name: string,").is_err());
    assert!(parse_schema(",name: string").is_err());
    assert!(parse_schema("name: string,,age: int").is_err());

    assert!(parse_schema("name: string(").is_err());
    assert!(parse_schema("name: string)").is_err());
    assert!(parse_schema("name: string()").is_err());
    assert!(parse_schema("name: string(nullable").is_err());
    assert!(parse_schema("name: string nullable)").is_err());

    assert!(parse_schema("name: array<").is_err());
    assert!(parse_schema("name: array>").is_err());
    assert!(parse_schema("name: array<>").is_err());
    assert!(parse_schema("name: array string>").is_err());
    assert!(parse_schema("name: array<string").is_err());

    assert!(parse_schema("name: ref<").is_err());
    assert!(parse_schema("name: ref>").is_err());
    assert!(parse_schema("name: ref<>").is_err());
    assert!(parse_schema("name: ref users>").is_err());
    assert!(parse_schema("name: ref<users").is_err());

    assert!(parse_schema("name: string(= value)").is_err());
    assert!(parse_schema("name: string(default value)").is_err());
}

#[test]
fn invalid_field_types() {
    assert!(parse_schema("name: text").is_err());
    assert!(parse_schema("age: integer").is_err());
    assert!(parse_schema("price: double").is_err());
    assert!(parse_schema("active: bool").is_err());
    assert!(parse_schema("id: id").is_err());
    assert!(parse_schema("id: identifier").is_err());

    assert!(parse_schema("items: list<string>").is_err());
    assert!(parse_schema("items: vector<int>").is_err());
    assert!(parse_schema("items: array").is_err());

    assert!(parse_schema("user_ref: reference<users>").is_err());
    assert!(parse_schema("user_ref: link<users>").is_err());
    assert!(parse_schema("user_ref: ref").is_err());

    assert!(parse_schema("data: object").is_err());
    assert!(parse_schema("data: map").is_err());
    assert!(parse_schema("data: json").is_err());
    assert!(parse_schema("timestamp: datetime").is_err());
    assert!(parse_schema("timestamp: date").is_err());
}

#[test]
fn invalid_constraints() {
    assert!(parse_schema("name: string(required)").is_err());
    assert!(parse_schema("name: string(optional)").is_err());
    assert!(parse_schema("name: string(unique)").is_err());
    assert!(parse_schema("name: string(indexed)").is_err());
    assert!(parse_schema("age: int(min = 0)").is_err());
    assert!(parse_schema("age: int(max = 100)").is_err());
    assert!(parse_schema("name: string(length = 50)").is_err());

    assert!(parse_schema("name: string(invalid_constraint)").is_err());
    assert!(parse_schema("name: string(constraint_without_value = )").is_err());
    assert!(parse_schema("name: string( = value)").is_err());

    assert!(parse_schema("name: string(not_nullable)").is_err());
    assert!(parse_schema("name: string(non_null)").is_err());
    assert!(parse_schema("name: string(null)").is_err());

    assert!(parse_schema("name: string(default)").is_err());
    assert!(parse_schema("name: string(= test)").is_err());
}

#[test]
fn field_modifications() {
    let modifications = parse_field_modifications("name: string, age: drop").unwrap();
    assert_eq!(modifications.len(), 2);

    match &modifications["name"] {
        FieldModification::Set(field_def) => {
            assert_eq!(field_def.field_type, FieldType::String);
        }
        _ => panic!("Expected FieldModification::Set for name field"),
    }

    match &modifications["age"] {
        FieldModification::Drop => {}
        _ => panic!("Expected FieldModification::Drop for age field"),
    }
}
