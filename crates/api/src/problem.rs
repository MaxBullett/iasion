//! Typed errors as RFC 9457 problem details (ADR-0036, decision 9). Each `type` is a
//! stable relative URI under `/problems/`.

/// Media type of a problem response.
pub const CONTENT_TYPE: &str = "application/problem+json";

/// 401: the request has no valid Access token. The response never says why; the log
/// does (ADR-0037, decision 14).
pub const UNAUTHENTICATED: &str = "/problems/unauthenticated";

/// 403: the token is valid, but its email may not use the app (ADR-0024, ADR-0037,
/// decisions 7 and 9).
pub const ACCESS_DENIED: &str = "/problems/access-denied";

/// 403: a request other than GET without the app's own `Origin` (ADR-0037,
/// decision 11).
pub const CROSS_ORIGIN: &str = "/problems/cross-origin";

/// 415: a request other than GET whose body is not JSON (ADR-0037, decision 10).
pub const UNSUPPORTED_MEDIA_TYPE: &str = "/problems/unsupported-media-type";

/// 503: no valid Access key is available, so nothing can be verified and nothing is
/// served (ADR-0037, decision 6).
pub const ACCESS_KEYS_UNAVAILABLE: &str = "/problems/access-keys-unavailable";
