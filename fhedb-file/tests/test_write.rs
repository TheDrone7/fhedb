mod test_setup;

use test_setup::{setup_once, teardown};

#[test]
fn write_db_to_file() {
	let path = "tests/test_write.fhedb";
	setup_once(path);
	use fhedb_core::prelude::*;
	use fhedb_file::prelude::*;

	let db = Database::new("test_write".to_owned());
	db.to_file(path).unwrap();

	let db = Database::from_file(path).unwrap();
	assert_eq!(db.name, "test_write");

	teardown(path);
}
