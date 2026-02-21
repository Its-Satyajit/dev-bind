use std::path::PathBuf;

pub fn get_config_path() -> PathBuf {
    let mut path = if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        PathBuf::from(format!("/home/{}/.config", sudo_user))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
    };
    path.push("devbind");
    path.push("config.toml");
    path
}

pub fn get_config_dir() -> PathBuf {
    let mut p = get_config_path();
    p.pop();
    p
}

/// Resolve the installed devbind binary path (prefers ~/.local/bin, falls back to PATH).
pub fn which_devbind() -> String {
    if let Some(p) = dirs::home_dir()
        .map(|h| h.join(".local/bin/devbind"))
        .filter(|p| p.exists())
    {
        return p.to_string_lossy().into_owned();
    }
    "devbind".to_string()
}

/// Check whether the devbind proxy is actually listening on port 443.
pub fn is_proxy_running() -> bool {
    std::net::TcpStream::connect("127.0.0.1:443").is_ok()
}

#[cfg(test)]
#[path = "utils_tests.rs"]
mod tests;
