use std::path::PathBuf;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn socket_path() -> PathBuf {
    PathBuf::from("/var/run/gnosis-vpn.sock")
}

// #[cfg(windows)]
// pub fn socket_path() -> PathBuf {
// PathBuf::from("//./pipe/Gnosis VPN")
// }
