use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};
use tiny_http::{Header, Request, Response, Server};
use url::Url;

#[derive(Default)]
struct State {
    files: HashMap<String, (Vec<u8>, u64)>,
    revision: u64,
    tokens: u64,
    valid: u64,
}

pub struct FakeCloud {
    address: String,
    state: Arc<Mutex<State>>,
    server: Arc<Server>,
}

type Reply = (u16, Vec<u8>, Option<String>);

impl FakeCloud {
    pub fn start() -> Self {
        let server = Arc::new(Server::http("127.0.0.1:0").unwrap());
        let address = format!("http://{}", server.server_addr());
        let state = Arc::new(Mutex::new(State::default()));
        let (listener, shared) = (server.clone(), state.clone());
        thread::spawn(move || {
            for mut request in listener.incoming_requests() {
                let (status, body, metadata) = reply(&mut request, &mut shared.lock().unwrap());
                let mut response = Response::from_data(body).with_status_code(status);
                if let Some(metadata) = metadata {
                    response
                        .add_header(Header::from_bytes("Dropbox-API-Result", metadata).unwrap());
                }
                let _ = request.respond(response);
            }
        });
        Self {
            address,
            state,
            server,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.address)
    }

    pub fn expire_tokens(&self) {
        self.state.lock().unwrap().valid = u64::MAX;
    }

    pub fn issued_tokens(&self) -> u64 {
        self.state.lock().unwrap().tokens
    }
}

impl Drop for FakeCloud {
    fn drop(&mut self) {
        self.server.unblock();
    }
}

fn reply(request: &mut Request, state: &mut State) -> Reply {
    let url = Url::parse(&format!("http://fake{}", request.url())).unwrap();
    let mut body = Vec::new();
    request.as_reader().read_to_end(&mut body).unwrap();
    if url.path() == "/token" {
        state.tokens += 1;
        state.valid = state.tokens;
        return ok(json!({ "access_token": format!("t{}", state.tokens) }));
    }
    if header(request, "Authorization") != format!("Bearer t{}", state.valid) {
        return (401, b"expired".to_vec(), None);
    }
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
    match url.path() {
        "/account" => ok(json!({ "email": "a@b.c" })),
        "/download" => download(request, state),
        "/upload" => upload(request, state, body),
        "/files" => files(request, state, &query),
        "/drive-upload" => create(state, &body),
        path => drive_file(state, path, body),
    }
}

fn download(request: &Request, state: &State) -> Reply {
    match state.files.get(&path_of(request)) {
        Some((content, revision)) => (
            200,
            content.clone(),
            Some(json!({ "rev": revision.to_string() }).to_string()),
        ),
        None => (409, b"path/not_found".to_vec(), None),
    }
}

fn upload(request: &Request, state: &mut State, body: Vec<u8>) -> Reply {
    let argument: Value = serde_json::from_str(&header(request, "Dropbox-API-Arg")).unwrap();
    let name = path_of(request);
    let current = state.files.get(&name).map(|file| file.1.to_string());
    if argument["mode"]["update"].as_str() != current.as_deref() {
        return (409, b"path/conflict".to_vec(), None);
    }
    save(state, &name, body);
    ok(json!({}))
}

fn files(request: &Request, state: &State, query: &HashMap<String, String>) -> Reply {
    if request.method().as_str() == "POST" {
        return ok(json!({ "id": "folder" }));
    }
    let Some(filter) = query.get("q") else {
        return (400, Vec::new(), None);
    };
    let name = filter.split('\'').nth(1).unwrap_or_default();
    let found: Vec<Value> = state
        .files
        .get(name)
        .map(|file| json!({ "id": name, "name": name, "md5Checksum": file.1.to_string() }))
        .into_iter()
        .collect();
    ok(json!({ "files": found }))
}

fn create(state: &mut State, body: &[u8]) -> Reply {
    let text = String::from_utf8(body.to_vec()).unwrap();
    let sections: Vec<&str> = text.split("\r\n\r\n").collect();
    let metadata: Value = serde_json::from_str(sections[1].split("\r\n").next().unwrap()).unwrap();
    let content = sections[2].rsplit_once("\r\n--").unwrap().0;
    save(
        state,
        metadata["name"].as_str().unwrap(),
        content.as_bytes().to_vec(),
    );
    ok(json!({}))
}

fn drive_file(state: &mut State, path: &str, body: Vec<u8>) -> Reply {
    if let Some(name) = path.strip_prefix("/drive-upload/") {
        save(state, name, body);
        return ok(json!({}));
    }
    let name = path.strip_prefix("/files/").unwrap_or_default();
    match state.files.get(name) {
        Some((content, _)) => (200, content.clone(), None),
        None => (404, Vec::new(), None),
    }
}

fn save(state: &mut State, name: &str, content: Vec<u8>) {
    state.revision += 1;
    state
        .files
        .insert(name.to_owned(), (content, state.revision));
}

fn path_of(request: &Request) -> String {
    let argument: Value = serde_json::from_str(&header(request, "Dropbox-API-Arg")).unwrap();
    argument["path"]
        .as_str()
        .unwrap()
        .trim_start_matches('/')
        .to_owned()
}

fn header(request: &Request, name: &str) -> String {
    request
        .headers()
        .iter()
        .find(|header| header.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str().to_owned())
        .unwrap_or_default()
}

fn ok(body: Value) -> Reply {
    (200, body.to_string().into_bytes(), None)
}
