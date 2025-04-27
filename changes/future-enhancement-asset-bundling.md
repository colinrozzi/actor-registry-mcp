# Future Enhancement: Static Asset Bundling

This document outlines a plan for enhancing the HTTP template with automatic static asset bundling.

## Overview

Currently, the HTTP template provides a basic structure for an HTTP server actor, but it doesn't include a way to bundle static assets (HTML, CSS, JS, images, etc.) with the actor. This enhancement would add functionality to copy template assets to the new actor during creation.

## Implementation Plan

### 1. Asset Directory Structure

Create a standard directory structure for assets:

```
/templates/http/assets/
  ├── index.html
  ├── styles.css
  ├── app.js
  └── images/
      └── logo.png
```

### 2. Enhanced Actor Creation

Update the `create()` method in `actor.rs` to copy assets:

```rust
pub fn create<P: AsRef<Path>>(name: &str, path: P, template: Option<&str>) -> Result<Self> {
    // ...existing code...
    
    // Copy static assets for HTTP template
    if template_name == "http" {
        let assets_dir = path.join("assets");
        fs::create_dir_all(&assets_dir)?;
        
        // Copy template assets if they exist
        let template_assets_dir = PathBuf::from("/Users/colinrozzi/work/mcp-servers/actor-registry-mcp/templates/http/assets");
        if template_assets_dir.exists() {
            copy_dir_recursively(&template_assets_dir, &assets_dir)?;
        }
        
        // Add a note about assets to the README
        fs::write(
            path.join("assets").join("README.md"),
            "# Static Assets\n\nThis directory contains static assets for your HTTP actor.\n",
        )?;
    }
    
    // ...rest of the code...
}

// Helper function to copy directories recursively
fn copy_dir_recursively(source: &Path, destination: &Path) -> Result<()> {
    if !destination.exists() {
        fs::create_dir_all(destination)?;
    }

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = destination.join(file_name);

        if entry_path.is_dir() {
            copy_dir_recursively(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}
```

### 3. Update HTTP Template to Serve Static Files

Enhance the HTTP template to serve static files from the assets directory:

```rust
fn handle_request(
    state: Option<Vec<u8>>,
    params: (u64, HttpRequest),
) -> Result<(Option<Vec<u8>>, (HttpResponse,)), String> {
    // ...existing code...
    
    // Add a function to serve static files from assets directory
    fn serve_static_file(path: &str, state: Option<Vec<u8>>) -> Result<(Option<Vec<u8>>, (HttpResponse,)), String> {
        // Map file extensions to MIME types
        let content_type = match path.split('.').last() {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            _ => "application/octet-stream",
        };
        
        // Try to read the file from the filesystem
        match fs::read(format!("./assets/{}", path)) {
            Ok(content) => {
                let response = HttpResponse {
                    status: 200,
                    headers: vec![
                        ("Content-Type".to_string(), content_type.to_string()),
                        ("Cache-Control".to_string(), "max-age=3600".to_string()),
                    ],
                    body: Some(content),
                };
                Ok((state, (response,)))
            },
            Err(_) => {
                // Return 404 if file not found
                let response = HttpResponse {
                    status: 404,
                    headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                    body: Some(format!("File not found: {}", path).into_bytes()),
                };
                Ok((state, (response,)))
            }
        }
    }
    
    // Route handling with static file support
    let response = match path {
        "/" => {
            // Redirect to index.html or serve it directly
            serve_static_file("index.html", state)?
        },
        path if path.starts_with("/assets/") => {
            // Strip the /assets/ prefix and serve the file
            let asset_path = path.trim_start_matches("/assets/");
            serve_static_file(asset_path, state)?
        },
        "/api/hello" => {
            // API endpoints remain the same
            // ...
        },
        // ...other routes...
    };
    
    // ...rest of the code...
}
```

### 4. Use bindings::filesystem for File Access

For proper file system access in WebAssembly, update the template to use the Theater filesystem bindings:

```rust
use crate::bindings::ntwk::theater::filesystem;

// ...

// Read file from filesystem
fn read_file(path: &str) -> Result<Vec<u8>, String> {
    match filesystem::read_file(path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to read file {}: {}", path, e)),
    }
}

// Enhance the serve_static_file function
fn serve_static_file(path: &str, state: Option<Vec<u8>>) -> Result<(Option<Vec<u8>>, (HttpResponse,)), String> {
    // Map file extensions to MIME types
    let content_type = match path.split('.').last() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };
    
    // Try to read the file using the filesystem binding
    match read_file(&format!("./assets/{}", path)) {
        Ok(content) => {
            let response = HttpResponse {
                status: 200,
                headers: vec![
                    ("Content-Type".to_string(), content_type.to_string()),
                    ("Cache-Control".to_string(), "max-age=3600".to_string()),
                ],
                body: Some(content),
            };
            Ok((state, (response,)))
        },
        Err(_) => {
            // Return 404 if file not found
            let response = HttpResponse {
                status: 404,
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: Some(format!("File not found: {}", path).into_bytes()),
            };
            Ok((state, (response,)))
        }
    }
}
```

### 5. Template Substitution

Add a template substitution step for static assets to replace placeholders like `{{actor_name}}`:

```rust
// During actor creation, process template files
fn process_template_file(path: &Path, replacements: &[(&str, &str)]) -> Result<()> {
    if let Ok(content) = fs::read_to_string(path) {
        let mut processed = content;
        
        // Apply all replacements
        for (placeholder, value) in replacements {
            processed = processed.replace(placeholder, value);
        }
        
        // Write the processed content back
        fs::write(path, processed)?;
    }
    
    Ok(())
}

// In the create() method
if template_name == "http" {
    // ...copy assets...
    
    // Process HTML, CSS, and JS files to replace placeholders
    let replacements = [
        ("{{actor_name}}", name),
        ("{{server_port}}", "8080"),
    ];
    
    for entry in walkdir::WalkDir::new(&assets_dir) {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            // Only process text files
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ["html", "css", "js", "json", "txt", "md"].contains(&ext) {
                    process_template_file(path, &replacements)?;
                }
            }
        }
    }
}
```

## 6. Update Actor Documentation

Enhance the generated README to include information about static assets:

```rust
let readme_content = match template_name {
    "http" => format!(
        "# {}\\n\\nA Theater HTTP server actor created from the {} template.\\n\\n## Features\\n\\n- HTTP server running on port 8080\\n- REST API endpoints\\n- WebSocket support\\n- Static asset serving\\n\\n## Static Assets\\n\\nThis actor includes static assets in the `assets/` directory:\\n\\n- `index.html` - Main page\\n- `styles.css` - Stylesheet\\n- `app.js` - JavaScript for WebSocket demo\\n\\nYou can add your own assets to this directory.\\n\\n## Building\\n\\n```bash\\ncargo build --target wasm32-unknown-unknown --release\\n```\\n\\n## Running\\n\\n```bash\\ntheater start manifest.toml\\n```\\n\\n## API Endpoints\\n\\n- GET / - Serves index.html\\n- GET /assets/* - Serves static assets\\n- GET /api/hello - Returns a JSON greeting message\\n- WS /ws - WebSocket endpoint that echoes messages\\n",
        name, template_name
    ),
    // ... other templates ...
}
```

## Testing the Enhancement

1. Create a new actor with the HTTP template
2. Verify that the assets directory is created and populated
3. Build and run the actor
4. Access the actor in a web browser to confirm assets are served correctly
5. Modify an asset and verify the changes are reflected

## Future Work

1. Add support for asset bundling and minification during the build process
2. Implement asset versioning for better caching
3. Add support for more asset types (fonts, videos, etc.)
4. Create a simple asset management API for runtime modifications
