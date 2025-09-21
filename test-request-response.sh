#!/bin/bash

# Test script for request-response example
# This script starts a server, sends a request, and verifies the response

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧪 Testing fastn-p2p request-response implementation${NC}"
echo "================================================"

# Clean up function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        echo "Killing server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
    fi
    # Clean up any key files created during test
    rm -f .fastn.* 2>/dev/null || true
}

# Set up trap to clean up on exit
trap cleanup EXIT INT TERM

# Pre-compilation stage
echo -e "\n${YELLOW}📦 Pre-compilation stage...${NC}"
echo "Building all dependencies (this may take a while on first run)..."

# Build everything in release mode for better performance
cargo build --package examples --bin request_response --release || {
    echo -e "${RED}❌ Failed to build request_response example${NC}"
    exit 1
}

echo -e "${GREEN}✅ Pre-compilation complete${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Starting actual P2P tests...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Start the server in the background (using compiled binary directly)
echo -e "\n${YELLOW}🚀 Starting server...${NC}"
./target/release/request_response server > server.log 2>&1 &
SERVER_PID=$!

# Give server time to start and generate ID52
echo "Waiting for server to start..."
for i in {1..10}; do
    if grep -q "🎧 Server listening on:" server.log 2>/dev/null; then
        echo "Server started!"
        break
    fi
    sleep 1
done

# Check if server is still running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Server process died. Check server.log for details:${NC}"
    tail -20 server.log
    exit 1
fi

# Extract the server ID52 from the log
SERVER_ID52=$(grep "🎧 Server listening on:" server.log | sed 's/.*🎧 Server listening on: //' | head -1)

if [ -z "$SERVER_ID52" ]; then
    echo -e "${RED}❌ Could not find server ID52 in server.log${NC}"
    echo "Server log contents:"
    cat server.log
    exit 1
fi

echo -e "${GREEN}✅ Server started with ID52: $SERVER_ID52${NC}"

# Test 1: Send a simple message
echo -e "\n${YELLOW}📤 Test 1: Sending 'Hello P2P!' message...${NC}"
OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Hello P2P!" 2>&1 | tail -5)

if echo "$OUTPUT" | grep -q "✅ Response: Echo: Hello P2P!"; then
    echo -e "${GREEN}✅ Test 1 passed: Got expected echo response${NC}"
else
    echo -e "${RED}❌ Test 1 failed: Unexpected response${NC}"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi

# Test 2: Send a different message
echo -e "\n${YELLOW}📤 Test 2: Sending 'Testing 123' message...${NC}"
OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Testing 123" 2>&1 | tail -5)

if echo "$OUTPUT" | grep -q "✅ Response: Echo: Testing 123"; then
    echo -e "${GREEN}✅ Test 2 passed: Got expected echo response${NC}"
else
    echo -e "${RED}❌ Test 2 failed: Unexpected response${NC}"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi

# Test 3: Multiple rapid requests
echo -e "\n${YELLOW}📤 Test 3: Sending multiple rapid requests...${NC}"
SUCCESS_COUNT=0
for i in {1..5}; do
    OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Message $i" 2>&1 | tail -5)
    if echo "$OUTPUT" | grep -q "✅ Response: Echo: Message $i"; then
        echo -e "  ${GREEN}✓ Request $i successful${NC}"
        ((SUCCESS_COUNT++))
    else
        echo -e "  ${RED}✗ Request $i failed${NC}"
    fi
done

if [ $SUCCESS_COUNT -eq 5 ]; then
    echo -e "${GREEN}✅ Test 3 passed: All 5 requests successful${NC}"
else
    echo -e "${RED}❌ Test 3 failed: Only $SUCCESS_COUNT/5 requests successful${NC}"
    exit 1
fi

# Check server is still healthy
if ps -p $SERVER_PID > /dev/null; then
    echo -e "\n${GREEN}✅ Server still running after all tests${NC}"
else
    echo -e "\n${RED}❌ Server crashed during tests${NC}"
    exit 1
fi

# Final summary
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "\n${YELLOW}Server log preview:${NC}"
tail -10 server.log

echo -e "\n${YELLOW}Test completed. Server will be shut down.${NC}"