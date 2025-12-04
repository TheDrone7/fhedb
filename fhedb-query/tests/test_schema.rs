use fhedb_query::prelude::parse_contextual_query;

fn parse_schema(input: &str) -> Result<(), ()> {
    let wrapped = format!("CREATE COLLECTION test {{{}}}", input);
    parse_contextual_query(&wrapped).map(|_| ()).map_err(|_| ())
}

#[test]
fn field_types() {
    assert!(parse_schema("id: id_int, name: string, age: int, height: float, active: boolean, email_id: id_string").is_ok());
    assert!(parse_schema("numbers: array<int>, names: array<string>").is_ok());
    assert!(parse_schema("matrix: array<array<int>>").is_ok());
    assert!(parse_schema("user_ref: ref<users>, company_ref: ref<companies>").is_ok());
    assert!(
        parse_schema("tags: array<ref<tag_collection>>, metadata: array<array<string>>").is_ok()
    );
    assert!(
        parse_schema(
            "name: STRING, age: INT, height: FLOAT, active: BOOLEAN, id: ID_INT, email: ID_STRING"
        )
        .is_ok()
    );
    assert!(parse_schema("items: ARRAY<STRING>, refs: Array<Ref<users>>").is_ok());
}

#[test]
fn nullable_constraint() {
    assert!(parse_schema("name: string(nullable), age: int(nullable)").is_ok());
    assert!(parse_schema("items: array<string>(nullable)").is_ok());
    assert!(parse_schema("user_ref: ref<users>(nullable)").is_ok());
    assert!(parse_schema("name: String(NULLABLE), active: Boolean(default = true)").is_ok());
}

#[test]
fn default_values_valid() {
    assert!(parse_schema("age: int(default = 25), active: boolean(default = true)").is_ok());
    assert!(parse_schema("name: string(default = \"John Doe\")").is_ok());
    assert!(parse_schema("score: float(default = 95.5)").is_ok());
    assert!(parse_schema("description: string(nullable, default = null)").is_ok());
    assert!(parse_schema("tags: array<string>(default = [\"tag1\", \"tag2\"])").is_ok());
    assert!(parse_schema("owner: ref<users>(default = \"admin\")").is_ok());
    assert!(parse_schema("id: id_int, name: string(default = \"Anonymous\"), age: int").is_ok());
}

#[test]
fn default_values_invalid() {
    assert!(parse_schema("id: id_int(default = 100)").is_err());
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
    assert!(parse_schema("numbers: array<int>(default = [\"one\", \"two\"])").is_err());
}

#[test]
fn empty() {
    assert!(parse_schema("").is_ok());
    assert!(parse_schema("   ").is_ok());
    assert!(parse_schema("\t\n  \r\n").is_ok());
}

#[test]
fn extra_whitespace() {
    assert!(parse_schema("  name:string,age:int  ").is_ok());
    assert!(parse_schema("\tname\t:\tstring\t,\tage\t:\tint\t").is_ok());
    assert!(parse_schema(" items : array < string > ").is_ok());
    assert!(parse_schema(" user_ref : ref < users > ").is_ok());
}

#[test]
fn invalid_syntax() {
    assert!(parse_schema("name string").is_err());
    assert!(parse_schema("name:").is_err());
    assert!(parse_schema(":string").is_err());
    assert!(parse_schema(",name: string").is_err());
    assert!(parse_schema("name: string,,age: int").is_err());
    assert!(parse_schema("name: string(").is_err());
    assert!(parse_schema("name: string)").is_err());
    assert!(parse_schema("name: string()").is_err());
    assert!(parse_schema("name: string nullable)").is_err());
    assert!(parse_schema("name: string(nullable").is_err());
    assert!(parse_schema("name: array<").is_err());
    assert!(parse_schema("name: array>").is_err());
    assert!(parse_schema("name: array<>").is_err());
    assert!(parse_schema("name: array<string").is_err());
    assert!(parse_schema("name: ref<").is_err());
    assert!(parse_schema("name: ref>").is_err());
    assert!(parse_schema("name: ref<>").is_err());
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
    assert!(parse_schema("name: string, age: drop").is_err());
}
