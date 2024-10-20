mod test_setup;

use test_setup::{setup_once, teardown};

#[test]
fn create_db_in_file() {
    let path = "tests/test_create.fhedb";
    setup_once(path);
    use fhedb_core::prelude::*;
    use fhedb_file::prelude::*;

    // Make sure it overwrites the existing file
    let db = DbMetadata::new("test_cr".to_owned());
    db.create_file(path).unwrap();

    let db = DbMetadata::from_file(path).unwrap();
    assert_eq!(db.name, "test_cr");

    teardown(path);
}
