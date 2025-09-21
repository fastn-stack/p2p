#!/bin/bash

# Test script for request-response example
# This script starts a server, sends a request, and verifies the response
# Usage: ./test-request-response.sh [--quiet] [--retry]
#   --quiet: Minimal output (only show results and performance)
#   --retry: Enable retry on discovery failures

set -e  # Exit on error

# Parse command line arguments
QUIET_MODE=false
RETRY_ON_DISCOVERY=false

for arg in "$@"; do
    case $arg in
        --quiet)
            QUIET_MODE=true
            ;;
        --retry)
            RETRY_ON_DISCOVERY=true
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

if [ "$QUIET_MODE" = false ]; then
    echo -e "${YELLOW}🧪 Testing fastn-p2p request-response implementation${NC}"
    echo "================================================"
fi

# Clean up function
cleanup() {
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
        if [ ! -z "$SERVER_PID" ]; then
            echo "Killing server (PID: $SERVER_PID)"
        fi
    fi
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
    # Clean up any key files created during test
    rm -f .fastn.* 2>/dev/null || true
}

# Set up trap to clean up on exit
trap cleanup EXIT INT TERM

# Pre-compilation stage
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}📦 Pre-compilation stage...${NC}"
    echo "Building all dependencies (this may take a while on first run)..."
fi

# Build everything in release mode for better performance
if [ "$QUIET_MODE" = true ]; then
    cargo build --package examples --bin request_response --release >/dev/null 2>&1 || {
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    }
else
    cargo build --package examples --bin request_response --release || {
        echo -e "${RED}❌ Failed to build request_response example${NC}"
        exit 1
    }
    echo -e "${GREEN}✅ Pre-compilation complete${NC}"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}Starting actual P2P tests...${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

# Start the server in the background (using compiled binary directly)
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}🚀 Starting server...${NC}"
fi
TEST_START_TIME=$(date +%s)
./target/release/request_response server > server.log 2>&1 &
SERVER_PID=$!

# Give server time to start and generate ID52
if [ "$QUIET_MODE" = false ]; then
    echo "Waiting for server to start..."
fi
for i in {1..10}; do
    if grep -q "🎧 Server listening on:" server.log 2>/dev/null; then
        if [ "$QUIET_MODE" = false ]; then
            echo "Server started!"
        fi
        break
    fi
    sleep 1
done

# Check if server is still running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Server process died${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Check server.log for details:"
        tail -20 server.log
    fi
    exit 1
fi

# Extract the server ID52 from the log
SERVER_ID52=$(grep "🎧 Server listening on:" server.log | sed 's/.*🎧 Server listening on: //' | head -1)

if [ -z "$SERVER_ID52" ]; then
    echo -e "${RED}❌ Could not find server ID52${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Server log contents:"
        cat server.log
    fi
    exit 1
fi

if [ "$QUIET_MODE" = false ]; then
    echo -e "${GREEN}✅ Server started with ID52: $SERVER_ID52${NC}"
    echo "Waiting for discovery services to register server..."
fi
sleep 3

# Initialize test results
TEST_RESULTS=()
FAILED=false

# Test 1: Send a simple message
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}📤 Test 1: Sending 'Hello P2P!' message...${NC}"
fi
START_TIME=$(date +%s%N)
OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Hello P2P!" 2>&1 | tail -5)
END_TIME=$(date +%s%N)
LATENCY_NS=$((END_TIME - START_TIME))
LATENCY_MS=$(echo "scale=2; $LATENCY_NS / 1000000" | bc)

# Retry only if flag is set and discovery fails
if [[ "$RETRY_ON_DISCOVERY" == true ]] && echo "$OUTPUT" | grep -q "Discovery"; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${YELLOW}⚠️  Discovery issue - retrying after 2 seconds...${NC}"
    fi
    sleep 2
    START_TIME=$(date +%s%N)
    OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Hello P2P!" 2>&1 | tail -5)
    END_TIME=$(date +%s%N)
    LATENCY_NS=$((END_TIME - START_TIME))
    LATENCY_MS=$(echo "scale=2; $LATENCY_NS / 1000000" | bc)
fi

if echo "$OUTPUT" | grep -q "✅ Response: Echo: Hello P2P!"; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${GREEN}✅ Test 1 passed: Got expected echo response (latency: ${LATENCY_MS}ms)${NC}"
    fi
    TEST_RESULTS+=("Test 1: PASS (${LATENCY_MS}ms)")
else
    echo -e "${RED}❌ Test 1 failed${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Output was:"
        echo "$OUTPUT"
    fi
    TEST_RESULTS+=("Test 1: FAIL")
    FAILED=true
    exit 1
fi

# Test 2: Send a different message
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}📤 Test 2: Sending 'Testing 123' message...${NC}"
fi
START_TIME=$(date +%s%N)
OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Testing 123" 2>&1 | tail -5)
END_TIME=$(date +%s%N)
LATENCY2_NS=$((END_TIME - START_TIME))
LATENCY2_MS=$(echo "scale=2; $LATENCY2_NS / 1000000" | bc)

# Retry only if flag is set and discovery fails
if [[ "$RETRY_ON_DISCOVERY" == true ]] && echo "$OUTPUT" | grep -q "Discovery"; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${YELLOW}⚠️  Discovery issue - retrying after 2 seconds...${NC}"
    fi
    sleep 2
    START_TIME=$(date +%s%N)
    OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Testing 123" 2>&1 | tail -5)
    END_TIME=$(date +%s%N)
    LATENCY2_NS=$((END_TIME - START_TIME))
    LATENCY2_MS=$(echo "scale=2; $LATENCY2_NS / 1000000" | bc)
fi

if echo "$OUTPUT" | grep -q "✅ Response: Echo: Testing 123"; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${GREEN}✅ Test 2 passed: Got expected echo response (latency: ${LATENCY2_MS}ms)${NC}"
    fi
    TEST_RESULTS+=("Test 2: PASS (${LATENCY2_MS}ms)")
else
    echo -e "${RED}❌ Test 2 failed${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Output was:"
        echo "$OUTPUT"
    fi
    TEST_RESULTS+=("Test 2: FAIL")
    FAILED=true
    exit 1
fi

# Test 3: Multiple rapid requests
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}📤 Test 3: Sending multiple rapid requests...${NC}"
fi
SUCCESS_COUNT=0
TOTAL_LATENCY_NS=0
for i in {1..5}; do
    START_TIME=$(date +%s%N)
    OUTPUT=$(./target/release/request_response client "$SERVER_ID52" "Message $i" 2>&1 | tail -5)
    END_TIME=$(date +%s%N)
    REQ_LATENCY_NS=$((END_TIME - START_TIME))
    REQ_LATENCY_MS=$(echo "scale=2; $REQ_LATENCY_NS / 1000000" | bc)
    TOTAL_LATENCY_NS=$((TOTAL_LATENCY_NS + REQ_LATENCY_NS))
    if echo "$OUTPUT" | grep -q "✅ Response: Echo: Message $i"; then
        if [ "$QUIET_MODE" = false ]; then
            echo -e "  ${GREEN}✓ Request $i successful (${REQ_LATENCY_MS}ms)${NC}"
        fi
        ((SUCCESS_COUNT++))
    else
        if [ "$QUIET_MODE" = false ]; then
            echo -e "  ${RED}✗ Request $i failed${NC}"
        fi
    fi
done
AVG_LATENCY_MS=$(echo "scale=2; $TOTAL_LATENCY_NS / 5000000" | bc)

if [ $SUCCESS_COUNT -eq 5 ]; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${GREEN}✅ Test 3 passed: All 5 requests successful (avg latency: ${AVG_LATENCY_MS}ms)${NC}"
    fi
    TEST_RESULTS+=("Test 3: PASS (avg ${AVG_LATENCY_MS}ms)")
else
    echo -e "${RED}❌ Test 3 failed: Only $SUCCESS_COUNT/5 requests successful${NC}"
    TEST_RESULTS+=("Test 3: FAIL ($SUCCESS_COUNT/5)")
    FAILED=true
    exit 1
fi

# Check server is still healthy
if ps -p $SERVER_PID > /dev/null; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${GREEN}✅ Server still running after all tests${NC}"
    fi
else
    echo -e "${RED}❌ Server crashed during tests${NC}"
    exit 1
fi

# Final summary
TEST_END_TIME=$(date +%s)
TOTAL_DURATION=$((TEST_END_TIME - TEST_START_TIME))

if [ "$QUIET_MODE" = true ]; then
    # In quiet mode, just show summary
    if [ "$FAILED" = false ]; then
        echo -e "${GREEN}✅ Request-Response: PASS${NC}"
        echo "  • Single request: ${LATENCY_MS}ms"
        echo "  • Avg (5 requests): ${AVG_LATENCY_MS}ms"
        echo "  • Duration: ${TOTAL_DURATION}s"
    else
        echo -e "${RED}❌ Request-Response: FAIL${NC}"
    fi
else
    # Verbose mode - show everything
    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Performance Summary
    echo -e "\n${YELLOW}📊 Performance Metrics:${NC}"
    echo "  • Single request latency: ${LATENCY_MS}ms"
    echo "  • Average latency (5 rapid requests): ${AVG_LATENCY_MS}ms"
    echo "  • Total test duration: ${TOTAL_DURATION}s"
    
    echo -e "\n${YELLOW}Server log preview:${NC}"
    tail -10 server.log

    echo -e "\n${YELLOW}Test completed. Server will be shut down.${NC}"
fi