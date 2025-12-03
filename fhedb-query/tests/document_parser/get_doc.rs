#[test]
fn basic() {
    "GET DOCUMENT FROM users {id = 1, name}";
}

#[test]
fn variations() {
    "GET DOC FROM products {id = \"prod_123\", price}";
    "   GeT    DoCs    fRoM    MyCollection   {status = 'active', name}   ";
    "GET DOCUMENTS FROM test_collection {*}";
}

#[test]
fn all_operators() {
    "GET DOCUMENT FROM users {
        id = 1,
        age > 18,
        salary >= 50000,
        rating < 5.0,
        experience <= 10,
        name != 'admin',
        bio == 'developer',
        username
    }";
}

#[test]
fn wildcard_selectors() {
    "GET DOCUMENT FROM users {id = 1, *}";
    "GET DOCUMENT FROM users {id = 1, **}";
}

#[test]
fn simple_nested() {
    "GET DOCUMENT FROM users {id = 1, address {city, country}}";
}

#[test]
fn nested_with_conditions() {
    "GET DOCUMENT FROM users {
        id = 1,
        address {city = 'NYC', country, zipcode != '10001'}
    }";
}

#[test]
fn deep_nesting() {
    "GET DOCUMENT FROM users {
        id = 1,
        profile {
            name,
            address {
                city = 'NYC',
                location {
                    lat > 40.0,
                    lng < -73.0,
                    details {*}
                }
            }
        }
    }";
}

#[test]
fn nested_wildcards() {
    "GET DOCUMENT FROM users {
        id = 1,
        profile {
            *,
            address {**}
        }
    }";
}

#[test]
fn mixed_syntax() {
    "GET DOCUMENT FROM users {
        id = 1,
        name,
        age > 18,
        address {city = 'NYC'},
        *
    }";
}

#[test]
fn empty_braces() {
    "GET DOCUMENT FROM users {id = 1, address {}}";
}

#[test]
fn only_conditions() {
    "GET DOCUMENT FROM users {id = 1, age > 18, status = 'active'}";
}

#[test]
fn only_selectors() {
    "GET DOCUMENT FROM users {name, email, age}";
}

#[test]
fn complex_values() {
    "GET DOCUMENT FROM products {
        tags == '[\"electronics\", \"mobile\"]',
        config = \"{\\\"theme\\\": \\\"dark\\\"}\",
        path != \"C:\\Users\\Admin\",
        matrix = [[1, 2], [3, 4]],
        name,
        price
    }";
}

#[test]
fn invalid_syntax() {
    "";
    "GET";
    "GET DOCUMENT";
    "GET DOCUMENT FROM";
    "GET DOC FROM users";
    "DOCUMENT GET FROM users {id = 1}";
    "GET DOCUMENT IN users {id = 1}";
}

#[test]
fn invalid_field_structure() {
    "GET DOC FROM users {id 1}";
    "GET DOC FROM users {id = 1";
    "GET DOCUMENT FROM users {id = 1} EXTRA";
}

#[test]
fn invalid_assignments_in_get() {
    "GET DOC FROM users {id = 1, name: 'John'}";
}

#[test]
fn invalid_assignments_in_nested() {
    "GET DOC FROM users {id = 1, address {city: 'NYC'}}";
}

#[test]
fn invalid_malformed_nested() {
    "GET DOC FROM users {address {city, country}";
    "GET DOC FROM users {address city, country}}";
    "GET DOC FROM users {address {{city}}";
}

#[test]
fn invalid_empty_conditions() {
    "GET DOC FROM users {id =}";
    "GET DOC FROM users {= 1}";
}

#[test]
fn invalid_unsupported_operators() {
    "GET DOC FROM users {id ~ 1}";
    "GET DOC FROM users {id & 1}";
}
