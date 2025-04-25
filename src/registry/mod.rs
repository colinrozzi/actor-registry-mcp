pub mod actor;
pub mod config;

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use self::actor::Actor;
use self::config::RegistryConfig;

#[derive(Clone)]
pub struct Registry {
    path: PathBuf,
    config: Arc<Mutex<RegistryConfig>>,
}

impl Registry {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(anyhow!("Registry path does not exist: {:?}", path));
        }

        if !path.is_dir() {
            return Err(anyhow!("Registry path is not a directory: {:?}", path));
        }

        // Load or create default config
        let config_path = path.join(".registry.config.toml");
        let config = if config_path.exists() {
            RegistryConfig::load(&config_path)?
        } else {
            info!("Creating default registry config at {:?}", config_path);
            let config = RegistryConfig::default();
            config.save(&config_path)?;
            config
        };

        Ok(Self {
            path,
            config: Arc::new(Mutex::new(config)),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn find_actor(&self, name: &str) -> Result<Actor> {
        let actor_path = self.path.join(name);

        if !actor_path.exists() {
            return Err(anyhow!("Actor '{}' not found in registry", name));
        }

        Actor::from_path(actor_path)
    }

    pub fn list_actors(&self) -> Result<Vec<Actor>> {
        let mut actors = Vec::new();

        for entry in WalkDir::new(&self.path).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let path = entry.path();

                // Skip directories that don't contain a manifest.toml
                let manifest_path = path.join("manifest.toml");
                if !manifest_path.exists() {
                    debug!("Skipping directory without manifest.toml: {:?}", path);
                    continue;
                }

                match Actor::from_path(path) {
                    Ok(actor) => actors.push(actor),
                    Err(e) => warn!("Failed to load actor from {}: {}", path.display(), e),
                }
            }
        }

        Ok(actors)
    }

    pub fn create_actor(&self, name: &str, template: Option<&str>) -> Result<Actor> {
        let actor_path = self.path.join(name);

        if actor_path.exists() {
            return Err(anyhow!(
                "Actor '{}' already exists at {:?}",
                name,
                actor_path
            ));
        }

        // Create the actor using the template system
        Actor::create(name, actor_path, template)
    }

    pub fn build_actor(&self, name: &str, release: bool) -> Result<()> {
        let actor = self.find_actor(name)?;
        actor.build(release)
    }

    pub fn get_templates(&self) -> Vec<String> {
        // For now, just return a static list of templates
        // In the future, this would scan a templates directory
        vec![
            "basic".to_string(),
            "http".to_string(),
            "supervisor".to_string(),
        ]
    }

    pub fn get_available_interfaces(&self) -> Vec<String> {
        // For now, return a static list of common interfaces
        // In the future, this would be loaded from WIT definitions
        vec![
            "ntwk:theater/actor".to_string(),
            "ntwk:theater/message-server-client".to_string(),
            "ntwk:theater/http-handlers".to_string(),
            "ntwk:theater/supervisor".to_string(),
        ]
    }
}

