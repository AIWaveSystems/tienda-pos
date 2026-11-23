use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{Result, StorageError};

const PORT: u16 = 53682;
pub const REDIRECT_URI: &str = "http://localhost:53682/callback";
const TIMEOUT: Duration = Duration::from_secs(120);

pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl Pkce {
    pub fn generate() -> Self {
        let verifier = random_token(40);
        let challenge = BASE64_URL_SAFE_NO_PAD.encode(Sha256::digest(&verifier));
        Self {
            verifier,
            challenge,
        }
    }
}

pub struct Authorization {
    pub code: String,
    pub verifier: String,
}

#[derive(Debug, Default, PartialEq)]
pub struct Callback {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

pub fn authorize(build_url: impl FnOnce(&str, &str) -> Result<Url>) -> Result<Authorization> {
    let pkce = Pkce::generate();
    let state = random_token(24);
    let listener = TcpListener::bind(("127.0.0.1", PORT)).map_err(|_| {
        StorageError::Connection(format!(
            "No se pudo abrir el puerto local {PORT} para recibir la autorizacion."
        ))
    })?;
    listener.set_nonblocking(true)?;
    open::that(build_url(&state, &pkce.challenge)?.as_str()).map_err(|_| {
        StorageError::Connection("No se pudo abrir el navegador predeterminado.".to_owned())
    })?;
    let callback = wait_callback(&listener, Instant::now() + TIMEOUT)?;
    Ok(Authorization {
        code: validate(callback, &state)?,
        verifier: pkce.verifier,
    })
}

pub fn validate(callback: Callback, state: &str) -> Result<String> {
    if let Some(error) = callback.error {
        return Err(StorageError::Connection(format!(
            "El proveedor rechazo la autorizacion: {error}"
        )));
    }
    if callback.state.as_deref() != Some(state) {
        return Err(StorageError::Connection(
            "La respuesta de autorizacion no es valida (el 'state' no coincide).".to_owned(),
        ));
    }
    callback.code.ok_or_else(|| {
        StorageError::Connection("El proveedor no devolvio un codigo de autorizacion.".to_owned())
    })
}

pub fn parse_callback(target: &str) -> Option<Callback> {
    let url = Url::parse("http://localhost").ok()?.join(target).ok()?;
    if url.path() != "/callback" {
        return None;
    }
    let mut callback = Callback::default();
    for (name, value) in url.query_pairs() {
        let value = Some(value.into_owned());
        match name.as_ref() {
            "code" => callback.code = value,
            "state" => callback.state = value,
            "error" => callback.error = value,
            _ => {}
        }
    }
    Some(callback)
}

fn wait_callback(listener: &TcpListener, deadline: Instant) -> Result<Callback> {
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(callback) = answer(stream) {
                    return Ok(callback);
                }
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    }
    Err(StorageError::Connection(
        "Tiempo de espera agotado esperando la autorizacion en el navegador.".to_owned(),
    ))
}

fn answer(mut stream: TcpStream) -> Option<Callback> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut line = String::new();
    BufReader::new(&stream).read_line(&mut line).ok()?;
    let callback = line.split_whitespace().nth(1).and_then(parse_callback);
    let (status, title) = match &callback {
        Some(found) if found.error.is_none() => ("200 OK", "Conexion exitosa"),
        Some(_) => ("200 OK", "No se pudo conectar"),
        None => ("404 Not Found", "No encontrado"),
    };
    let body =
        format!("<html><body><h2>{title}</h2><p>Puede cerrar esta ventana.</p></body></html>");
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    callback
}

fn random_token(length: usize) -> String {
    let mut bytes = vec![0u8; length];
    OsRng.fill_bytes(&mut bytes);
    BASE64_URL_SAFE_NO_PAD.encode(bytes)
}
