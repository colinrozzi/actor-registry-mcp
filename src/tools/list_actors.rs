use mcp_protocol::types::tool::{ToolCallResult, ToolContent};
use mcp_server::ServerBuilder;
use serde_json::json;
use tracing::{info, debug, error};
use anyhow::{Result, anyhow};
use regex::Regex;

use crate::registry::Registry;
use crate::registry::actor::BuildStatus;

pub fn register_list_actors_tool(
    builder: ServerBuilder,
    registry: Registry
) -> ServerBuilder {
    builder.with_tool(
        "list-actors",
        Some("Lists all actors in the registry with their basic information"),
        json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "string",
                    "description": "Filter by interface, status, or pattern (optional)"
                },
                "format": {
                    "type": "string",
                    "enum": ["text", "json"],
                    "description": "Output format (optional: text, json)"
                },
                "detailed": {
                    "type": "boolean",
                    "description": "Include additional metadata (optional)"
                }
            }
        }),
        move |args| {
            let filter = args.get("filter").and_then(|v| v.as_str());
            let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("text");
            let detailed = args.get("detailed").and_then(|v| v.as_bool()).unwrap_or(false);
            
            debug!("Listing actors with filter '{:?}', format '{}', detailed: {}", 
                   filter, format, detailed);
            
            match registry.list_actors() {
                Ok(mut actors) => {
                    // Apply filter if provided
                    if let Some(filter_str) = filter {
                        let filter_str = filter_str.to_lowercase();
                        
                        // Check if it's an interface filter
                        if filter_str.starts_with("interface:") {
                            let interface = filter_str.trim_start_matches("interface:");
                            actors.retain(|actor| {
                                if let Some(ref manifest) = actor.manifest {
                                    manifest.interface.implements.iter()
                                        .any(|i| i.to_lowercase().contains(interface))
                                } else {
                                    false
                                }
                            });
                        }
                        // Check if it's a status filter
                        else if filter_str.starts_with("status:") {
                            let status = filter_str.trim_start_matches("status:");
                            actors.retain(|actor| {
                                match status {
                                    "built" => actor.build_info.build_status == BuildStatus::Success,
                                    "notbuilt" => actor.build_info.build_status == BuildStatus::NotBuilt,
                                    "failed" => actor.build_info.build_status == BuildStatus::Failed,
                                    _ => true,
                                }
                            });
                        }
                        // Otherwise treat as a pattern
                        else {
                            if let Ok(re) = Regex::new(&filter_str) {
                                actors.retain(|actor| {
                                    re.is_match(&actor.name) || 
                                    actor.manifest.as_ref()
                                        .and_then(|m| m.description.as_ref())
                                        .map(|d| re.is_match(d))
                                        .unwrap_or(false)
                                });
                            } else {
                                // If regex fails, do simple contains match
                                actors.retain(|actor| {
                                    actor.name.to_lowercase().contains(&filter_str) ||
                                    actor.manifest.as_ref()
                                        .and_then(|m| m.description.as_ref())
                                        .map(|d| d.to_lowercase().contains(&filter_str))
                                        .unwrap_or(false)
                                });
                            }
                        }
                    }
                    
                    if format == "json" {
                        let actors_json: Vec<serde_json::Value> = actors.iter().map(|actor| {
                            let mut json = json!({
                                "name": actor.name,
                                "path": actor.path.to_string_lossy(),
                                "build_status": format!("{:?}", actor.build_info.build_status),
                            });
                            
                            if let Some(ref manifest) = actor.manifest {
                                if let Some(ref desc) = manifest.description {
                                    json["description"] = json!(desc);
                                }
                                json["interfaces"] = json!(manifest.interface.implements);
                            }
                            
                            if detailed {
                                if let Some(ref cargo) = actor.cargo_config {
                                    json["version"] = json!(cargo.package.version);
                                    json["dependencies_count"] = json!(cargo.dependencies.len());
                                }
                            }
                            
                            json
                        }).collect();
                        
                        let content = vec![
                            ToolContent::Json {
                                json: json!({
                                    "actors": actors_json,
                                    "count": actors.len(),
                                })
                            }
                        ];
                        
                        Ok(ToolCallResult {
                            content,
                            is_error: Some(false)
                        })
                    } else {
                        // Format as text
                        let mut text = format!("Found {} actors:\n\n", actors.len());
                        
                        for actor in &actors {
                            text.push_str(&format!("- {} ", actor.name));
                            
                            if let Some(ref manifest) = actor.manifest {
                                if let Some(ref desc) = manifest.description {
                                    text.push_str(&format!("- {}", desc));
                                }
                            }
                            
                            text.push_str(&format!(" [{}]", actor.build_info.build_status));
                            
                            if detailed {
                                text.push_str("\n  Path: ");
                                text.push_str(&actor.path.to_string_lossy());
                                
                                if let Some(ref manifest) = actor.manifest {
                                    text.push_str("\n  Interfaces: ");
                                    text.push_str(&manifest.interface.implements.join(", "));
                                }
                                
                                if let Some(ref cargo) = actor.cargo_config {
                                    text.push_str(&format!("\n  Version: {}", cargo.package.version));
                                    text.push_str(&format!("\n  Dependencies: {}", cargo.dependencies.len()));
                                }
                            }
                            
                            text.push_str("\n");
                        }
                        
                        let content = vec![
                            ToolContent::Text {
                                text
                            }
                        ];
                        
                        Ok(ToolCallResult {
                            content,
                            is_error: Some(false)
                        })
                    }
                },
                Err(e) => {
                    error!("Failed to list actors: {}", e);
                    let content = vec![
                        ToolContent::Text {
                            text: format!("Failed to list actors: {}", e)
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