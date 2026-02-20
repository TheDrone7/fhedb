use chumsky::Parser;
use fhedb_query::lexer::{Token, lexer};

fn parse_identifier(input: &str) -> Option<String> {
    let tokens = lexer().parse(input).into_result().ok()?;
    if tokens.len() == 1
        && let Token::Ident(s) = &tokens[0].0
    {
        return Some(s.clone());
    }
    None
}

fn is_identifier(input: &str) -> bool {
    parse_identifier(input).is_some()
}

#[test]
fn valid_identifiers() {
    assert!(is_identifier("database_123"));
    assert!(is_identifier("größe_tabelle"),);
    assert!(is_identifier("数据库"));
    assert!(is_identifier("データベース"));
    assert!(is_identifier("قاعدة_البيانات"));
    assert!(is_identifier("база_данных"));
}

#[test]
fn invalid_identifiers() {
    assert!(!is_identifier(""));
    assert!(!is_identifier(" database"));
    assert!(!is_identifier("-database"));
    assert!(!is_identifier(".database"));
    assert!(!is_identifier("@user"));
    assert!(!is_identifier("#collection"));
    assert!(!is_identifier("()"));
}

#[test]
fn partial_identifiers() {
    assert!(!is_identifier("database-name"));
    assert!(!is_identifier("database.name"));
    assert!(!is_identifier("database name"));
    assert!(!is_identifier("func()"));
}

#[test]
fn reserved_query_keywords() {
    assert!(!is_identifier("create"));
    assert!(!is_identifier("drop"));
    assert!(!is_identifier("list"));
    assert!(!is_identifier("database"));
    assert!(!is_identifier("databases"));
    assert!(!is_identifier("collection"));
    assert!(!is_identifier("collections"));
    assert!(!is_identifier("if"));
    assert!(!is_identifier("exists"));
    assert!(!is_identifier("schema"));
    assert!(!is_identifier("from"));
    assert!(!is_identifier("get"));
    assert!(!is_identifier("modify"));
    assert!(!is_identifier("alter"));

    assert!(!is_identifier("CREATE"));
    assert!(!is_identifier("DrOp"));
    assert!(!is_identifier("DATABASE"));
}

#[test]
fn reserved_type_keywords() {
    assert!(!is_identifier("int"));
    assert!(!is_identifier("float"));
    assert!(!is_identifier("string"));
    assert!(!is_identifier("boolean"));
    assert!(!is_identifier("array"));
    assert!(!is_identifier("ref"));
    assert!(!is_identifier("id_int"));
    assert!(!is_identifier("id_string"));

    assert!(!is_identifier("INT"));
    assert!(!is_identifier("String"));
    assert!(!is_identifier("BOOLEAN"));
    assert!(!is_identifier("ID_INT"));
}

#[test]
fn reserved_constraint_keywords() {
    assert!(!is_identifier("nullable"));
    assert!(!is_identifier("default"));

    assert!(!is_identifier("NULLABLE"));
    assert!(!is_identifier("Default"));
}

#[test]
fn reserved_literal_keywords() {
    assert!(!is_identifier("true"));
    assert!(!is_identifier("false"));
    assert!(!is_identifier("null"));

    assert!(!is_identifier("TRUE"));
    assert!(!is_identifier("False"));
    assert!(!is_identifier("NULL"));
}

#[test]
fn keywords_as_substrings_allowed() {
    assert!(is_identifier("created_at"));
    assert!(is_identifier("is_dropped"));
    assert!(is_identifier("listing"));
    assert!(is_identifier("integer_value"));
    assert!(is_identifier("floating_point"));
    assert!(is_identifier("string_value"));
    assert!(is_identifier("arrays_data"));
    assert!(is_identifier("reference_id"));
    assert!(is_identifier("is_nullable"));
    assert!(is_identifier("default_value"));
    assert!(is_identifier("is_true"));
    assert!(is_identifier("not_false"));
    assert!(is_identifier("null_count"));
    assert!(is_identifier("my_int_field"));
    assert!(is_identifier("get_data"));
    assert!(is_identifier("create_user"));
}
