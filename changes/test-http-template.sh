#!/bin/bash
# Make the script executable with: chmod +x test-http-template.sh
# Test script for HTTP template implementation

set -e  # Exit on any error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing HTTP template implementation${NC}"

# Set the actors directory
export THEATER_ACTORS_PATH="/tmp/theater-test-actors"
ACTOR_NAME="test-http-actor-$(date +%s)"

echo -e "\n${YELLOW}Creating test directory at $THEATER_ACTORS_PATH${NC}"
mkdir -p $THEATER_ACTORS_PATH

# Build the actor registry MCP server
echo -e "\n${YELLOW}Building actor-registry-mcp${NC}"
cargo build

# Start the MCP server in the background
echo -e "\n${YELLOW}Starting actor-registry-mcp server${NC}"
./target/debug/actor-registry-mcp > /tmp/actor-registry-mcp.log 2>&1 &
MCP_PID=$!

# Give the server time to start
sleep 2

# Create a test actor using the HTTP template
echo -e "\n${YELLOW}Creating test actor '$ACTOR_NAME' with HTTP template${NC}"
mcp-client call create-new-actor --params "{\"name\": \"$ACTOR_NAME\", \"template\": \"http\"}"

# Verify the actor was created
if [ -d "$THEATER_ACTORS_PATH/$ACTOR_NAME" ]; then
    echo -e "${GREEN}✓ Actor directory created${NC}"
else
    echo -e "${RED}✗ Failed to create actor directory${NC}"
    kill $MCP_PID
    exit 1
fi

# Check for required files
for file in "src/lib.rs" "wit/world.wit" "manifest.toml" "Cargo.toml" "flake.nix" "README.md"; do
    if [ -f "$THEATER_ACTORS_PATH/$ACTOR_NAME/$file" ]; then
        echo -e "${GREEN}✓ File $file exists${NC}"
    else
        echo -e "${RED}✗ File $file missing${NC}"
        kill $MCP_PID
        exit 1
    fi
done

# Check HTTP-specific configuration in manifest.toml
if grep -q "http-framework" "$THEATER_ACTORS_PATH/$ACTOR_NAME/manifest.toml"; then
    echo -e "${GREEN}✓ manifest.toml contains HTTP framework configuration${NC}"
else
    echo -e "${RED}✗ manifest.toml missing HTTP framework configuration${NC}"
    kill $MCP_PID
    exit 1
fi

# Check HTTP routes in lib.rs
if grep -q "add_route" "$THEATER_ACTORS_PATH/$ACTOR_NAME/src/lib.rs"; then
    echo -e "${GREEN}✓ lib.rs contains HTTP route configuration${NC}"
else
    echo -e "${RED}✗ lib.rs missing HTTP route configuration${NC}"
    kill $MCP_PID
    exit 1
fi

# Check WebSocket support
if grep -q "enable_websocket" "$THEATER_ACTORS_PATH/$ACTOR_NAME/src/lib.rs"; then
    echo -e "${GREEN}✓ lib.rs contains WebSocket configuration${NC}"
else
    echo -e "${RED}✗ lib.rs missing WebSocket configuration${NC}"
    kill $MCP_PID
    exit 1
fi

# Build the actor (if nix is available)
if command -v nix &> /dev/null; then
    echo -e "\n${YELLOW}Building actor with nix${NC}"
    cd "$THEATER_ACTORS_PATH/$ACTOR_NAME"
    if nix build; then
        echo -e "${GREEN}✓ Actor built successfully${NC}"
    else
        echo -e "${RED}✗ Actor build failed${NC}"
        cd -
        kill $MCP_PID
        exit 1
    fi
    cd -
else
    echo -e "\n${YELLOW}Nix not available, skipping build test${NC}"
fi

# Clean up
echo -e "\n${YELLOW}Stopping MCP server${NC}"
kill $MCP_PID

echo -e "\n${GREEN}All tests passed!${NC}"
echo -e "Test actor created at $THEATER_ACTORS_PATH/$ACTOR_NAME"
echo -e "${YELLOW}You can manually inspect the generated files or remove the test directory${NC}"
