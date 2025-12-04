use chumsky::Parser;
use fhedb_query::lexer::{Token, lexer};

fn parse_identifier(input: &str) -> Option<String> {
    let tokens = lexer().parse(input).into_result().ok()?;
    if tokens.len() == 1 {
        if let Token::Ident(s) = &tokens[0].0 {
            return Some(s.clone());
        }
    }
    None
}

#[test]
fn valid_identifiers() {
    assert_eq!(
        parse_identifier("database_123"),
        Some("database_123".to_string())
    );
    assert_eq!(
        parse_identifier("größe_tabelle"),
        Some("größe_tabelle".to_string())
    );
    assert_eq!(parse_identifier("数据库"), Some("数据库".to_string()));
    assert_eq!(
        parse_identifier("データベース"),
        Some("データベース".to_string())
    );
    assert_eq!(
        parse_identifier("قاعدة_البيانات"),
        Some("قاعدة_البيانات".to_string())
    );
    assert_eq!(
        parse_identifier("база_данных"),
        Some("база_данных".to_string())
    );
}

#[test]
fn invalid_identifiers() {
    assert_eq!(parse_identifier(""), None);
    assert_eq!(parse_identifier(" database"), None);
    assert_eq!(parse_identifier("-database"), None);
    assert_eq!(parse_identifier(".database"), None);
    assert_eq!(parse_identifier("@user"), None);
    assert_eq!(parse_identifier("#collection"), None);
    assert_eq!(parse_identifier("()"), None);
}

#[test]
fn partial_identifiers() {
    assert_eq!(parse_identifier("database-name"), None);
    assert_eq!(parse_identifier("database.name"), None);
    assert_eq!(parse_identifier("database name"), None);
    assert_eq!(parse_identifier("func()"), None);
}
