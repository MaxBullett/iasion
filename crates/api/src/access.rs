//! Verifying Cloudflare Access tokens (ADR-0037).
//!
//! Only the `Cf-Access-Jwt-Assertion` header is read. A token is accepted with RS256
//! only, signed by a key from the team's certs endpoint whose `kid` matches, with
//! `aud`, `iss`, `exp` and `nbf` all present and valid, and a tolerance of 60 seconds.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use iasion_app::BoxFuture;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, KeyAlgorithm};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::Deserialize;

/// The request header that carries the Access token (ADR-0037, decision 1).
pub const TOKEN_HEADER: &str = "cf-access-jwt-assertion";

/// Tolerance for `exp` and `nbf` (ADR-0037, decision 2).
pub const LEEWAY: Duration = Duration::from_secs(60);

/// A token with an unknown `kid` fetches the keys again at most this often
/// (ADR-0037, decision 5).
pub const UNKNOWN_KID_REFETCH_INTERVAL: Duration = Duration::from_secs(60);

/// `server` refreshes the keys this often (ADR-0037, decision 5).
pub const REFRESH_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Cloudflare Access settings of the instance (ADR-0037, decision 3).
#[derive(Debug, Clone)]
pub struct Settings {
    /// `IASION_ACCESS_TEAM_DOMAIN`, e.g. `example-team.cloudflareaccess.com`. The token's
    /// `iss` must be `https://` followed by it.
    pub team_domain: String,
    /// `IASION_ACCESS_AUD`: the application's AUD tag, which `aud` must contain.
    pub aud: String,
}

/// Where the keys come from: the certs endpoint in production, `access-dev` in
/// development (ADR-0038, decision 8), a fake in tests. `server` implements it over
/// HTTP.
pub trait KeySource: Send + Sync {
    /// The body of the certs endpoint: JSON with the keys as a JWK set under `keys`.
    fn fetch(&self) -> BoxFuture<'_, Result<Vec<u8>, KeySourceError>>;
}

/// Fetching the keys failed. The message must not contain secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySourceError(pub String);

/// A monotonic clock, provided by `server`, because only `app` and `server` may read
/// the clock (ADR-0034, decision 6). Tests use a fake.
pub trait Clock: Send + Sync {
    /// Time since an arbitrary, fixed start. Never decreases.
    fn elapsed(&self) -> Duration;
}

/// The Access keys currently held, by `kid`.
pub struct Keys {
    source: Arc<dyn KeySource>,
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    keys: Arc<HashMap<String, DecodingKey>>,
    /// Clock time of the latest fetch, successful or not.
    last_fetch: Option<Duration>,
}

impl Keys {
    /// Holds no keys until the first [`Keys::refresh`]. Until then, and whenever no
    /// valid key is held, requests are refused with 503 (ADR-0037, decision 6).
    pub fn new(source: Arc<dyn KeySource>, clock: Arc<dyn Clock>) -> Self {
        Self {
            source,
            clock,
            state: Mutex::new(State::default()),
        }
    }

    /// Fetches the keys and replaces the ones held. On failure, the keys held so far
    /// stay. `server` calls it at startup and every [`REFRESH_INTERVAL`].
    pub async fn refresh(&self) -> Result<(), KeysError> {
        let now = self.clock.elapsed();
        self.lock().last_fetch = Some(now);
        self.fetch().await
    }

    async fn fetch(&self) -> Result<(), KeysError> {
        let body = self.source.fetch().await.map_err(KeysError::Fetch)?;
        let keys = parse_certs(&body)?;
        self.lock().keys = Arc::new(keys);
        Ok(())
    }

    /// Fetches the keys again unless the latest fetch was less than a minute ago. The
    /// slot is taken before fetching, so concurrent requests fetch once.
    async fn refetch_if_due(&self) {
        let now = self.clock.elapsed();
        {
            let mut state = self.lock();
            let due = state
                .last_fetch
                .is_none_or(|last| now.saturating_sub(last) >= UNKNOWN_KID_REFETCH_INTERVAL);
            if !due {
                return;
            }
            state.last_fetch = Some(now);
        }
        if let Err(error) = self.fetch().await {
            tracing::warn!(%error, "refetching the Access keys failed");
        }
    }

    fn held(&self) -> Arc<HashMap<String, DecodingKey>> {
        self.lock().keys.clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        // The state stays consistent at every point a holder could panic.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The key for `kid`, fetching the keys again if it is unknown or none is held.
    async fn key(&self, kid: &str) -> Result<DecodingKey, Failure> {
        if let Some(key) = self.held().get(kid) {
            return Ok(key.clone());
        }
        self.refetch_if_due().await;
        let held = self.held();
        if held.is_empty() {
            return Err(Failure::KeysUnavailable);
        }
        held.get(kid)
            .cloned()
            .ok_or(Failure::Unauthenticated("unknown key id"))
    }
}

/// Refreshing the keys failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeysError {
    /// The key source failed.
    Fetch(KeySourceError),
    /// The response held no usable RS256 key.
    Invalid(String),
}

impl fmt::Display for KeysError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fetch(KeySourceError(message)) => {
                write!(formatter, "cannot fetch the Access keys: {message}")
            }
            Self::Invalid(message) => write!(formatter, "the Access keys are unusable: {message}"),
        }
    }
}

impl std::error::Error for KeysError {}

#[derive(Deserialize)]
struct Certs {
    keys: Vec<serde_json::Value>,
}

/// The RS256 keys with a `kid` in a certs response. Other keys are skipped; a response
/// without any usable key is an error, so it never replaces usable keys.
fn parse_certs(body: &[u8]) -> Result<HashMap<String, DecodingKey>, KeysError> {
    let certs: Certs = serde_json::from_slice(body)
        .map_err(|_| KeysError::Invalid("not a JSON object with `keys`".into()))?;
    let keys: HashMap<_, _> = certs
        .keys
        .into_iter()
        .filter_map(|value| serde_json::from_value::<Jwk>(value).ok())
        .filter(|jwk| matches!(jwk.algorithm, AlgorithmParameters::RSA(_)))
        .filter(|jwk| matches!(jwk.common.key_algorithm, None | Some(KeyAlgorithm::RS256)))
        .filter_map(|jwk| {
            let kid = jwk.common.key_id.clone()?;
            let key = DecodingKey::from_jwk(&jwk).ok()?;
            Some((kid, key))
        })
        .collect();
    if keys.is_empty() {
        return Err(KeysError::Invalid("no RS256 key with a key id".into()));
    }
    Ok(keys)
}

/// Verifies tokens against the keys held, for one instance's settings.
pub(crate) struct Verifier {
    keys: Arc<Keys>,
    validation: Validation,
}

/// Why a request is not served. The reasons go to the log only, never to the client
/// (ADR-0037, decision 14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    /// 401: no valid token.
    Unauthenticated(&'static str),
    /// 503: no valid key is held.
    KeysUnavailable,
}

/// The claims Iasion reads from a verified token.
#[derive(Deserialize)]
struct Claims {
    email: Option<String>,
}

impl Verifier {
    pub(crate) fn new(settings: &Settings, keys: Arc<Keys>) -> Self {
        // jsonwebtoken's defaults do not fit and are set explicitly (decision 4).
        let mut validation = Validation::new(Algorithm::RS256);
        validation.algorithms = vec![Algorithm::RS256];
        validation.set_audience(&[settings.aud.trim()]);
        validation.set_issuer(&[format!("https://{}", settings.team_domain.trim())]);
        validation.set_required_spec_claims(&["aud", "iss", "exp", "nbf"]);
        validation.validate_aud = true;
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = LEEWAY.as_secs();
        Self { keys, validation }
    }

    /// The token's `email` claim as given, if the token is valid.
    pub(crate) async fn verify(&self, token: &str) -> Result<Option<String>, Failure> {
        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| Failure::Unauthenticated("malformed token"))?;
        if header.alg != Algorithm::RS256 {
            return Err(Failure::Unauthenticated("algorithm is not RS256"));
        }
        let kid = header.kid.ok_or(Failure::Unauthenticated("no key id"))?;
        let key = self.keys.key(&kid).await?;
        let data = jsonwebtoken::decode::<Claims>(token, &key, &self.validation)
            .map_err(|error| Failure::Unauthenticated(reason(error.kind())))?;
        Ok(data.claims.email)
    }
}

fn reason(kind: &ErrorKind) -> &'static str {
    match kind {
        ErrorKind::InvalidSignature => "invalid signature",
        ErrorKind::ExpiredSignature => "expired",
        ErrorKind::ImmatureSignature => "not yet valid",
        ErrorKind::InvalidAudience => "wrong audience",
        ErrorKind::InvalidIssuer => "wrong issuer",
        ErrorKind::MissingRequiredClaim(_) => "missing required claim",
        ErrorKind::InvalidAlgorithm => "algorithm is not RS256",
        _ => "invalid token",
    }
}

/// An email as it is stored and compared: trimmed and lowercased (ADR-0037,
/// decision 8).
pub(crate) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
