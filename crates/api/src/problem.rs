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

/// 404: no API route has this path.
pub const NOT_FOUND: &str = "/problems/not-found";

/// 503: the installation's status could not be read from the database.
pub const DATABASE_UNAVAILABLE: &str = "/problems/database-unavailable";

/// A problem response's body (RFC 9457). Nothing in it comes from the request.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub(crate) struct Problem {
    #[serde(rename = "type")]
    #[schema(rename = "type")]
    problem_type: ProblemType,
    /// A short summary, the same for every occurrence of the type.
    title: String,
    /// The HTTP status code.
    status: u16,
}

/// The typed errors, each a relative URI under `/problems/` (ADR-0036, decision 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub enum ProblemType {
    #[serde(rename = "/problems/unauthenticated")]
    Unauthenticated,
    #[serde(rename = "/problems/access-denied")]
    AccessDenied,
    #[serde(rename = "/problems/cross-origin")]
    CrossOrigin,
    #[serde(rename = "/problems/unsupported-media-type")]
    UnsupportedMediaType,
    #[serde(rename = "/problems/access-keys-unavailable")]
    AccessKeysUnavailable,
    #[serde(rename = "/problems/database-unavailable")]
    DatabaseUnavailable,
    #[serde(rename = "/problems/not-found")]
    NotFound,
}

impl ProblemType {
    /// Every type, for the check that each matches its constant.
    pub const ALL: [ProblemType; 7] = [
        Self::Unauthenticated,
        Self::AccessDenied,
        Self::CrossOrigin,
        Self::UnsupportedMediaType,
        Self::AccessKeysUnavailable,
        Self::DatabaseUnavailable,
        Self::NotFound,
    ];

    /// The type's URI, as its constant above.
    pub fn uri(self) -> &'static str {
        match self {
            Self::Unauthenticated => UNAUTHENTICATED,
            Self::AccessDenied => ACCESS_DENIED,
            Self::CrossOrigin => CROSS_ORIGIN,
            Self::UnsupportedMediaType => UNSUPPORTED_MEDIA_TYPE,
            Self::AccessKeysUnavailable => ACCESS_KEYS_UNAVAILABLE,
            Self::DatabaseUnavailable => DATABASE_UNAVAILABLE,
            Self::NotFound => NOT_FOUND,
        }
    }
}

/// A problem response with its `type`, `title` and `status`, and nothing that comes
/// from the request.
pub(crate) fn response(
    status: axum::http::StatusCode,
    problem_type: ProblemType,
    title: &'static str,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    let body = Problem {
        problem_type,
        title: title.to_string(),
        status: status.as_u16(),
    };
    let body = serde_json::to_string(&body).unwrap_or_default();
    (
        status,
        [(axum::http::header::CONTENT_TYPE, CONTENT_TYPE)],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_type_serializes_as_its_uri() {
        for problem_type in ProblemType::ALL {
            assert_eq!(
                serde_json::to_value(problem_type).unwrap(),
                problem_type.uri()
            );
        }
    }
}
