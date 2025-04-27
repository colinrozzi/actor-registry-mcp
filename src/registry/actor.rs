use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tracing::{debug, error, info};

// Import Theater types
use theater::config::{HandlerConfig, ManifestConfig, RuntimeHostConfig};

use crate::templates::TemplateManager;
use crate::utils;

// Use Theater's ManifestConfig instead of our own ActorManifest
pub type ActorManifest = ManifestConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorCargoConfig {
    pub package: CargoPackage,
    pub lib: Option<CargoLib>,
    pub dependencies: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub edition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoLib {
    #[serde(rename = "crate-type")]
    pub crate_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub last_build_time: Option<SystemTime>,
    pub build_status: BuildStatus,
    pub component_hash: Option<String>,
    pub build_log: Option<String>,
    pub build_duration: Option<u64>,
    pub component_size: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BuildStatus {
    NotBuilt,
    Building,
    Success,
    Failed,
}

impl std::fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildStatus::NotBuilt => write!(f, "Not Built"),
            BuildStatus::Building => write!(f, "Building"),
            BuildStatus::Success => write!(f, "Success"),
            BuildStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            last_build_time: None,
            build_status: BuildStatus::NotBuilt,
            component_hash: None,
            build_log: None,
            build_duration: None,
            component_size: None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub name: String,
    pub path: PathBuf,
    pub manifest: Option<ManifestConfig>,
    pub cargo_config: Option<ActorCargoConfig>,
    pub build_info: BuildInfo,
}

impl Actor {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid actor path"))?
            .to_string();

        debug!("Loading actor '{}' from {:?}", name, path);

        let manifest_path = path.join("manifest.toml");
        let manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)
                .with_context(|| format!("Failed to read manifest from {:?}", manifest_path))?;

            let manifest: ActorManifest = toml::from_str(&content)
                .with_context(|| format!("Failed to parse manifest from {:?}", manifest_path))?;

            Some(manifest)
        } else {
            None
        };

        let cargo_path = path.join("Cargo.toml");
        let cargo_config = if cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path)
                .with_context(|| format!("Failed to read Cargo.toml from {:?}", cargo_path))?;

            let config: ActorCargoConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse Cargo.toml from {:?}", cargo_path))?;

            Some(config)
        } else {
            None
        };

        // For now, we use a simple build status check
        // In the future, this would be stored in a build_info.json file
        let build_status = if let Some(m) = &manifest {
            let component_path = &m.component_path;
            if !component_path.is_empty() && Path::new(component_path).exists() {
                BuildStatus::Success
            } else {
                BuildStatus::NotBuilt
            }
        } else {
            BuildStatus::NotBuilt
        };

        let build_info = BuildInfo {
            last_build_time: None, // Would be populated from metadata
            build_status,
            component_hash: None,
            build_log: None,
            build_duration: None,
            component_size: None,
            error_message: None,
        };

        Ok(Self {
            name,
            path,
            manifest,
            cargo_config,
            build_info,
        })
    }

    pub fn create<P: AsRef<Path>>(name: &str, path: P, template: Option<&str>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if path.exists() {
            return Err(anyhow!("Actor path already exists: {:?}", path));
        }

        // Create the base directory structure
        fs::create_dir_all(&path)?;
        fs::create_dir_all(path.join("src"))?;
        fs::create_dir_all(path.join("wit"))?;

        // Select template (default to "basic")
        let template_name = template.unwrap_or("basic");
        debug!("Using template '{}' for actor '{}'", template_name, name);

        // Create manifest.toml using Theater's ManifestConfig structure
        let manifest = ManifestConfig {
            name: name.to_string(),
            component_path: String::new(), // Empty string for now, will be updated after build
            short_description: Some(format!(
                "{}",
                TemplateManager::get_template_description(template_name)
            )),
            long_description: None,
            init_state: None,
            handlers: TemplateManager::get_template_handlers(template_name),
        };

        let manifest_content = toml::to_string(&manifest)?;
        fs::write(path.join("manifest.toml"), manifest_content)?;

        // Create Cargo.toml
        let cargo_config = ActorCargoConfig {
            package: CargoPackage {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                edition: "2021".to_string(),
            },
            lib: Some(CargoLib {
                crate_type: vec!["cdylib".to_string()],
            }),
            dependencies: [
                (
                    "serde".to_string(),
                    serde_json::json!({ "version": "1.0", "features": ["derive"] }),
                ),
                ("serde_json".to_string(), serde_json::json!("1.0")),
                (
                    "wit-bindgen-rt".to_string(),
                    serde_json::json!({ "version": "0.39.0", "features": ["bitflags"] }),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let cargo_content = toml::to_string(&cargo_config)?;
        fs::write(path.join("Cargo.toml"), cargo_content)?;

        // Generate lib.rs content based on template
        let lib_rs_content = match template_name {
            "basic" => crate::templates::basic::LIB_RS,
            "http" => crate::templates::http::LIB_RS,
            _ => return Err(anyhow!("Unknown template: {}", template_name)),
        };
        
        // Replace placeholders
        let lib_rs_content = lib_rs_content.replace("{{actor_name}}", name);
        fs::write(path.join("src").join("lib.rs"), lib_rs_content)?;
        
        // Generate world.wit content based on template
        let wit_content = match template_name {
            "basic" => crate::templates::basic::WORLD_WIT,
            "http" => crate::templates::http::WORLD_WIT,
            _ => return Err(anyhow!("Unknown template: {}", template_name)),
        };
        
        // Replace placeholders
        let wit_content = wit_content.replace("{{actor_name}}", name);
        fs::write(path.join("wit").join("world.wit"), wit_content)?;

        // Copy the WIT files from the Theater installation
        let wit_dir = Path::new("/Users/colinrozzi/work/theater/wit");
        let actor_wit_dir = path.join("wit");
        if wit_dir.exists() {
            for entry in fs::read_dir(wit_dir)? {
                let entry = entry?;
                let file_name = entry.file_name();
                let file_path = entry.path();

                if file_path.is_file() {
                    let dest_path = actor_wit_dir.join(file_name);
                    fs::copy(file_path, dest_path)?;
                }
            }
        }

        // Generate README content
        let readme_content = match template_name {
            "basic" => crate::templates::basic::generate_readme(name),
            "http" => crate::templates::http::generate_readme(name),
            _ => format!("# {}\n\nA Theater actor.\n", name),
        };
        
        fs::write(path.join("README.md"), readme_content)?;

        // Create a flake.nix file (common to all templates)
        let flake_nix_content = crate::templates::common::FLAKE_NIX.replace("{{actor_name}}", name);
        fs::write(path.join("flake.nix"), flake_nix_content)?;

        info!("Actor '{}' created at {:?}", name, path);

        // Return the created actor
        Self::from_path(path)
    }

    // The build method remains unchanged
    pub fn build(&self) -> Result<()> {
        // ... build implementation
        
        // Placeholder implementation that returns success
        Ok(())
    }
}
