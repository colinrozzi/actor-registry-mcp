use anyhow::{anyhow, Result};
use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use modelcontextprotocol_server::ServerBuilder;
use serde_json::json;

use std::process::Command;
use tracing::{debug, error, info};

use crate::registry::Registry;

pub fn register_build_actor_tool(builder: ServerBuilder, registry: Registry) -> ServerBuilder {
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
                "release": {
                    "type": "boolean",
                    "description": "Build in release mode (optional)"
                },
                "clean": {
                    "type": "boolean",
                    "description": "Clean the target directory before building (optional)"
                },
                "force": {
                    "type": "boolean",
                    "description": "Force rebuild even if the component is up to date (optional)"
                },
                "verbose": {
                    "type": "boolean",
                    "description": "Turn on verbose output (optional)"
                }
            },
            "required": ["name"]
        }),
        move |args| {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Missing required parameter: name"))?;

            debug!("Building actor '{}'", name);

            // First, find the actor to get its path
            match registry.find_actor(name) {
                Ok(actor) => {
                    // Prepare the theater build command with optional arguments
                    let mut cmd = Command::new("theater");
                    cmd.arg("build");

                    // Add optional flags
                    if args
                        .get("release")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        cmd.arg("--release");
                    }
                    if args.get("clean").and_then(|v| v.as_bool()).unwrap_or(false) {
                        cmd.arg("--clean");
                    }
                    if args.get("force").and_then(|v| v.as_bool()).unwrap_or(false) {
                        cmd.arg("--force");
                    }
                    if args
                        .get("verbose")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        cmd.arg("--verbose");
                    }

                    // Add the actor path as the final argument
                    cmd.arg(&actor.path);

                    // Execute the command
                    info!("Executing: {:?}", cmd);
                    match cmd.output() {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                            if output.status.success() {
                                info!("Successfully built actor '{}'", name);
                                let content = vec![ToolContent::Text {
                                    text: format!(
                                        "Actor '{}' successfully built.\n\nOutput:\n{}\n{}",
                                        name, stdout, stderr
                                    ),
                                }];

                                Ok(ToolCallResult {
                                    content,
                                    is_error: Some(false),
                                })
                            } else {
                                error!("Failed to build actor '{}': {}", name, stderr);
                                let content = vec![ToolContent::Text {
                                    text: format!(
                                        "Failed to build actor '{}':\n\nOutput:\n{}\n\nError:\n{}",
                                        name, stdout, stderr
                                    ),
                                }];

                                Ok(ToolCallResult {
                                    content,
                                    is_error: Some(true),
                                })
                            }
                        }
                        Err(e) => {
                            error!("Failed to execute theater build command: {}", e);
                            let content = vec![ToolContent::Text {
                                text: format!("Failed to execute theater build command: {}", e),
                            }];

                            Ok(ToolCallResult {
                                content,
                                is_error: Some(true),
                            })
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to find actor '{}' for building: {}", name, e);
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
