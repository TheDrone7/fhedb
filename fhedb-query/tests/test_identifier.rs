#[test]
fn valid_identifiers() {
    "database_123";
    "größe_tabelle";
    "数据库";
    "データベース";
    "قاعدة_البيانات";
    "база_данных";
}

#[test]
fn invalid_identifiers() {
    "";
    " database";
    "-database";
    ".database";
    "@user";
    "#collection";
    "()";
}

#[test]
fn partial_identifiers() {
    "database-name";
    "database.name";
    "database name";
    "func()";
}
