use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tracing::{debug, error, info};

// Import Theater types
use theater::config::{HandlerConfig, ManifestConfig, RuntimeHostConfig};

use crate::templates::templates;
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
                "A Theater actor created from the {} template.",
                template_name
            )),
            long_description: None,
            init_state: None,
            handlers: vec![HandlerConfig::Runtime(RuntimeHostConfig {})],
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

        // Create a basic lib.rs file based on the template
        let lib_rs_content = match template_name {
            "basic" => templates::BASIC_LIB_RS,
            _ => return Err(anyhow!("Unknown template: {}", template_name)),
        };

        let lib_rs_content = lib_rs_content.replace("{{actor_name}}", name);
        fs::write(path.join("src").join("lib.rs"), lib_rs_content)?;

        // Create the WIT world based on the template
        let wit_content = match template_name {
            "basic" => templates::BASIC_WIT,
            _ => return Err(anyhow!("Unknown template: {}", template_name)),
        };

        let wit_content = wit_content.replace("{{actor_name}}", name);
        fs::write(path.join("wit").join("world.wit"), wit_content)?;

        // copy the files from /Users/colinrozzi/work/theater/wit to the actor's wit directory
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

        // Create a README.md
        let readme_content = format!(
            "# {}\n\nA Theater actor created from the {} template.\n\n## Building\n\nTo build the actor:\n\n```bash\ncargo build --target wasm32-unknown-unknown --release\n```\n\n## Running\n\nTo run the actor with Theater:\n\n```bash\ntheater start manifest.toml\n```\n",
            name, template_name
        );
        fs::write(path.join("README.md"), readme_content)?;

        // Create a simple flake.nix
        let flake_nix_content = templates::FLAKE_NIX.replace("{{actor_name}}", name);
        fs::write(path.join("flake.nix"), flake_nix_content)?;

        info!("Actor '{}' created at {:?}", name, path);

        // Return the created actor
        Self::from_path(path)
    }

    pub fn build(&self) -> Result<()> {
        debug!("Building actor '{}'", self.name);

        // Create build_info directory if it doesn't exist
        let build_info_dir = self.path.join(".build_info");
        if !build_info_dir.exists() {
            fs::create_dir_all(&build_info_dir).expect("Failed to create build_info directory");
        }

        // Create log file
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let log_file = build_info_dir.join(format!("build_{}.log", timestamp));
        let log_file_path = log_file.to_string_lossy().to_string();

        // Start timing the build
        let build_start = std::time::Instant::now();

        // Update status to building
        let status_file = build_info_dir.join("status");
        fs::write(&status_file, "BUILDING").expect("Failed to write status file");

        // Execute nix build
        let output = match Command::new("/nix/var/nix/profiles/default/bin/nix")
            .args(["build", "--no-link", "--print-out-paths"])
            .current_dir(&self.path)
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                error!("Failed to execute nix build command: {}", e);

                // Create a failure log
                let mut log_content = format!("=== Build Log for {} ===\n", self.name);
                log_content.push_str(&format!("Date: {}\n", timestamp));
                log_content.push_str("Builder: nix\n");
                log_content.push_str(&format!(
                    "Duration: {} seconds\n\n",
                    build_start.elapsed().as_secs()
                ));
                log_content.push_str(&format!(
                    "ERROR: Failed to execute nix build command: {}\n",
                    e
                ));

                // Write the log file
                if let Err(write_err) = fs::write(&log_file, log_content) {
                    error!("Failed to write build log: {}", write_err);
                }

                // Update status
                if let Err(status_err) = fs::write(&status_file, "FAILED") {
                    error!("Failed to update status file: {}", status_err);
                }

                // Create simplified build_info
                let build_info = BuildInfo {
                    last_build_time: Some(SystemTime::now()),
                    build_status: BuildStatus::Failed,
                    component_hash: None,
                    build_log: Some(log_file_path),
                    build_duration: Some(build_start.elapsed().as_secs()),
                    component_size: None,
                    error_message: Some(format!("Failed to execute nix build command: {}", e)),
                };

                // Write build_info
                if let Ok(build_info_json) = serde_json::to_string_pretty(&build_info) {
                    let _ = fs::write(build_info_dir.join("build_info.json"), build_info_json);
                }

                return Err(anyhow!("Failed to execute nix build command: {}", e));
            }
        };

        // Function to update build info
        let update_build_info = |status: BuildStatus, wasm_path: Option<&str>| -> Result<()> {
            let build_duration = build_start.elapsed().as_secs();

            // Calculate component hash and size if available
            let (component_hash, component_size) = if let Some(path) = wasm_path {
                if Path::new(path).exists() {
                    match (utils::calculate_file_hash(path), utils::get_file_size(path)) {
                        (Ok(hash), Ok(size)) => (Some(hash), Some(size)),
                        _ => (None, None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            // Extract error message from stderr if build failed
            let error_message = if status == BuildStatus::Failed {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.is_empty() {
                    None
                } else {
                    // Extract a concise error message - first line that contains "error:"
                    stderr
                        .lines()
                        .find(|line| line.contains("error:"))
                        .map(|line| line.trim().to_string())
                }
            } else {
                None
            };

            let component_hash_clone = component_hash.clone();
            let build_info = BuildInfo {
                last_build_time: Some(SystemTime::now()),
                build_status: status,
                component_hash,
                build_log: Some(log_file_path.clone()),
                build_duration: Some(build_duration),
                component_size,
                error_message,
            };

            // Write output to log file
            let mut log_content = format!("=== Build Log for {} ===\n", self.name);
            log_content.push_str(&format!("Date: {}\n", timestamp));
            log_content.push_str(&format!("Builder: nix\n"));
            log_content.push_str(&format!("Duration: {} seconds\n\n", build_duration));

            log_content.push_str("=== STDOUT ===\n");
            log_content.push_str(&String::from_utf8_lossy(&output.stdout));

            log_content.push_str("\n=== STDERR ===\n");
            log_content.push_str(&String::from_utf8_lossy(&output.stderr));

            log_content.push_str(&format!("\n=== Exit Status: {} ===\n", output.status));

            if let Some(hash) = &component_hash_clone {
                log_content.push_str(&format!("\n=== Component Hash: {} ===\n", hash));
            }

            if let Some(size) = component_size {
                log_content.push_str(&format!("\n=== Component Size: {} bytes ===\n", size));
            }

            fs::write(&log_file, log_content)?;

            // Write build_info to JSON file
            let build_info_json = serde_json::to_string_pretty(&build_info)?;
            fs::write(build_info_dir.join("build_info.json"), build_info_json)?;

            Ok(())
        };

        // Check build status
        if !output.status.success() {
            error!("Nix build failed with status: {}", output.status);
            if let Err(e) = update_build_info(BuildStatus::Failed, None) {
                error!("Failed to update build info: {}", e);
            }
            if let Err(e) = fs::write(&status_file, "FAILED") {
                error!("Failed to write status file: {}", e);
            }
            return Err(anyhow!("Nix build failed with status: {}", output.status));
        }

        // Get the output path from stdout
        let nix_store_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if nix_store_path.is_empty() {
            error!("Failed to determine nix store path");
            if let Err(e) = update_build_info(BuildStatus::Failed, None) {
                error!("Failed to update build info: {}", e);
            }
            if let Err(e) = fs::write(&status_file, "FAILED") {
                error!("Failed to write status file: {}", e);
            }
            return Err(anyhow!("Failed to determine nix store path"));
        }

        // Construct the WASM file path
        // The filename in the nix store will match the actor name (with hyphens)
        // as we now handle the transformation in the flake.nix template
        let wasm_file_name = format!("{}.wasm", self.name);
        let wasm_path = format!("{}/lib/{}", nix_store_path, wasm_file_name);

        // Check if the WASM file exists
        if !Path::new(&wasm_path).exists() {
            error!("Built WASM file not found at expected path: {}", wasm_path);
            if let Err(e) = update_build_info(BuildStatus::Failed, Some(&wasm_path)) {
                error!("Failed to update build info: {}", e);
            }
            if let Err(e) = fs::write(&status_file, "FAILED") {
                error!("Failed to write status file: {}", e);
            }
            return Err(anyhow!(
                "Built WASM file not found at expected path: {}",
                wasm_path
            ));
        }

        // Update the manifest.toml with the new component path
        if let Some(mut manifest) = self.manifest.clone() {
            manifest.component_path = wasm_path.clone();

            match toml::to_string(&manifest) {
                Ok(manifest_content) => {
                    if let Err(e) = fs::write(self.path.join("manifest.toml"), manifest_content) {
                        error!("Failed to write manifest.toml: {}", e);
                        // Continue anyway to ensure we record build success
                    } else {
                        info!("Updated manifest.toml with new component path");
                    }
                }
                Err(e) => {
                    error!("Failed to serialize manifest: {}", e);
                    // Continue anyway to ensure we record build success
                }
            }
        }

        // Ensure we still write build info and status even if there are errors
        if let Err(e) = update_build_info(BuildStatus::Success, Some(&wasm_path)) {
            error!("Failed to update build info: {}", e);
        }

        if let Err(e) = fs::write(&status_file, "SUCCESS") {
            error!("Failed to write status file: {}", e);
        }

        info!("Build completed successfully");
        return Ok(());
    }
}
