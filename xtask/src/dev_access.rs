//! The local stand-in for Cloudflare Access (ADR-0038, decision 7): a dev key pair in
//! `.dev/access/`, its public half as a certs response that `access-dev` serves, and
//! tokens signed with it. Verification in the app is the same as in production; only
//! `IASION_ACCESS_CERTS_URL` points elsewhere (ADR-0037, decision 12).
//!
//! The key pair is generated on first use and never committed (`.dev/` is
//! gitignored).

use std::fs;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey};
use rsa::traits::PublicKeyParts;
use rsa::{Generate, PrivateKey};
use serde_json::{Map, Value, json};

const PRIVATE_KEY: &str = ".dev/access/private-key.der";
/// The directory `access-dev` serves; it holds only the public certs.
pub const PUBLIC_DIRECTORY: &str = ".dev/access/public";
const CERTS: &str = ".dev/access/public/certs";

/// Dev tokens are valid this long, in seconds.
const LIFETIME: u64 = 12 * 60 * 60;

/// Prints a token for `email` (the setup email if none is given), signed with the dev
/// key, for the instance settings in the environment or in `.env`.
pub fn dev_token(args: &[String]) -> Result<(), String> {
    let email = match args {
        [] => setting("IASION_SETUP_EMAIL")?,
        [flag, email] if flag == "--email" => email.clone(),
        _ => return Err("usage: cargo xtask dev-token [--email <email>]".into()),
    };
    let team_domain = setting("IASION_ACCESS_TEAM_DOMAIN")?;
    let aud = setting("IASION_ACCESS_AUD")?;

    let (encoding, kid) = key()?;
    let now = jsonwebtoken::get_current_timestamp();
    let mut claims = Map::new();
    claims.insert("aud".into(), json!([aud.trim()]));
    claims.insert(
        "iss".into(),
        json!(format!("https://{}", team_domain.trim())),
    );
    claims.insert("email".into(), json!(email));
    claims.insert("type".into(), json!("app"));
    claims.insert("iat".into(), json!(now));
    claims.insert("nbf".into(), json!(now));
    claims.insert("exp".into(), json!(now + LIFETIME));
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid);
    let token = jsonwebtoken::encode(&header, &Value::Object(claims), &encoding)
        .map_err(|error| format!("dev-token: cannot sign: {error}"))?;
    println!("{token}");
    Ok(())
}

/// Loads the dev key, generating it on first use, and writes its certs response.
fn key() -> Result<(EncodingKey, String), String> {
    let private = if Path::new(PRIVATE_KEY).exists() {
        let der = fs::read(PRIVATE_KEY).map_err(|error| format!("{PRIVATE_KEY}: {error}"))?;
        PrivateKey::from_pkcs1_der(&der).map_err(|error| format!("{PRIVATE_KEY}: {error}"))?
    } else {
        eprintln!("dev-token: generating the dev key pair in .dev/access/");
        let private = PrivateKey::generate();
        let der = private
            .to_pkcs1_der()
            .map_err(|error| format!("dev-token: {error}"))?;
        write_private(der.as_bytes())?;
        private
    };
    let der = private
        .to_pkcs1_der()
        .map_err(|error| format!("dev-token: {error}"))?;
    let encoding = EncodingKey::from_rsa_der(der.as_bytes());
    // A key id per key pair, so that a regenerated pair is fetched again.
    let kid: String = private.n_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let kid = format!("dev-{kid}");
    write_certs(&encoding, &kid)?;
    Ok((encoding, kid))
}

fn write_private(der: &[u8]) -> Result<(), String> {
    fs::create_dir_all(".dev/access").map_err(|error| format!(".dev/access: {error}"))?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(PRIVATE_KEY)
        .and_then(|mut file| file.write_all(der))
        .map_err(|error| format!("{PRIVATE_KEY}: {error}"))
}

/// The public key as Cloudflare's certs endpoint serves it: a JWK set under `keys`.
fn write_certs(encoding: &EncodingKey, kid: &str) -> Result<(), String> {
    let mut jwk = Jwk::from_encoding_key(encoding, Algorithm::RS256)
        .map_err(|error| format!("dev-token: {error}"))?;
    jwk.common.key_id = Some(kid.to_string());
    let body = serde_json::to_vec_pretty(&JwkSet { keys: vec![jwk] })
        .map_err(|error| format!("dev-token: {error}"))?;
    fs::create_dir_all(PUBLIC_DIRECTORY).map_err(|error| format!("{PUBLIC_DIRECTORY}: {error}"))?;
    fs::write(CERTS, body).map_err(|error| format!("{CERTS}: {error}"))
}

/// A setting from the environment, or else from `.env`, so that the command works
/// without direnv too.
fn setting(name: &str) -> Result<String, String> {
    if let Ok(value) = std::env::var(name)
        && !value.trim().is_empty()
    {
        return Ok(value);
    }
    let file = fs::read_to_string(".env").unwrap_or_default();
    file.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == name)
        .map(|(_, value)| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!("{name} is not set in the environment or in .env (see .env.example)")
        })
}
