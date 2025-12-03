#[test]
fn basic() {
    "DROP DATABASE test_db";
}

#[test]
fn case_insensitive() {
    "DrOp DaTaBaSe MyDatabase";
}

#[test]
fn with_extra_whitespace() {
    "   DROP    DATABASE    test_db   ";
}

#[test]
fn invalid_empty() {
    "";
}

#[test]
fn invalid_missing_name() {
    "DROP DATABASE";
}

#[test]
fn invalid_extra_input() {
    "DROP DATABASE test_db EXTRA_STUFF";
}

#[test]
fn invalid_no_keyword() {
    "DROP test_db";
}

#[test]
fn invalid_wrong_order() {
    "DATABASE DROP test_db";
}
