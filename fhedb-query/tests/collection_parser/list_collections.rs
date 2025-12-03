#[test]
fn basic() {
    "LIST COLLECTIONS";
}

#[test]
fn case_insensitive() {
    "LiSt CoLlEcTiOnS";
}

#[test]
fn with_extra_whitespace() {
    "   LIST    COLLECTIONS   ";
}

#[test]
fn invalid_empty() {
    "";
}

#[test]
fn invalid_missing_collections() {
    "LIST";
}

#[test]
fn invalid_extra_input() {
    "LIST COLLECTIONS EXTRA_STUFF";
}

#[test]
fn invalid_wrong_keyword() {
    "LIST COLLECTION";
}

#[test]
fn invalid_wrong_order() {
    "COLLECTIONS LIST";
}
