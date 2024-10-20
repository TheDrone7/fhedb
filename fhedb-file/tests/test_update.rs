mod test_setup;

use test_setup::{setup_once, teardown};

#[test]
fn update_db_metadata_in_file() {
    let path = "tests/test_update.fhedb";
    setup_once(path);
    use fhedb_core::prelude::*;
    use fhedb_file::prelude::*;

    // Make sure it overwrites the existing file
    let mut db = DbMetadata::from_file(path).unwrap();
    assert_eq!(db.name, "test");

	db.name = "test_update".to_owned();
	db.update_file(path).unwrap();

	let db = DbMetadata::from_file(path).unwrap();
	assert_eq!(db.name, "test_update");

    teardown(path);
}
