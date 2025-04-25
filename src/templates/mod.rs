// In a real implementation, these templates would be in separate files
pub(crate) mod templates {
    pub(crate) const BASIC_LIB_RS: &str = r#"mod bindings;

use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::ntwk::theater::runtime::log;
use crate::bindings::ntwk::theater::types::State;

struct Component;
impl Guest for Component {
    fn init(_state: State, params: (String,)) -> Result<(State,), String> {
        log("Initializing {{actor_name}} actor");
        let (param,) = params;
        log(&format!("Init parameter: {}", param));

        log("Hello from {{actor_name}}!");

        Ok((Some(vec![]),))
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message");
        let (data,) = params;
        log(&format!("Received data: {:?}", data));
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message");
        let (request_id, data) = params;
        log(&format!(
            "[req id] {} [data] {}",
            request_id,
            String::from_utf8(data.clone()).expect("Failed to convert data to string")
        ));

        Ok((state, (Some(data),)))
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (bindings::exports::ntwk::theater::message_server_client::Json,),
    ) -> Result<
        (
            Option<bindings::exports::ntwk::theater::message_server_client::Json>,
            (bindings::exports::ntwk::theater::message_server_client::ChannelAccept,),
        ),
        String,
    > {
        log("Handling channel open message");
        log(&format!("Channel open message: {:?}", params));
        Ok((
            state,
            (
                bindings::exports::ntwk::theater::message_server_client::ChannelAccept {
                    accepted: true,
                    message: None,
                },
            ),
        ))
    }

    fn handle_channel_close(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (String,),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Handling channel close message");
        log(&format!("Channel close message: {:?}", params));
        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (
            String,
            bindings::exports::ntwk::theater::message_server_client::Json,
        ),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Received channel message");
        log(&format!("Channel message: {:?}", params));
        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
"#;

    pub(crate) const HTTP_LIB_RS: &str = r#"mod bindings;

use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::http_handlers::Guest as HttpHandlersGuest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::ntwk::theater::http_client::HttpRequest as ClientHttpRequest;
use crate::bindings::ntwk::theater::http_framework::{
    add_route, create_server, register_handler, start_server, ServerConfig,
};
use crate::bindings::ntwk::theater::http_types::{
    HttpRequest as FrameworkHttpRequest, HttpResponse as FrameworkHttpResponse, MiddlewareResult,
};
use crate::bindings::ntwk::theater::runtime::log;
use crate::bindings::ntwk::theater::types::State;

struct Component;

impl Guest for Component {
    fn init(_state: State, params: (String,)) -> Result<(State,), String> {
        log("Initializing {{actor_name}} actor");
        let (param,) = params;
        log(&format!("Init parameter: {}", param));

        // Set up HTTP server
        let config = ServerConfig {
            port: Some(8080),
            host: Some("0.0.0.0".to_string()),
            tls_config: None,
        };

        // Create a new HTTP server
        let server_id = create_server(&config)?;
        log(&format!("Created server with ID: {}", server_id));

        // Register handlers
        let api_handler_id = register_handler("handle_request")?;
        log(&format!("Registered API handler: {}", api_handler_id));

        // Add routes
        add_route(server_id, "/", "GET", api_handler_id)?;
        add_route(server_id, "/api", "GET", api_handler_id)?;
        add_route(server_id, "/api", "POST", api_handler_id)?;

        // Start the server
        let port = start_server(server_id)?;
        log(&format!("Server started on port {}", port));

        Ok((Some(vec![]),))
    }
}

impl HttpHandlersGuest for Component {
    fn handle_request(
        state: Option<Vec<u8>>,
        params: (u64, FrameworkHttpRequest),
    ) -> Result<(Option<Vec<u8>>, (FrameworkHttpResponse,)), String> {
        let (handler_id, request) = params;
        log(&format!(
            "Handling HTTP request with handler ID: {}",
            handler_id
        ));
        log(&format!("Request URI: {}", request.uri));

        // Simple response for demo purposes
        let response = FrameworkHttpResponse {
            status: 200,
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: Some("Hello from {{actor_name}} HTTP handler!".as_bytes().to_vec()),
        };

        Ok((state, (response,)))
    }

    fn handle_middleware(
        state: Option<Vec<u8>>,
        params: (u64, FrameworkHttpRequest),
    ) -> Result<(Option<Vec<u8>>, (MiddlewareResult,)), String> {
        let (handler_id, request) = params;
        log(&format!(
            "Handling middleware with handler ID: {}",
            handler_id
        ));

        // For now, just pass all requests through
        Ok((
            state,
            (MiddlewareResult {
                proceed: true,
                request,
            },),
        ))
    }

    fn handle_websocket_connect(
        state: Option<Vec<u8>>,
        params: (u64, u64, String, Option<String>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (handler_id, connection_id, path, _query) = params;
        log(&format!(
            "WebSocket connected - Handler: {}, Connection: {}, Path: {}",
            handler_id, connection_id, path
        ));

        Ok((state,))
    }

    fn handle_websocket_message(
        state: Option<Vec<u8>>,
        params: (u64, u64, crate::bindings::ntwk::theater::websocket_types::WebsocketMessage),
    ) -> Result<(Option<Vec<u8>>, (Vec<crate::bindings::ntwk::theater::websocket_types::WebsocketMessage>,)), String> {
        let (handler_id, connection_id, _message) = params;
        log(&format!(
            "WebSocket message received - Handler: {}, Connection: {}",
            handler_id, connection_id
        ));

        Ok((state, (vec![],)))
    }

    fn handle_websocket_disconnect(
        state: Option<Vec<u8>>,
        params: (u64, u64),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (handler_id, connection_id) = params;
        log(&format!(
            "WebSocket disconnected - Handler: {}, Connection: {}",
            handler_id, connection_id
        ));

        Ok((state,))
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message");
        let (data,) = params;
        log(&format!("Received data: {:?}", data));
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message");
        let (request_id, data) = params;
        log(&format!(
            "[req id] {} [data] {}",
            request_id,
            String::from_utf8(data.clone()).expect("Failed to convert data to string")
        ));

        Ok((state, (Some(data),)))
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (bindings::exports::ntwk::theater::message_server_client::Json,),
    ) -> Result<
        (
            Option<bindings::exports::ntwk::theater::message_server_client::Json>,
            (bindings::exports::ntwk::theater::message_server_client::ChannelAccept,),
        ),
        String,
    > {
        log("Handling channel open message");
        log(&format!("Channel open message: {:?}", params));
        Ok((
            state,
            (
                bindings::exports::ntwk::theater::message_server_client::ChannelAccept {
                    accepted: true,
                    message: None,
                },
            ),
        ))
    }

    fn handle_channel_close(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (String,),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Handling channel close message");
        log(&format!("Channel close message: {:?}", params));
        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (
            String,
            bindings::exports::ntwk::theater::message_server_client::Json,
        ),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Received channel message");
        log(&format!("Channel message: {:?}", params));
        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
"#;

    pub(crate) const SUPERVISOR_LIB_RS: &str = r#"mod bindings;

use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::exports::ntwk::theater::supervisor::Guest as SupervisorGuest;
use crate::bindings::ntwk::theater::runtime::log;
use crate::bindings::ntwk::theater::supervisor as supervisor_host;
use crate::bindings::ntwk::theater::types::State;

struct Component;

impl Guest for Component {
    fn init(_state: State, params: (String,)) -> Result<(State,), String> {
        log("Initializing {{actor_name}} supervisor actor");
        let (param,) = params;
        log(&format!("Init parameter: {}", param));

        // Store initial state
        let state_data = serde_json::json!({
            "children": [],
            "parameters": param,
        });
        
        let state_bytes = serde_json::to_vec(&state_data)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        Ok((Some(state_bytes),))
    }
}

impl SupervisorGuest for Component {
    fn handle_spawn_child(
        state: Option<Vec<u8>>,
        params: (String, Option<String>, Option<Vec<u8>>),
    ) -> Result<(Option<Vec<u8>>, (String,)), String> {
        let (manifest_path, id, init_state) = params;
        log(&format!("Spawning child from manifest: {}", manifest_path));
        
        // Spawn the child actor
        let child_id = supervisor_host::spawn_child(
            &manifest_path,
            id.as_deref(),
            init_state.as_deref(),
        )?;
        
        log(&format!("Spawned child with ID: {}", child_id));
        
        // Update state to track the new child
        let mut state_data: serde_json::Value = if let Some(state_bytes) = state {
            serde_json::from_slice(&state_bytes)
                .map_err(|e| format!("Failed to deserialize state: {}", e))?
        } else {
            serde_json::json!({ "children": [] })
        };
        
        // Add the new child to the list
        if let Some(children) = state_data.get_mut("children").and_then(|c| c.as_array_mut()) {
            children.push(serde_json::json!({
                "id": child_id,
                "manifest": manifest_path,
                "status": "spawned"
            }));
        }
        
        let updated_state = serde_json::to_vec(&state_data)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        
        Ok((Some(updated_state), (child_id,)))
    }

    fn handle_stop_child(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (child_id,) = params;
        log(&format!("Stopping child: {}", child_id));
        
        // Stop the child actor
        supervisor_host::stop_child(&child_id)?;
        
        // Update state to mark the child as stopped
        let mut state_data: serde_json::Value = if let Some(state_bytes) = state {
            serde_json::from_slice(&state_bytes)
                .map_err(|e| format!("Failed to deserialize state: {}", e))?
        } else {
            serde_json::json!({ "children": [] })
        };
        
        // Update the child's status
        if let Some(children) = state_data.get_mut("children").and_then(|c| c.as_array_mut()) {
            for child in children {
                if let Some(id) = child.get("id").and_then(|id| id.as_str()) {
                    if id == child_id {
                        if let Some(status) = child.get_mut("status") {
                            *status = serde_json::json!("stopped");
                        }
                        break;
                    }
                }
            }
        }
        
        let updated_state = serde_json::to_vec(&state_data)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        
        Ok((Some(updated_state),))
    }

    fn handle_restart_child(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (child_id,) = params;
        log(&format!("Restarting child: {}", child_id));
        
        // Restart the child actor
        supervisor_host::restart_child(&child_id)?;
        
        // Update state to mark the child as restarted
        let mut state_data: serde_json::Value = if let Some(state_bytes) = state {
            serde_json::from_slice(&state_bytes)
                .map_err(|e| format!("Failed to deserialize state: {}", e))?
        } else {
            serde_json::json!({ "children": [] })
        };
        
        // Update the child's status
        if let Some(children) = state_data.get_mut("children").and_then(|c| c.as_array_mut()) {
            for child in children {
                if let Some(id) = child.get("id").and_then(|id| id.as_str()) {
                    if id == child_id {
                        if let Some(status) = child.get_mut("status") {
                            *status = serde_json::json!("running");
                        }
                        break;
                    }
                }
            }
        }
        
        let updated_state = serde_json::to_vec(&state_data)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        
        Ok((Some(updated_state),))
    }

    fn handle_list_children(
        state: Option<Vec<u8>>,
        _params: (),
    ) -> Result<(Option<Vec<u8>>, (Vec<String>,)), String> {
        log("Listing children");
        
        // Get all child actors from the supervisor
        let children = supervisor_host::list_children()?;
        log(&format!("Found {} children", children.len()));
        
        Ok((state, (children,)))
    }

    fn handle_get_child_status(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>, (String,)), String> {
        let (child_id,) = params;
        log(&format!("Getting status for child: {}", child_id));
        
        // Get the child's status
        let status = supervisor_host::get_child_status(&child_id)?;
        log(&format!("Child status: {}", status));
        
        Ok((state, (status,)))
    }

    fn handle_get_child_state(
        state: Option<Vec<u8>>,
        params: (String,),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        let (child_id,) = params;
        log(&format!("Getting state for child: {}", child_id));
        
        // Get the child's state
        let child_state = supervisor_host::get_child_state(&child_id)?;
        
        Ok((state, (child_state,)))
    }

    fn handle_update_child_state(
        state: Option<Vec<u8>>,
        params: (String, Option<Vec<u8>>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (child_id, new_state) = params;
        log(&format!("Updating state for child: {}", child_id));
        
        // Update the child's state
        supervisor_host::update_child_state(&child_id, new_state.as_deref())?;
        log("Child state updated");
        
        Ok((state,))
    }

    fn handle_get_child_events(
        state: Option<Vec<u8>>,
        params: (String, u32),
    ) -> Result<(Option<Vec<u8>>, (Vec<String>,)), String> {
        let (child_id, limit) = params;
        log(&format!("Getting events for child: {}, limit: {}", child_id, limit));
        
        // Get the child's events
        let events = supervisor_host::get_child_events(&child_id, limit)?;
        log(&format!("Retrieved {} events", events.len()));
        
        Ok((state, (events,)))
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message");
        let (data,) = params;
        log(&format!("Received data: {:?}", data));
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message");
        let (request_id, data) = params;
        log(&format!(
            "[req id] {} [data] {}",
            request_id,
            String::from_utf8(data.clone()).expect("Failed to convert data to string")
        ));

        Ok((state, (Some(data),)))
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (bindings::exports::ntwk::theater::message_server_client::Json,),
    ) -> Result<
        (
            Option<bindings::exports::ntwk::theater::message_server_client::Json>,
            (bindings::exports::ntwk::theater::message_server_client::ChannelAccept,),
        ),
        String,
    > {
        log("Handling channel open message");
        log(&format!("Channel open message: {:?}", params));
        Ok((
            state,
            (
                bindings::exports::ntwk::theater::message_server_client::ChannelAccept {
                    accepted: true,
                    message: None,
                },
            ),
        ))
    }

    fn handle_channel_close(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (String,),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Handling channel close message");
        log(&format!("Channel close message: {:?}", params));
        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (
            String,
            bindings::exports::ntwk::theater::message_server_client::Json,
        ),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        log("Received channel message");
        log(&format!("Channel message: {:?}", params));
        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
"#;

    pub(crate) const FLAKE_NIX: &str = r#"{
  description = "{{actor_name}} - A Theater actor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustVersion = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustVersion;
          rustc = rustVersion;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        buildActor = args: rustPlatform.buildRustPackage (rec {
          inherit pname version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
            };
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          postBuild = ''
            mkdir -p $out/lib
            cp target/wasm32-unknown-unknown/release/*.wasm $out/lib/
          '';
        } // args);
      in
      {
        packages = {
          default = buildActor {};

          ${pname} = buildActor {};
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rustVersion.override { targets = [ "wasm32-unknown-unknown" ]; })
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "{{actor_name}} development environment"
            echo "Run 'cargo build --target wasm32-unknown-unknown --release' to build"
          '';
        };
      }
    );
}
"#;
}
