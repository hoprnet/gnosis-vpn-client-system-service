use std::path;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn socket_path() -> path::Path {
    Path::new("/var/run/gnosis-vpn.sock");
}

#[cfg(windows)]
pub fn socket_path() -> path::Path {
    Path::new("//./pipe/Mullvad VPN")
}
