use fhedb_core::index::primary::PrimaryIndex;
use fhedb_core::prelude::DocId;
use tempfile::tempdir;

#[test]
fn new_creates_index() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("user_id", dir.path());
    assert!(index.is_ok());
    assert_eq!(index.unwrap().field_name(), "user_id");
}

#[test]
fn is_empty_on_new_index() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();
    assert!(index.is_empty());
}

#[test]
fn len_empty() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();
    assert_eq!(index.len(), 0);
}

#[test]
fn insert_and_get_string_id() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_string("doc-abc-123".to_string());
    index.insert(&id, 100).unwrap();

    let result = index.get(&id).unwrap();
    assert_eq!(result, Some(100));
}

#[test]
fn insert_and_get_u64_id() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(42);
    index.insert(&id, 200).unwrap();

    let result = index.get(&id).unwrap();
    assert_eq!(result, Some(200));
}

#[test]
fn get_missing_id() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_string("nonexistent".to_string());
    let result = index.get(&id).unwrap();
    assert_eq!(result, None);
}

#[test]
fn contains_id_after_insert() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(7);
    assert!(!index.contains_id(&id).unwrap());

    index.insert(&id, 300).unwrap();
    assert!(index.contains_id(&id).unwrap());
}

#[test]
fn contains_id_missing() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_string("missing".to_string());
    assert!(!index.contains_id(&id).unwrap());
}

#[test]
fn is_empty_after_insert() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    index.insert(&id, 50).unwrap();
    assert!(!index.is_empty());
}

#[test]
fn len_after_inserts() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    for i in 0..5u64 {
        let id = DocId::from_u64(i);
        index.insert(&id, i * 100).unwrap();
    }

    assert_eq!(index.len(), 5);
}

#[test]
fn insert_multiple_and_get_all_ids() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let ids: Vec<DocId> = (0..3u64).map(DocId::from_u64).collect();
    for (i, id) in ids.iter().enumerate() {
        index.insert(id, (i as u64) * 10).unwrap();
    }

    let mut all = index
        .all_ids()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    all.sort_by_key(|id| match id {
        DocId::U64(v) => *v,
        _ => panic!("Expected u64 DocId"),
    });

    assert_eq!(all.len(), 3);
    assert_eq!(all[0], DocId::from_u64(0));
    assert_eq!(all[1], DocId::from_u64(1));
    assert_eq!(all[2], DocId::from_u64(2));
}

#[test]
fn all_entries() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id1 = DocId::from_u64(10);
    let id2 = DocId::from_u64(20);
    index.insert(&id1, 1000).unwrap();
    index.insert(&id2, 2000).unwrap();

    let mut entries = index
        .all_entries()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    entries.sort_by_key(|(id, _)| match id {
        DocId::U64(v) => *v,
        _ => panic!("Expected u64 DocId"),
    });

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], (DocId::from_u64(10), 1000));
    assert_eq!(entries[1], (DocId::from_u64(20), 2000));
}

#[test]
fn remove_existing() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(5);
    index.insert(&id, 500).unwrap();
    assert!(index.contains_id(&id).unwrap());

    let removed = index.remove(&id).unwrap();
    assert_eq!(removed, Some(500));
    assert!(!index.contains_id(&id).unwrap());
    assert_eq!(index.get(&id).unwrap(), None);
}

#[test]
fn remove_missing() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(99);
    let removed = index.remove(&id).unwrap();
    assert_eq!(removed, None);
}

#[test]
fn is_empty_after_remove_all() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id1 = DocId::from_u64(1);
    let id2 = DocId::from_u64(2);
    index.insert(&id1, 100).unwrap();
    index.insert(&id2, 200).unwrap();
    assert!(!index.is_empty());

    index.remove(&id1).unwrap();
    index.remove(&id2).unwrap();
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
}

#[test]
fn update_existing() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(3);
    index.insert(&id, 300).unwrap();
    assert_eq!(index.get(&id).unwrap(), Some(300));

    index.update(&id, 999).unwrap();
    assert_eq!(index.get(&id).unwrap(), Some(999));
}

#[test]
fn update_nonexistent_id_error() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(77);
    let result = index.update(&id, 123);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn insert_and_get_with_string_ids() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id_a = DocId::from_string("alpha".to_string());
    let id_b = DocId::from_string("beta".to_string());
    let id_g = DocId::from_string("gamma".to_string());

    index.insert(&id_a, 10).unwrap();
    index.insert(&id_b, 20).unwrap();
    index.insert(&id_g, 30).unwrap();

    assert_eq!(index.get(&id_a).unwrap(), Some(10));
    assert_eq!(index.get(&id_b).unwrap(), Some(20));
    assert_eq!(index.get(&id_g).unwrap(), Some(30));
    assert_eq!(index.len(), 3);
}

#[test]
fn remove_does_not_affect_other_entries() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id1 = DocId::from_u64(1);
    let id2 = DocId::from_u64(2);
    let id3 = DocId::from_u64(3);
    index.insert(&id1, 100).unwrap();
    index.insert(&id2, 200).unwrap();
    index.insert(&id3, 300).unwrap();

    index.remove(&id2).unwrap();

    assert_eq!(index.get(&id1).unwrap(), Some(100));
    assert_eq!(index.get(&id2).unwrap(), None);
    assert_eq!(index.get(&id3).unwrap(), Some(300));
    assert_eq!(index.len(), 2);
}

#[test]
fn insert_many_entries() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    for i in 0..100u64 {
        let id = DocId::from_u64(i);
        index.insert(&id, i * 10).unwrap();
    }

    assert_eq!(index.len(), 100);
    assert!(!index.is_empty());

    for i in 0..100u64 {
        let id = DocId::from_u64(i);
        assert_eq!(index.get(&id).unwrap(), Some(i * 10));
        assert!(index.contains_id(&id).unwrap());
    }
}

#[test]
fn update_preserves_other_entries() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id1 = DocId::from_u64(10);
    let id2 = DocId::from_u64(20);
    index.insert(&id1, 100).unwrap();
    index.insert(&id2, 200).unwrap();

    index.update(&id1, 999).unwrap();

    assert_eq!(index.get(&id1).unwrap(), Some(999));
    assert_eq!(index.get(&id2).unwrap(), Some(200));
    assert_eq!(index.len(), 2);
}

#[test]
fn all_ids_empty_index() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();
    let ids = index
        .all_ids()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(ids.is_empty());
}

#[test]
fn all_entries_empty_index() {
    let dir = tempdir().unwrap();
    let index = PrimaryIndex::new("id", dir.path()).unwrap();
    let entries = index
        .all_entries()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries.is_empty());
}

#[test]
fn insert_duplicate_id_rejected() {
    let dir = tempdir().unwrap();
    let mut index = PrimaryIndex::new("id", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    index.insert(&id, 100).unwrap();

    let result = index.insert(&id, 200);
    assert!(result.is_err());
    assert_eq!(index.len(), 1);
}

#[test]
fn separate_field_indices_are_independent() {
    let dir = tempdir().unwrap();
    let mut index_a = PrimaryIndex::new("field_a", dir.path()).unwrap();
    let index_b = PrimaryIndex::new("field_b", dir.path()).unwrap();

    let id = DocId::from_u64(1);
    index_a.insert(&id, 100).unwrap();

    assert!(index_a.contains_id(&id).unwrap());
    assert!(!index_b.contains_id(&id).unwrap());
    assert_eq!(index_a.len(), 1);
    assert_eq!(index_b.len(), 0);
}
