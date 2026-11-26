//! Test support for the API tests: test key pairs, tokens, fakes for the ports, and a
//! capture of the logs (ADR-0037, decision 13: API tests use a test key pair).
//!
//! Key pairs are generated at runtime and never committed (ADR-0038).

#![allow(dead_code)]

use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use http::{HeaderMap, Request, StatusCode};
use http_body_util::BodyExt;
use iasion_api::access::{self, Clock, KeySource, KeySourceError, Keys};
use iasion_api::{Settings, router};
use iasion_app::{BoxFuture, StatusError, SystemStatus};
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::{Generate, PrivateKey};
use serde_json::{Map, Value, json};
use tower::ServiceExt;

pub const TEAM_DOMAIN: &str = "example-team.cloudflareaccess.com";
pub const ISSUER: &str = "https://example-team.cloudflareaccess.com";
pub const AUD: &str = "test-aud-tag";
pub const PUBLIC_ORIGIN: &str = "https://iasion.example.com";
pub const SETUP_EMAIL: &str = "first.member@example.com";
/// A member of the reference household (`fixtures/README.md`) who is not the setup
/// email.
pub const OTHER_EMAIL: &str = "alex@example.com";
pub const SCHEMA_VERSION: i64 = 20261126000004;

/// A test signing key with its `kid`.
pub struct TestKey {
    pub kid: &'static str,
    pub encoding: EncodingKey,
    pub jwk: Jwk,
}

fn generate(kid: &'static str) -> TestKey {
    let private = PrivateKey::generate();
    let der = private.to_pkcs1_der().unwrap();
    let encoding = EncodingKey::from_rsa_der(der.as_bytes());
    let mut jwk = Jwk::from_encoding_key(&encoding, Algorithm::RS256).unwrap();
    jwk.common.key_id = Some(kid.to_string());
    TestKey { kid, encoding, jwk }
}

/// The key the tests' key source serves first.
pub static KEY_A: LazyLock<TestKey> = LazyLock::new(|| generate("key-a"));
/// The key a rotation adds.
pub static KEY_B: LazyLock<TestKey> = LazyLock::new(|| generate("key-b"));
/// A key nobody serves, presented under KEY_A's `kid`.
pub static IMPOSTOR: LazyLock<TestKey> = LazyLock::new(|| generate("key-a"));

/// The body of a certs endpoint serving `keys`, in Cloudflare's format: the JWK set
/// under `keys`, and the same keys as certificates under `public_cert(s)`, which
/// Iasion does not read.
pub fn certs(keys: &[&TestKey]) -> Vec<u8> {
    let set = JwkSet {
        keys: keys.iter().map(|key| key.jwk.clone()).collect(),
    };
    let mut body = serde_json::to_value(set).unwrap();
    body["public_certs"] = json!([]);
    serde_json::to_vec(&body).unwrap()
}

pub fn now() -> u64 {
    jsonwebtoken::get_current_timestamp()
}

/// Claims of a valid token for `email`, issued a minute ago, expiring in ten minutes.
pub fn claims(email: &str) -> Map<String, Value> {
    let now = now();
    let mut claims = Map::new();
    claims.insert("aud".into(), json!([AUD]));
    claims.insert("iss".into(), json!(ISSUER));
    claims.insert("sub".into(), json!("00000000-0000-0000-0000-000000000001"));
    claims.insert("email".into(), json!(email));
    claims.insert("type".into(), json!("app"));
    claims.insert("iat".into(), json!(now - 60));
    claims.insert("nbf".into(), json!(now - 60));
    claims.insert("exp".into(), json!(now + 600));
    claims
}

/// A token with `claims`, signed with `key` under its `kid` with RS256.
pub fn sign(key: &TestKey, claims: &Map<String, Value>) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(key.kid.to_string());
    jsonwebtoken::encode(&header, claims, &key.encoding).unwrap()
}

/// A valid token for `email`, signed with KEY_A.
pub fn token(email: &str) -> String {
    sign(&KEY_A, &claims(email))
}

/// A key source whose response the test controls, counting its fetches.
pub struct FakeKeySource {
    response: Mutex<Result<Vec<u8>, KeySourceError>>,
    fetches: AtomicUsize,
}

impl FakeKeySource {
    pub fn serving(body: Vec<u8>) -> Arc<Self> {
        Arc::new(Self {
            response: Mutex::new(Ok(body)),
            fetches: AtomicUsize::new(0),
        })
    }

    pub fn failing() -> Arc<Self> {
        let source = Self::serving(Vec::new());
        source.fail();
        source
    }

    pub fn serve(&self, body: Vec<u8>) {
        *self.response.lock().unwrap() = Ok(body);
    }

    pub fn fail(&self) {
        *self.response.lock().unwrap() = Err(KeySourceError("connection refused".into()));
    }

    pub fn fetches(&self) -> usize {
        self.fetches.load(Ordering::SeqCst)
    }
}

impl KeySource for FakeKeySource {
    fn fetch(&self) -> BoxFuture<'_, Result<Vec<u8>, KeySourceError>> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        let response = self.response.lock().unwrap().clone();
        Box::pin(async move { response })
    }
}

/// A clock that moves only when the test advances it.
#[derive(Default)]
pub struct FakeClock {
    elapsed: Mutex<Duration>,
}

impl FakeClock {
    pub fn advance(&self, by: Duration) {
        *self.elapsed.lock().unwrap() += by;
    }
}

impl Clock for FakeClock {
    fn elapsed(&self) -> Duration {
        *self.elapsed.lock().unwrap()
    }
}

/// Reports the migrations as applied up to SCHEMA_VERSION.
pub struct FakeStatus;

impl SystemStatus for FakeStatus {
    fn schema_version(&self) -> BoxFuture<'_, Result<Option<i64>, StatusError>> {
        Box::pin(async { Ok(Some(SCHEMA_VERSION)) })
    }
}

pub fn settings() -> Settings {
    Settings {
        access: access::Settings {
            team_domain: TEAM_DOMAIN.into(),
            aud: AUD.into(),
        },
        public_origin: PUBLIC_ORIGIN.into(),
        setup_email: SETUP_EMAIL.into(),
    }
}

/// The API as `server` starts it: keys fetched once at startup, at clock time zero.
pub struct TestApi {
    pub router: Router,
    pub keys: Arc<Keys>,
    pub source: Arc<FakeKeySource>,
    pub clock: Arc<FakeClock>,
}

impl TestApi {
    /// Started with a key source serving KEY_A.
    pub async fn start() -> Self {
        Self::start_with(settings(), FakeKeySource::serving(certs(&[&KEY_A]))).await
    }

    /// Started with `source`; the startup fetch may fail, as it does in production.
    pub async fn start_with(settings: Settings, source: Arc<FakeKeySource>) -> Self {
        let clock = Arc::new(FakeClock::default());
        let keys = Arc::new(Keys::new(source.clone(), clock.clone()));
        let _ = keys.refresh().await;
        let router = router(settings, keys.clone(), Arc::new(FakeStatus));
        Self {
            router,
            keys,
            source,
            clock,
        }
    }

    pub async fn send(&self, request: Request<Body>) -> Reply {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        Reply {
            status,
            headers,
            body: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }

    /// `GET /api/session` with `token` in the Access header.
    pub async fn session(&self, token: &str) -> Reply {
        self.send(
            get_session()
                .header(access::TOKEN_HEADER, token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }
}

pub fn get_session() -> http::request::Builder {
    Request::get("/api/session")
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: String,
}

impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap()
    }

    /// Asserts a problem response (RFC 9457, ADR-0036, decision 9) of `type`.
    pub fn assert_problem(&self, status: StatusCode, problem_type: &str) {
        assert_eq!(self.status, status, "body: {}", self.body);
        assert_eq!(
            self.headers.get(http::header::CONTENT_TYPE).unwrap(),
            iasion_api::problem::CONTENT_TYPE
        );
        let body = self.json();
        assert_eq!(body["type"], problem_type);
        assert_eq!(body["status"], status.as_u16());
        assert!(body["title"].is_string());
    }
}

/// Captures what the code under test logs while the returned guard lives.
pub fn capture_logs() -> (Logs, tracing::subscriber::DefaultGuard) {
    let logs = Logs::default();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(logs.clone())
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .finish();
    (logs, tracing::subscriber::set_default(subscriber))
}

#[derive(Clone, Default)]
pub struct Logs(Arc<Mutex<Vec<u8>>>);

impl Logs {
    pub fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl io::Write for Logs {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Logs {
    type Writer = Logs;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
