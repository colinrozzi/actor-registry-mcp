use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error};
use anyhow::{Result, anyhow};

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
                },
                "check": {
                    "type": "boolean",
                    "description": "Only check if build would succeed (optional)"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force rebuild (optional)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            let release = args.get("release").and_then(|v| v.as_bool()).unwrap_or(true);
            let check = args.get("check").and_then(|v| v.as_bool()).unwrap_or(false);
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            
            debug!("Building actor '{}' with release: {}, check: {}, force: {}", 
                   name, release, check, force);
            
            // For now, we're ignoring check and force flags
            if check {
                // Just simulate a check
                match registry.find_actor(name) {
                    Ok(actor) => {
                        let content = vec![
                            ToolContent::Text {
                                text: format!("Actor '{}' is ready to build", name)
                            }
                        ];
                        
                        Ok(ToolCallResult {
                            content,
                            is_error: Some(false)
                        })
                    },
                    Err(e) => {
                        error!("Failed to find actor for build check: {}", e);
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
            } else {
                // Perform actual build
                match registry.find_actor(name) {
                    Ok(actor) => {
                        match actor.build(release) {
                            Ok(()) => {
                                let content = vec![
                                    ToolContent::Text {
                                        text: format!("Actor '{}' successfully built", name)
                                    }
                                ];
                                
                                Ok(ToolCallResult {
                                    content,
                                    is_error: Some(false)
                                })
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
        }
    )
}