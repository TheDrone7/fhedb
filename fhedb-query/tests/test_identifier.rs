use fhedb_query::parser::utilities::identifier;

#[test]
fn valid_identifiers() {
    let (remaining, result) = identifier("database_123").unwrap();
    assert_eq!(result, "database_123");
    assert_eq!(remaining, "");

    let (remaining, result) = identifier("größe_tabelle").unwrap();
    assert_eq!(result, "größe_tabelle");
    assert_eq!(remaining, "");

    let (remaining, result) = identifier("数据库").unwrap();
    assert_eq!(result, "数据库");
    assert_eq!(remaining, "");

    let (remaining, result) = identifier("データベース").unwrap();
    assert_eq!(result, "データベース");
    assert_eq!(remaining, "");

    let (remaining, result) = identifier("قاعدة_البيانات").unwrap();
    assert_eq!(result, "قاعدة_البيانات");
    assert_eq!(remaining, "");

    let (remaining, result) = identifier("база_данных").unwrap();
    assert_eq!(result, "база_данных");
    assert_eq!(remaining, "");
}

#[test]
fn invalid_identifiers() {
    assert!(identifier("").is_err());
    assert!(identifier(" database").is_err());
    assert!(identifier("-database").is_err());
    assert!(identifier(".database").is_err());
    assert!(identifier("@user").is_err());
    assert!(identifier("#collection").is_err());
    assert!(identifier("()").is_err());
}

#[test]
fn partial_identifiers() {
    let (remaining, result) = identifier("database-name").unwrap();
    assert_eq!(result, "database");
    assert_eq!(remaining, "-name");

    let (remaining, result) = identifier("database.name").unwrap();
    assert_eq!(result, "database");
    assert_eq!(remaining, ".name");

    let (remaining, result) = identifier("database name").unwrap();
    assert_eq!(result, "database");
    assert_eq!(remaining, " name");

    let (remaining, result) = identifier("func()").unwrap();
    assert_eq!(result, "func");
    assert_eq!(remaining, "()");
}
