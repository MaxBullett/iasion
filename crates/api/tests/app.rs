//! The whole app: the built frontend next to the API, and the security headers on
//! every response (ADR-0043, decisions 12 to 14).

// clippy.toml allows unwrap in test functions only; the helpers here are test code too.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::Router;
use axum::body::Body;
use http::{Method, Request, StatusCode, header};
use iasion_api::access::TOKEN_HEADER;
use iasion_api::problem;
use support::*;
use tower::ServiceExt;

const INDEX: &str = "<!doctype html><title>Iasion</title>";
const SCRIPT: &str = "console.log('iasion')";

/// A built frontend in a directory of its own, as `pnpm run build` leaves it.
fn web_dir() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "iasion-web-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("assets")).unwrap();
    fs::write(dir.join("index.html"), INDEX).unwrap();
    fs::write(dir.join("assets/index-abc123.js"), SCRIPT).unwrap();
    dir
}

async fn app() -> Router {
    let api = TestApi::start().await;
    iasion_api::app(api.router, &web_dir())
}

async fn get(app: &Router, path: &str, token: Option<&str>) -> Reply {
    let mut request = Request::get(path);
    if let Some(token) = token {
        request = request.header(TOKEN_HEADER, token);
    }
    send(app, request.body(Body::empty()).unwrap()).await
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    use http_body_util::BodyExt;
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

fn header<'a>(reply: &'a Reply, name: &str) -> &'a str {
    reply
        .headers
        .get(name)
        .unwrap_or_else(|| panic!("no {name} header"))
        .to_str()
        .unwrap()
}

// --- The frontend -----------------------------------------------------------------

/// Decision 12: the frontend at `/`; decision 13: without a token.
#[tokio::test]
async fn serves_the_frontend_without_a_token() {
    let app = app().await;

    let reply = get(&app, "/", None).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, INDEX);
    assert!(header(&reply, "content-type").starts_with("text/html"));
}

/// Decision 12: client-side routes load `index.html`.
#[tokio::test]
async fn serves_the_frontend_for_client_side_routes() {
    let app = app().await;

    for path in ["/accounts", "/accounts/joint-current-account/transactions"] {
        let reply = get(&app, path, None).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert_eq!(reply.body, INDEX, "{path}");
    }
}

#[tokio::test]
async fn serves_built_assets() {
    let app = app().await;

    let reply = get(&app, "/assets/index-abc123.js", None).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, SCRIPT);
    assert!(header(&reply, "content-type").contains("javascript"));
}

/// A missing file is a 404, never `index.html` served as a script or an image.
#[tokio::test]
async fn a_missing_file_is_not_found() {
    let app = app().await;

    for path in ["/assets/index-gone.js", "/favicon.ico"] {
        let reply = get(&app, path, None).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{path}");
        assert_ne!(reply.body, INDEX, "{path}");
    }
}

/// Built assets carry a content hash in their names and never change; `index.html`
/// names the current ones and must be revalidated.
#[tokio::test]
async fn caches_assets_but_not_the_page() {
    let app = app().await;

    let asset = get(&app, "/assets/index-abc123.js", None).await;
    let page = get(&app, "/", None).await;
    let route = get(&app, "/accounts", None).await;

    assert_eq!(
        header(&asset, "cache-control"),
        "public, max-age=31536000, immutable"
    );
    assert_eq!(header(&page, "cache-control"), "no-cache");
    assert_eq!(header(&route, "cache-control"), "no-cache");
}

/// A path under `/assets/` that names no file gets the page, which changes with every
/// release, so it is revalidated like the page anywhere else.
#[tokio::test]
async fn the_page_under_assets_is_not_cached() {
    let app = app().await;

    let reply = get(&app, "/assets/missing", None).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, INDEX);
    assert_eq!(header(&reply, "cache-control"), "no-cache");
}

// --- The API next to it -----------------------------------------------------------

/// Decision 13: `/api` is verified as before, whatever the path.
#[tokio::test]
async fn the_api_still_needs_a_token() {
    let app = app().await;

    for path in ["/api/session", "/api/no-such-route"] {
        let reply = get(&app, path, None).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

#[tokio::test]
async fn the_api_is_served_next_to_the_frontend() {
    let app = app().await;

    let reply = get(&app, "/api/session", Some(&token(SETUP_EMAIL))).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(reply.json()["email"], SETUP_EMAIL);
}

/// An unknown API path is a problem, not the frontend's page.
#[tokio::test]
async fn an_unknown_api_path_is_not_found() {
    let app = app().await;

    let reply = get(&app, "/api/no-such-route", Some(&token(SETUP_EMAIL))).await;

    reply.assert_problem(StatusCode::NOT_FOUND, problem::NOT_FOUND);
}

// --- Security headers -------------------------------------------------------------

/// Decision 14: on every response, the page, an asset, a 404, the API and a problem.
#[tokio::test]
async fn every_response_carries_the_security_headers() {
    let app = app().await;
    let replies = [
        get(&app, "/", None).await,
        get(&app, "/assets/index-abc123.js", None).await,
        get(&app, "/favicon.ico", None).await,
        get(&app, "/api/session", Some(&token(SETUP_EMAIL))).await,
        get(&app, "/api/session", None).await,
    ];

    for reply in &replies {
        let csp = header(reply, "content-security-policy");
        assert!(csp.contains("default-src 'self'"), "{csp}");
        assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
        assert_eq!(header(reply, "x-content-type-options"), "nosniff");
        assert_eq!(header(reply, "referrer-policy"), "no-referrer");
    }
}

/// The cross-site checks of the API do not apply to the frontend, but the frontend
/// takes nothing but GET and HEAD.
#[tokio::test]
async fn the_frontend_takes_only_get_and_head() {
    let app = app().await;

    let head = send(
        &app,
        Request::builder()
            .method(Method::HEAD)
            .uri("/")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    let post = send(
        &app,
        Request::builder()
            .method(Method::POST)
            .uri("/")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap(),
    )
    .await;

    assert_eq!(head.status, StatusCode::OK);
    assert_eq!(post.status, StatusCode::METHOD_NOT_ALLOWED);
}
