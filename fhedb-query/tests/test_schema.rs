use bson::Bson;
use fhedb_core::db::schema::FieldType;
use fhedb_query::prelude::parse_schema;

#[test]
fn field_types() {
    let (remaining, schema) = parse_schema(
        "id: id_int, name: string, age: int, height: float, active: boolean, email_id: id_string",
    )
    .unwrap();
    assert_eq!(remaining, "");
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

    let (remaining, schema) = parse_schema("numbers: array<int>, names: array<string>").unwrap();
    assert_eq!(remaining, "");
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

    let (remaining, schema) = parse_schema("matrix: array<array<int>>").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["matrix"].field_type,
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int))))
    );

    let (remaining, schema) =
        parse_schema("user_ref: ref<users>, company_ref: ref<companies>").unwrap();
    assert_eq!(remaining, "");
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

    let (remaining, schema) =
        parse_schema("tags: array<ref<tag_collection>>, metadata: array<array<string>>").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 2);

    assert_eq!(
        schema.fields["tags"].field_type,
        FieldType::Array(Box::new(FieldType::Reference("tag_collection".to_string())))
    );

    assert_eq!(
        schema.fields["metadata"].field_type,
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::String))))
    );
}

#[test]
fn nullable_constraint() {
    let (remaining, schema) =
        parse_schema("name: string(nullable), age: int ( nullable )").unwrap();
    assert_eq!(remaining, "");
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

    let (remaining, schema) = parse_schema("items: array<string>(nullable)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["items"].field_type,
        FieldType::Nullable(Box::new(FieldType::Array(Box::new(FieldType::String))))
    );

    let (remaining, schema) = parse_schema("user_ref: ref<users>(nullable)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Nullable(Box::new(FieldType::Reference("users".to_string())))
    );
}

#[test]
fn default_values_int() {
    let (remaining, schema) = parse_schema("age: int(default = 18)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
    assert_eq!(schema.fields["age"].default_value, Some(Bson::Int64(18)));

    let (remaining, schema) = parse_schema("negative: int(default = -42)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["negative"].field_type, FieldType::Int);
    assert_eq!(
        schema.fields["negative"].default_value,
        Some(Bson::Int64(-42))
    );
}

#[test]
fn default_values_float() {
    let (remaining, schema) = parse_schema("pi: float(default = 3.14159)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["pi"].field_type, FieldType::Float);
    assert_eq!(
        schema.fields["pi"].default_value,
        Some(Bson::Double(3.14159))
    );

    let (remaining, schema) = parse_schema("negative: float(default = -1.5)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["negative"].field_type, FieldType::Float);
    assert_eq!(
        schema.fields["negative"].default_value,
        Some(Bson::Double(-1.5))
    );
}

#[test]
fn default_values_boolean() {
    let (remaining, schema) = parse_schema("active: boolean(default = true)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["active"].default_value,
        Some(Bson::Boolean(true))
    );

    let (remaining, schema) = parse_schema("disabled: boolean ( default = false )").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["disabled"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["disabled"].default_value,
        Some(Bson::Boolean(false))
    );
}

#[test]
fn default_values_string() {
    let (remaining, schema) = parse_schema("name: string(default = John)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("John".to_string()))
    );

    let (remaining, schema) =
        parse_schema("title: string(default = \"Software Engineer\")").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["title"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["title"].default_value,
        Some(Bson::String("Software Engineer".to_string()))
    );

    let (remaining, schema) = parse_schema("message: string(default = 'Hello World')").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["message"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["message"].default_value,
        Some(Bson::String("Hello World".to_string()))
    );

    let (remaining, schema) = parse_schema("empty: string(default = \"\")").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["empty"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["empty"].default_value,
        Some(Bson::String("".to_string()))
    );

    let (remaining, schema) = parse_schema("null_string: string(default = null)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["null_string"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["null_string"].default_value,
        Some(Bson::String("null".to_string()))
    );
}

#[test]
fn default_values_null() {
    let (remaining, schema) =
        parse_schema("description: string(nullable)(default = null)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["description"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["description"].default_value, Some(Bson::Null));

    let (remaining, schema) = parse_schema("notes: int(nullable)(default = null)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["notes"].field_type,
        FieldType::Nullable(Box::new(FieldType::Int))
    );
    assert_eq!(schema.fields["notes"].default_value, Some(Bson::Null));

    let (remaining, schema) = parse_schema("score: float(nullable)(default = null)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["score"].field_type,
        FieldType::Nullable(Box::new(FieldType::Float))
    );
    assert_eq!(schema.fields["score"].default_value, Some(Bson::Null));
}

#[test]
fn default_values_reference() {
    let (remaining, schema) = parse_schema("user_ref: ref<users>(default = admin)").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Reference("users".to_string())
    );
    assert_eq!(
        schema.fields["user_ref"].default_value,
        Some(Bson::String("admin".to_string()))
    );

    let (remaining, schema) =
        parse_schema("owner: ref<companies>(default = \"default-company\")").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["owner"].field_type,
        FieldType::Reference("companies".to_string())
    );
    assert_eq!(
        schema.fields["owner"].default_value,
        Some(Bson::String("default-company".to_string()))
    );

    let (remaining, schema) =
        parse_schema("category: ref<categories>(default = 'uncategorized')").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["category"].field_type,
        FieldType::Reference("categories".to_string())
    );
    assert_eq!(
        schema.fields["category"].default_value,
        Some(Bson::String("uncategorized".to_string()))
    );
}

#[test]
fn default_values_invalid() {
    let res = parse_schema("id: id_int(default = 1)");
    assert!(res.is_err());
    assert!(parse_schema("user_id: id_string(default = abc123)").is_err());
    assert!(parse_schema("uuid: id_string(default = \"uuid-123\")").is_err());
    
    assert!(parse_schema("tags: array<string>(default = [])").is_err());
    assert!(parse_schema("numbers: array<int>(default = [1,2,3])").is_err());
    assert!(parse_schema("matrix: array<array<int>>(default = [[1]])").is_err());
    
    assert!(parse_schema("age: int(default = abc)").is_err());
    assert!(parse_schema("age: int(default = 3.14)").is_err());
    assert!(parse_schema("height: float(default = not_a_number)").is_err());
    assert!(parse_schema("active: boolean(default = maybe)").is_err());
    assert!(parse_schema("active: boolean(default = 1)").is_err());
    
    assert!(parse_schema("age: int(default = null)").is_err());
    assert!(parse_schema("score: float(default = null)").is_err());
    assert!(parse_schema("active: boolean(default = null)").is_err());
}

#[test]
fn schema_empty() {
    let (remaining, schema) = parse_schema("").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 0);

    let (remaining, schema) = parse_schema("   ").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 0);

    let (remaining, schema) = parse_schema("\t\n  \r\n").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 0);
}

#[test]
fn schema_whitespace_handling() {
    let (remaining, schema) = parse_schema("  name:string,age:int  ").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);

    let (remaining, schema) = parse_schema("\tname\t:\tstring\t,\tage\t:\tint\t").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 2);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);

    let (remaining, schema) = parse_schema("name : string ( nullable ) ( default = John )").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["name"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("John".to_string()))
    );

    let (remaining, schema) = parse_schema(" items : array < string > ").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["items"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );

    let (remaining, schema) = parse_schema(" user_ref : ref < users > ").unwrap();
    assert_eq!(remaining, "");
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Reference("users".to_string())
    );
}

#[test]
fn schema_invalid_syntax() {
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
fn schema_invalid_field_types() {
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

    assert!(parse_schema("name: String").is_err());
    assert!(parse_schema("age: INT").is_err());
    assert!(parse_schema("height: Float").is_err());
    assert!(parse_schema("active: Boolean").is_err());
}

#[test]
fn schema_invalid_constraints() {
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
