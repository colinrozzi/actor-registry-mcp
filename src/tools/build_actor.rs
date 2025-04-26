use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error, warn};
use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::registry::actor::BuildStatus;

use crate::registry::Registry;

pub fn register_build_actor_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "build-actor",
        Some("Builds an actor using nix flakes and validates the output"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the actor (required)"
                },
                "release": {
                    "type": "boolean",
                    "description": "Build in release mode (optional)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            let release = args.get("release").and_then(|v| v.as_bool()).unwrap_or(true);
            
            debug!("Building actor '{}' with release: {}", name, release);
            
            // First, find the actor
            match registry.find_actor(name) {
                Ok(actor) => {
                    // Perform build
                    match actor.build(release) {
                        Ok(()) => {
                            // Reload the actor to get updated build info
                            match Registry::new(registry.path())?.find_actor(name) {
                                Ok(updated_actor) => {
                                    // Get component path from the updated actor
                                    let component_path = updated_actor.manifest
                                        .as_ref()
                                        .and_then(|m| m.component_path.clone())
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    
                                    let content = vec![
                                        ToolContent::Text {
                                            text: format!("Actor '{}' successfully built.\nComponent path: {}", 
                                                         name, component_path)
                                        },
                                        ToolContent::Resource {
                                            resource: json!({
                                                "name": updated_actor.name,
                                                "build_status": format!("{}", updated_actor.build_info.build_status),
                                                "component_path": component_path,
                                                "last_build_time": updated_actor.build_info.last_build_time.map(|t| {
                                                    t.duration_since(UNIX_EPOCH).unwrap().as_secs()
                                                }),
                                                "build_log": updated_actor.build_info.build_log
                                            })
                                        }
                                    ];
                                    
                                    Ok(ToolCallResult {
                                        content,
                                        is_error: Some(false)
                                    })
                                },
                                Err(e) => {
                                    warn!("Build succeeded but failed to reload actor info: {}", e);
                                    let content = vec![
                                        ToolContent::Text {
                                            text: format!("Actor '{}' successfully built, but failed to reload actor info: {}", name, e)
                                        }
                                    ];
                                    
                                    Ok(ToolCallResult {
                                        content,
                                        is_error: Some(false)
                                    })
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to build actor: {}", e);
                            let content = vec![
                                ToolContent::Text {
                                    text: format!("Failed to build actor: {}", e)
                                }
                            ];
                            
                            Ok(ToolCallResult {
                                content,
                                is_error: Some(true)
                            })
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to find actor for build: {}", e);
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Failed to find actor: {}", e)
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