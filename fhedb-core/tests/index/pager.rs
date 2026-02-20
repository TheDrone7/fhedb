use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

use fhedb_core::prelude::{PAGE_SIZE, Pager};
use tempfile::tempdir;

#[test]
fn create_file_and_metadata_page() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");
    let pager = Pager::new(&path).unwrap();

    assert_eq!(pager.page_count(), 1);
    assert_eq!(pager.root_page_num(), 0);
    assert_eq!(pager.free_page_num(), 0);
    assert_eq!(path.metadata().unwrap().len(), PAGE_SIZE as u64);
}

#[test]
fn reopen_existing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    {
        let mut pager = Pager::new(&path).unwrap();
        pager.set_root(5).unwrap();
    }

    let pager = Pager::new(&path).unwrap();
    assert_eq!(pager.root_page_num(), 5);
    assert_eq!(pager.free_page_num(), 0);
    assert_eq!(pager.page_count(), 1);
}

#[test]
fn reject_corrupted_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut file = File::create(&path).unwrap();
    file.write_all(&[0u8; PAGE_SIZE + 1]).unwrap();
    drop(file);

    let result = Pager::new(&path);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn accept_empty_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    File::create(&path).unwrap();
    assert_eq!(path.metadata().unwrap().len(), 0);

    let pager = Pager::new(&path).unwrap();
    assert_eq!(pager.page_count(), 1);
    assert_eq!(pager.root_page_num(), 0);
    assert_eq!(pager.free_page_num(), 0);
}

#[test]
fn persist_metadata_across_reopens() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    {
        let mut pager = Pager::new(&path).unwrap();
        let page1 = pager.allocate_page().unwrap();
        pager.set_root(page1).unwrap();

        let page2 = pager.allocate_page().unwrap();
        pager.free_page(page2).unwrap();
    }

    let pager = Pager::new(&path).unwrap();
    assert_eq!(pager.root_page_num(), 1);
    assert_eq!(pager.free_page_num(), 2);
    assert_eq!(pager.page_count(), 3);
}

#[test]
fn write_page_stores_data_on_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();
    let page_num = pager.allocate_page().unwrap();

    let page_data = [0xABu8; PAGE_SIZE];
    pager.write_page(page_num, &page_data).unwrap();

    let mut file = File::open(&path).unwrap();
    file.seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))
        .unwrap();
    let mut buf = [0u8; PAGE_SIZE];
    file.read_exact(&mut buf).unwrap();
    assert_eq!(buf, page_data);
}

#[test]
fn write_page_rejects_out_of_bounds() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();

    let result = pager.write_page(100, &[0u8; PAGE_SIZE]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn read_page_loads_data_from_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();
    let page_num = pager.allocate_page().unwrap();

    let page_data = [0xCDu8; PAGE_SIZE];
    {
        let mut file = File::options().write(true).open(&path).unwrap();
        file.seek(SeekFrom::Start(page_num as u64 * PAGE_SIZE as u64))
            .unwrap();
        file.write_all(&page_data).unwrap();
    }

    let result = pager.read_page(page_num).unwrap();
    assert_eq!(result, page_data);
}

#[test]
fn read_page_rejects_out_of_bounds() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();

    let result = pager.read_page(100);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn allocate_page_appends_sequentially() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();
    assert_eq!(pager.page_count(), 1);

    let p1 = pager.allocate_page().unwrap();
    let p2 = pager.allocate_page().unwrap();
    let p3 = pager.allocate_page().unwrap();

    assert_eq!(p1, 1);
    assert_eq!(p2, 2);
    assert_eq!(p3, 3);
    assert_eq!(pager.page_count(), 4);
    assert_eq!(path.metadata().unwrap().len(), 4 * PAGE_SIZE as u64);
}

#[test]
fn allocate_page_reuses_freed_pages() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();
    let p1 = pager.allocate_page().unwrap();
    let _p2 = pager.allocate_page().unwrap();

    let page_count_before = pager.page_count();
    pager.free_page(p1).unwrap();

    let reused = pager.allocate_page().unwrap();
    assert_eq!(reused, p1);
    assert_eq!(pager.page_count(), page_count_before);
}

#[test]
fn free_list_is_lifo() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();
    let p1 = pager.allocate_page().unwrap();
    let p2 = pager.allocate_page().unwrap();
    let p3 = pager.allocate_page().unwrap();

    pager.free_page(p1).unwrap();
    pager.free_page(p2).unwrap();
    pager.free_page(p3).unwrap();

    assert_eq!(pager.allocate_page().unwrap(), p3);
    assert_eq!(pager.allocate_page().unwrap(), p2);
    assert_eq!(pager.allocate_page().unwrap(), p1);
}

#[test]
fn free_page_rejects_invalid_pages() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let mut pager = Pager::new(&path).unwrap();

    let result = pager.free_page(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);

    let result = pager.free_page(100);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
}
