use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub default_template: String,
    pub default_interfaces: Vec<String>,
    pub build_cache_enabled: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            default_template: "basic".to_string(),
            default_interfaces: vec!["ntwk:theater/actor".to_string()],
            build_cache_enabled: true,
        }
    }
}

impl RegistryConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path.as_ref()))?;
        
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {:?}", path.as_ref()))?;
        
        Ok(config)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path.as_ref()))?;
        
        Ok(())
    }
}