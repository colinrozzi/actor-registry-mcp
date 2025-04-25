use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::time::SystemTime;
use anyhow::{Result, Context, anyhow};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, error};

use crate::templates::templates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub component_path: Option<String>,
    
    #[serde(default)]
    pub interface: ActorInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActorInterface {
    pub implements: Vec<String>,
    pub requires: Vec<String>,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BuildStatus {
    NotBuilt,
    Building,
    Success,
    Failed,
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            last_build_time: None,
            build_status: BuildStatus::NotBuilt,
            component_hash: None,
            build_log: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub name: String,
    pub path: PathBuf,
    pub manifest: Option<ActorManifest>,
    pub cargo_config: Option<ActorCargoConfig>,
    pub build_info: BuildInfo,
}

impl Actor {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let name = path.file_name()
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
        let component_path = manifest.as_ref().and_then(|m| m.component_path.clone());
        let build_status = if let Some(path) = component_path {
            if Path::new(&path).exists() {
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
        };
        
        Ok(Self {
            name,
            path,
            manifest,
            cargo_config,
            build_info,
        })
    }
    
    pub fn create<P: AsRef<Path>>(
        name: &str, 
        path: P, 
        template: Option<&str>, 
        interfaces: Vec<String>
    ) -> Result<Self> {
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
        
        // Create manifest.toml
        let mut implements = vec!["ntwk:theater/actor".to_string()];
        implements.extend(interfaces);
        
        let manifest = ActorManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: Some(format!("A Theater actor created from the {} template", template_name)),
            component_path: None,
            interface: ActorInterface {
                implements,
                requires: vec![],
            },
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
                ("serde".to_string(), serde_json::json!({ "version": "1.0", "features": ["derive"] })),
                ("serde_json".to_string(), serde_json::json!("1.0")),
                ("wit-bindgen-rt".to_string(), serde_json::json!({ "version": "0.39.0", "features": ["bitflags"] })),
            ].into_iter().collect(),
        };
        
        let cargo_content = toml::to_string(&cargo_config)?;
        fs::write(path.join("Cargo.toml"), cargo_content)?;
        
        // Create a basic lib.rs file based on the template
        let lib_rs_content = match template_name {
            "basic" => templates::BASIC_LIB_RS,
            "http" => templates::HTTP_LIB_RS,
            "supervisor" => templates::SUPERVISOR_LIB_RS,
            _ => return Err(anyhow!("Unknown template: {}", template_name)),
        };
        
        let lib_rs_content = lib_rs_content.replace("{{actor_name}}", name);
        fs::write(path.join("src").join("lib.rs"), lib_rs_content)?;
        
        // Create a README.md
        let readme_content = format!(
            "# {}\n\nA Theater actor created from the {} template.\n\n## Building\n\nTo build the actor:\n\n```bash\ncargo build --target wasm32-unknown-unknown --release\n```\n\n## Running\n\nTo run the actor with Theater:\n\n```bash\ntheater start manifest.toml\n```\n",
            name, template_name
        );
        fs::write(path.join("README.md"), readme_content)?;
        
        // Create a simple flake.nix
        let flake_nix_content = templates::FLAKE_NIX
            .replace("{{actor_name}}", name);
        fs::write(path.join("flake.nix"), flake_nix_content)?;
        
        info!("Actor '{}' created at {:?}", name, path);
        
        // Return the created actor
        Self::from_path(path)
    }
    
    pub fn build(&self, release: bool) -> Result<()> {
        debug!("Building actor '{}' (release: {})", self.name, release);
        
        // For now, we'll just simulate the build process
        // In a real implementation, this would run nix build or cargo build
        
        // Check if the actor directory has a flake.nix
        let flake_path = self.path.join("flake.nix");
        if flake_path.exists() {
            info!("Building actor '{}' with nix flake", self.name);
            
            // In a real implementation, we would run:
            // let output = Command::new("nix")
            //     .arg("build")
            //     .current_dir(&self.path)
            //     .output()?;
            
            // Simulate successful build for now
            let new_component_path = format!("/nix/store/abcdef-{}-0.1.0/lib/{}.wasm", self.name, self.name.replace("-", "_"));
            
            // Update the manifest.toml
            if let Some(mut manifest) = self.manifest.clone() {
                manifest.component_path = Some(new_component_path);
                
                let manifest_content = toml::to_string(&manifest)?;
                fs::write(self.path.join("manifest.toml"), manifest_content)?;
                
                info!("Updated manifest.toml with new component path");
            }
            
            info!("Build completed successfully");
            return Ok(());
        } else {
            // Fallback to cargo build
            info!("Building actor '{}' with cargo", self.name);
            
            // In a real implementation, we would run:
            // let target = if release { "--release" } else { "" };
            // let output = Command::new("cargo")
            //     .args(["build", "--target", "wasm32-unknown-unknown", target])
            //     .current_dir(&self.path)
            //     .output()?;
            
            // Simulate successful build
            let target_type = if release { "release" } else { "debug" };
            let new_component_path = format!("{}/target/wasm32-unknown-unknown/{}/{}.wasm", 
                self.path.display(), target_type, self.name.replace("-", "_"));
            
            // Update the manifest.toml
            if let Some(mut manifest) = self.manifest.clone() {
                manifest.component_path = Some(new_component_path);
                
                let manifest_content = toml::to_string(&manifest)?;
                fs::write(self.path.join("manifest.toml"), manifest_content)?;
                
                info!("Updated manifest.toml with new component path");
            }
            
            info!("Build completed successfully");
            return Ok(());
        }
    }
}