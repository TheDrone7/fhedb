use fhedb_query::error::create_span;
use fhedb_query::parser::utilities::identifier;

#[test]
fn valid_identifiers() {
    let (remaining, result) = identifier(create_span("database_123")).unwrap();
    assert_eq!(*result.fragment(), "database_123");
    assert_eq!(*remaining.fragment(), "");

    let (remaining, result) = identifier(create_span("größe_tabelle")).unwrap();
    assert_eq!(*result.fragment(), "größe_tabelle");
    assert_eq!(*remaining.fragment(), "");

    let (remaining, result) = identifier(create_span("数据库")).unwrap();
    assert_eq!(*result.fragment(), "数据库");
    assert_eq!(*remaining.fragment(), "");

    let (remaining, result) = identifier(create_span("データベース")).unwrap();
    assert_eq!(*result.fragment(), "データベース");
    assert_eq!(*remaining.fragment(), "");

    let (remaining, result) = identifier(create_span("قاعدة_البيانات")).unwrap();
    assert_eq!(*result.fragment(), "قاعدة_البيانات");
    assert_eq!(*remaining.fragment(), "");

    let (remaining, result) = identifier(create_span("база_данных")).unwrap();
    assert_eq!(*result.fragment(), "база_данных");
    assert_eq!(*remaining.fragment(), "");
}

#[test]
fn invalid_identifiers() {
    assert!(identifier(create_span("")).is_err());
    assert!(identifier(create_span(" database")).is_err());
    assert!(identifier(create_span("-database")).is_err());
    assert!(identifier(create_span(".database")).is_err());
    assert!(identifier(create_span("@user")).is_err());
    assert!(identifier(create_span("#collection")).is_err());
    assert!(identifier(create_span("()")).is_err());
}

#[test]
fn partial_identifiers() {
    let (remaining, result) = identifier(create_span("database-name")).unwrap();
    assert_eq!(*result.fragment(), "database");
    assert_eq!(*remaining.fragment(), "-name");

    let (remaining, result) = identifier(create_span("database.name")).unwrap();
    assert_eq!(*result.fragment(), "database");
    assert_eq!(*remaining.fragment(), ".name");

    let (remaining, result) = identifier(create_span("database name")).unwrap();
    assert_eq!(*result.fragment(), "database");
    assert_eq!(*remaining.fragment(), " name");

    let (remaining, result) = identifier(create_span("func()")).unwrap();
    assert_eq!(*result.fragment(), "func");
    assert_eq!(*remaining.fragment(), "()");
}
