extern crate fhedb_file;
extern crate lazy_static;

use lazy_static::lazy_static;
use std::fs::write;
use std::sync::Mutex;

lazy_static! {
    pub static ref TEST_SETUP: Mutex<bool> = Mutex::new(false);
    pub static ref PROJECT_ROOT: String = std::env::var("CARGO_MANIFEST_DIR").unwrap();
}

pub fn setup_once(path: &str) {
    let mut setup_done = TEST_SETUP.lock().unwrap();
    if !*setup_done {
        // Perform setup
        create_test_file(path);

        // Mark it as done
        *setup_done = true;
    }
}

pub fn create_test_file(path: &str) {
    pub use fhedb_core::prelude::*;

    let project_root = PROJECT_ROOT.to_string();
    let test_file = format!("{}/{}", project_root, path);

    let init = DbMetadata::new("test".to_owned());

    write(test_file, init.to_bytes()).unwrap();
}

pub fn teardown(path: &str) {
    use std::fs;
    fs::remove_file(path).unwrap();
}
