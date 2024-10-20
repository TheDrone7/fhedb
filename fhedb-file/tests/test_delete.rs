mod test_setup;

use fhedb_core::prelude::*;
use fhedb_file::prelude::*;
use test_setup::{setup_once, teardown};

#[test]
fn delete_db_file() {
    let path = "tests/test_delete.fhedb";
    setup_once(path);

    let db = DbMetadata::from_file(path).unwrap();
    assert_eq!(db.name, "test");

    db.delete_file(path).unwrap();
    let db = DbMetadata::from_file(path);

    if db.is_err() {
        assert!(true);
        return;
    } else {
        teardown(path);
        assert!(false);
    }
}
