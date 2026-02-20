use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AppTheme {
    #[default]
    BreezeDark,
    BreezeLight,
    AdwaitaDark,
    AdwaitaLight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: AppTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBindConfig {
    pub proxy: ProxyConfig,
    pub routes: Vec<RouteConfig>,
    pub ui: UIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_port: u16,
    pub use_mkcert: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub domain: String,
    pub port: u16,
}

impl Default for DevBindConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                listen_port: 443,
                use_mkcert: true,
            },
            routes: Vec::new(),
            ui: UIConfig {
                theme: AppTheme::BreezeDark,
            },
        }
    }
}

impl DevBindConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
