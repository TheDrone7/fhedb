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
fn default_values_int() {
    let schema = parse_schema("age: int(default = 18)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["age"].field_type, FieldType::Int);
    assert_eq!(schema.fields["age"].default_value, Some(Bson::Int64(18)));

    let schema = parse_schema("negative: int(default = -42)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["negative"].field_type, FieldType::Int);
    assert_eq!(
        schema.fields["negative"].default_value,
        Some(Bson::Int64(-42))
    );
}

#[test]
fn default_values_float() {
    let schema = parse_schema("pi: float(default = 3.14159)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["pi"].field_type, FieldType::Float);
    assert_eq!(
        schema.fields["pi"].default_value,
        Some(Bson::Double(3.14159))
    );

    let schema = parse_schema("negative: float(default = -1.5)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["negative"].field_type, FieldType::Float);
    assert_eq!(
        schema.fields["negative"].default_value,
        Some(Bson::Double(-1.5))
    );
}

#[test]
fn default_values_boolean() {
    let schema = parse_schema("active: boolean(default = true)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["active"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["active"].default_value,
        Some(Bson::Boolean(true))
    );

    let schema = parse_schema("disabled: boolean ( default = false )").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["disabled"].field_type, FieldType::Boolean);
    assert_eq!(
        schema.fields["disabled"].default_value,
        Some(Bson::Boolean(false))
    );
}

#[test]
fn default_values_string() {
    let schema = parse_schema("name: string(default = \"John\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["name"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["name"].default_value,
        Some(Bson::String("John".to_string()))
    );

    let schema = parse_schema("title: string(default = \"Software Engineer\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["title"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["title"].default_value,
        Some(Bson::String("Software Engineer".to_string()))
    );

    let schema = parse_schema("message: string(default = 'Hello World')").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["message"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["message"].default_value,
        Some(Bson::String("Hello World".to_string()))
    );

    let schema = parse_schema("empty: string(default = \"\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["empty"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["empty"].default_value,
        Some(Bson::String("".to_string()))
    );

    let schema = parse_schema("null_string: string(default = \"null\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["null_string"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["null_string"].default_value,
        Some(Bson::String("null".to_string()))
    );

    let schema = parse_schema("escaped: string(default = \"Hello\\nWorld\\t!\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(schema.fields["escaped"].field_type, FieldType::String);
    assert_eq!(
        schema.fields["escaped"].default_value,
        Some(Bson::String("Hello\nWorld\t!".to_string()))
    );
}

#[test]
fn default_values_null() {
    let schema = parse_schema("description: string(nullable)(default = null)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["description"].field_type,
        FieldType::Nullable(Box::new(FieldType::String))
    );
    assert_eq!(schema.fields["description"].default_value, Some(Bson::Null));

    let schema = parse_schema("notes: int(nullable)(default = null)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["notes"].field_type,
        FieldType::Nullable(Box::new(FieldType::Int))
    );
    assert_eq!(schema.fields["notes"].default_value, Some(Bson::Null));

    let schema = parse_schema("score: float(nullable)(default = null)").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["score"].field_type,
        FieldType::Nullable(Box::new(FieldType::Float))
    );
    assert_eq!(schema.fields["score"].default_value, Some(Bson::Null));
}

#[test]
fn default_values_reference() {
    let schema = parse_schema("user_ref: ref<users>(default = \"admin\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["user_ref"].field_type,
        FieldType::Reference("users".to_string())
    );
    assert_eq!(
        schema.fields["user_ref"].default_value,
        Some(Bson::String("admin".to_string()))
    );

    let schema = parse_schema("owner: ref<companies>(default = \"default-company\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["owner"].field_type,
        FieldType::Reference("companies".to_string())
    );
    assert_eq!(
        schema.fields["owner"].default_value,
        Some(Bson::String("default-company".to_string()))
    );

    let schema = parse_schema("category: ref<categories>(default = 'uncategorized')").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["category"].field_type,
        FieldType::Reference("categories".to_string())
    );
    assert_eq!(
        schema.fields["category"].default_value,
        Some(Bson::String("uncategorized".to_string()))
    );

    let schema = parse_schema("path_ref: ref<paths>(default = \"data\\\\user\\tinfo\")").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["path_ref"].field_type,
        FieldType::Reference("paths".to_string())
    );
    assert_eq!(
        schema.fields["path_ref"].default_value,
        Some(Bson::String("data\\user\tinfo".to_string()))
    );
}

#[test]
fn default_values_array() {
    let schema = parse_schema("tags: array<string>(default = [])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["tags"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["tags"].default_value,
        Some(Bson::Array(vec![]))
    );

    let schema =
        parse_schema("names: array<string>(default = [\"Alice\", \"Bob\", \"Charlie\"])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["names"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["names"].default_value,
        Some(Bson::Array(vec![
            Bson::String("Alice".to_string()),
            Bson::String("Bob".to_string()),
            Bson::String("Charlie".to_string())
        ]))
    );

    let schema = parse_schema("numbers: array<int>(default = [1, 2, 3])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["numbers"].field_type,
        FieldType::Array(Box::new(FieldType::Int))
    );
    assert_eq!(
        schema.fields["numbers"].default_value,
        Some(Bson::Array(vec![
            Bson::Int64(1),
            Bson::Int64(2),
            Bson::Int64(3)
        ]))
    );

    let schema = parse_schema("flags: array<boolean>(default = [true, false, true])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["flags"].field_type,
        FieldType::Array(Box::new(FieldType::Boolean))
    );
    assert_eq!(
        schema.fields["flags"].default_value,
        Some(Bson::Array(vec![
            Bson::Boolean(true),
            Bson::Boolean(false),
            Bson::Boolean(true)
        ]))
    );

    let schema = parse_schema("coordinates: array<float>(default = [1.5, 2.7, 3.14])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["coordinates"].field_type,
        FieldType::Array(Box::new(FieldType::Float))
    );
    assert_eq!(
        schema.fields["coordinates"].default_value,
        Some(Bson::Array(vec![
            Bson::Double(1.5),
            Bson::Double(2.7),
            Bson::Double(3.14)
        ]))
    );

    let schema = parse_schema("matrix: array<array<int>>(default = [[1, 2], [3, 4]])").unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["matrix"].field_type,
        FieldType::Array(Box::new(FieldType::Array(Box::new(FieldType::Int))))
    );
    assert_eq!(
        schema.fields["matrix"].default_value,
        Some(Bson::Array(vec![
            Bson::Array(vec![Bson::Int64(1), Bson::Int64(2)]),
            Bson::Array(vec![Bson::Int64(3), Bson::Int64(4)])
        ]))
    );

    let schema =
        parse_schema(r#"paths: array<string>(default = ["data[0]", "config[env]", "array[key]"])"#)
            .unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["paths"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["paths"].default_value,
        Some(Bson::Array(vec![
            Bson::String("data[0]".to_string()),
            Bson::String("config[env]".to_string()),
            Bson::String("array[key]".to_string())
        ]))
    );

    let schema =
        parse_schema(r#"quotes: array<string>(default = ["He said \"Hello\"", "She said 'Hi'"])"#)
            .unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["quotes"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["quotes"].default_value,
        Some(Bson::Array(vec![
            Bson::String("He said \"Hello\"".to_string()),
            Bson::String("She said 'Hi'".to_string())
        ]))
    );

    let schema = parse_schema(
        r#"escaped: array<string>(default = ["Line1\nLine2", "Tab\tSeparated", "Back\\slash"])"#,
    )
    .unwrap();
    assert_eq!(schema.fields.len(), 1);
    assert_eq!(
        schema.fields["escaped"].field_type,
        FieldType::Array(Box::new(FieldType::String))
    );
    assert_eq!(
        schema.fields["escaped"].default_value,
        Some(Bson::Array(vec![
            Bson::String("Line1\nLine2".to_string()),
            Bson::String("Tab\tSeparated".to_string()),
            Bson::String("Back\\slash".to_string())
        ]))
    );
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
