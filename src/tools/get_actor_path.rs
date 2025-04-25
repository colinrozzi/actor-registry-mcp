use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error};
use anyhow::{Result, anyhow};
use std::path::PathBuf;

use crate::registry::Registry;

pub fn register_get_actor_path_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "get-actor-path",
        Some("Retrieves the path to an actor or specific actor files"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the actor (required)"
                },
                "file": {
                    "type": "string",
                    "description": "Specific file to locate (optional: manifest.toml, component.wasm, etc.)"
                },
                "absolute": {
                    "type": "boolean",
                    "description": "Return absolute path (optional, defaults to relative)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            let file = args.get("file").and_then(|v| v.as_str());
            let absolute = args.get("absolute").and_then(|v| v.as_bool()).unwrap_or(true);
            
            debug!("Getting path for actor '{}', file: '{:?}', absolute: {}", 
                   name, file, absolute);
            
            match registry.find_actor(name) {
                Ok(actor) => {
                    let base_path = actor.path.clone();
                    
                    // Determine the target path
                    let target_path = if let Some(specific_file) = file {
                        match specific_file {
                            "manifest.toml" => base_path.join("manifest.toml"),
                            "cargo.toml" => base_path.join("Cargo.toml"),
                            "component.wasm" => {
                                // Try to get from manifest
                                if let Some(ref manifest) = actor.manifest {
                                    if let Some(ref component_path) = manifest.component_path {
                                        PathBuf::from(component_path)
                                    } else {
                                        base_path.join("target").join("wasm32-unknown-unknown").join("release")
                                            .join(format!("{}.wasm", actor.name.replace("-", "_")))
                                    }
                                } else {
                                    base_path.join("target").join("wasm32-unknown-unknown").join("release")
                                        .join(format!("{}.wasm", actor.name.replace("-", "_")))
                                }
                            },
                            "lib.rs" => base_path.join("src").join("lib.rs"),
                            "readme.md" => base_path.join("README.md"),
                            "flake.nix" => base_path.join("flake.nix"),
                            _ => base_path.join(specific_file),
                        }
                    } else {
                        base_path
                    };
                    
                    // Format the path string
                    let path_str = if absolute {
                        target_path.to_string_lossy().to_string()
                    } else {
                        // Try to make relative to registry path
                        if let Ok(rel_path) = target_path.strip_prefix(registry.path()) {
                            rel_path.to_string_lossy().to_string()
                        } else {
                            target_path.to_string_lossy().to_string()
                        }
                    };
                    
                    // Check if the path exists
                    let exists = target_path.exists();
                    
                    let content = vec![
                        ToolContent::Text {
                            text: if exists {
                                path_str.clone()
                            } else {
                                format!("Path does not exist: {}", path_str)
                            }
                        },
                        ToolContent::Json {
                            json: json!({
                                "path": path_str,
                                "exists": exists,
                                "is_file": target_path.is_file(),
                                "is_dir": target_path.is_dir(),
                            })
                        }
                    ];
                    
                    Ok(ToolCallResult {
                        content,
                        is_error: Some(!exists)
                    })
                },
                Err(e) => {
                    error!("Failed to get actor path: {}", e);
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Failed to get actor path: {}", e)
                        }
                    ];
                    
                    Ok(ToolCallResult {
                        content,
                        is_error: Some(true)
                    })
                }
            }
        }
    )
}