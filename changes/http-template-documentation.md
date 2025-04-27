# HTTP Template Documentation

The HTTP template provides a foundation for creating HTTP server actors in the Theater system. This documentation explains how to use the HTTP template and what features it offers.

## Creating an HTTP Actor

To create a new HTTP actor using the template:

```bash
# Using the MCP client
mcp-client call create-new-actor --params '{"name": "my-http-actor", "template": "http"}'

# Or using the theater CLI (if available)
theater create-actor my-http-actor --template http
```

## Features

The HTTP template provides the following features out of the box:

1. **HTTP Server**: Pre-configured HTTP server running on port 8080
2. **API Endpoints**: Basic API endpoint structure with routing
3. **WebSocket Support**: Built-in WebSocket support with message echo functionality
4. **Error Handling**: Standard error responses and middleware

## Default Endpoints

The template includes these endpoints:

- `GET /` - Returns a simple HTML welcome page
- `GET /api/hello` - Returns a JSON greeting message
- `WS /ws` - WebSocket endpoint that echoes messages

## Customizing

### Changing the HTTP Port

To change the default HTTP port (8080), modify the `ServerConfig` in the `init` function:

```rust
// Set up HTTP server with custom port
let config = ServerConfig {
    port: Some(3000), // Change this to your desired port
    host: Some("0.0.0.0".to_string()),
    tls_config: None,
};
```

### Adding New Routes

To add new routes, add more `add_route` calls in the `init` function:

```rust
// Add a new route
add_route(server_id, "/api/users", "GET", api_handler_id)?;
```

### Handling Routes

Modify the route handling in the `handle_request` implementation:

```rust
// Route handling
let response = match path {
    // ... existing routes ...
    
    "/api/users" => {
        // Your implementation here
        HttpResponse {
            status: 200,
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            body: Some(json!({ "users": ["user1", "user2"] }).to_string().as_bytes().to_vec()),
        }
    },
    
    // ... other routes ...
}
```

## WebSocket Handling

The template includes basic WebSocket handling. To customize WebSocket behavior, modify the following functions:

1. `handle_websocket_connect` - Called when a client connects
2. `handle_websocket_message` - Called when a message is received
3. `handle_websocket_disconnect` - Called when a client disconnects

## Building and Running

Build the actor:

```bash
cd my-http-actor
cargo component build --release --target wasm32-unknown-unknown
```

Run the actor with Theater:

```bash
theater start manifest.toml
```

## Testing

You can test the HTTP endpoints with curl:

```bash
# Test the homepage
curl http://localhost:8080/

# Test the API endpoint
curl http://localhost:8080/api/hello
```

And test WebSockets with any WebSocket client by connecting to `ws://localhost:8080/ws`.
