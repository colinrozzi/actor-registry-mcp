use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error};
use anyhow::{Result, anyhow};

use crate::registry::Registry;

pub fn register_create_actor_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "create-new-actor",
        Some("Creates a new actor with the required file structure and configurations"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string", 
                    "description": "Name of the actor (required)"
                },
                "template": {
                    "type": "string", 
                    "description": "Template to use (optional, defaults to basic)"
                },
                "interfaces": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "List of interfaces to implement (optional)"
                },
                "supervisor": {
                    "type": "boolean",
                    "description": "Flag to add supervision capabilities (optional)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;
            
            // Optional parameters
            let template = args.get("template").and_then(|v| v.as_str());
            
            let interfaces_value = args.get("interfaces");
            let mut interfaces = Vec::new();
            
            if let Some(interfaces_arr) = interfaces_value.and_then(|v| v.as_array()) {
                for interface in interfaces_arr {
                    if let Some(interface_str) = interface.as_str() {
                        interfaces.push(interface_str.to_string());
                    }
                }
            }
            
            // Add supervisor interface if requested
            let supervisor = args.get("supervisor").and_then(|v| v.as_bool()).unwrap_or(false);
            if supervisor && !interfaces.contains(&"ntwk:theater/supervisor".to_string()) {
                interfaces.push("ntwk:theater/supervisor".to_string());
            }
            
            debug!("Creating actor '{}' with template '{:?}' and interfaces {:?}", 
                   name, template, interfaces);
            
            match registry.create_actor(name, template, interfaces) {
                Ok(actor) => {
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Actor '{}' successfully created at {}", 
                                          name, actor.path.display())
                        },
                        ToolContent::Json {
                            json: json!({
                                "name": actor.name,
                                "path": actor.path.to_string_lossy(),
                                "template": template.unwrap_or("basic"),
                                "interfaces": interfaces,
                            })
                        }
                    ];
                    
                    Ok(ToolCallResult {
                        content,
                        is_error: Some(false)
                    })
                },
                Err(e) => {
                    error!("Failed to create actor: {}", e);
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Failed to create actor: {}", e)
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