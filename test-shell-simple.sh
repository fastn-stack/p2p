#!/bin/bash

# Test script for shell_simple example
# Tests remote command execution over P2P
# Usage: ./test-shell-simple.sh [--retry]

set -e

# Check for retry flag
RETRY_ON_DISCOVERY=false
if [[ "$1" == "--retry" ]]; then
    RETRY_ON_DISCOVERY=true
    echo "Note: Retry mode enabled for discovery issues"
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧪 Testing fastn-p2p shell_simple implementation${NC}"
echo "================================================"

# Build first
echo -e "${YELLOW}📦 Building shell_simple...${NC}"
cargo build --release --bin shell_simple 2>&1 | tail -3
echo -e "${GREEN}✅ Build complete${NC}"
echo ""

# Start daemon
echo -e "${YELLOW}🚀 Starting shell daemon...${NC}"
./target/release/shell_simple daemon > daemon.log 2>&1 &
DAEMON_PID=$!

# Wait for daemon to start
sleep 3

# Get daemon ID
DAEMON_ID=$(grep "🐚 Shell daemon listening on:" daemon.log | sed 's/.*🐚 Shell daemon listening on: //' | head -1)

if [ -z "$DAEMON_ID" ]; then
    echo -e "${RED}❌ Failed to start daemon${NC}"
    cat daemon.log
    exit 1
fi

echo -e "${GREEN}✅ Daemon started with ID52: $DAEMON_ID${NC}"

# Give discovery services time to register the daemon
echo "Waiting for discovery services to register daemon..."
sleep 3
echo ""

# Function to test a command
test_command() {
    local cmd="$1"
    local expected="$2"
    local description="$3"
    
    echo -e "${YELLOW}📤 Test: $description${NC}"
    echo "   Command: $cmd"
    
    OUTPUT=$(./target/release/shell_simple exec "$DAEMON_ID" $cmd 2>&1 || true)
    
    # Retry only if flag is set and discovery fails
    if [[ "$RETRY_ON_DISCOVERY" == true ]] && echo "$OUTPUT" | grep -q "Discovery"; then
        echo -e "${YELLOW}⚠️  Discovery issue - retrying...${NC}"
        sleep 2
        OUTPUT=$(./target/release/shell_simple exec "$DAEMON_ID" $cmd 2>&1 || true)
    fi
    
    if echo "$OUTPUT" | grep -q "$expected"; then
        echo -e "${GREEN}✅ Success: Got expected output${NC}"
        echo "   Output: $(echo "$OUTPUT" | head -1)"
    else
        echo -e "${RED}❌ Failed: Unexpected output${NC}"
        echo "   Expected: $expected"
        echo "   Got: $OUTPUT"
    fi
    echo ""
}

# Test 1: Simple echo
test_command "echo Hello_P2P" "Hello_P2P" "Simple echo command"

# Test 2: List files
test_command "ls target" "release" "List directory contents"

# Test 3: Show current date
test_command "date" "202" "Show current date (checking for year)"

# Test 4: Command with args
test_command "echo Multiple arguments test" "Multiple arguments test" "Command with multiple args"

# Test 5: Command that fails
echo -e "${YELLOW}📤 Test: Command that should fail${NC}"
OUTPUT=$(./target/release/shell_simple exec "$DAEMON_ID" nonexistent_command 2>&1 || true)
if echo "$OUTPUT" | grep -q "not found\|failed\|error"; then
    echo -e "${GREEN}✅ Success: Command failed as expected${NC}"
else
    echo -e "${RED}❌ Failed: Should have shown error${NC}"
fi
echo ""

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up...${NC}"
kill $DAEMON_PID 2>/dev/null || true

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 Shell simple tests completed!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"