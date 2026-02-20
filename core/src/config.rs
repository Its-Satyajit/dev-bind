use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBindConfig {
    pub proxy: ProxyConfig,
    pub routes: Vec<RouteConfig>,
    pub ui: UIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    pub port_http: u16,
    pub port_https: u16,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            port_http: 80,
            port_https: 443,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RouteConfig {
    pub domain: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UIConfig {
    // Current UI configuration is system-synced (Light/Dark Breeze)
}

impl Default for DevBindConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                port_http: 80,
                port_https: 443,
            },
            routes: vec![],
            ui: UIConfig::default(),
        }
    }
}

impl DevBindConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse config from {:?}", path))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content).with_context(|| format!("Failed to write config to {:?}", path))
    }

    /// Add a route (used for ephemeral sessions — caller decides whether to persist).
    pub fn add_route(&mut self, domain: String, port: u16) {
        if let Some(route) = self.routes.iter_mut().find(|r| r.domain == domain) {
            route.port = port;
        } else {
            self.routes.push(RouteConfig { domain, port });
        }
    }

    /// Remove a route by domain name.
    pub fn remove_route(&mut self, domain: &str) {
        self.routes.retain(|r| r.domain != domain);
    }
}
