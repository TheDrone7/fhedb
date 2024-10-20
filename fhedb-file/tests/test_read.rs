mod test_setup;

use fhedb_core::prelude::*;
use fhedb_file::prelude::*;
use test_setup::{setup_once, teardown};

#[test]
fn read_file_to_db() {
    let path = "tests/test.fhedb";
    setup_once(path);

    let db = DbMetadata::read_file(path).unwrap();
    assert_eq!(db.name, "test");

    teardown(path);
}
