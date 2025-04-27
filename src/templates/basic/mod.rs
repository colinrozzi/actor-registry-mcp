// Basic template module
mod lib_rs_template;
mod wit_template;

// Re-export template constants
pub use lib_rs_template::LIB_RS;
pub use wit_template::WORLD_WIT;

// Basic template information
pub fn get_description() -> &'static str {
    "A simple actor with message handling capabilities"
}

// Template-specific handler configurations
pub fn get_handlers() -> Vec<theater::config::HandlerConfig> {
    vec![theater::config::HandlerConfig::Runtime(
        theater::config::RuntimeHostConfig {},
    )]
}

// Generate README content for a basic actor
pub fn generate_readme(name: &str) -> String {
    format!(
        "# {}\n\nA Theater actor created from the basic template.\n\n## Building\n\nTo build the actor:\n\n```bash\ncargo build --target wasm32-unknown-unknown --release\n```\n\n## Running\n\nTo run the actor with Theater:\n\n```bash\ntheater start manifest.toml\n```\n",
        name
    )
}
