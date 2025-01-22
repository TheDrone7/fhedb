mod test_setup;

use test_setup::{setup_once, teardown};

#[test]
fn update_db_metadata_in_file() {
    let path = "tests/test_update.fhedb";
    setup_once(path);
    use fhedb_core::prelude::*;
    use fhedb_file::prelude::*;

    let mut db = DbMetadata::read_file(path).unwrap();
    assert_eq!(db.name, "test_update");

    // Extend the contents
    db.version = "longer_version".to_owned();
    db.update_file(path).unwrap();
    let mut db = DbMetadata::read_file(path).unwrap();
    assert_eq!(db.version, "longer_version");

    // Shorten the contents
    db.version = "sv".to_owned();
    db.update_file(path).unwrap();
    let db = DbMetadata::read_file(path).unwrap();
    assert_eq!(db.version, "sv");

    teardown(path);
}
