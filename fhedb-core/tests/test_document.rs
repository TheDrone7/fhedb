use fhedb_core::prelude::{DocId, Document};
use bson::doc;
use uuid::Uuid;

#[test]
fn test_docid_from_and_into_uuid() {
    let uuid = Uuid::new_v4();
    let doc_id: DocId = uuid.into();
    let uuid2: Uuid = doc_id.clone().into();
    assert_eq!(uuid, uuid2);
}

#[test]
fn test_docid_new_is_unique() {
    let id1 = DocId::new();
    let id2 = DocId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_document_new_and_fields() {
    let id = DocId::new();
    let data = doc! { "foo": 42 };
    let doc = Document::new(id.clone(), data.clone());
    assert_eq!(doc.id, id);
    assert_eq!(doc.data, data);
}

#[test]
fn test_document_with_random_id() {
    let data = doc! { "bar": true };
    let doc1 = Document::with_random_id(data.clone());
    let doc2 = Document::with_random_id(data.clone());
    assert_ne!(doc1.id, doc2.id);
    assert_eq!(doc1.data, data);
}

#[test]
fn test_document_from_tuple() {
    let id = DocId::new();
    let data = doc! { "baz": "qux" };
    let doc: Document = (id.clone(), data.clone()).into();
    assert_eq!(doc.id, id);
    assert_eq!(doc.data, data);
}

#[test]
fn test_document_from_bson_document() {
    let data = doc! { "hello": "world" };
    let doc: Document = data.clone().into();
    assert_eq!(doc.data, data);
}

#[test]
fn test_document_into_parts() {
    let id = DocId::new();
    let data = doc! { "x": 1 };
    let doc = Document::new(id.clone(), data.clone());
    let (id2, data2) = doc.into_parts();
    assert_eq!(id2, id);
    assert_eq!(data2, data);
}
