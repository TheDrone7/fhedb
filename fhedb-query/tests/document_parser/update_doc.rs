#[test]
fn basic() {
    "UPDATE DOCUMENT IN users {id = 1, name: \"Jane Doe\", age: 30}";
}

#[test]
fn variations() {
    "UPDATE DOC IN products {id = \"prod_123\", price: 1099.99, stock: 50}";
    "   UpDaTe    DoC    iN    MyCollection   {field: 'value', num: 42}   ";
}

#[test]
fn complex_data_types() {
    "UPDATE DOCUMENT IN products {
        id = 42,
        name: \"Gaming Laptop\",
        price: 1299.99,
        in_stock: true,
        tags: [\"gaming\", \"laptop\"],
        config: \"{\\\"theme\\\": \\\"dark\\\"}\",
        matrix: [[1, 2], [3, 4]],
        path: \"C:\\Users\\Admin\"
    }";
}

#[test]
fn nested_assignments() {
    "UPDATE DOCUMENT IN users {
        id = 1,
        address: {city: \"Boston\", zipcode: \"02101\"},
        profile: {bio: \"Developer\", age: 30}
    }";
}

#[test]
fn with_conditions() {
    "UPDATE DOCUMENT IN users {
        id = 1,
        age > 18,
        name: \"Updated Name\",
        status: \"active\"
    }";
}

#[test]
fn with_selectors() {
    "UPDATE DOCUMENT IN users {
        id = 1,
        name: \"John\",
        email,
        age
    }";
}

#[test]
fn nested_with_wildcards() {
    "UPDATE DOCUMENT IN users {
        id = 1,
        profile: {name: \"John\"},
        address {*}
    }";
}

#[test]
fn invalid_syntax() {
    "";
    "UPDATE";
    "UPDATE DOCUMENT";
    "UPDATE DOCUMENT IN";
    "UPDATE DOC IN users";
    "DOCUMENT UPDATE IN users {id = 1, name: \"test\"}";
    "UPDATE DOCUMENT ON users {id = 1, name: \"test\"}";
}

#[test]
fn invalid_field_structure() {
    "UPDATE DOC IN users {id 1}";
    "UPDATE DOC IN users {id = 1";
    "UPDATE DOCUMENT IN users {id = 1} EXTRA";
}

#[test]
fn invalid_mixed_assignment_selector() {
    "UPDATE DOC IN users {id = 1, name}";
}
