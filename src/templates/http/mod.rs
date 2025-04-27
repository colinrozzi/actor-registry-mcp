// HTTP template module
mod lib_rs_template;
mod wit_template;

// Re-export template constants
pub use lib_rs_template::LIB_RS;
pub use wit_template::WORLD_WIT;

// HTTP template information
pub fn get_description() -> &'static str {
    "An HTTP server actor with REST API and WebSocket support"
}

// Template-specific handler configurations
pub fn get_handlers() -> Vec<theater::config::HandlerConfig> {
    vec![
        theater::config::HandlerConfig::Runtime(theater::config::RuntimeHostConfig {}),
        // For HTTP handlers, we'll use a more basic approach until we know
        // the correct way to define HTTP framework handlers
    ]
}

// Generate README content for an HTTP actor
pub fn generate_readme(name: &str) -> String {
    format!(
        "# {}\n\nA Theater HTTP server actor.\n\n## Features\n\n- HTTP server running on port 8080\n- REST API endpoints\n- WebSocket support\n\n## Building\n\nTo build the actor:\n\n```bash\ncargo build --target wasm32-unknown-unknown --release\n```\n\n## Running\n\nTo run the actor with Theater:\n\n```bash\ntheater start manifest.toml\n```\n\n## API Endpoints\n\n- GET / - Returns a simple HTML welcome page\n- GET /api/hello - Returns a JSON greeting message\n- WS /ws - WebSocket endpoint that echoes messages\n",
        name
    )
}
