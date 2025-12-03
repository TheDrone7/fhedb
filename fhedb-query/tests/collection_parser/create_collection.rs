#[test]
fn basic() {
    "CREATE COLLECTION users {id: id_int, name: string, age: int}";
}

#[test]
fn case_insensitive() {
    "CrEaTe CoLlEcTiOn MyCollection {id: iD_stRiNg, title: string}";
}

#[test]
fn with_drop_if_exists() {
    "CREATE COLLECTION test_collection DROP IF EXISTS {id: id_int, data: string}";
}

#[test]
fn with_extra_whitespace() {
    "   CREATE    COLLECTION    test_collection   {id: id_int, name: string}   ";
    "   CREATE    COLLECTION    test_collection    DROP   IF   EXISTS   {id: id_string}   ";
}

#[test]
fn complex_schema() {
    "CREATE COLLECTION products {
        id: id_string,
        name: string(default = \"Unnamed\"),
        price: float,
        in_stock: boolean(default = true),
        tags: array<string>,
        category_ref: ref<categories>(nullable)
    }";
}

#[test]
fn nested_braces_in_strings() {
    "CREATE COLLECTION test {
        id: id_int,
        config: string(default = \"{\\\"key\\\": {\\\"nested\\\": \\\"value\\\"}}\"),
        template: string(default = \"Hello {name}, welcome to {place}!\")
    }";
}

#[test]
fn empty_schema() {
    "CREATE COLLECTION empty_collection {}";
}

#[test]
fn invalid_empty() {
    "";
}

#[test]
fn invalid_missing_name() {
    "CREATE COLLECTION";
}

#[test]
fn invalid_missing_schema() {
    "CREATE COLLECTION test_collection";
}

#[test]
fn invalid_extra_input() {
    "CREATE COLLECTION test_collection {id: id_int} EXTRA_STUFF";
}

#[test]
fn invalid_no_keyword() {
    "CREATE test_collection {id: id_int}";
}

#[test]
fn invalid_wrong_order() {
    "COLLECTION CREATE test_collection {id: id_int}";
}

#[test]
fn invalid_malformed_schema() {
    "CREATE COLLECTION test_collection {id: id_int,}";
}

#[test]
fn invalid_missing_braces() {
    "CREATE COLLECTION test_collection id: id_int";
}
