#[test]
fn field_types() {
    "id: id_int, name: string, age: int, height: float, active: boolean, email_id: id_string";
    "numbers: array<int>, names: array<string>";
    "matrix: array<array<int>>";
    "user_ref: ref<users>, company_ref: ref<companies>";
    "tags: array<ref<tag_collection>>, metadata: array<array<string>>";
    "name: STRING, age: INT, height: FLOAT, active: BOOLEAN, id: ID_INT, email: ID_STRING";
    "items: ARRAY<STRING>, refs: Array<Ref<users>>";
}

#[test]
fn nullable_constraint() {
    "name: string(nullable), age: int ( nullable )";
    "items: array<string>(nullable)";
    "user_ref: ref<users>(nullable)";
    "name: String(NULLABLE), active: Boolean(Default = true)";
}

#[test]
fn default_values_valid() {
    "age: int(default = 25), active: boolean(default = true)";
    "name: string(default = \"John Doe\")";
    "score: float(default = 95.5)";
    "description: string(nullable)(default = null)";
    "tags: array<string>(default = [\"dev\", \"test\"])";
    "owner: ref<users>(default = \"admin\")";
    "id: id_int, name: string(default = \"Anonymous\"), age: int";
}

#[test]
fn default_values_invalid() {
    "id: id_int(default = 1)";
    "user_id: id_string(default = abc123)";
    "uuid: id_string(default = \"uuid-123\")";
    "age: int(default = abc)";
    "age: int(default = 3.14)";
    "height: float(default = not_a_number)";
    "active: boolean(default = maybe)";
    "active: boolean(default = 1)";
    "age: int(default = null)";
    "score: float(default = null)";
    "active: boolean(default = null)";
    "tags: array<string>(default = [1, 2, 3])";
    "numbers: array<int>(default = [\"a\", \"b\"])";
    "items: array<string>(default = 1, 2, 3)";
}

#[test]
fn empty() {
    "";
    "   ";
    "\t\n  \r\n";
}

#[test]
fn extra_whitespace() {
    "  name:string,age:int  ";
    "\tname\t:\tstring\t,\tage\t:\tint\t";
    "name : string ( nullable ) ( default = \"John\" )";
    " items : array < string > ";
    " user_ref : ref < users > ";
}

#[test]
fn invalid_syntax() {
    "name string";
    "name:";
    ":string";
    "name: string,";
    ",name: string";
    "name: string,,age: int";
    "name: string(";
    "name: string)";
    "name: string()";
    "name: string(nullable";
    "name: string nullable)";
    "name: array<";
    "name: array>";
    "name: array<>";
    "name: array string>";
    "name: array<string";
    "name: ref<";
    "name: ref>";
    "name: ref<>";
    "name: ref users>";
    "name: ref<users";
    "name: string(= value)";
    "name: string(default value)";
}

#[test]
fn invalid_field_types() {
    "name: text";
    "age: integer";
    "price: double";
    "active: bool";
    "id: id";
    "id: identifier";
    "items: list<string>";
    "items: vector<int>";
    "items: array";
    "user_ref: reference<users>";
    "user_ref: link<users>";
    "user_ref: ref";
    "data: object";
    "data: map";
    "data: json";
    "timestamp: datetime";
    "timestamp: date";
}

#[test]
fn invalid_constraints() {
    "name: string(required)";
    "name: string(optional)";
    "name: string(unique)";
    "name: string(indexed)";
    "age: int(min = 0)";
    "age: int(max = 100)";
    "name: string(length = 50)";
    "name: string(invalid_constraint)";
    "name: string(constraint_without_value = )";
    "name: string( = value)";
    "name: string(not_nullable)";
    "name: string(non_null)";
    "name: string(null)";
    "name: string(default)";
    "name: string(= test)";
}

#[test]
fn field_modifications() {
    "name: string, age: drop";
}
