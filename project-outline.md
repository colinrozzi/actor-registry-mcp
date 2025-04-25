# Theater Actor Registry

## Overview
The Theater Actor Registry is a local file system-based registry for managing WebAssembly actors in the Theater system. It provides a set of tools for creating, managing, and building actors while maintaining a consistent structure and workflow.

## Core Tools

### create-new-actor
Creates a new actor with the required file structure and configurations.

**Parameters:**
- `name`: Name of the actor (required)
- `path`: Target directory (optional, defaults to current directory)
- `template`: Template to use (optional, defaults to basic)
- `--interfaces`: List of interfaces to implement (optional)
- `--supervisor`: Flag to add supervision capabilities (optional)

**Actions:**
1. Create directory structure
2. Generate Cargo.toml with required dependencies
3. Generate manifest.toml with basic configuration
4. Set up WIT files for specified interfaces
5. Create basic source files with template implementation
6. Initialize nix flake
7. Add README with actor description

### get-actor-path
Retrieves the path to an actor or specific actor files.

**Parameters:**
- `name`: Name of the actor (required)
- `--file`: Specific file to locate (optional: manifest.toml, component.wasm, etc.)
- `--absolute`: Return absolute path (optional, defaults to relative)

**Actions:**
1. Locate actor in registry
2. Validate actor structure
3. Return requested path

### list-actors
Lists all actors in the registry with their basic information.

**Parameters:**
- `--filter`: Filter by interface, status, or pattern (optional)
- `--format`: Output format (optional: text, json)
- `--detailed`: Include additional metadata (optional)

**Actions:**
1. Scan registry directory
2. Collect actor metadata
3. Format and display results
4. Show build status and interface information

### get-actor-info
Provides detailed information about a specific actor.

**Parameters:**
- `name`: Name of the actor (required)
- `--format`: Output format (optional: text, json)

**Actions:**
1. Read actor manifest
2. Parse documentation
3. Show interface information
4. Display build status and history
5. List dependencies and requirements

### build-actor
Builds an actor using nix flakes and validates the output.

**Parameters:**
- `name`: Name of the actor (required)
- `--release`: Build in release mode (optional)
- `--check`: Only check if build would succeed (optional)
- `--force`: Force rebuild (optional)

**Actions:**
1. Validate actor structure
2. Check dependencies
3. Execute nix build
4. Validate built component
5. Update manifest with new component path

## Additional Tools (Future)

### validate-actor
Validates an actor's structure, interfaces, and configuration.

**Parameters:**
- `name`: Name of the actor (required)
- `--fix`: Attempt to fix issues (optional)

### clean-actor
Removes build artifacts and temporary files.

**Parameters:**
- `name`: Name of the actor (required)
- `--all`: Remove all generated files (optional)

### copy-actor
Creates a new actor based on an existing one.

**Parameters:**
- `source`: Source actor name (required)
- `destination`: New actor name (required)
- `--deep`: Copy implementation details (optional)

### update-actor-interface
Updates an actor's WIT interfaces.

**Parameters:**
- `name`: Name of the actor (required)
- `--add`: Interfaces to add (optional)
- `--remove`: Interfaces to remove (optional)

## Registry Structure
```
actors/
├── actor-name/
│   ├── Cargo.toml
│   ├── manifest.toml
│   ├── flake.nix
│   ├── flake.lock
│   ├── README.md
│   ├── src/
│   │   └── lib.rs
│   └── wit/
│       └── interfaces.wit
└── another-actor/
    └── ...
```

## Implementation Priorities

### Phase 1: Core Tools
1. create-new-actor
2. list-actors
3. get-actor-path
4. build-actor
5. get-actor-info

### Phase 2: Enhanced Functionality
1. Template system for create-new-actor
2. Build validation and testing
3. Interface management
4. Actor metadata and documentation

### Phase 3: Additional Tools
1. validate-actor
2. clean-actor
3. copy-actor
4. update-actor-interface

## Future Considerations
- Remote registry support
- Version management
- Dependency tracking
- Interface compatibility checking
- Build caching
- CI/CD integration
