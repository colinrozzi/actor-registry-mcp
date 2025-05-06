use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use modelcontextprotocol_server::ServerBuilder;
use serde_json::json;
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::registry::Registry;

pub fn register_get_actor_path_tool(builder: ServerBuilder, registry: Registry) -> ServerBuilder {
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
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;

            debug!("Getting path for actor '{}'", name);

            match registry.find_actor(name) {
                Ok(actor) => {
                    let base_path = actor.path.clone();

                    // Format the path string
                    let path_str = base_path.to_string_lossy().to_string();

                    let content = vec![ToolContent::Text { text: path_str }];

                    Ok(ToolCallResult {
                        content,
                        is_error: None,
                    })
                }
                Err(e) => {
                    error!("Failed to get actor path: {}", e);
                    let content = vec![ToolContent::Text {
                        text: format!("Failed to get actor path: {}", e),
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
