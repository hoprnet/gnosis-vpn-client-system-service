use std::path::PathBuf;

#[cfg(target_family = "unix")]
pub fn socket_path() -> PathBuf {
    PathBuf::from("/var/run/gnosis-vpn.sock")
}

// #[cfg(target_family = "windows")]
// pub fn socket_path() -> PathBuf {
// PathBuf::from("//./pipe/Gnosis VPN")
// }
