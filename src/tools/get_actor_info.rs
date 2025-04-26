use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use std::fs;
use tracing::{debug, error, info};

use crate::registry::Registry;

pub fn register_get_actor_info_tool(builder: ServerBuilder, registry: Registry) -> ServerBuilder {
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
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;

            debug!("Getting info for actor '{}'", name);

            match registry.find_actor(name) {
                Ok(actor) => {
                    // Format as text
                    let mut text = format!("# Actor: {}\n\n", actor.name);

                    if let Some(ref manifest) = actor.manifest {
                        // We no longer have version or description in Theater's manifest
                        text.push_str("Theater Component\n");

                        text.push_str("\n## Interfaces\n\n");
                        text.push_str("Implements:\n");
                        text.push_str(&format!("- {}\n", manifest.interface.implements));

                        if !manifest.interface.requires.is_empty() {
                            text.push_str("\nRequires:\n");
                            for interface in &manifest.interface.requires {
                                text.push_str(&format!("- {}\n", interface));
                            }
                        }

                        if !manifest.component_path.is_empty() {
                            text.push_str(&format!("\nComponent path: {}\n", manifest.component_path));
                        }
                    }

                    text.push_str(&format!(
                        "\n## Build Status\n\n{:?}\n",
                        actor.build_info.build_status
                    ));

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

                    let content = vec![ToolContent::Text { text }];

                    Ok(ToolCallResult {
                        content,
                        is_error: Some(false),
                    })
                }
                Err(e) => {
                    error!("Failed to get actor info: {}", e);
                    let content = vec![ToolContent::Text {
                        text: format!("Failed to get actor info: {}", e),
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

