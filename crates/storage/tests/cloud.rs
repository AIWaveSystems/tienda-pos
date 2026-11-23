mod common;
mod fake_cloud;

use std::sync::Arc;

use fake_cloud::FakeCloud;
use pos_config::OAuthApp;
use pos_storage::cloud::{oauth, DriveEndpoints, Dropbox, DropboxEndpoints, GoogleDrive};
use pos_storage::{DocumentStore, FileBackend, StorageError};

fn app() -> OAuthApp {
    OAuthApp {
        client_id: "id".into(),
        client_secret: "secreto".into(),
    }
}

fn dropbox(cloud: &FakeCloud) -> Dropbox {
    let endpoints = DropboxEndpoints {
        token: cloud.url("/token"),
        account: cloud.url("/account"),
        download: cloud.url("/download"),
        upload: cloud.url("/upload"),
        ..DropboxEndpoints::default()
    };
    Dropbox::connect(app(), "refresco", endpoints).unwrap()
}

fn drive(cloud: &FakeCloud) -> GoogleDrive {
    let endpoints = DriveEndpoints {
        token: cloud.url("/token"),
        files: cloud.url("/files"),
        upload: cloud.url("/drive-upload"),
        ..DriveEndpoints::default()
    };
    GoogleDrive::connect(app(), "refresco", endpoints).unwrap()
}

fn rejects_stale_writes(files: &dyn FileBackend) {
    files.write("prueba.json", b"[1]", None).unwrap();
    let first = files.read("prueba.json").unwrap().unwrap();
    files
        .write("prueba.json", b"[2]", Some(&first.revision))
        .unwrap();
    let stale = files.write("prueba.json", b"[3]", Some(&first.revision));
    assert!(matches!(stale, Err(StorageError::Conflict(_))));
    let duplicate = files.write("prueba.json", b"[4]", None);
    assert!(matches!(duplicate, Err(StorageError::Conflict(_))));
    assert_eq!(files.read("prueba.json").unwrap().unwrap().content, b"[2]");
}

#[test]
fn dropbox_fulfills_the_contract() {
    let cloud = FakeCloud::start();
    common::contract(&DocumentStore::new(Arc::new(dropbox(&cloud))));
}

#[test]
fn drive_fulfills_the_contract() {
    let cloud = FakeCloud::start();
    common::contract(&DocumentStore::new(Arc::new(drive(&cloud))));
}

#[test]
fn concurrent_changes_are_detected() {
    let cloud = FakeCloud::start();
    rejects_stale_writes(&dropbox(&cloud));
    let cloud = FakeCloud::start();
    rejects_stale_writes(&drive(&cloud));
}

#[test]
fn expired_access_is_renewed() {
    let cloud = FakeCloud::start();
    let files = dropbox(&cloud);
    let others = drive(&cloud);
    cloud.expire_tokens();
    files.write("a.json", b"[]", None).unwrap();
    cloud.expire_tokens();
    assert!(others.read("a.json").unwrap().is_some());
    assert!(cloud.issued_tokens() >= 3);
}

#[test]
fn unreadable_remote_file_is_not_overwritten() {
    let cloud = FakeCloud::start();
    let files = Arc::new(dropbox(&cloud));
    files.write("clients.json", b"{roto", None).unwrap();
    let store = DocumentStore::new(files.clone());
    let outcome = pos_storage::Store::clients(&store).upsert(&Default::default());
    assert!(matches!(outcome, Err(StorageError::Data(_))));
    assert_eq!(
        files.read("clients.json").unwrap().unwrap().content,
        b"{roto"
    );
}

#[test]
fn callback_is_parsed_and_validated() {
    let callback = oauth::parse_callback("/callback?code=a%20b&state=s").unwrap();
    assert_eq!(oauth::validate(callback, "s").unwrap(), "a b");
    let callback = oauth::parse_callback("/callback?code=a&state=otro").unwrap();
    assert!(oauth::validate(callback, "s").is_err());
    let callback = oauth::parse_callback("/callback?error=denied").unwrap();
    assert!(oauth::validate(callback, "s").is_err());
    assert!(oauth::parse_callback("/favicon.ico").is_none());
    assert_ne!(
        oauth::Pkce::generate().verifier,
        oauth::Pkce::generate().verifier
    );
}
