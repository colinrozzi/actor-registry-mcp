use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{debug, error, info};

use crate::registry::Registry;

pub fn register_create_actor_tool(builder: ServerBuilder, registry: Registry) -> ServerBuilder {
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
                    "description": "Template to use (defaults to basic, only basic is supported)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;

            // Optional parameters
            let template = args.get("template").and_then(|v| v.as_str());

            debug!("Creating actor '{}' with template '{:?}'", name, template);

            // Clone interfaces to avoid losing ownership
            match registry.create_actor(name, template) {
                Ok(actor) => {
                    let content = vec![ToolContent::Text {
                        text: format!(
                            "Actor '{}' successfully created at {}",
                            name,
                            actor.path.display()
                        ),
                    }];

                    Ok(ToolCallResult {
                        content,
                        is_error: Some(false),
                    })
                }
                Err(e) => {
                    error!("Failed to create actor: {}", e);
                    let content = vec![ToolContent::Text {
                        text: format!("Failed to create actor: {}", e),
                    }];

                    Ok(ToolCallResult {
                        content,
                        is_error: Some(true),
                    })
                }
            }
        },
    )
}
