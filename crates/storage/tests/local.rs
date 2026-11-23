mod common;

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use pos_domain::{Client, Product};
use pos_storage::{copy_all, DocumentStore, MirroredStore, Store};

#[test]
fn local_store_fulfills_the_contract() {
    let folder = tempfile::tempdir().unwrap();
    common::contract(&DocumentStore::local(folder.path()).unwrap());
}

#[test]
fn corrupt_file_is_set_aside_and_never_lost() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("clients.json"), "{roto").unwrap();
    let store = DocumentStore::local(folder.path()).unwrap();
    assert!(store.clients().all().unwrap().is_empty());
    let backup = fs::read_to_string(folder.path().join("clients.json.bak")).unwrap();
    assert_eq!(backup, "{roto");
}

#[test]
fn unreadable_record_is_reported_without_touching_the_file() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("products.json");
    let content = r#"{"schema_version":1,"items":[{"id":"a","stock":"muchos"}]}"#;
    fs::write(&path, content).unwrap();
    let store = DocumentStore::local(folder.path()).unwrap();
    assert!(store.products().all().is_err());
    assert!(store.products().upsert(&Product::default()).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), content);
}

#[test]
fn mirror_keeps_a_local_copy() {
    let primary_folder = tempfile::tempdir().unwrap();
    let mirror_folder = tempfile::tempdir().unwrap();
    let primary = Arc::new(DocumentStore::local(primary_folder.path()).unwrap());
    let mirror = DocumentStore::local(mirror_folder.path()).unwrap();
    let store = MirroredStore::with_interval(primary, mirror, Duration::from_secs(60));
    common::contract(&store);
    let client = Client::default();
    store.clients().upsert(&client).unwrap();
    store.close();
    store.close();
    let copy = DocumentStore::local(mirror_folder.path()).unwrap();
    assert_eq!(copy.clients().all().unwrap(), vec![client]);
}

#[test]
fn migration_copies_every_collection() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let source = DocumentStore::local(first.path()).unwrap();
    let target = DocumentStore::local(second.path()).unwrap();
    source.clients().upsert(&Client::default()).unwrap();
    source.products().upsert(&Product::default()).unwrap();
    copy_all(&source, &target).unwrap();
    assert_eq!(
        source.clients().all().unwrap(),
        target.clients().all().unwrap()
    );
    assert_eq!(
        source.products().all().unwrap(),
        target.products().all().unwrap()
    );
}
