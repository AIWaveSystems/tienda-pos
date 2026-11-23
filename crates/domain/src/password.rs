use pbkdf2::pbkdf2_hmac;
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

const ROUNDS: u32 = 120_000;
const MAX_ROUNDS: u32 = 10_000_000;

pub fn hash(password: &str) -> String {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    derive(password, &salt, ROUNDS)
}

pub fn verify(password: &str, stored: &str) -> bool {
    expected(password, stored).is_some_and(|value| constant_time_eq(&value, stored))
}

pub fn needs_rehash(stored: &str) -> bool {
    !stored.is_empty() && !stored.starts_with("pbkdf2$")
}

fn expected(password: &str, stored: &str) -> Option<String> {
    if let Some(body) = stored.strip_prefix("pbkdf2$") {
        let mut parts = body.splitn(3, '$');
        let rounds: u32 = parts.next()?.parse().ok()?;
        let salt = hex::decode(parts.next()?).ok()?;
        if rounds == 0 || rounds > MAX_ROUNDS {
            return None;
        }
        return Some(derive(password, &salt, rounds));
    }
    let (salt, _) = stored.split_once(':')?;
    let digest = Sha256::digest(format!("{salt}{password}"));
    Some(format!("{salt}:{}", hex::encode(digest)))
}

fn derive(password: &str, salt: &[u8], rounds: u32) -> String {
    let mut digest = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, rounds, &mut digest);
    format!(
        "pbkdf2${rounds}${}${}",
        hex::encode(salt),
        hex::encode(digest)
    )
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .fold(0, |difference, (a, b)| difference | (a ^ b))
            == 0
}
