// Templates module - provides templates for different types of actors
pub mod basic;
pub mod http;
pub mod common;

use std::path::Path;
use anyhow::Result;
use std::fs;

// Template manager
pub struct TemplateManager;

impl TemplateManager {
    // Get a list of available templates
    pub fn list_templates() -> Vec<String> {
        vec![
            "basic".to_string(),
            "http".to_string(),
            "supervisor".to_string(), // Listed but not fully implemented yet
        ]
    }
    
    // Get template description
    pub fn get_template_description(template_name: &str) -> &'static str {
        match template_name {
            "basic" => basic::get_description(),
            "http" => http::get_description(),
            "supervisor" => "An actor with supervisor capabilities for managing child actors",
            _ => "Unknown template",
        }
    }
    
    // Apply template to create an actor
    pub fn apply_template(
        template_name: &str, 
        name: &str, 
        path: &Path,
    ) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(path)?;
        fs::create_dir_all(path.join("src"))?;
        fs::create_dir_all(path.join("wit"))?;
        
        // Generate lib.rs content based on template
        let lib_rs_content = match template_name {
            "basic" => basic::LIB_RS,
            "http" => http::LIB_RS,
            _ => return Err(anyhow::anyhow!("Unknown template: {}", template_name)),
        };
        
        // Replace placeholders
        let lib_rs_content = lib_rs_content.replace("{{actor_name}}", name);
        fs::write(path.join("src").join("lib.rs"), lib_rs_content)?;
        
        // Generate world.wit content based on template
        let wit_content = match template_name {
            "basic" => basic::WORLD_WIT,
            "http" => http::WORLD_WIT,
            _ => return Err(anyhow::anyhow!("Unknown template: {}", template_name)),
        };
        
        // Replace placeholders
        let wit_content = wit_content.replace("{{actor_name}}", name);
        fs::write(path.join("wit").join("world.wit"), wit_content)?;
        
        // Generate README content
        let readme_content = match template_name {
            "basic" => basic::generate_readme(name),
            "http" => http::generate_readme(name),
            _ => format!("# {}\n\nA Theater actor.\n", name),
        };
        
        fs::write(path.join("README.md"), readme_content)?;
        
        // Create a flake.nix file (common to all templates)
        let flake_nix_content = common::FLAKE_NIX.replace("{{actor_name}}", name);
        fs::write(path.join("flake.nix"), flake_nix_content)?;
        
        Ok(())
    }
    
    // Get template-specific handler configurations
    pub fn get_template_handlers(template_name: &str) -> Vec<theater::config::HandlerConfig> {
        match template_name {
            "basic" => basic::get_handlers(),
            "http" => http::get_handlers(),
            _ => vec![theater::config::HandlerConfig::Runtime(
                theater::config::RuntimeHostConfig {},
            )],
        }
    }
}
