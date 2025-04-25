use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error};
use anyhow::{Result, anyhow};
use std::fs;

use crate::registry::Registry;

pub fn register_get_actor_info_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "get-actor-info",
        Some("Provides detailed information about a specific actor"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the actor (required)"
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (optional: text, json)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("text");
            
            debug!("Getting info for actor '{}' with format '{}'", name, format);
            
            match registry.find_actor(name) {
                Ok(actor) => {
                    if format == "json" {
                        let mut actor_info = json!({
                            "name": actor.name,
                            "path": actor.path.to_string_lossy(),
                            "build_status": format!("{:?}", actor.build_info.build_status),
                        });
                        
                        if let Some(ref manifest) = actor.manifest {
                            actor_info["manifest"] = json!({
                                "name": manifest.name,
                                "version": manifest.version,
                                "description": manifest.description,
                                "component_path": manifest.component_path,
                                "interfaces": {
                                    "implements": manifest.interface.implements,
                                    "requires": manifest.interface.requires,
                                }
                            });
                        }
                        
                        if let Some(ref cargo) = actor.cargo_config {
                            actor_info["cargo"] = json!({
                                "name": cargo.package.name,
                                "version": cargo.package.version,
                                "edition": cargo.package.edition,
                                "dependencies": cargo.dependencies,
                            });
                        }
                        
                        // Read README if exists
                        let readme_path = actor.path.join("README.md");
                        if readme_path.exists() {
                            if let Ok(readme) = fs::read_to_string(readme_path) {
                                actor_info["readme"] = json!(readme);
                            }
                        }
                        
                        let content = vec![
                            ToolContent::Json {
                                json: actor_info
                            }
                        ];
                        
                        Ok(ToolCallResult {
                            content,
                            is_error: Some(false)
                        })
                    } else {
                        // Format as text
                        let mut text = format!("# Actor: {}\n\n", actor.name);
                        
                        if let Some(ref manifest) = actor.manifest {
                            text.push_str(&format!("Version: {}\n", manifest.version));
                            
                            if let Some(ref desc) = manifest.description {
                                text.push_str(&format!("Description: {}\n", desc));
                            }
                            
                            text.push_str("\n## Interfaces\n\n");
                            text.push_str("Implements:\n");
                            for interface in &manifest.interface.implements {
                                text.push_str(&format!("- {}\n", interface));
                            }
                            
                            if !manifest.interface.requires.is_empty() {
                                text.push_str("\nRequires:\n");
                                for interface in &manifest.interface.requires {
                                    text.push_str(&format!("- {}\n", interface));
                                }
                            }
                            
                            if let Some(ref component_path) = manifest.component_path {
                                text.push_str(&format!("\nComponent path: {}\n", component_path));
                            }
                        }
                        
                        text.push_str(&format!("\n## Build Status\n\n{:?}\n", actor.build_info.build_status));
                        
                        if let Some(ref cargo) = actor.cargo_config {
                            text.push_str("\n## Dependencies\n\n");
                            for (name, version) in &cargo.dependencies {
                                text.push_str(&format!("- {}: {}\n", name, version));
                            }
                        }
                        
                        // Read README if exists
                        let readme_path = actor.path.join("README.md");
                        if readme_path.exists() {
                            if let Ok(readme) = fs::read_to_string(readme_path) {
                                text.push_str("\n## README\n\n");
                                text.push_str(&readme);
                            }
                        }
                        
                        let content = vec![
                            ToolContent::Text {
                                text
                            }
                        ];
                        
                        Ok(ToolCallResult {
                            content,
                            is_error: Some(false)
                        })
                    }
                },
                Err(e) => {
                    error!("Failed to get actor info: {}", e);
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Failed to get actor info: {}", e)
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