#[test]
fn basic() {
    "DELETE DOCUMENT FROM users {id = 1}";
}

#[test]
fn with_remove_keyword() {
    "REMOVE DOC FROM products {id = \"prod_123\"}";
}

#[test]
fn invalid_empty_delete_query() {
    "DELETE DOCUMENT FROM users {}";
}

#[test]
fn variations() {
    "DELETE DOC FROM products {id = \"prod_123\"}";
    "   DeLeTe    DoCs    fRoM    MyCollection   {status = 'active'}   ";
    "REMOVE DOCUMENTS FROM test_collection {active = true}";
}

#[test]
fn all_operators() {
    "DELETE DOCUMENT FROM users {
        id = 1,
        age > 18,
        salary >= 50000,
        rating < 5.0,
        experience <= 10,
        name != 'admin',
        bio == 'developer'
    }";
}

#[test]
fn with_selectors() {
    "DELETE DOCUMENT FROM users {id = 1, name, email}";
}

#[test]
fn wildcard_selectors() {
    "DELETE DOCUMENT FROM users {id = 1, *}";
    "DELETE DOCUMENT FROM users {id = 1, **}";
}

#[test]
fn simple_nested() {
    "DELETE DOCUMENT FROM users {id = 1, address {city, country}}";
}

#[test]
fn nested_with_conditions() {
    "DELETE DOCUMENT FROM users {
        id = 1,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
}

#[test]
fn only_conditions() {
    "DELETE DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
}

#[test]
fn only_selectors() {
    "DELETE DOCUMENT FROM users {name, email, age}";
}

#[test]
fn mixed_syntax() {
    "DELETE DOCUMENT FROM users {
        id = 1,
        name,
        age > 18,
        address {city = 'NYC'},
        *
    }";
}

#[test]
fn complex_values() {
    "DELETE DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\\\Users\\\\Admin\",
        matrix = [[1, 2], [3, 4]]
    }";
}

#[test]
fn invalid_syntax() {
    "";
    "DELETE";
    "DELETE DOCUMENT";
    "DELETE DOCUMENT FROM";
    "DELETE DOC FROM users";
    "DOCUMENT DELETE FROM users {id = 1}";
    "DELETE DOCUMENT IN users {id = 1}";
}

#[test]
fn invalid_field_structure() {
    "DELETE DOC FROM users {id 1}";
    "DELETE DOC FROM users {id = 1";
    "DELETE DOCUMENT FROM users {id = 1} EXTRA";
}

#[test]
fn invalid_empty_query() {
    "DELETE DOCUMENT FROM users {}";
}

#[test]
fn invalid_assignments() {
    "DELETE DOC FROM users {id = 1, name: 'John'}";
}

#[test]
fn invalid_assignments_in_nested() {
    "DELETE DOC FROM users {id = 1, address {city: 'NYC'}}";
}

#[test]
fn invalid_malformed_nested() {
    "DELETE DOC FROM users {address {city, country}";
    "DELETE DOC FROM users {address city, country}}";
    "DELETE DOC FROM users {address {{city}}";
}

#[test]
fn invalid_empty_conditions() {
    "DELETE DOC FROM users {id =}";
    "DELETE DOC FROM users {= 1}";
}

#[test]
fn invalid_unsupported_operators() {
    "DELETE DOC FROM users {id ~ 1}";
    "DELETE DOC FROM users {id & 1}";
}
