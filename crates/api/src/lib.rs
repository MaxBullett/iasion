//! HTTP API and its OpenAPI description (ADR-0035, ADR-0036).
//!
//! Every route under `/api` is served only for a request with a valid Cloudflare
//! Access token (ADR-0037). Requests other than GET must also come from the app's own
//! origin and carry JSON (ADR-0037, decisions 10 and 11).

pub mod access;
pub mod problem;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use iasion_app::SystemStatus;
use serde::Serialize;
use tracing::Instrument;
use utoipa::openapi::OpenApi;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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

#[derive(Clone)]
struct AppState {
    verifier: Arc<access::Verifier>,
    public_origin: Arc<str>,
    setup_email: Arc<str>,
    status: Arc<dyn SystemStatus>,
    request_ids: Arc<AtomicU64>,
}

/// The verified email of the request, trimmed and lowercased.
#[derive(Clone)]
struct VerifiedEmail(String);

/// The API's routes with authentication, the cross-site checks and request logging.
pub fn router(
    settings: Settings,
    keys: Arc<access::Keys>,
    status: Arc<dyn SystemStatus>,
) -> Router {
    let state = AppState {
        verifier: Arc::new(access::Verifier::new(&settings.access, keys)),
        public_origin: settings.public_origin.trim().trim_end_matches('/').into(),
        setup_email: access::normalize_email(&settings.setup_email).into(),
        status,
        request_ids: Arc::new(AtomicU64::new(0)),
    };
    let (routes, _) = routes().split_for_parts();
    // The layer added last runs first: logging, then the cross-site checks, then
    // authentication.
    routes
        .layer(middleware::from_fn_with_state(state.clone(), authenticate))
        .layer(middleware::from_fn_with_state(state.clone(), cross_site))
        .layer(middleware::from_fn_with_state(state.clone(), request_log))
        .with_state(state)
}

/// The API description, generated from the routes (ADR-0036, decision 1). Committed
/// as `crates/api/openapi.json`; `cargo xtask openapi` regenerates it.
pub fn openapi() -> OpenApi {
    let (_, openapi) = routes().split_for_parts();
    openapi
}

/// Every route, with its part of the description.
fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(<Description as utoipa::OpenApi>::openapi())
        .routes(routes!(session))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Iasion",
        description = "The API of Iasion's frontend. Every route needs a Cloudflare Access \
            token; requests other than GET must come from the app's own origin and carry \
            JSON. Errors are RFC 9457 problem details.",
        license(name = "AGPL-3.0-or-later", identifier = "AGPL-3.0-or-later"),
    ),
    components(schemas(problem::Problem, problem::ProblemType)),
    security(("cloudflare_access" = [])),
    modifiers(&AccessToken),
)]
struct Description;

/// The Access token as the security scheme of every route (ADR-0037, decision 1).
struct AccessToken;

impl Modify for AccessToken {
    fn modify(&self, openapi: &mut OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "cloudflare_access",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "Cf-Access-Jwt-Assertion",
                "The JWT that Cloudflare Access adds to every request it lets through.",
            ))),
        );
    }
}

/// Gives each request an ID of its own, logs under it, and returns it in a header. An
/// ID the client sends is replaced, so the logs never carry client-chosen IDs.
async fn request_log(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let number = state.request_ids.fetch_add(1, Ordering::Relaxed) + 1;
    let id = format!("{number:016x}");
    request.headers_mut().remove(REQUEST_ID_HEADER);
    let span = tracing::info_span!("request", request_id = %id, method = %request.method());
    let mut response = next.run(request).instrument(span.clone()).await;
    span.in_scope(|| tracing::info!(status = response.status().as_u16(), "response"));
    if let Ok(value) = HeaderValue::from_str(&id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, value);
    }
    response
}

/// Requests other than GET come from the app's own origin and carry JSON (ADR-0037,
/// decisions 10 and 11).
async fn cross_site(State(state): State<AppState>, request: Request, next: Next) -> Response {
    if request.method() == Method::GET {
        return next.run(request).await;
    }
    let origin = request.headers().get(header::ORIGIN);
    if origin.and_then(|value| value.to_str().ok()) != Some(&*state.public_origin) {
        tracing::info!(reason = "not the own origin", "request denied");
        return problem::response(
            StatusCode::FORBIDDEN,
            problem::ProblemType::CrossOrigin,
            "Requests other than GET must come from the app's own origin",
        );
    }
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if !content_type.is_some_and(|essence| essence.eq_ignore_ascii_case("application/json")) {
        tracing::info!(reason = "not JSON", "request denied");
        return problem::response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            problem::ProblemType::UnsupportedMediaType,
            "Requests other than GET must carry application/json",
        );
    }
    next.run(request).await
}

/// Verifies the Access token and admits the setup email only: no household exists in
/// the walking skeleton (ADR-0024, ADR-0037, decision 9). S-014 replaces the check
/// with the login identities of the Access context.
async fn authenticate(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(access::TOKEN_HEADER)
        .and_then(|value| value.to_str().ok());
    let Some(token) = token else {
        return unauthenticated("no token");
    };
    let email = match state.verifier.verify(token).await {
        Ok(email) => email,
        Err(access::Failure::Unauthenticated(reason)) => return unauthenticated(reason),
        Err(access::Failure::KeysUnavailable) => {
            tracing::warn!(reason = "no Access keys", "request refused");
            return problem::response(
                StatusCode::SERVICE_UNAVAILABLE,
                problem::ProblemType::AccessKeysUnavailable,
                "No Access keys are available to verify the request",
            );
        }
    };
    let Some(email) = email.map(|email| access::normalize_email(&email)) else {
        return denied("no email claim");
    };
    if *email != *state.setup_email {
        return denied("not the setup email");
    }
    request.extensions_mut().insert(VerifiedEmail(email));
    next.run(request).await
}

fn unauthenticated(reason: &'static str) -> Response {
    tracing::info!(reason, "request denied");
    problem::response(
        StatusCode::UNAUTHORIZED,
        problem::ProblemType::Unauthenticated,
        "A valid Cloudflare Access token is required",
    )
}

fn denied(reason: &'static str) -> Response {
    tracing::info!(reason, "request denied");
    problem::response(
        StatusCode::FORBIDDEN,
        problem::ProblemType::AccessDenied,
        "This sign-in may not use Iasion",
    )
}

/// The signed-in session.
#[derive(Serialize, ToSchema)]
struct Session {
    /// The verified email, trimmed and lowercased.
    email: String,
    /// No household exists yet; only first setup can follow (ADR-0024).
    first_setup_pending: bool,
    /// Version of the latest applied migration; `null` before the first.
    #[schema(required = true)]
    schema_version: Option<i64>,
}

/// Who is signed in, and the state of the installation.
#[utoipa::path(
    get,
    path = "/api/session",
    responses(
        (status = OK, body = Session, description = "The session"),
        (status = UNAUTHORIZED, body = problem::Problem, content_type = problem::CONTENT_TYPE,
            description = "No valid Access token"),
        (status = FORBIDDEN, body = problem::Problem, content_type = problem::CONTENT_TYPE,
            description = "The sign-in may not use Iasion"),
        (status = SERVICE_UNAVAILABLE, body = problem::Problem, content_type = problem::CONTENT_TYPE,
            description = "No Access keys, or no database"),
    ),
)]
async fn session(
    State(state): State<AppState>,
    Extension(VerifiedEmail(email)): Extension<VerifiedEmail>,
) -> Response {
    match state.status.schema_version().await {
        Ok(schema_version) => Json(Session {
            email,
            first_setup_pending: true,
            schema_version,
        })
        .into_response(),
        Err(_) => {
            tracing::warn!(reason = "status unavailable", "request failed");
            problem::response(
                StatusCode::SERVICE_UNAVAILABLE,
                problem::ProblemType::DatabaseUnavailable,
                "The database is unavailable",
            )
        }
    }
}
