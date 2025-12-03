#[test]
fn basic() {
    "GET SCHEMA FROM users";
}

#[test]
fn case_insensitive() {
    "GeT sChEmA fRoM MyCollection";
}

#[test]
fn with_extra_whitespace() {
    "   GET    SCHEMA    FROM    test_collection   ";
}

#[test]
fn invalid_empty() {
}

#[test]
fn invalid_missing_schema() {
    "GET";
}

#[test]
fn invalid_missing_from() {
    "GET SCHEMA";
}

#[test]
fn invalid_missing_collection_name() {
    "GET SCHEMA FROM";
}

#[test]
fn invalid_extra_input() {
    "GET SCHEMA FROM users EXTRA_STUFF";
}

#[test]
fn invalid_wrong_keyword() {
    "GET SCHEMAS FROM users";
}

#[test]
fn invalid_wrong_order() {
    "SCHEMA GET FROM users";
}

