use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{debug, error};

use crate::registry::Registry;

pub fn register_list_actors_tool(builder: ServerBuilder, registry: Registry) -> ServerBuilder {
    builder.with_tool(
        "list-actors-in-registry",
        Some("Lists all actors in the registry with their basic information"),
        json!({
            "type": "object",
            "properties": {
            }
        }),
        move |_| {
            debug!("Listing actors in registry",);

            match registry.list_actors() {
                Ok(actors) => {
                    // Apply filter if provided
                    let mut text = format!("Found {} actors:\n\n", actors.len());

                    for actor in &actors {
                        text.push_str(&format!("- {} ", actor.name));

                        if let Some(ref manifest) = actor.manifest {
                            if let Some(ref desc) = manifest.description {
                                text.push_str(&format!("- {}", desc));
                            }
                        }

                        text.push_str(&format!(" [{}]", actor.build_info.build_status));

                        text.push_str("\n");
                    }

                    let content = vec![ToolContent::Text { text }];

                    Ok(ToolCallResult {
                        content,
                        is_error: Some(false),
                    })
                }
                Err(e) => {
                    error!("Failed to list actors: {}", e);
                    let content = vec![ToolContent::Text {
                        text: format!("Failed to list actors: {}", e),
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

