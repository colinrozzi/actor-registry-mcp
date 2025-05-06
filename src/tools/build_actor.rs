use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use modelcontextprotocol_server::ServerBuilder;
use serde_json::json;
use tracing::{debug, error, warn};
use anyhow::anyhow;

use crate::registry::Registry;

pub fn register_build_actor_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "build-actor",
        Some("Builds an actor using its flake and updates its manifest"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the actor (required)"
                },
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            debug!("Building actor '{}'", name);
            
            // First, find the actor
            match registry.find_actor(name) {
                Ok(actor) => {
                    // Perform build
                    match actor.build() {
                        Ok(()) => {
                            // Reload the actor to get updated build info
                            match Registry::new(registry.path())?.find_actor(name) {
                                Ok(updated_actor) => {
                                    // Get component path from the updated actor
                                    let component_path = updated_actor.manifest
                                        .as_ref()
                                        .map(|m| m.component_path.clone())
                                        .unwrap_or_else(|| "Unknown".to_string());
                                    
                                    let content = vec![
                                        ToolContent::Text {
                                            text: format!("Actor '{}' successfully built.\nComponent path: {}\nCheck .build_info for logs", 
                                                         name, component_path)
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
                                            text: format!("Actor '{}' successfully built, but failed to reload actor info: {}\nCheck .build_info for logs", name, e)
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
                            
                            // Try to get build info anyway to provide more details to the user
                            let additional_info = match Registry::new(registry.path()) {
                                Ok(reg) => match reg.find_actor(name) {
                                    Ok(actor) => {
                                        let build_log = actor.build_info.build_log
                                            .map(|log| format!("\nBuild log: {}", log))
                                            .unwrap_or_default();
                                            
                                        let error_msg = actor.build_info.error_message
                                            .map(|msg| format!("\nError message: {}", msg))
                                            .unwrap_or_default();
                                            
                                        format!("{}{}", build_log, error_msg)
                                    },
                                    Err(_) => String::new()
                                },
                                Err(_) => String::new()
                            };
                            
                            let content = vec![
                                ToolContent::Text {
                                    text: format!("Failed to build actor: {}{}\nCheck .build_info for logs", e, additional_info)
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
                            text: format!("Failed to find actor: {}\nCheck .build_info for logs", e)
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
