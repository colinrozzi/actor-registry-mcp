use std::path::Path;
use std::fs;
use std::io::Read;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use anyhow::{Result, Context};
use tracing::debug;

/// Ensures that a directory exists, creating it if necessary
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        debug!("Creating directory: {:?}", path);
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {:?}", path))?;
    }
    
    Ok(())
}

/// Gets the relative path for an actor file
pub fn get_actor_file_path(actor_name: &str, file_type: &str) -> String {
    match file_type {
        "manifest" => format!("{}/manifest.toml", actor_name),
        "cargo" => format!("{}/Cargo.toml", actor_name),
        "src" => format!("{}/src/lib.rs", actor_name),
        "readme" => format!("{}/README.md", actor_name),
        "flake" => format!("{}/flake.nix", actor_name),
        _ => format!("{}/{}", actor_name, file_type),
    }
}

/// Parses the actor name from a path
pub fn actor_name_from_path<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}

/// Formats a list of actors for display
pub fn format_actor_list(names: &[String], detailed: bool) -> String {
    if names.is_empty() {
        return "No actors found".to_string();
    }
    
    if detailed {
        let header = format!("Found {} actors:\n\n", names.len());
        let actor_items = names.iter()
            .map(|name| format!("- {}", name))
            .collect::<Vec<String>>()
            .join("\n");
        
        format!("{}{}", header, actor_items)
    } else {
        names.join(", ")
    }
}

/// Calculate a hash for a file
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    // Create a hasher
    let mut hasher = DefaultHasher::new();
    
    // Open the file
    let mut file = fs::File::open(path)?;
    
    // Read the file in chunks to avoid loading large files into memory
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        buffer[..bytes_read].hash(&mut hasher);
    }
    
    // Get the hash
    let hash = hasher.finish();
    
    // Return the hash as a hex string
    Ok(format!("{:016x}", hash))
}

/// Get the size of a file in bytes
pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}