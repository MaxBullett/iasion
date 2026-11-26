//! The built frontend next to the API, and the security headers on every response
//! (ADR-0043, decisions 12 to 14).

use std::path::Path;

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

/// Content from the app's own origin only, and never inside a frame. The built
/// frontend has no inline scripts or styles, so none are allowed.
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; base-uri 'self'; form-action 'self'; \
     frame-ancestors 'none'; object-src 'none'";

/// Built assets carry a content hash in their names, so they never change.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";
/// `index.html` names the current assets, so it is revalidated on every load.
const REVALIDATE: &str = "no-cache";

/// The API with the built frontend in `web_dir` as its fallback, and the security
/// headers on every response. Static files need no token: they are the published code
/// (decision 13). Paths under `/api` never reach the frontend.
pub fn app(api: Router, web_dir: &Path) -> Router {
    let files = ServeDir::new(web_dir);
    let page = ServeFile::new(web_dir.join("index.html"));
    api.fallback(move |request: Request| frontend(files.clone(), page.clone(), request))
        .layer(header_layer(
            header::CONTENT_SECURITY_POLICY,
            CONTENT_SECURITY_POLICY,
        ))
        .layer(header_layer(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(header_layer(header::REFERRER_POLICY, "no-referrer"))
}

fn header_layer(name: HeaderName, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
}

/// A path whose last segment names a file is served from the directory, and is a 404
/// if missing. Any other path is a client-side route and gets `index.html`.
async fn frontend(files: ServeDir, page: ServeFile, request: Request) -> Response {
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let path = request.uri().path();
    let names_a_file = path
        .rsplit('/')
        .next()
        .is_some_and(|last| last.contains('.'));
    let cache = if path.starts_with("/assets/") {
        IMMUTABLE
    } else {
        REVALIDATE
    };
    let response = if names_a_file {
        files
            .oneshot(request)
            .await
            .map(|response| response.map(Body::new))
    } else {
        page.oneshot(request)
            .await
            .map(|response| response.map(Body::new))
    };
    let mut response = match response {
        Ok(response) => response,
        Err(never) => match never {},
    };
    if response.status().is_success() {
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    }
    response
}
