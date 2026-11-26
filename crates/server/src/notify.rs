//! Tells systemd the app is ready, so that a failed start is detected and Podman rolls
//! back (ADR-0040, decision 9). The protocol is one datagram to `NOTIFY_SOCKET`.

use std::env;
use std::os::unix::net::UnixDatagram;

/// Sends `READY=1` if systemd (or Podman on its behalf) asked for notifications.
pub fn ready() -> Result<(), String> {
    let Some(path) = env::var_os("NOTIFY_SOCKET") else {
        return Ok(());
    };
    let socket =
        UnixDatagram::unbound().map_err(|error| format!("cannot notify systemd: {error}"))?;
    let path = path.to_string_lossy().into_owned();
    let sent = match path.strip_prefix('@') {
        Some(name) => send_abstract(&socket, name),
        None => socket.send_to(b"READY=1", &path).map(|_| ()),
    };
    sent.map_err(|error| format!("cannot notify systemd: {error}"))
}

#[cfg(target_os = "linux")]
fn send_abstract(socket: &UnixDatagram, name: &str) -> std::io::Result<()> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;
    let address = SocketAddr::from_abstract_name(name.as_bytes())?;
    socket.send_to_addr(b"READY=1", &address).map(|_| ())
}

#[cfg(not(target_os = "linux"))]
fn send_abstract(_socket: &UnixDatagram, _name: &str) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "abstract sockets exist on Linux only",
    ))
}
