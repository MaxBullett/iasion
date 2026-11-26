//! Verifying Cloudflare Access tokens (ADR-0037).
//!
//! Only the `Cf-Access-Jwt-Assertion` header is read. A token is accepted with RS256
//! only, signed by a key from the team's certs endpoint whose `kid` matches, with
//! `aud`, `iss`, `exp` and `nbf` all present and valid, and a tolerance of 60 seconds.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use iasion_app::BoxFuture;

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
    _private: (),
}

impl Keys {
    /// Holds no keys until the first [`Keys::refresh`]. Until then, and whenever no
    /// valid key is held, requests are refused with 503 (ADR-0037, decision 6).
    pub fn new(source: Arc<dyn KeySource>, clock: Arc<dyn Clock>) -> Self {
        let _ = (source, clock);
        todo!("implemented after the tests are reviewed")
    }

    /// Fetches the keys and replaces the ones held. On failure, the keys held so far
    /// stay. `server` calls it at startup and every [`REFRESH_INTERVAL`].
    pub async fn refresh(&self) -> Result<(), KeysError> {
        todo!("implemented after the tests are reviewed")
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
