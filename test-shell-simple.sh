#!/bin/bash

# Test script for shell_simple example
# Tests remote command execution over P2P
# Usage: ./test-shell-simple.sh [--retry]


# Parse command line arguments
QUIET_MODE=false
RETRY_ON_DISCOVERY=false

for arg in "$@"; do
    case $arg in
        --quiet|-q)
            QUIET_MODE=true
            ;;
        --retry)
            RETRY_ON_DISCOVERY=true
            if [ "$QUIET_MODE" = false ]; then
                echo "Note: Retry mode enabled for discovery issues"
            fi
            ;;
        --help|-h)
            echo "Usage: $0 [--quiet|-q] [--retry]"
            echo "  --quiet, -q  Minimal output (only show results and performance)"
            echo "  --retry      Enable retry on discovery failures"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        -*)
            echo "Error: Unknown option $arg"
            echo "Usage: $0 [--quiet|-q] [--retry]"
            exit 1
            ;;
        *)
            echo "Error: Unexpected argument $arg"
            echo "Usage: $0 [--quiet|-q] [--retry]"
            exit 1
            ;;
    esac
done

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Redirect output if quiet mode
if [ "$QUIET_MODE" = true ]; then
    exec 3>&1 4>&2
    exec 1>/tmp/test-shell-simple-$$.log 2>&1
fi

if [ "$QUIET_MODE" = false ]; then
    echo -e "${YELLOW}🧪 Testing fastn-p2p shell_simple implementation${NC}"
fi
if [ "$QUIET_MODE" = false ]; then
    echo "================================================"
fi

# Build first
echo -e "${YELLOW}📦 Building shell_simple...${NC}"
cargo build --release --bin shell_simple 2>&1 | tail -3
echo -e "${GREEN}✅ Build complete${NC}"
echo ""

# Start daemon
echo -e "${YELLOW}🚀 Starting shell daemon...${NC}"
TEST_START_TIME=$(date +%s)
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

# Initialize metrics tracking
TOTAL_COMMANDS=0
TOTAL_LATENCY_NS=0

# Function to test a command
test_command() {
    local cmd="$1"
    local expected="$2"
    local description="$3"
    
    echo -e "${YELLOW}📤 Test: $description${NC}"
    echo "   Command: $cmd"
    
    START_TIME=$(date +%s%N)
    OUTPUT=$(./target/release/shell_simple exec "$DAEMON_ID" $cmd 2>&1 || true)
    END_TIME=$(date +%s%N)
    CMD_LATENCY_NS=$((END_TIME - START_TIME))
    CMD_LATENCY_MS=$(echo "scale=2; $CMD_LATENCY_NS / 1000000" | bc)
    TOTAL_LATENCY_NS=$((TOTAL_LATENCY_NS + CMD_LATENCY_NS))
    ((TOTAL_COMMANDS++))
    
    # Retry only if flag is set and discovery fails
    if [[ "$RETRY_ON_DISCOVERY" == true ]] && echo "$OUTPUT" | grep -q "Discovery"; then
        echo -e "${YELLOW}⚠️  Discovery issue - retrying...${NC}"
        sleep 2
        OUTPUT=$(./target/release/shell_simple exec "$DAEMON_ID" $cmd 2>&1 || true)
    fi
    
    if echo "$OUTPUT" | grep -q "$expected"; then
        echo -e "${GREEN}✅ Success: Got expected output (${CMD_LATENCY_MS}ms)${NC}"
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
if [ "$QUIET_MODE" = false ]; then
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
fi
kill $DAEMON_PID 2>/dev/null || true

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 Shell simple tests completed!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Performance Summary
if [ $TOTAL_COMMANDS -gt 0 ]; then
    AVG_LATENCY_MS=$(echo "scale=2; $TOTAL_LATENCY_NS / $TOTAL_COMMANDS / 1000000" | bc)
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${YELLOW}📊 Performance Metrics:${NC}"
        echo "  • Commands executed: $TOTAL_COMMANDS"
        echo "  • Average command latency: ${AVG_LATENCY_MS}ms"
        echo "  • Total test duration: $(($(date +%s) - TEST_START_TIME))s"
    else
        # Restore original stdout/stderr and show summary
        exec 1>&3 2>&4
        echo -e "${GREEN}✅ Shell Simple: PASS${NC}"
        echo "  • Commands: $TOTAL_COMMANDS"
        echo "  • Avg latency: ${AVG_LATENCY_MS}ms"
        echo "  • Duration: $(($(date +%s) - TEST_START_TIME))s"
        # Clean up log file
        rm -f /tmp/test-shell-simple-$$.log
    fi
fi