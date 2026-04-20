use bson::Bson;
use fhedb_core::index::secondary::SecondaryIndex;
use fhedb_core::prelude::DocId;
use tempfile::tempdir;

#[test]
fn new_creates_index() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path());
    assert!(index.is_ok());
    assert_eq!(index.unwrap().field_name(), "status");
}

#[test]
fn is_empty_on_new_index() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path()).unwrap();
    assert!(index.is_empty());
}

#[test]
fn len_empty() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path()).unwrap();
    assert_eq!(index.len(), 0);
}

#[test]
fn insert_and_get_entry() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_string("doc-abc-123".to_string());
    let val = Bson::String("active".to_string());

    index.insert(&val, &id, 100).unwrap();

    let result = index.get(&val, &id).unwrap();
    assert_eq!(result, Some(100));
}

#[test]
fn get_missing_entry() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_string("nonexistent".to_string());
    let val = Bson::String("missing".to_string());
    let result = index.get(&val, &id).unwrap();
    assert_eq!(result, None);
}

#[test]
fn contains_entry_after_insert() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(7);
    let val = Bson::Int32(50);
    assert!(!index.contains_entry(&val, &id).unwrap());

    index.insert(&val, &id, 300).unwrap();
    assert!(index.contains_entry(&val, &id).unwrap());
}

#[test]
fn contains_entry_missing() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_string("missing".to_string());
    let val = Bson::Boolean(false);
    assert!(!index.contains_entry(&val, &id).unwrap());
}

#[test]
fn is_empty_after_insert() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    let val = Bson::Int64(10);
    index.insert(&val, &id, 50).unwrap();
    assert!(!index.is_empty());
}

#[test]
fn len_after_inserts() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    for i in 0..5i64 {
        let id = DocId::from_u64(i as u64);
        let val = Bson::Int64(i * 10);
        index.insert(&val, &id, (i as u64) * 100).unwrap();
    }

    assert_eq!(index.len(), 5);
}

#[test]
fn all_entries() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id1 = DocId::from_u64(10);
    let id2 = DocId::from_u64(20);
    let val1 = Bson::Int32(5);
    let val2 = Bson::Int32(15);

    index.insert(&val1, &id1, 1000).unwrap();
    index.insert(&val2, &id2, 2000).unwrap();

    let mut entries = index
        .all_entries()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    entries.sort_by_key(|(_, id, _)| match id {
        DocId::U64(v) => *v,
        _ => panic!("Expected u64 DocId"),
    });

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (Bson::Int64(5), DocId::from_u64(10), 1000));
    assert_eq!(entries[1], (Bson::Int64(15), DocId::from_u64(20), 2000));
}

#[test]
fn remove_existing() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(5);
    let val = Bson::Boolean(true);

    index.insert(&val, &id, 500).unwrap();
    assert!(index.contains_entry(&val, &id).unwrap());

    let removed = index.remove(&val, &id).unwrap();
    assert_eq!(removed, Some(500));
    assert!(!index.contains_entry(&val, &id).unwrap());
    assert_eq!(index.get(&val, &id).unwrap(), None);
}

#[test]
fn remove_missing() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(99);
    let val = Bson::Double(3.14);
    let removed = index.remove(&val, &id).unwrap();
    assert_eq!(removed, None);
}

#[test]
fn is_empty_after_remove_all() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id1 = DocId::from_u64(1);
    let id2 = DocId::from_u64(2);
    let val1 = Bson::Int32(100);
    let val2 = Bson::Int32(200);

    index.insert(&val1, &id1, 100).unwrap();
    index.insert(&val2, &id2, 200).unwrap();
    assert!(!index.is_empty());

    index.remove(&val1, &id1).unwrap();
    index.remove(&val2, &id2).unwrap();
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
}

#[test]
fn update_existing() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(3);
    let old_val = Bson::String("old".to_string());
    let new_val = Bson::String("new".to_string());

    index.insert(&old_val, &id, 300).unwrap();
    assert_eq!(index.get(&old_val, &id).unwrap(), Some(300));

    index.update(&old_val, &new_val, &id, 999).unwrap();

    assert_eq!(index.get(&old_val, &id).unwrap(), None);
    assert_eq!(index.get(&new_val, &id).unwrap(), Some(999));
}

#[test]
fn update_nonexistent_entry_error() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(77);
    let val = Bson::Int32(5);

    let result = index.update(&val, &val, &id, 123);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn remove_does_not_affect_other_entries() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id1 = DocId::from_u64(1);
    let id2 = DocId::from_u64(2);
    let id3 = DocId::from_u64(3);
    let val = Bson::Int32(1);

    index.insert(&val, &id1, 100).unwrap();
    index.insert(&val, &id2, 200).unwrap();
    index.insert(&val, &id3, 300).unwrap();

    index.remove(&val, &id2).unwrap();

    assert_eq!(index.get(&val, &id1).unwrap(), Some(100));
    assert_eq!(index.get(&val, &id2).unwrap(), None);
    assert_eq!(index.get(&val, &id3).unwrap(), Some(300));
    assert_eq!(index.len(), 2);
}

#[test]
fn insert_many_entries() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    for i in 0..100i64 {
        let id = DocId::from_u64(i as u64);
        let val = Bson::Int64(i * 10);
        index.insert(&val, &id, (i as u64) * 10).unwrap();
    }

    assert_eq!(index.len(), 100);
    assert!(!index.is_empty());

    for i in 0..100i64 {
        let id = DocId::from_u64(i as u64);
        let val = Bson::Int64(i * 10);
        assert_eq!(index.get(&val, &id).unwrap(), Some((i as u64) * 10));
        assert!(index.contains_entry(&val, &id).unwrap());
    }
}

#[test]
fn update_preserves_other_entries() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id1 = DocId::from_u64(10);
    let id2 = DocId::from_u64(20);
    let val1 = Bson::Int32(1);
    let val2 = Bson::Int32(2);

    index.insert(&val1, &id1, 100).unwrap();
    index.insert(&val2, &id2, 200).unwrap();

    index.update(&val1, &val1, &id1, 999).unwrap();

    assert_eq!(index.get(&val1, &id1).unwrap(), Some(999));
    assert_eq!(index.get(&val2, &id2).unwrap(), Some(200));
    assert_eq!(index.len(), 2);
}

#[test]
fn all_entries_empty_index() {
    let dir = tempdir().unwrap();
    let index = SecondaryIndex::new("status", dir.path()).unwrap();
    let entries = index
        .all_entries()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries.is_empty());
}

#[test]
fn insert_duplicate_entry_rejected() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    let val = Bson::Int32(100);

    index.insert(&val, &id, 100).unwrap();

    let result = index.insert(&val, &id, 200);
    assert!(result.is_err());
    assert_eq!(index.len(), 1);
}

#[test]
fn separate_field_indices_are_independent() {
    let dir = tempdir().unwrap();
    let mut index_a = SecondaryIndex::new("field_a", dir.path()).unwrap();
    let index_b = SecondaryIndex::new("field_b", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    let val = Bson::Int32(100);

    index_a.insert(&val, &id, 100).unwrap();

    assert!(index_a.contains_entry(&val, &id).unwrap());
    assert!(!index_b.contains_entry(&val, &id).unwrap());
    assert_eq!(index_a.len(), 1);
    assert_eq!(index_b.len(), 0);
}

#[test]
fn large_string_cap_index() {
    let dir = tempdir().unwrap();
    let mut index = SecondaryIndex::new("status", dir.path()).unwrap();

    let id1 = DocId::from_u64(3);
    let text_501 = Bson::String("a".repeat(501));
    let text_500 = Bson::String("a".repeat(500));

    index.insert(&text_501, &id1, 1000).unwrap();

    let result = index.get(&text_500, &id1).unwrap();
    assert_eq!(result, Some(1000));
}
