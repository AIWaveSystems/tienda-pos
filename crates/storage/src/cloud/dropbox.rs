use pos_config::{BackendKind, OAuthApp};
use serde_json::{json, Value};
use ureq::Response;
use url::Url;

use super::account::CloudAccount;
use super::http::{read_bytes, read_json, text, AccessToken, Http};
use super::oauth;
use crate::document::{FileBackend, Snapshot};
use crate::{Result, StorageError};

#[derive(Clone)]
pub struct DropboxEndpoints {
    pub authorize: String,
    pub token: String,
    pub account: String,
    pub download: String,
    pub upload: String,
}

impl Default for DropboxEndpoints {
    fn default() -> Self {
        Self {
            authorize: "https://www.dropbox.com/oauth2/authorize".to_owned(),
            token: "https://api.dropboxapi.com/oauth2/token".to_owned(),
            account: "https://api.dropboxapi.com/2/users/get_current_account".to_owned(),
            download: "https://content.dropboxapi.com/2/files/download".to_owned(),
            upload: "https://content.dropboxapi.com/2/files/upload".to_owned(),
        }
    }
}

pub struct Dropbox {
    http: Http,
    app: OAuthApp,
    endpoints: DropboxEndpoints,
    refresh_token: String,
    access: AccessToken,
}

impl Dropbox {
    pub fn connect(
        app: OAuthApp,
        refresh_token: &str,
        endpoints: DropboxEndpoints,
    ) -> Result<Self> {
        let dropbox = Self {
            http: Http::new("Dropbox"),
            app,
            endpoints,
            refresh_token: refresh_token.to_owned(),
            access: AccessToken::default(),
        };
        dropbox.token()?;
        Ok(dropbox)
    }

    pub fn authorize(app: &OAuthApp, endpoints: &DropboxEndpoints) -> Result<CloudAccount> {
        let http = Http::new("Dropbox");
        let authorization = oauth::authorize(|state, challenge| {
            Url::parse_with_params(
                &endpoints.authorize,
                [
                    ("client_id", app.client_id.as_str()),
                    ("redirect_uri", oauth::REDIRECT_URI),
                    ("response_type", "code"),
                    ("token_access_type", "offline"),
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
                ("grant_type", "authorization_code"),
                ("client_id", &app.client_id),
                ("client_secret", &app.client_secret),
                ("redirect_uri", oauth::REDIRECT_URI),
                ("code_verifier", &authorization.verifier),
            ],
        )?;
        let access = text(&tokens, "access_token")?;
        let account = account(&http, endpoints, access).map_err(|error| http.failure(error))?;
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
                    ("grant_type", "refresh_token"),
                    ("client_id", &self.app.client_id),
                    ("client_secret", &self.app.client_secret),
                ],
            )?;
            Ok(text(&tokens, "access_token")?.to_owned())
        })
    }

    fn renewing(
        &self,
        perform: impl Fn(&str) -> std::result::Result<Response, ureq::Error>,
    ) -> Result<std::result::Result<Response, ureq::Error>> {
        match perform(&self.token()?) {
            Err(ureq::Error::Status(401, _)) => {
                self.access.forget();
                Ok(perform(&self.token()?))
            }
            other => Ok(other),
        }
    }

    fn content(
        &self,
        url: &str,
        argument: &Value,
        body: &[u8],
    ) -> Result<std::result::Result<Response, ureq::Error>> {
        self.renewing(|token| {
            self.http
                .request("POST", url)
                .set("Authorization", &format!("Bearer {token}"))
                .set("Dropbox-API-Arg", &argument.to_string())
                .set("Content-Type", "application/octet-stream")
                .send_bytes(body)
        })
    }
}

impl FileBackend for Dropbox {
    fn kind(&self) -> BackendKind {
        BackendKind::Dropbox
    }

    fn read(&self, name: &str) -> Result<Option<Snapshot>> {
        let argument = json!({ "path": format!("/{name}") });
        let response = match self.content(&self.endpoints.download, &argument, b"")? {
            Ok(response) => response,
            Err(ureq::Error::Status(409, response)) => {
                let detail = response.into_string().unwrap_or_default();
                if detail.contains("not_found") {
                    return Ok(None);
                }
                return Err(StorageError::Connection(format!(
                    "Error de Dropbox (409): {detail}"
                )));
            }
            Err(error) => return Err(self.http.failure(error)),
        };
        let metadata: Value =
            serde_json::from_str(response.header("Dropbox-API-Result").unwrap_or("{}"))?;
        Ok(Some(Snapshot {
            revision: text(&metadata, "rev")?.to_owned(),
            content: read_bytes(response)?,
        }))
    }

    fn write(&self, name: &str, content: &[u8], expected: Option<&str>) -> Result<()> {
        let mode = match expected {
            Some(revision) => json!({ ".tag": "update", "update": revision }),
            None => json!({ ".tag": "add" }),
        };
        let argument = json!({
            "path": format!("/{name}"),
            "mode": mode,
            "autorename": false,
            "mute": true,
            "strict_conflict": true
        });
        match self.content(&self.endpoints.upload, &argument, content)? {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(409, _)) => Err(StorageError::Conflict(
                "Los datos en Dropbox cambiaron desde la ultima lectura. Intente de nuevo."
                    .to_owned(),
            )),
            Err(error) => Err(self.http.failure(error)),
        }
    }

    fn ping(&self) -> Result<()> {
        self.renewing(|token| account(&self.http, &self.endpoints, token))?
            .map_err(|error| self.http.failure(error))?;
        Ok(())
    }
}

fn account(
    http: &Http,
    endpoints: &DropboxEndpoints,
    token: &str,
) -> std::result::Result<Response, ureq::Error> {
    http.request("POST", &endpoints.account)
        .set("Authorization", &format!("Bearer {token}"))
        .set("Content-Type", "application/json")
        .send_string("null")
}
