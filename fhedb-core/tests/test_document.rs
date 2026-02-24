use bson::doc;
use fhedb_core::prelude::{DocId, Document};
use uuid::Uuid;

#[test]
fn docid_from_and_into_uuid() {
    let uuid = Uuid::new_v4();
    let doc_id: DocId = uuid.into();
    let uuid2: Uuid = doc_id.clone().into();
    assert_eq!(uuid, uuid2);
}

#[test]
fn docid_new_is_unique() {
    let id1 = DocId::new();
    let id2 = DocId::new();
    assert_ne!(id1, id2);
}

#[test]
fn document_new_and_fields() {
    let id = DocId::new();
    let data = doc! { "foo": 42 };
    let doc = Document::new(id.clone(), data.clone());
    assert_eq!(doc.id, id);
    assert_eq!(doc.data, data);
}

#[test]
fn document_with_random_id() {
    let data = doc! { "bar": true };
    let doc1 = Document::with_random_id(data.clone());
    let doc2 = Document::with_random_id(data.clone());
    assert_ne!(doc1.id, doc2.id);
    assert_eq!(doc1.data, data);
}

#[test]
fn document_from_tuple() {
    let id = DocId::new();
    let data = doc! { "baz": "qux" };
    let doc: Document = (id.clone(), data.clone()).into();
    assert_eq!(doc.id, id);
    assert_eq!(doc.data, data);
}

#[test]
fn document_from_bson_document() {
    let data = doc! { "hello": "world" };
    let doc: Document = data.clone().into();
    assert_eq!(doc.data, data);
}

#[test]
fn document_into_parts() {
    let id = DocId::new();
    let data = doc! { "x": 1 };
    let doc = Document::new(id.clone(), data.clone());
    let (id2, data2) = doc.into_parts();
    assert_eq!(id2, id);
    assert_eq!(data2, data);
}

#[test]
fn docid_to_bytes() {
    let bytes1 = DocId::from_string("hello".to_string()).to_bytes();
    assert_eq!(bytes1[0], 0u8);
    assert_eq!(&bytes1[1..], b"hello");

    let bytes2 = DocId::from_u64(42).to_bytes();
    assert_eq!(bytes2[0], 1u8);
    assert_eq!(bytes2.len(), 9);
    assert_eq!(&bytes2[1..], &42u64.to_be_bytes());

    let bytes3 = DocId::from_string(String::new()).to_bytes();
    assert_eq!(bytes3.len(), 1);
    assert_eq!(bytes3[0], 0u8);
}

#[test]
fn docid_from_bytes() {
    let id1 = DocId::from_bytes(&[0u8, b'h', b'e', b'l', b'l', b'o']);
    assert_eq!(id1, DocId::from_string("hello".to_string()));

    let mut u64_bytes = vec![1u8];
    u64_bytes.extend_from_slice(&42u64.to_be_bytes());
    let id2 = DocId::from_bytes(&u64_bytes);
    assert_eq!(id2, DocId::from_u64(42));

    let id3 = DocId::from_bytes(&[0u8]);
    assert_eq!(id3, DocId::from_string(String::new()));
}

#[test]
fn docid_bytes_roundtrip() {
    let id1 = DocId::from_string("test-doc-id".to_string());
    assert_eq!(DocId::from_bytes(&id1.to_bytes()), id1);

    let id2 = DocId::from_u64(9876543210);
    assert_eq!(DocId::from_bytes(&id2.to_bytes()), id2);

    let id3 = DocId::from_u64(0);
    assert_eq!(DocId::from_bytes(&id3.to_bytes()), id3);

    let id4 = DocId::from_u64(u64::MAX);
    assert_eq!(DocId::from_bytes(&id4.to_bytes()), id4);

    let id5 = DocId::from_uuid(Uuid::new_v4());
    assert_eq!(DocId::from_bytes(&id5.to_bytes()), id5);

    let id6 = DocId::from_string(String::new());
    assert_eq!(DocId::from_bytes(&id6.to_bytes()), id6);
}

#[test]
#[should_panic(expected = "Invalid bytes for DocId")]
fn docid_from_bytes_panics_on_invalid_prefix() {
    DocId::from_bytes(&[2u8, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
#[should_panic(expected = "Invalid bytes for DocId")]
fn docid_from_bytes_panics_on_empty_slice() {
    DocId::from_bytes(&[]);
}

#[test]
#[should_panic(expected = "Invalid bytes for u64 DocId")]
fn docid_from_bytes_panics_on_truncated_u64() {
    DocId::from_bytes(&[1u8, 0, 0, 0, 0]);
}
