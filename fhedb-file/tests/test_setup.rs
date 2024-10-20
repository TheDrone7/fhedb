extern crate fhedb_file;

use std::fs::write;
use std::sync::OnceLock;

static PROJECT_ROOT: OnceLock<String> = OnceLock::new();
static TEST_SETUP: OnceLock<bool> = OnceLock::new();

pub fn setup_once(path: &str) {
    let setup_done = TEST_SETUP.get();
    if setup_done.is_none() {
        // Perform setup
        PROJECT_ROOT.get_or_init(|| std::env::var("CARGO_MANIFEST_DIR").unwrap());
        create_test_file(path);

        // Mark it as done
        let _ = TEST_SETUP.set(true);
    }
}

pub fn create_test_file(path: &str) {
    pub use fhedb_core::prelude::*;

    let project_root = PROJECT_ROOT.get().unwrap();
    let test_file = format!("{}/{}", project_root, path);

    let init = DbMetadata::new("test".to_owned());
    write(test_file, init.to_bytes()).unwrap();
}

pub fn teardown(path: &str) {
    use std::fs;
    fs::remove_file(path).unwrap();
}
