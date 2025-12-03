#[test]
fn basic() {
    "MODIFY COLLECTION users {name: string, age: drop}";
    "ALTER COLLECTION products {price: float, old_field: drop}";
}

#[test]
fn case_insensitive() {
    "MoDiFy CoLlEcTiOn MyCollection {FiElD: dROp}";
}

#[test]
fn with_extra_whitespace() {
    "   MODIFY    COLLECTION    test_collection   {field1: int, field2: drop}   ";
}

#[test]
fn invalid_empty() {
}

#[test]
fn invalid_missing_name() {
    "MODIFY COLLECTION";
}

#[test]
fn invalid_extra_input() {
    "MODIFY COLLECTION test_collection {field: int} EXTRA_STUFF";
}

#[test]
fn invalid_no_keyword() {
    "MODIFY test_collection {field: int}";
}

#[test]
fn invalid_wrong_order() {
    "COLLECTION MODIFY test_collection {field: int}";
}

