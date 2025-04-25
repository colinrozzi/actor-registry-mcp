# Theater Actor Registry MCP Server

A Model Context Protocol (MCP) server that enables easy management and creation of Theater actors.

## Overview

The Theater Actor Registry MCP Server provides tools for creating, managing, and building WebAssembly actors in the Theater system through the Model Context Protocol. It offers a set of tools accessible via the MCP server that enable easy actor creation, management, and building while maintaining a consistent structure and workflow.

## Features

- **Create new actors**: Generate new actor projects with the required file structure and configurations
- **List actors**: View all available actors with filtering options
- **Build actors**: Build actors using Nix flakes or Cargo
- **Get actor info**: Retrieve detailed information about specific actors
- **Get actor paths**: Find the path to specific actor files

## Installation

1. Clone the repository:
```bash
git clone https://github.com/your-username/actor-registry-mcp.git
cd actor-registry-mcp
```

2. Build the project:
```bash
cargo build --release
```

## Running the MCP Server

Set the environment variable `THEATER_ACTORS_PATH` to point to your actors directory:

```bash
export THEATER_ACTORS_PATH=/path/to/your/actors
cargo run --release
```

If you do not set the environment variable, the server will use the default path `/Users/colinrozzi/work/actor-registry`.

## Available Tools

### create-new-actor

Creates a new actor with the required file structure and configurations.

**Parameters:**
- `name`: Name of the actor (required)
- `template`: Template to use (optional, defaults to basic)
- `interfaces`: List of interfaces to implement (optional)
- `supervisor`: Flag to add supervision capabilities (optional)

**Example:**
```json
{
  "name": "my-new-actor",
  "template": "http",
  "interfaces": ["ntwk:theater/http-handlers"],
  "supervisor": false
}
```

### list-actors

Lists all actors in the registry with their basic information.

**Parameters:**
- `filter`: Filter by interface, status, or pattern (optional)
- `format`: Output format (optional: text, json)
- `detailed`: Include additional metadata (optional)

**Example:**
```json
{
  "filter": "interface:http-handlers",
  "format": "json",
  "detailed": true
}
```

### build-actor

Builds an actor using nix flakes and validates the output.

**Parameters:**
- `name`: Name of the actor (required)
- `release`: Build in release mode (optional)
- `check`: Only check if build would succeed (optional)
- `force`: Force rebuild (optional)

**Example:**
```json
{
  "name": "my-actor",
  "release": true,
  "check": false,
  "force": false
}
```

### get-actor-info

Provides detailed information about a specific actor.

**Parameters:**
- `name`: Name of the actor (required)
- `format`: Output format (optional: text, json)

**Example:**
```json
{
  "name": "my-actor",
  "format": "json"
}
```

### get-actor-path

Retrieves the path to an actor or specific actor files.

**Parameters:**
- `name`: Name of the actor (required)
- `file`: Specific file to locate (optional: manifest.toml, component.wasm, etc.)
- `absolute`: Return absolute path (optional, defaults to relative)

**Example:**
```json
{
  "name": "my-actor",
  "file": "manifest.toml",
  "absolute": true
}
```

## Templates

The Actor Registry supports several templates for new actors:

- **basic**: A simple actor with message handling capabilities
- **http**: An actor with HTTP server functionality
- **supervisor**: An actor with supervisor capabilities for managing child actors

## Integrating with MCP Clients

The Actor Registry MCP Server works with any MCP client that implements the Model Context Protocol. You can use it with the `mcp-client` crate from the `rust-mcp` project:

```rust
use mcp_client::{ClientBuilder, transport::StdioTransport};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a client that connects to the Theater Actor Registry MCP Server
    let (transport, receiver) = StdioTransport::new("path/to/actor-registry-mcp", vec![]);

    let client = ClientBuilder::new("my-client", "0.1.0")
        .with_transport(transport)
        .build()?;

    // Call the create-new-actor tool
    let result = client.call_tool("create-new-actor", &json!({
        "name": "my-new-actor",
        "template": "http"
    })).await?;

    println!("Actor created: {:?}", result);

    Ok(())
}
```

## License

This project is licensed under the MIT License.
