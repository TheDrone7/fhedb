use fhedb_core::prelude::{BPlusTree, InternalCell, LeafCell, Node, NodeType, PAGE_SIZE, Pager};
use tempfile::tempdir;

#[test]
fn initialize_empty_tree() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    assert!(tree.get(b"anything").unwrap().is_none());
}

#[test]
fn reopen_existing_tree() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    tree.insert(b"alpha", &[1u8; 16]).unwrap();
    tree.insert(b"beta", &[2u8; 16]).unwrap();
    tree.insert(b"gamma", &[3u8; 16]).unwrap();

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    assert_eq!(tree.get(b"alpha").unwrap(), Some([1u8; 16]));
    assert_eq!(tree.get(b"beta").unwrap(), Some([2u8; 16]));
    assert_eq!(tree.get(b"gamma").unwrap(), Some([3u8; 16]));
    assert!(tree.get(b"delta").unwrap().is_none());
}

#[test]
fn insert_duplicate_key_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    tree.insert(b"key1", &[1u8; 16]).unwrap();

    let result = tree.insert(b"key1", &[2u8; 16]);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().kind(),
        std::io::ErrorKind::AlreadyExists
    );

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    assert_eq!(tree.get(b"key1").unwrap(), Some([1u8; 16]));
}

#[test]
fn insert_key_too_large() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let big_key = [0xFFu8; 4096];
    let result = tree.insert(&big_key, &[1u8; 16]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);

    let pager = tree.pager();
    assert_eq!(pager.page_count(), 2);
    assert_eq!(pager.root_page_num(), 1);
    let mut root_page = pager.read_page(1).unwrap();
    let root = Node::new(&mut root_page);
    assert_eq!(root.get_header().keys_count, 0);
}

#[test]
fn insert_triggers_splits() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let initial_root = tree.pager().root_page_num();

    for i in 0..20u32 {
        let key = format!("key_{:0>200}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    let root_after_leaf_splits = tree.pager().root_page_num();
    assert_ne!(root_after_leaf_splits, initial_root);

    for i in 20..300u32 {
        let key = format!("key_{:0>200}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    let root_after_internal_splits = tree.pager().root_page_num();
    assert_ne!(root_after_internal_splits, root_after_leaf_splits);

    for i in 0..300u32 {
        let key = format!("key_{:0>200}", i);
        assert!(tree.get(key.as_bytes()).unwrap().is_some());
    }
}

#[test]
fn split_leaf_distributes_cells() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let mut left_page = [0u8; PAGE_SIZE];
    let mut left = Node::new(&mut left_page);
    left.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 6] = [b"aaa", b"bbb", b"ccc", b"ddd", b"eee", b"fff"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        left.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    let mut right_page = [0u8; PAGE_SIZE];
    let separator = tree
        .split_leaf(&mut left_page, &mut right_page, 42)
        .unwrap();

    let left = Node::new(&mut left_page);
    let left_header = left.get_header();
    assert_eq!(left_header.keys_count, 3);
    assert_eq!(left_header.next_page, 42);
    assert_eq!(left.get_key_at(0), b"aaa");
    assert_eq!(left.get_key_at(1), b"bbb");
    assert_eq!(left.get_key_at(2), b"ccc");

    let right = Node::new(&mut right_page);
    let right_header = right.get_header();
    assert_eq!(right_header.keys_count, 3);
    assert_eq!(right_header.node_type, NodeType::Leaf);
    assert_eq!(right.get_key_at(0), b"ddd");
    assert_eq!(right.get_key_at(1), b"eee");
    assert_eq!(right.get_key_at(2), b"fff");

    assert_eq!(separator, b"ddd");
}

#[test]
fn split_internal_distributes_cells() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let mut left_page = [0u8; PAGE_SIZE];
    let mut left = Node::new(&mut left_page);
    left.init(NodeType::Internal, 0);
    let mut header = left.get_header();
    header.first_child = 100;
    left.set_header(header);

    let keys: [&[u8]; 5] = [b"bbb", b"ddd", b"fff", b"hhh", b"jjj"];
    let children: [u32; 5] = [101, 102, 103, 104, 105];
    for (i, (key, child)) in keys.iter().zip(children.iter()).enumerate() {
        let cell = InternalCell {
            key,
            child_page: *child,
        };
        left.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    let mut right_page = [0u8; PAGE_SIZE];
    let (separator, adopted) = tree
        .split_internal(&mut left_page, &mut right_page, 42)
        .unwrap();

    let left = Node::new(&mut left_page);
    let left_header = left.get_header();
    assert_eq!(left_header.keys_count, 2);
    assert_eq!(left_header.first_child, 100);
    assert_eq!(left_header.next_page, 42);
    assert_eq!(left.get_key_at(0), b"bbb");
    assert_eq!(left.get_key_at(1), b"ddd");

    let right = Node::new(&mut right_page);
    let right_header = right.get_header();
    assert_eq!(right_header.keys_count, 2);
    assert_eq!(right_header.node_type, NodeType::Internal);
    assert_eq!(right_header.first_child, 103);
    assert_eq!(right.get_key_at(0), b"hhh");
    assert_eq!(right.get_key_at(1), b"jjj");

    assert_eq!(separator, b"fff");
    assert_eq!(adopted, vec![103, 104, 105]);
}

#[test]
fn update_existing_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    tree.insert(b"key1", &[1u8; 16]).unwrap();

    let page_count_before = tree.pager().page_count();
    assert_eq!(tree.get(b"key1").unwrap(), Some([1u8; 16]));

    tree.update(b"key1", &[9u8; 16]).unwrap();

    assert_eq!(tree.pager().page_count(), page_count_before);
    assert_eq!(tree.get(b"key1").unwrap(), Some([9u8; 16]));
}

#[test]
fn update_nonexistent_key_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let page_count_before = tree.pager().page_count();
    let result = tree.update(b"missing", &[1u8; 16]);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    assert_eq!(tree.pager().page_count(), page_count_before);
}

#[test]
fn delete_existing_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    tree.insert(b"key1", &[1u8; 16]).unwrap();
    tree.insert(b"key2", &[2u8; 16]).unwrap();

    let root = tree.pager().root_page_num();
    let mut root_page = tree.pager().read_page(root).unwrap();
    assert_eq!(Node::new(&mut root_page).get_header().keys_count, 2);

    let page_count_before = tree.pager().page_count();
    tree.delete(b"key1").unwrap();

    let mut root_page = tree.pager().read_page(root).unwrap();
    assert_eq!(Node::new(&mut root_page).get_header().keys_count, 1);

    assert!(tree.get(b"key1").unwrap().is_none());
    assert_eq!(tree.get(b"key2").unwrap(), Some([2u8; 16]));
    assert_eq!(tree.pager().page_count(), page_count_before);
}

#[test]
fn delete_nonexistent_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();
    tree.insert(b"key1", &[1u8; 16]).unwrap();

    let page_count_before = tree.pager().page_count();
    tree.delete(b"missing").unwrap();

    assert_eq!(tree.get(b"key1").unwrap(), Some([1u8; 16]));
    assert_eq!(tree.pager().page_count(), page_count_before);
}

#[test]
fn delete_triggers_merge() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    for i in 0..20u32 {
        let key = format!("key_{:0>200}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    assert_eq!(tree.pager().free_page_num(), 0);

    for i in 0..15u32 {
        let key = format!("key_{:0>200}", i);
        tree.delete(key.as_bytes()).unwrap();
    }

    assert_ne!(tree.pager().free_page_num(), 0);

    for i in 15..20u32 {
        let key = format!("key_{:0>200}", i);
        assert!(tree.get(key.as_bytes()).unwrap().is_some());
    }
}

#[test]
fn merge_leaves_combines_nodes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    let left_page_num = tree.pager().allocate_page().unwrap();
    let right_page_num = tree.pager().allocate_page().unwrap();
    let parent_page_num = tree.pager().allocate_page().unwrap();

    let mut left_page = [0u8; PAGE_SIZE];
    let mut left = Node::new(&mut left_page);
    left.init(NodeType::Leaf, parent_page_num);
    let mut lh = left.get_header();
    lh.next_page = right_page_num;
    left.set_header(lh);
    let cell_a = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    let cell_b = LeafCell {
        key: b"bbb",
        value: &[2u8; 16],
    };
    left.insert_cell(0, &cell_a.to_bytes()).unwrap();
    left.insert_cell(1, &cell_b.to_bytes()).unwrap();
    tree.pager().write_page(left_page_num, &left_page).unwrap();

    let mut right_page = [0u8; PAGE_SIZE];
    let mut right = Node::new(&mut right_page);
    right.init(NodeType::Leaf, parent_page_num);
    let cell_c = LeafCell {
        key: b"ccc",
        value: &[3u8; 16],
    };
    let cell_d = LeafCell {
        key: b"ddd",
        value: &[4u8; 16],
    };
    right.insert_cell(0, &cell_c.to_bytes()).unwrap();
    right.insert_cell(1, &cell_d.to_bytes()).unwrap();
    tree.pager()
        .write_page(right_page_num, &right_page)
        .unwrap();

    let mut parent_page = [0u8; PAGE_SIZE];
    let mut parent = Node::new(&mut parent_page);
    parent.init(NodeType::Internal, 0);
    let mut ph = parent.get_header();
    ph.first_child = left_page_num;
    parent.set_header(ph);
    let sep = InternalCell {
        key: b"ccc",
        child_page: right_page_num,
    };
    parent.insert_cell(0, &sep.to_bytes()).unwrap();
    tree.pager()
        .write_page(parent_page_num, &parent_page)
        .unwrap();

    let result = tree.merge_leaves(left_page_num, right_page_num, parent_page_num, 0);
    assert!(result.is_ok());
    assert!(result.unwrap());

    let mut merged_page = tree.pager().read_page(left_page_num).unwrap();
    let merged = Node::new(&mut merged_page);
    assert_eq!(merged.get_header().keys_count, 4);
    assert_eq!(merged.get_header().next_page, 0);
    assert_eq!(merged.get_key_at(0), b"aaa");
    assert_eq!(merged.get_key_at(1), b"bbb");
    assert_eq!(merged.get_key_at(2), b"ccc");
    assert_eq!(merged.get_key_at(3), b"ddd");

    assert_ne!(tree.pager().free_page_num(), 0);
}

#[test]
fn scan_range() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    for i in 0..10u32 {
        let key = format!("key_{:03}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    let results = tree.scan(b"key_003", b"key_006");
    assert!(results.is_ok());

    let mut iter = results.unwrap();
    for i in 3u32..=6 {
        let entry = iter.next();
        assert!(entry.is_some());
        let (key, value) = entry.unwrap().unwrap();
        let expected_key = format!("key_{:03}", i);
        let expected_val: [u8; 16] = i.to_le_bytes().repeat(4).try_into().unwrap();
        assert_eq!(key, expected_key.as_bytes());
        assert_eq!(value, expected_val);
    }
    assert!(iter.next().is_none());
}

#[test]
fn scan_empty_range() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    for i in 0..5u32 {
        let key = format!("key_{:03}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    let mut iter = tree.scan(b"zzz_start", b"zzz_end").unwrap();
    assert!(iter.next().is_none());
}

#[test]
fn scan_across_leaves() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.idx");

    let pager = Pager::new(&path).unwrap();
    let mut tree = BPlusTree::open(pager).unwrap();

    for i in 0..20u32 {
        let key = format!("key_{:0>200}", i);
        tree.insert(
            key.as_bytes(),
            &i.to_le_bytes().repeat(4).try_into().unwrap(),
        )
        .unwrap();
    }

    let root_after = tree.pager().root_page_num();
    assert_ne!(root_after, 1);

    let start = format!("key_{:0>200}", 5u32);
    let end = format!("key_{:0>200}", 18u32);
    let mut iter = tree.scan(start.as_bytes(), end.as_bytes()).unwrap();
    for i in 5u32..=18 {
        let entry = iter.next();
        assert!(entry.is_some());
        let (key, value) = entry.unwrap().unwrap();
        let expected_key = format!("key_{:0>200}", i);
        let expected_val: [u8; 16] = i.to_le_bytes().repeat(4).try_into().unwrap();
        assert_eq!(key, expected_key.as_bytes());
        assert_eq!(value, expected_val);
    }
    assert!(iter.next().is_none());
}
