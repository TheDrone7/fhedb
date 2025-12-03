#[test]
fn basic() {
    "DROP COLLECTION test_collection";
}

#[test]
fn case_insensitive() {
    "DrOp CoLlEcTiOn MyCollection";
}

#[test]
fn with_extra_whitespace() {
    "   DROP    COLLECTION    test_collection   ";
}

#[test]
fn invalid_empty() {
    "";
}

#[test]
fn invalid_missing_name() {
    "DROP COLLECTION";
}

#[test]
fn invalid_extra_input() {
    "DROP COLLECTION test_collection EXTRA_STUFF";
}

#[test]
fn invalid_no_keyword() {
    "DROP test_collection";
}

#[test]
fn invalid_wrong_order() {
    "COLLECTION DROP test_collection";
}
