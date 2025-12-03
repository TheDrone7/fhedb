#[test]
fn basic() {
    "LIST DATABASES";
}

#[test]
fn case_insensitive() {
    "LiSt DaTaBaSeS";
}

#[test]
fn with_extra_whitespace() {
    "   LIST    DATABASES   ";
}

#[test]
fn invalid_missing_databases() {
    "LIST";
}

#[test]
fn invalid_extra_input() {
    "LIST DATABASES EXTRA_STUFF";
}

#[test]
fn invalid_wrong_order() {
    "DATABASES LIST";
}
