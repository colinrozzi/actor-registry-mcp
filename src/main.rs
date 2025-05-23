use anyhow::Result;
use modelcontextprotocol_server::{transport::StdioTransport, ServerBuilder};
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use tracing::{debug, info, Level};
use tracing_subscriber::fmt;

// Import theater for manifest types
use theater;

// Import our tool implementations
mod registry;
mod templates;
mod tools;
mod utils;

use tools::{
    build_actor::register_build_actor_tool, create_actor::register_create_actor_tool,
    get_actor_info::register_get_actor_info_tool, get_actor_path::register_get_actor_path_tool,
    list_actors::register_list_actors_tool,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let log_file = "/Users/colinrozzi/work/mcp-servers/actor-registry-mcp/actor-registry-mcp.log";
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_writer(move || -> Box<dyn io::Write> {
            Box::new(io::BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .unwrap(),
            ))
        })
        .with_ansi(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set default tracing subscriber");

    debug!("Logging initialized to {}", log_file);

    // Set up registry configuration
    let registry_path = match env::var("THEATER_ACTORS_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let default_path = PathBuf::from("/Users/colinrozzi/work/actor-registry");
            info!(
                "THEATER_ACTORS_PATH not set, using default path: {:?}",
                default_path
            );
            default_path
        }
    };

    // Initialize the registry (shared state)
    let registry = registry::Registry::new(registry_path)?;

    info!("Starting Actor Registry MCP server");
    debug!("Registry path: {:?}", registry.path());

    // Create server builder
    let mut server_builder =
        ServerBuilder::new("theater-actor-registry", "0.1.0").with_transport(StdioTransport::new());

    // Register tools
    server_builder = register_create_actor_tool(server_builder, registry.clone());
    server_builder = register_list_actors_tool(server_builder, registry.clone());
    server_builder = register_build_actor_tool(server_builder, registry.clone());
    server_builder = register_get_actor_info_tool(server_builder, registry.clone());
    server_builder = register_get_actor_path_tool(server_builder, registry.clone());

    // Build the server
    let server = server_builder.build()?;

    info!("Server initialized. Waiting for client connection...");

    // Run server (blocks until shutdown)
    server.run().await?;

    info!("Server shutting down");

    Ok(())
}
