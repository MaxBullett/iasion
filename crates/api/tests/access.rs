//! Authentication and the cross-site checks of the API (ADR-0037, S-014), on
//! `GET /api/session`, the walking skeleton's only route.
//!
//! No household exists in the walking skeleton, so only the setup email is served
//! (ADR-0024, ADR-0037, decision 9). Decision numbers below are ADR-0037's.

// clippy.toml allows unwrap in test functions only; the helpers here are test code too.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use std::time::Duration;

use axum::body::Body;
use http::{Method, Request, StatusCode, header};
use iasion_api::access::TOKEN_HEADER;
use iasion_api::{REQUEST_ID_HEADER, problem};
use serde_json::json;
use support::*;

// --- Served ---------------------------------------------------------------------

/// S-014: a request with a valid Access JWT is served. Decision 9: while no household
/// exists, for the setup email.
#[tokio::test]
async fn serves_the_setup_email_with_a_valid_token() {
    let api = TestApi::start().await;

    let reply = api.session(&token(SETUP_EMAIL)).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(
        reply.json(),
        json!({
            "email": SETUP_EMAIL,
            "first_setup_pending": true,
            "schema_version": SCHEMA_VERSION,
        })
    );
}

/// Decision 8: emails are compared trimmed and lowercased, the setup email included.
#[tokio::test]
async fn compares_emails_trimmed_and_lowercased() {
    let mut settings = settings();
    settings.setup_email = " First.Member@EXAMPLE.com ".into();
    let api = TestApi::start_with(settings, FakeKeySource::serving(certs(&[&KEY_A]))).await;

    let reply = api.session(&token("  FIRST.member@Example.COM")).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(reply.json()["email"], SETUP_EMAIL);
}

/// Decision 2: `aud` must contain the AUD tag; Access sends a list.
#[tokio::test]
async fn accepts_an_aud_list_that_contains_the_aud_tag() {
    let api = TestApi::start().await;
    let mut claims = claims(SETUP_EMAIL);
    claims.insert("aud".into(), json!(["another-application", AUD]));

    let reply = api.session(&sign(&KEY_A, &claims)).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
}

/// Decision 2: `exp` and `nbf` hold with a tolerance of 60 seconds.
#[tokio::test]
async fn tolerates_60_seconds_of_clock_skew() {
    let api = TestApi::start().await;

    let mut expired_lately = claims(SETUP_EMAIL);
    expired_lately.insert("exp".into(), json!(now() - 30));
    let mut valid_soon = claims(SETUP_EMAIL);
    valid_soon.insert("nbf".into(), json!(now() + 30));

    for claims in [expired_lately, valid_soon] {
        let reply = api.session(&sign(&KEY_A, &claims)).await;
        assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    }
}

// --- Rejected without a valid token: 401 ------------------------------------------

/// S-014: without a valid Access JWT a request is rejected.
#[tokio::test]
async fn rejects_a_request_without_a_token() {
    let api = TestApi::start().await;

    let reply = api.send(get_session().body(Body::empty()).unwrap()).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

/// S-014: rejected even if an email header is present. Decision 1: never an email
/// header.
#[tokio::test]
async fn ignores_the_email_header() {
    let api = TestApi::start().await;

    let request = get_session()
        .header("cf-access-authenticated-user-email", SETUP_EMAIL)
        .body(Body::empty())
        .unwrap();
    let reply = api.send(request).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

/// Decision 1: never the cookie, even with a valid token in it.
#[tokio::test]
async fn ignores_the_authorization_cookie() {
    let api = TestApi::start().await;

    let request = get_session()
        .header(
            header::COOKIE,
            format!("CF_Authorization={}", token(SETUP_EMAIL)),
        )
        .body(Body::empty())
        .unwrap();
    let reply = api.send(request).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

#[tokio::test]
async fn rejects_a_malformed_token() {
    let api = TestApi::start().await;

    for malformed in ["", "not-a-token", "a.b.c", "Bearer x.y.z"] {
        let reply = api.session(malformed).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

/// Decision 2: the signature must come from the key with the token's `kid`.
#[tokio::test]
async fn rejects_a_token_signed_with_another_key() {
    let api = TestApi::start().await;

    let reply = api.session(&sign(&IMPOSTOR, &claims(SETUP_EMAIL))).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

/// Decision 2: a token without a `kid` matches no key.
#[tokio::test]
async fn rejects_a_token_without_a_kid() {
    let api = TestApi::start().await;
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(&header, &claims(SETUP_EMAIL), &KEY_A.encoding).unwrap();

    let reply = api.session(&token).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

/// Decision 2: RS256 only. An HMAC token keyed with something public, and an unsigned
/// token, are both rejected.
#[tokio::test]
async fn accepts_rs256_only() {
    let api = TestApi::start().await;
    let claims = claims(SETUP_EMAIL);

    let mut hmac = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256);
    hmac.kid = Some(KEY_A.kid.into());
    let secret = jsonwebtoken::EncodingKey::from_secret(&certs(&[&KEY_A])).unwrap();
    let hmac_token = jsonwebtoken::encode(&hmac, &claims, &secret).unwrap();

    // `{"alg":"none","kid":"key-a"}`, the claims of a valid token, no signature.
    let valid = sign(&KEY_A, &claims);
    let payload = valid.split('.').nth(1).unwrap();
    let unsigned = format!("eyJhbGciOiJub25lIiwia2lkIjoia2V5LWEifQ.{payload}.");

    for token in [hmac_token, unsigned] {
        let reply = api.session(&token).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

/// Decision 2: `aud` must contain the application's AUD tag.
#[tokio::test]
async fn rejects_a_token_for_another_application() {
    let api = TestApi::start().await;
    let mut claims = claims(SETUP_EMAIL);
    claims.insert("aud".into(), json!(["another-application"]));

    let reply = api.session(&sign(&KEY_A, &claims)).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
}

/// Decision 2: `iss` must be the team domain.
#[tokio::test]
async fn rejects_a_token_from_another_team() {
    let api = TestApi::start().await;

    for issuer in ["https://another-team.cloudflareaccess.com", TEAM_DOMAIN] {
        let mut claims = claims(SETUP_EMAIL);
        claims.insert("iss".into(), json!(issuer));
        let reply = api.session(&sign(&KEY_A, &claims)).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

/// Decision 2: all four claims are required.
#[tokio::test]
async fn requires_aud_iss_exp_and_nbf() {
    let api = TestApi::start().await;

    for claim in ["aud", "iss", "exp", "nbf"] {
        let mut claims = claims(SETUP_EMAIL);
        claims.remove(claim);
        let reply = api.session(&sign(&KEY_A, &claims)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "without {claim}");
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

/// Decision 2: beyond the 60 seconds of tolerance, `exp` and `nbf` reject.
#[tokio::test]
async fn rejects_an_expired_or_not_yet_valid_token() {
    let api = TestApi::start().await;

    let mut expired = claims(SETUP_EMAIL);
    expired.insert("exp".into(), json!(now() - 120));
    let mut not_yet_valid = claims(SETUP_EMAIL);
    not_yet_valid.insert("nbf".into(), json!(now() + 120));

    for claims in [expired, not_yet_valid] {
        let reply = api.session(&sign(&KEY_A, &claims)).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
}

/// The response says nothing that identifies the token or its email.
#[tokio::test]
async fn a_rejection_does_not_echo_the_token_or_the_email() {
    let api = TestApi::start().await;
    let token = sign(&IMPOSTOR, &claims(SETUP_EMAIL));

    let reply = api.session(&token).await;

    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    assert!(!reply.body.contains(&token));
    assert!(!reply.body.contains(SETUP_EMAIL));
}

// --- Valid token, denied: 403 -----------------------------------------------------

/// S-014: a valid token for an email that is not a login identity is denied.
/// Decision 9: while no household exists, everyone but the setup email.
#[tokio::test]
async fn denies_any_email_but_the_setup_email_before_first_setup() {
    let api = TestApi::start().await;

    let reply = api.session(&token(OTHER_EMAIL)).await;

    reply.assert_problem(StatusCode::FORBIDDEN, problem::ACCESS_DENIED);
    assert!(!reply.body.contains(OTHER_EMAIL));
}

/// Decision 7: a token without an email claim is denied, whatever kind of token.
#[tokio::test]
async fn denies_a_token_without_an_email() {
    let api = TestApi::start().await;
    let mut claims = claims(SETUP_EMAIL);
    claims.remove("email");
    claims.insert("common_name".into(), json!("service-token-client-id"));

    let reply = api.session(&sign(&KEY_A, &claims)).await;

    reply.assert_problem(StatusCode::FORBIDDEN, problem::ACCESS_DENIED);
}

// --- Keys -------------------------------------------------------------------------

/// Decision 6: without a valid key nothing is served, not even a valid token.
#[tokio::test]
async fn fails_closed_without_keys() {
    let api = TestApi::start_with(settings(), FakeKeySource::failing()).await;

    let reply = api.session(&token(SETUP_EMAIL)).await;

    reply.assert_problem(
        StatusCode::SERVICE_UNAVAILABLE,
        problem::ACCESS_KEYS_UNAVAILABLE,
    );
}

/// Decision 6: a certs response without a usable RS256 key counts as no keys.
#[tokio::test]
async fn fails_closed_on_an_unusable_certs_response() {
    for body in [b"not json".to_vec(), br#"{"keys":[]}"#.to_vec()] {
        let api = TestApi::start_with(settings(), FakeKeySource::serving(body)).await;
        let reply = api.session(&token(SETUP_EMAIL)).await;
        reply.assert_problem(
            StatusCode::SERVICE_UNAVAILABLE,
            problem::ACCESS_KEYS_UNAVAILABLE,
        );
    }
}

/// Decision 5: once the certs endpoint answers again, requests are served again,
/// fetching at most once a minute.
#[tokio::test]
async fn recovers_once_keys_can_be_fetched() {
    let api = TestApi::start_with(settings(), FakeKeySource::failing()).await;
    api.source.serve(certs(&[&KEY_A]));

    api.clock.advance(Duration::from_secs(30));
    let reply = api.session(&token(SETUP_EMAIL)).await;
    reply.assert_problem(
        StatusCode::SERVICE_UNAVAILABLE,
        problem::ACCESS_KEYS_UNAVAILABLE,
    );
    assert_eq!(
        api.source.fetches(),
        1,
        "within a minute of the startup fetch"
    );

    api.clock.advance(Duration::from_secs(31));
    let reply = api.session(&token(SETUP_EMAIL)).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(api.source.fetches(), 2);
}

/// Decision 5: a token with an unknown `kid` fetches the keys again, so a rotated key
/// is accepted without waiting for the hourly refresh.
#[tokio::test]
async fn fetches_the_keys_again_for_an_unknown_kid() {
    let api = TestApi::start().await;
    api.source.serve(certs(&[&KEY_A, &KEY_B]));
    api.clock.advance(Duration::from_secs(61));

    let reply = api.session(&sign(&KEY_B, &claims(SETUP_EMAIL))).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(api.source.fetches(), 2);
    let reply = api.session(&token(SETUP_EMAIL)).await;
    assert_eq!(reply.status, StatusCode::OK, "the previous key stays valid");
}

/// Decision 5: at most once a minute, however many tokens name unknown keys.
#[tokio::test]
async fn fetches_for_unknown_kids_at_most_once_a_minute() {
    let api = TestApi::start().await;
    api.source.serve(certs(&[&KEY_A, &KEY_B]));

    for _ in 0..3 {
        api.clock.advance(Duration::from_secs(10));
        let reply = api.session(&sign(&KEY_B, &claims(SETUP_EMAIL))).await;
        reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    }
    assert_eq!(api.source.fetches(), 1, "only the startup fetch");

    api.clock.advance(Duration::from_secs(31));
    let reply = api.session(&sign(&KEY_B, &claims(SETUP_EMAIL))).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    assert_eq!(api.source.fetches(), 2);
}

/// Decision 5: the hourly refresh replaces the keys; a key no longer served is no
/// longer accepted.
#[tokio::test]
async fn a_refresh_replaces_the_keys() {
    let api = TestApi::start().await;
    api.source.serve(certs(&[&KEY_B]));

    api.keys.refresh().await.unwrap();

    let reply = api.session(&token(SETUP_EMAIL)).await;
    reply.assert_problem(StatusCode::UNAUTHORIZED, problem::UNAUTHENTICATED);
    let reply = api.session(&sign(&KEY_B, &claims(SETUP_EMAIL))).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
}

/// A failed refresh keeps the keys held so far: they stay valid for days after a
/// rotation, and failing closed is for when there are none.
#[tokio::test]
async fn a_failed_refresh_keeps_the_keys() {
    let api = TestApi::start().await;
    api.source.fail();

    assert!(api.keys.refresh().await.is_err());

    let reply = api.session(&token(SETUP_EMAIL)).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
}

/// A certs response that cannot be used does not replace usable keys either.
#[tokio::test]
async fn an_unusable_refresh_keeps_the_keys() {
    let api = TestApi::start().await;
    api.source.serve(br#"{"keys":[]}"#.to_vec());

    assert!(api.keys.refresh().await.is_err());

    let reply = api.session(&token(SETUP_EMAIL)).await;
    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
}

// --- Cross-site requests ----------------------------------------------------------

fn post_session(origin: Option<&str>, content_type: &str) -> Request<Body> {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/api/session")
        .header(TOKEN_HEADER, token(SETUP_EMAIL))
        .header(header::CONTENT_TYPE, content_type);
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    request.body(Body::from("{}")).unwrap()
}

/// Decision 11: a request other than GET without an `Origin` header is rejected, even
/// with a valid token.
#[tokio::test]
async fn rejects_a_command_without_an_origin() {
    let api = TestApi::start().await;

    let reply = api.send(post_session(None, "application/json")).await;

    reply.assert_problem(StatusCode::FORBIDDEN, problem::CROSS_ORIGIN);
}

/// Decision 11: and with another site's origin.
#[tokio::test]
async fn rejects_a_command_from_another_origin() {
    let api = TestApi::start().await;

    for origin in ["https://evil.example", "http://iasion.example.com", "null"] {
        let reply = api
            .send(post_session(Some(origin), "application/json"))
            .await;
        reply.assert_problem(StatusCode::FORBIDDEN, problem::CROSS_ORIGIN);
    }
}

/// Decision 10: commands accept only JSON.
#[tokio::test]
async fn rejects_a_command_that_is_not_json() {
    let api = TestApi::start().await;

    for content_type in [
        "text/plain",
        "application/x-www-form-urlencoded",
        "multipart/form-data",
    ] {
        let reply = api
            .send(post_session(Some(PUBLIC_ORIGIN), content_type))
            .await;
        reply.assert_problem(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            problem::UNSUPPORTED_MEDIA_TYPE,
        );
    }
}

/// Decisions 10 and 11: from the app's own origin, with JSON, a command passes both
/// checks. `/api/session` has no commands, so routing answers.
#[tokio::test]
async fn passes_a_json_command_from_the_own_origin() {
    let api = TestApi::start().await;

    for content_type in ["application/json", "application/json; charset=utf-8"] {
        let reply = api
            .send(post_session(Some(PUBLIC_ORIGIN), content_type))
            .await;
        assert_eq!(
            reply.status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{content_type}"
        );
    }
}

/// Decision 10: no CORS headers that would let another origin read a response.
#[tokio::test]
async fn sends_no_cors_headers() {
    let api = TestApi::start().await;

    let get = get_session()
        .header(TOKEN_HEADER, token(SETUP_EMAIL))
        .header(header::ORIGIN, "https://evil.example")
        .body(Body::empty())
        .unwrap();
    let preflight = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/session")
        .header(header::ORIGIN, "https://evil.example")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();

    for request in [get, preflight] {
        let reply = api.send(request).await;
        let cors: Vec<_> = reply
            .headers
            .keys()
            .filter(|name| name.as_str().starts_with("access-control-"))
            .collect();
        assert!(cors.is_empty(), "CORS headers: {cors:?}");
    }
}

// --- Logs -------------------------------------------------------------------------

/// Decision 14, S-014: a denied request is logged with its reason and a request ID,
/// never with its email or token. The response carries the same request ID.
#[tokio::test]
async fn logs_a_denial_with_reason_and_request_id_but_no_email_or_token() {
    let api = TestApi::start().await;
    let (logs, _guard) = capture_logs();
    let rejected = sign(&IMPOSTOR, &claims(SETUP_EMAIL));
    let denied = token(OTHER_EMAIL);

    for token in [&rejected, &denied] {
        let reply = api.session(token).await;
        let request_id = reply
            .headers
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        let text = logs.text();
        let line = text
            .lines()
            .find(|line| line.contains(request_id))
            .unwrap_or_else(|| panic!("no log line with request ID {request_id}:\n{text}"));
        assert!(line.contains("reason="), "no reason: {line}");
    }

    let text = logs.text();
    for secret in [rejected.as_str(), denied.as_str(), SETUP_EMAIL, OTHER_EMAIL] {
        assert!(!text.contains(secret), "logged an email or token:\n{text}");
    }
}

/// Decision 14: a served request logs no email or token either.
#[tokio::test]
async fn logs_no_email_or_token_when_serving() {
    let api = TestApi::start().await;
    let (logs, _guard) = capture_logs();
    let token = token(SETUP_EMAIL);

    let reply = api.session(&token).await;

    assert_eq!(reply.status, StatusCode::OK, "body: {}", reply.body);
    let text = logs.text();
    assert!(
        !text.contains(&token) && !text.contains(SETUP_EMAIL),
        "logged:\n{text}"
    );
}

/// Every response carries a request ID, served or not, and each is distinct.
#[tokio::test]
async fn every_response_has_its_own_request_id() {
    let api = TestApi::start().await;

    let served = api.session(&token(SETUP_EMAIL)).await;
    let rejected = api.send(get_session().body(Body::empty()).unwrap()).await;

    let served = served.headers.get(REQUEST_ID_HEADER).unwrap();
    let rejected = rejected.headers.get(REQUEST_ID_HEADER).unwrap();
    assert_ne!(served, rejected);
}
