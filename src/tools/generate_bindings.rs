use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use modelcontextprotocol_server::ServerBuilder;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info};

use crate::registry::Registry;

pub fn register_generate_bindings_tool(
    builder: ServerBuilder,
    registry: Registry,
) -> ServerBuilder {
    builder.with_tool(
        "generate-bindings",
        Some("Generates Rust bindings from an actor's WIT definitions"),
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the actor to generate bindings for (required)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;

            debug!("Generating bindings for actor '{}'", name);

            // Find the actor in the registry
            match registry.find_actor(name) {
                Ok(actor) => {
                    // Create bindings directory if it doesn't exist
                    let bindings_dir = actor.path.join("bindings");
                    if !bindings_dir.exists() {
                        fs::create_dir_all(&bindings_dir)
                            .map_err(|e| anyhow!("Failed to create bindings directory: {}", e))?;
                    }

                    // Execute cargo component bindings command
                    let output = Command::new("cargo")
                        .arg("component")
                        .arg("bindings")
                        .current_dir(&actor.path)
                        .output()
                        .map_err(|e| {
                            anyhow!("Failed to execute cargo component bindings command: {}", e)
                        })?;

                    // Check if command was successful
                    if !output.status.success() {
                        let error = String::from_utf8_lossy(&output.stderr).to_string();
                        error!("Binding generation failed: {}", error);

                        let content = vec![ToolContent::Text {
                            text: format!("Failed to generate bindings: {}", error),
                        }];

                        return Ok(ToolCallResult {
                            content,
                            is_error: Some(true),
                        });
                    }

                    // Log success
                    info!("Successfully generated bindings for actor '{}'", name);

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let success_message = format!(
                        "Successfully generated Rust bindings for actor '{}' at {}\n\n{}",
                        name,
                        bindings_dir.display(),
                        stdout
                    );

                    let content = vec![ToolContent::Text {
                        text: success_message,
                    }];

                    Ok(ToolCallResult {
                        content,
                        is_error: Some(false),
                    })
                }
                Err(e) => {
                    error!("Failed to find actor '{}': {}", name, e);

                    let content = vec![ToolContent::Text {
                        text: format!("Failed to find actor '{}': {}", name, e),
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
