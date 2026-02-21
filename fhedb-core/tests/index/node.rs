use fhedb_core::prelude::*;

#[test]
fn header_bytes_round_trip() {
    let header = NodeHeader {
        node_type: NodeType::Leaf,
        keys_count: 42,
        heap_pointer: 3000,
        parent_page: 7,
        next_page: 12,
        first_child: 99,
    };

    let bytes = header.to_bytes();
    let restored = NodeHeader::from_bytes(&bytes);

    assert_eq!(restored.node_type, NodeType::Leaf);
    assert_eq!(restored.keys_count, 42);
    assert_eq!(restored.heap_pointer, 3000);
    assert_eq!(restored.parent_page, 7);
    assert_eq!(restored.next_page, 12);
    assert_eq!(restored.first_child, 99);
}

#[test]
fn leaf_cell_bytes_round_trip() {
    let key = b"hello";
    let value = &[0x2Bu8; 16];
    let cell = LeafCell { key, value };

    let bytes = cell.to_bytes();
    let restored = LeafCell::from_bytes(&bytes);

    assert_eq!(restored.key, b"hello");
    assert_eq!(restored.value, &[0x2Bu8; 16]);
}

#[test]
fn internal_cell_bytes_round_trip() {
    let key = b"world";
    let cell = InternalCell {
        key,
        child_page: 49,
    };

    let bytes = cell.to_bytes();
    let restored = InternalCell::from_bytes(&bytes);

    assert_eq!(restored.key, b"world");
    assert_eq!(restored.child_page, 49);
}

#[test]
fn init() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 5);
    let header = node.get_header();
    assert_eq!(header.node_type, NodeType::Leaf);
    assert_eq!(header.keys_count, 0);
    assert_eq!(header.heap_pointer, PAGE_SIZE as u16);
    assert_eq!(header.parent_page, 5);
    assert_eq!(header.next_page, 0);
    assert_eq!(header.first_child, 0);

    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Internal, 3);
    let header = node.get_header();
    assert_eq!(header.node_type, NodeType::Internal);
}

#[test]
fn insert_single_cell() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell = LeafCell {
        key: b"abc",
        value: &[1u8; 16],
    };
    node.insert_cell(0, &cell.to_bytes()).unwrap();

    assert_eq!(node.get_header().keys_count, 1);
    assert_eq!(node.get_key_at(0), b"abc");
}

#[test]
fn insert_multiple_cells_in_order() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 4] = [b"aaa", b"bbb", b"ccc", b"ddd"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    assert_eq!(node.get_header().keys_count, 4);
    for (i, key) in keys.iter().enumerate() {
        assert_eq!(node.get_key_at(i as u16), *key);
    }
}

#[test]
fn insert_cell_at_beginning() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell_b = LeafCell {
        key: b"bbb",
        value: &[2u8; 16],
    };
    node.insert_cell(0, &cell_b.to_bytes()).unwrap();

    let cell_a = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    node.insert_cell(0, &cell_a.to_bytes()).unwrap();

    assert_eq!(node.get_header().keys_count, 2);
    assert_eq!(node.get_key_at(0), b"aaa");
    assert_eq!(node.get_key_at(1), b"bbb");
}

#[test]
fn insert_cell_at_middle() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell_a = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    let cell_c = LeafCell {
        key: b"ccc",
        value: &[3u8; 16],
    };
    node.insert_cell(0, &cell_a.to_bytes()).unwrap();
    node.insert_cell(1, &cell_c.to_bytes()).unwrap();

    let cell_b = LeafCell {
        key: b"bbb",
        value: &[2u8; 16],
    };
    node.insert_cell(1, &cell_b.to_bytes()).unwrap();

    assert_eq!(node.get_header().keys_count, 3);
    assert_eq!(node.get_key_at(0), b"aaa");
    assert_eq!(node.get_key_at(1), b"bbb");
    assert_eq!(node.get_key_at(2), b"ccc");
}

#[test]
fn insert_cell_overflow() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let big_key = [0xFFu8; 2500];
    let cell = LeafCell {
        key: &big_key,
        value: &[0u8; 16],
    };
    let bytes = cell.to_bytes();
    node.insert_cell(0, &bytes).unwrap();

    let big_key2 = [0xAFu8; 2500];
    let cell2 = LeafCell {
        key: &big_key2,
        value: &[1u8; 16],
    };
    let result = node.insert_cell(1, &cell2.to_bytes());
    assert!(result.is_err());
    assert_eq!(node.get_header().keys_count, 1);
}

#[test]
fn binary_search_existing() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 4] = [b"aaa", b"ccc", b"eee", b"ggg"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    assert_eq!(node.binary_search(b"aaa"), (0, true));
    assert_eq!(node.binary_search(b"ccc"), (1, true));
    assert_eq!(node.binary_search(b"eee"), (2, true));
    assert_eq!(node.binary_search(b"ggg"), (3, true));
}

#[test]
fn binary_search_insert_position() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 3] = [b"bbb", b"ddd", b"fff"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    assert_eq!(node.binary_search(b"aaa"), (0, false));
    assert_eq!(node.binary_search(b"ccc"), (1, false));
    assert_eq!(node.binary_search(b"eee"), (2, false));
    assert_eq!(node.binary_search(b"zzz"), (3, false));
}

#[test]
fn binary_search_empty() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    assert_eq!(node.binary_search(b"anything"), (0, false));
}

#[test]
fn delete_cell_end() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 3] = [b"aaa", b"bbb", b"ccc"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    node.delete_cell(2);

    assert_eq!(node.get_header().keys_count, 2);
    assert_eq!(node.get_key_at(0), b"aaa");
    assert_eq!(node.get_key_at(1), b"bbb");
}

#[test]
fn delete_cell_beginning() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 3] = [b"aaa", b"bbb", b"ccc"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    node.delete_cell(0);

    assert_eq!(node.get_header().keys_count, 2);
    assert_eq!(node.get_key_at(0), b"bbb");
    assert_eq!(node.get_key_at(1), b"ccc");
}

#[test]
fn delete_cell_middle() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let keys: [&[u8]; 3] = [b"aaa", b"bbb", b"ccc"];
    for (i, key) in keys.iter().enumerate() {
        let cell = LeafCell {
            key,
            value: &[i as u8; 16],
        };
        node.insert_cell(i as u16, &cell.to_bytes()).unwrap();
    }

    node.delete_cell(1);

    assert_eq!(node.get_header().keys_count, 2);
    assert_eq!(node.get_key_at(0), b"aaa");
    assert_eq!(node.get_key_at(1), b"ccc");
}

#[test]
fn delete_cell_out_of_bounds() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    node.insert_cell(0, &cell.to_bytes()).unwrap();

    node.delete_cell(5);

    assert_eq!(node.get_header().keys_count, 1);
    assert_eq!(node.get_key_at(0), b"aaa");
}

#[test]
fn update_leaf_value() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell = LeafCell {
        key: b"key",
        value: &[1u8; 16],
    };
    node.insert_cell(0, &cell.to_bytes()).unwrap();

    let data = node.get_cell_data(0);
    let restored = LeafCell::from_bytes(data);
    assert_eq!(restored.key, b"key");
    assert_eq!(restored.value, &[1u8; 16]);

    let new_value = [9u8; 16];
    node.update_leaf_value(0, &new_value);

    let data = node.get_cell_data(0);
    let restored = LeafCell::from_bytes(data);
    assert_eq!(restored.key, b"key");
    assert_eq!(restored.value, &[9u8; 16]);
}

#[test]
fn update_leaf_value_out_of_bounds() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell = LeafCell {
        key: b"key",
        value: &[1u8; 16],
    };
    node.insert_cell(0, &cell.to_bytes()).unwrap();

    node.update_leaf_value(5, &[9u8; 16]);

    let data = node.get_cell_data(0);
    let restored = LeafCell::from_bytes(data);
    assert_eq!(restored.value, &[1u8; 16]);
}

#[test]
fn used_space_empty() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    assert_eq!(node.used_space(), 0);
}

#[test]
fn used_space_after_inserts() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell_a = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    let cell_b = LeafCell {
        key: b"bbb",
        value: &[2u8; 16],
    };
    node.insert_cell(0, &cell_a.to_bytes()).unwrap();
    node.insert_cell(1, &cell_b.to_bytes()).unwrap();

    let cell_size = 2 + 3 + 16;
    let expected = 2 * SLOT_SIZE + 2 * cell_size;
    assert_eq!(node.used_space(), expected);
}

#[test]
fn used_space_after_delete() {
    let mut page = [0u8; PAGE_SIZE];
    let mut node = Node::new(&mut page);
    node.init(NodeType::Leaf, 0);

    let cell_a = LeafCell {
        key: b"aaa",
        value: &[1u8; 16],
    };
    let cell_b = LeafCell {
        key: b"bbb",
        value: &[2u8; 16],
    };
    node.insert_cell(0, &cell_a.to_bytes()).unwrap();
    node.insert_cell(1, &cell_b.to_bytes()).unwrap();

    let before = node.used_space();
    node.delete_cell(0);
    let after = node.used_space();

    let cell_size = 2 + 3 + 16;
    assert_eq!(before - after, SLOT_SIZE + cell_size);
}
