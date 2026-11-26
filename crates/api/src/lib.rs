//! HTTP API and its OpenAPI description (ADR-0035, ADR-0036).
//!
//! Every route under `/api` is served only for a request with a valid Cloudflare
//! Access token (ADR-0037). Requests other than GET must also come from the app's own
//! origin and, if they carry a body, carry JSON (ADR-0037, decisions 10 and 11).

pub mod access;
pub mod problem;

use std::sync::Arc;

use axum::Router;
use iasion_app::SystemStatus;

/// Settings of the API, read from the environment by `server` (`.env.example`).
#[derive(Debug, Clone)]
pub struct Settings {
    /// Cloudflare Access: team domain and AUD tag (ADR-0037, decision 3).
    pub access: access::Settings,
    /// The origin the app is served from, e.g. `https://iasion.example.com`
    /// (`IASION_PUBLIC_ORIGIN`, ADR-0037, decision 11).
    pub public_origin: String,
    /// The email allowed to run first setup while no household exists
    /// (`IASION_SETUP_EMAIL`, ADR-0024). Compared trimmed and lowercased (ADR-0037,
    /// decision 8).
    pub setup_email: String,
}

/// Response header that carries the request ID, which the logs carry as well
/// (ADR-0037, decision 14).
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The API's routes with authentication, the cross-site checks and request logging.
pub fn router(
    settings: Settings,
    keys: Arc<access::Keys>,
    status: Arc<dyn SystemStatus>,
) -> Router {
    let _ = (settings, keys, status);
    todo!("implemented after the tests are reviewed")
}
