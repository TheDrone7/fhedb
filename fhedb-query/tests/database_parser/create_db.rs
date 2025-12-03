#[test]
fn basic() {
    "CREATE DATABASE test_db";
}

#[test]
fn case_insensitive() {
    "CrEaTe DaTaBaSe MyDatabase";
}

#[test]
fn with_drop_if_exists() {
    "CREATE DATABASE test_db DROP IF EXISTS";
}

#[test]
fn with_extra_whitespace() {
    "   CREATE    DATABASE    test_db   ";
    "   CREATE    DATABASE    test_db    DROP   IF   EXISTS   ";
}

#[test]
fn invalid_empty() {
    "";
}

#[test]
fn invalid_missing_name() {
    "CREATE DATABASE";
}

#[test]
fn invalid_extra_input() {
    "CREATE DATABASE test_db EXTRA_STUFF";
}

#[test]
fn invalid_no_keyword() {
    "CREATE test_db";
}

#[test]
fn invalid_wrong_order() {
    "DATABASE CREATE test_db";
}
