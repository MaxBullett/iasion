//! The Access key source over HTTPS and the monotonic clock for `api` (ADR-0037).

use std::time::{Duration, Instant};

use iasion_api::access::{Clock, KeySource, KeySourceError};
use iasion_app::BoxFuture;

/// Fetches the certs endpoint: the team's in production, `access-dev` in development
/// (ADR-0038, decision 8).
pub struct HttpKeySource {
    client: reqwest::Client,
    url: String,
}

impl HttpKeySource {
    pub fn new(url: String) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|error| format!("cannot set up the HTTP client: {error}"))?;
        Ok(Self { client, url })
    }
}

impl KeySource for HttpKeySource {
    fn fetch(&self) -> BoxFuture<'_, Result<Vec<u8>, KeySourceError>> {
        Box::pin(async {
            let failed = |error: reqwest::Error| KeySourceError(error.without_url().to_string());
            let response = self
                .client
                .get(&self.url)
                .send()
                .await
                .and_then(reqwest::Response::error_for_status)
                .map_err(failed)?;
            Ok(response.bytes().await.map_err(failed)?.to_vec())
        })
    }
}

/// Time since the process started.
pub struct ProcessClock(Instant);

impl ProcessClock {
    #[expect(
        clippy::disallowed_methods,
        reason = "`server` provides the clock to `api` (ADR-0034, decision 6)"
    )]
    pub fn start() -> Self {
        Self(Instant::now())
    }
}

impl Clock for ProcessClock {
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}
