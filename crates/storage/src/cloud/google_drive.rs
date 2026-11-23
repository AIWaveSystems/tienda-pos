use std::sync::{Mutex, PoisonError};

use pos_config::{BackendKind, OAuthApp};
use serde_json::{json, Value};
use ureq::{Request, Response};
use url::Url;

use super::account::CloudAccount;
use super::http::{read_bytes, read_json, text, AccessToken, Http};
use super::oauth;
use crate::document::{FileBackend, Snapshot};
use crate::{Result, StorageError};

const FOLDER: &str = "VentasPOSRust";
const FOLDER_TYPE: &str = "application/vnd.google-apps.folder";
const SCOPE: &str = "https://www.googleapis.com/auth/drive.file \
                     https://www.googleapis.com/auth/userinfo.email";

#[derive(Clone)]
pub struct DriveEndpoints {
    pub authorize: String,
    pub token: String,
    pub account: String,
    pub files: String,
    pub upload: String,
}

impl Default for DriveEndpoints {
    fn default() -> Self {
        Self {
            authorize: "https://accounts.google.com/o/oauth2/v2/auth".to_owned(),
            token: "https://oauth2.googleapis.com/token".to_owned(),
            account: "https://www.googleapis.com/oauth2/v2/userinfo".to_owned(),
            files: "https://www.googleapis.com/drive/v3/files".to_owned(),
            upload: "https://www.googleapis.com/upload/drive/v3/files".to_owned(),
        }
    }
}

pub struct GoogleDrive {
    http: Http,
    app: OAuthApp,
    endpoints: DriveEndpoints,
    refresh_token: String,
    access: AccessToken,
    folder: Mutex<Option<String>>,
}

impl GoogleDrive {
    pub fn connect(app: OAuthApp, refresh_token: &str, endpoints: DriveEndpoints) -> Result<Self> {
        let drive = Self {
            http: Http::new("Google Drive"),
            app,
            endpoints,
            refresh_token: refresh_token.to_owned(),
            access: AccessToken::default(),
            folder: Mutex::new(None),
        };
        drive.folder()?;
        Ok(drive)
    }

    pub fn authorize(app: &OAuthApp, endpoints: &DriveEndpoints) -> Result<CloudAccount> {
        let http = Http::new("Google Drive");
        let authorization = oauth::authorize(|state, challenge| {
            Url::parse_with_params(
                &endpoints.authorize,
                [
                    ("client_id", app.client_id.as_str()),
                    ("redirect_uri", oauth::REDIRECT_URI),
                    ("response_type", "code"),
                    ("scope", SCOPE),
                    ("access_type", "offline"),
                    ("prompt", "consent"),
                    ("state", state),
                    ("code_challenge", challenge),
                    ("code_challenge_method", "S256"),
                ],
            )
            .map_err(|failure| StorageError::Config(failure.to_string()))
        })?;
        let tokens = http.post_form(
            &endpoints.token,
            &[
                ("code", &authorization.code),
                ("client_id", &app.client_id),
                ("client_secret", &app.client_secret),
                ("redirect_uri", oauth::REDIRECT_URI),
                ("grant_type", "authorization_code"),
                ("code_verifier", &authorization.verifier),
            ],
        )?;
        let access = text(&tokens, "access_token")?;
        let account = http
            .request("GET", &endpoints.account)
            .set("Authorization", &format!("Bearer {access}"))
            .call()
            .map_err(|error| http.failure(error))?;
        Ok(CloudAccount {
            refresh_token: text(&tokens, "refresh_token")?.to_owned(),
            email: read_json(account)?["email"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
        })
    }

    fn token(&self) -> Result<String> {
        self.access.get(|| {
            let tokens = self.http.post_form(
                &self.endpoints.token,
                &[
                    ("refresh_token", &self.refresh_token),
                    ("client_id", &self.app.client_id),
                    ("client_secret", &self.app.client_secret),
                    ("grant_type", "refresh_token"),
                ],
            )?;
            Ok(text(&tokens, "access_token")?.to_owned())
        })
    }

    fn send(
        &self,
        method: &str,
        url: &str,
        perform: impl Fn(Request) -> std::result::Result<Response, ureq::Error>,
    ) -> Result<Response> {
        let attempt = || -> Result<std::result::Result<Response, ureq::Error>> {
            let token = self.token()?;
            let request = self
                .http
                .request(method, url)
                .set("Authorization", &format!("Bearer {token}"));
            Ok(perform(request))
        };
        let outcome = match attempt()? {
            Err(ureq::Error::Status(401, _)) => {
                self.access.forget();
                attempt()?
            }
            other => other,
        };
        outcome.map_err(|error| self.http.failure(error))
    }

    fn search(&self, query: &str) -> Result<Option<Value>> {
        let response = self.send("GET", &self.endpoints.files, |request| {
            request
                .query("q", query)
                .query("fields", "files(id,name,md5Checksum)")
                .call()
        })?;
        let mut found = read_json(response)?;
        Ok(found["files"]
            .as_array_mut()
            .filter(|files| !files.is_empty())
            .map(|files| files.remove(0)))
    }

    fn folder(&self) -> Result<String> {
        let mut slot = self.folder.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(id) = slot.as_ref() {
            return Ok(id.clone());
        }
        let query = format!("name='{FOLDER}' and mimeType='{FOLDER_TYPE}' and trashed=false");
        let folder = match self.search(&query)? {
            Some(folder) => folder,
            None => {
                let metadata = json!({ "name": FOLDER, "mimeType": FOLDER_TYPE });
                read_json(self.send("POST", &self.endpoints.files, |request| {
                    request.send_json(&metadata)
                })?)?
            }
        };
        let id = text(&folder, "id")?.to_owned();
        *slot = Some(id.clone());
        Ok(id)
    }

    fn locate(&self, name: &str) -> Result<Option<Value>> {
        let folder = self.folder()?;
        self.search(&format!(
            "name='{name}' and '{folder}' in parents and trashed=false"
        ))
    }

    fn create(&self, name: &str, content: &[u8]) -> Result<()> {
        let boundary = format!("ventaspos-{}", pos_domain::new_id());
        let metadata = json!({ "name": name, "parents": [self.folder()?] });
        let mut body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n\
             --{boundary}\r\nContent-Type: application/json\r\n\r\n"
        )
        .into_bytes();
        body.extend_from_slice(content);
        body.extend_from_slice(format!("\r\n--{boundary}--").as_bytes());
        self.send("POST", &self.endpoints.upload, |request| {
            request
                .query("uploadType", "multipart")
                .query("fields", "id,md5Checksum")
                .set(
                    "Content-Type",
                    &format!("multipart/related; boundary={boundary}"),
                )
                .send_bytes(&body)
        })?;
        Ok(())
    }

    fn update(&self, id: &str, content: &[u8]) -> Result<()> {
        let url = format!("{}/{id}", self.endpoints.upload);
        self.send("PATCH", &url, |request| {
            request
                .query("uploadType", "media")
                .query("fields", "md5Checksum")
                .set("Content-Type", "application/json; charset=utf-8")
                .send_bytes(content)
        })?;
        Ok(())
    }
}

impl FileBackend for GoogleDrive {
    fn kind(&self) -> BackendKind {
        BackendKind::GoogleDrive
    }

    fn read(&self, name: &str) -> Result<Option<Snapshot>> {
        let Some(file) = self.locate(name)? else {
            return Ok(None);
        };
        let url = format!("{}/{}", self.endpoints.files, text(&file, "id")?);
        let response = self.send("GET", &url, |request| request.query("alt", "media").call())?;
        Ok(Some(Snapshot {
            content: read_bytes(response)?,
            revision: text(&file, "md5Checksum")?.to_owned(),
        }))
    }

    fn write(&self, name: &str, content: &[u8], expected: Option<&str>) -> Result<()> {
        let current = self.locate(name)?;
        let revision = current
            .as_ref()
            .map(|file| text(file, "md5Checksum"))
            .transpose()?;
        if revision != expected {
            return Err(StorageError::Conflict(
                "Los datos en Google Drive cambiaron desde la ultima lectura. Intente de nuevo."
                    .to_owned(),
            ));
        }
        match &current {
            Some(file) => self.update(text(file, "id")?, content),
            None => self.create(name, content),
        }
    }

    fn ping(&self) -> Result<()> {
        let query = format!("name='{FOLDER}' and trashed=false");
        self.search(&query)?;
        Ok(())
    }
}
