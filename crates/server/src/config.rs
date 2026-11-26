//! Settings from the environment (`.env.example` lists them all). Errors name the
//! variable, never its value.

use std::env;
use std::net::SocketAddr;

/// Listens for the app here unless `IASION_LISTEN_ADDR` says otherwise.
const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:8080";
/// Serves the health check here unless `IASION_HEALTH_ADDR` says otherwise: on the
/// loopback interface only, so never through the tunnel (ADR-0040, decision 16).
pub const DEFAULT_HEALTH_ADDR: &str = "127.0.0.1:8081";

pub struct Config {
    pub api: iasion_api::Settings,
    pub certs_url: String,
    pub listen_addr: SocketAddr,
    pub health_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let team_domain = required("IASION_ACCESS_TEAM_DOMAIN")?;
        let certs_url = optional("IASION_ACCESS_CERTS_URL")
            .unwrap_or_else(|| format!("https://{}/cdn-cgi/access/certs", team_domain.trim()));
        Ok(Self {
            api: iasion_api::Settings {
                access: iasion_api::access::Settings {
                    team_domain,
                    aud: required("IASION_ACCESS_AUD")?,
                },
                public_origin: required("IASION_PUBLIC_ORIGIN")?,
                setup_email: required("IASION_SETUP_EMAIL")?,
            },
            certs_url,
            listen_addr: address("IASION_LISTEN_ADDR", DEFAULT_LISTEN_ADDR)?,
            health_addr: health_addr()?,
        })
    }
}

pub fn health_addr() -> Result<SocketAddr, String> {
    address("IASION_HEALTH_ADDR", DEFAULT_HEALTH_ADDR)
}

/// A set, non-empty variable.
pub fn required(name: &str) -> Result<String, String> {
    optional(name).ok_or_else(|| format!("{name} is not set (see .env.example)"))
}

/// A variable that is set and not blank.
pub fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn address(name: &str, default: &str) -> Result<SocketAddr, String> {
    optional(name)
        .as_deref()
        .unwrap_or(default)
        .parse()
        .map_err(|_| format!("{name} is not an address such as {default}"))
}
