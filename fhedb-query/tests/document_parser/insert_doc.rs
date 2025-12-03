#[test]
fn basic() {
    "INSERT DOCUMENT INTO users {id: 1, name: \"John Doe\"}";
}

#[test]
fn variations() {
    "INSERT DOC INTO products {id: \"prod_123\", price: 999.99}";
    "   InSeRt    DoC    iNtO    MyCollection   {field: 'value'}   ";
    "INSERT DOCUMENT INTO empty_test {}";
}

#[test]
fn complex_data_types() {
    "INSERT DOCUMENT INTO products {
        id: 42,
        name: \"Gaming Laptop\",
        price: 1299.99,
        in_stock: true,
        tags: [\"gaming\", \"laptop\"],
        config: \"{\\\"theme\\\": \\\"dark\\\"}\",
        matrix: [[1, 2], [3, 4]],
        path: \"C:\\\\Users\\\\Admin\"
    }";
}

#[test]
fn invalid_syntax() {
    "";
    "INSERT";
    "INSERT DOCUMENT";
    "INSERT DOCUMENT INTO";
    "INSERT DOC INTO users";
    "DOCUMENT INSERT INTO users {id: 1}";
    "INSERT DOCUMENT IN users {id: 1}";
}

#[test]
fn invalid_field_structure() {
    "INSERT DOC INTO users {id 1}";
    "INSERT DOC INTO users {id: 1";
    "INSERT DOCUMENT INTO users {id: 1} EXTRA";
}

#[test]
fn duplicate_fields() {
    "INSERT DOC INTO users {id: 1, name: \"John\", id: 2}";
}
