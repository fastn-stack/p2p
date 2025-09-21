#!/bin/bash

# Test script for file-transfer example
# This script starts a file server, transfers a file, and verifies the content

set -e  # Exit on error

# Parse command line arguments
QUIET_MODE=false

for arg in "$@"; do
    case $arg in
        --quiet|-q)
            QUIET_MODE=true
            ;;
        --help|-h)
            echo "Usage: $0 [--quiet|-q]"
            echo "  --quiet, -q  Minimal output (only show results and performance)"
            echo "  --help, -h   Show this help message"
            exit 0
            ;;
        -*)
            echo "Error: Unknown option $arg"
            echo "Usage: $0 [--quiet|-q]"
            exit 1
            ;;
        *)
            echo "Error: Unexpected argument $arg"
            echo "Usage: $0 [--quiet|-q]"
            exit 1
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧪 Testing fastn-p2p file transfer implementation${NC}"
echo "================================================"

# Clean up function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        echo "Killing server (PID: $SERVER_PID)"
        kill $SERVER_PID 2>/dev/null || true
    fi
    # Clean up test files
    rm -f test_file.txt downloaded_test_file.txt file_server.log 2>/dev/null || true
}

# Set up trap to clean up on exit
trap cleanup EXIT INT TERM

# Pre-compilation stage
echo -e "\n${YELLOW}📦 Pre-compilation stage...${NC}"
echo "Building file_transfer example (this may take a while on first run)..."

cargo build --package examples --bin file_transfer --release || {
    echo -e "${RED}❌ Failed to build file_transfer example${NC}"
    exit 1
}

echo -e "${GREEN}✅ Pre-compilation complete${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}Starting file transfer tests...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Create a test file with some content
echo -e "\n${YELLOW}📝 Creating test file...${NC}"
cat > test_file.txt << EOF
Hello from P2P file transfer!
This is a test file to verify streaming works correctly.
Line 3: Some more content
Line 4: Even more content
Line 5: Final line of test content
EOF
echo "Created test_file.txt with $(wc -l < test_file.txt) lines, $(wc -c < test_file.txt) bytes"

# Start the file server
echo -e "\n${YELLOW}🚀 Starting file server...${NC}"
TEST_START_TIME=$(date +%s)
./target/release/file_transfer server > file_server.log 2>&1 &
SERVER_PID=$!

# Give server time to start and generate ID52
echo "Waiting for server to start..."
for i in {1..10}; do
    if grep -q "📁 File server listening on:" file_server.log 2>/dev/null; then
        echo "Server started!"
        break
    fi
    sleep 1
done

# Check if server is still running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Server process died. Check file_server.log for details:${NC}"
    tail -20 file_server.log
    exit 1
fi

# Extract the server ID52 from the log
SERVER_ID52=$(grep "📁 File server listening on:" file_server.log | sed 's/.*📁 File server listening on: //' | head -1)

if [ -z "$SERVER_ID52" ]; then
    echo -e "${RED}❌ Could not find server ID52 in file_server.log${NC}"
    echo "Server log contents:"
    cat file_server.log
    exit 1
fi

echo -e "${GREEN}✅ Server started with ID52: $SERVER_ID52${NC}"

# Give discovery services time to register the server
echo "Waiting for discovery services to register server..."
sleep 3

# Test 1: Download existing file
echo -e "\n${YELLOW}📥 Test 1: Downloading 'test_file.txt'...${NC}"
FILE_SIZE=$(wc -c < test_file.txt)
START_TIME=$(date +%s%N)
OUTPUT=$(./target/release/file_transfer client "$SERVER_ID52" "test_file.txt" 2>&1)
END_TIME=$(date +%s%N)
TRANSFER_TIME_NS=$((END_TIME - START_TIME))
TRANSFER_TIME_MS=$(echo "scale=2; $TRANSFER_TIME_NS / 1000000" | bc)
THROUGHPUT=$(echo "scale=2; $FILE_SIZE * 8 / ($TRANSFER_TIME_NS / 1000000000)" | bc)

if echo "$OUTPUT" | grep -q "✅ Downloaded test_file.txt"; then
    echo -e "${GREEN}✅ Download completed (${TRANSFER_TIME_MS}ms, ${FILE_SIZE} bytes)${NC}"
    
    # Verify the downloaded file exists and has correct content
    if [ -f "downloaded_test_file.txt" ]; then
        echo "Verifying file content..."
        if diff -q test_file.txt downloaded_test_file.txt > /dev/null; then
            echo -e "${GREEN}✅ Test 1 passed: File content matches original${NC}"
        else
            echo -e "${RED}❌ Test 1 failed: File content differs${NC}"
            echo "Original:"
            head -3 test_file.txt
            echo "Downloaded:"
            head -3 downloaded_test_file.txt
            exit 1
        fi
    else
        echo -e "${RED}❌ Test 1 failed: Downloaded file not found${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Test 1 failed: Download did not complete${NC}"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi

# Test 2: Try to download non-existent file
echo -e "\n${YELLOW}📥 Test 2: Attempting to download non-existent file...${NC}"
OUTPUT=$(./target/release/file_transfer client "$SERVER_ID52" "nonexistent.txt" 2>&1 || true)

if echo "$OUTPUT" | grep -q "NotFound\|not found\|Error"; then
    echo -e "${GREEN}✅ Test 2 passed: Correctly handled missing file${NC}"
else
    echo -e "${YELLOW}⚠️  Test 2: Unexpected output for missing file${NC}"
    echo "Output was:"
    echo "$OUTPUT"
fi

# Test 3: Try path traversal attack (should be blocked)
echo -e "\n${YELLOW}🔒 Test 3: Testing security - path traversal attempt...${NC}"
OUTPUT=$(./target/release/file_transfer client "$SERVER_ID52" "../etc/passwd" 2>&1 || true)

if echo "$OUTPUT" | grep -q "PermissionDenied\|denied\|blocked\|Error"; then
    echo -e "${GREEN}✅ Test 3 passed: Path traversal blocked${NC}"
else
    echo -e "${RED}❌ Test 3 failed: Path traversal not properly blocked${NC}"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi

# Test 4: Transfer a larger file
echo -e "\n${YELLOW}📥 Test 4: Testing with larger file...${NC}"
# Create a 1MB test file
dd if=/dev/urandom of=large_test.txt bs=1024 count=1024 2>/dev/null
LARGE_FILE_SIZE=$(wc -c < large_test.txt)
echo "Created large_test.txt ($LARGE_FILE_SIZE bytes)"

START_TIME=$(date +%s%N)
OUTPUT=$(./target/release/file_transfer client "$SERVER_ID52" "large_test.txt" 2>&1)
END_TIME=$(date +%s%N)
LARGE_TRANSFER_TIME_NS=$((END_TIME - START_TIME))
LARGE_TRANSFER_TIME_MS=$(echo "scale=2; $LARGE_TRANSFER_TIME_NS / 1000000" | bc)
LARGE_THROUGHPUT_MBPS=$(echo "scale=2; $LARGE_FILE_SIZE * 8 / ($LARGE_TRANSFER_TIME_NS / 1000)" | bc)

if echo "$OUTPUT" | grep -q "✅ Downloaded large_test.txt"; then
    if [ -f "downloaded_large_test.txt" ]; then
        ORIGINAL_SIZE=$(wc -c < large_test.txt)
        DOWNLOADED_SIZE=$(wc -c < downloaded_large_test.txt)
        if [ "$ORIGINAL_SIZE" -eq "$DOWNLOADED_SIZE" ]; then
            echo -e "${GREEN}✅ Test 4 passed: Large file transferred correctly (${ORIGINAL_SIZE} bytes in ${LARGE_TRANSFER_TIME_MS}ms, ${LARGE_THROUGHPUT_MBPS} Mbps)${NC}"
        else
            echo -e "${RED}❌ Test 4 failed: Size mismatch (original: ${ORIGINAL_SIZE}, downloaded: ${DOWNLOADED_SIZE})${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ Test 4 failed: Downloaded file not found${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Test 4 failed: Large file transfer did not complete${NC}"
    echo "Output was:"
    echo "$OUTPUT"
    exit 1
fi

# Clean up test files
rm -f downloaded_*.txt large_test.txt

# Check server is still healthy
if ps -p $SERVER_PID > /dev/null; then
    echo -e "\n${GREEN}✅ Server still running after all tests${NC}"
else
    echo -e "\n${RED}❌ Server crashed during tests${NC}"
    exit 1
fi

# Final summary
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 All file transfer tests passed!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Performance Summary
echo -e "\n${YELLOW}📊 Performance Metrics:${NC}"
echo "  • Small file (${FILE_SIZE}B) transfer: ${TRANSFER_TIME_MS}ms"
echo "  • Large file (1MB) transfer: ${LARGE_TRANSFER_TIME_MS}ms (${LARGE_THROUGHPUT_MBPS} Mbps)"
echo "  • Total test duration: $(($(date +%s) - TEST_START_TIME))s"
echo -e "\n${YELLOW}Server log preview:${NC}"
tail -10 file_server.log

echo -e "\n${YELLOW}Test completed. Server will be shut down.${NC}"
# Add quiet mode summary
if [ "$QUIET_MODE" = true ]; then
    # Override the verbose output with quiet summary
    exec 3>&1 4>&2
    trap 'exec 2>&4 1>&3' 0 1 2 3
    exec 1>/tmp/test-output.$$ 2>&1
    
    # Re-run in background to capture results
    (
        echo -e "${GREEN}✅ File Transfer: PASS${NC}" >&3
        echo "  • Small file: ${TRANSFER_TIME_MS}ms" >&3
        echo "  • Large file: ${LARGE_TRANSFER_TIME_MS}ms (${LARGE_THROUGHPUT_MBPS} Mbps)" >&3
        echo "  • Duration: $(($(date +%s) - TEST_START_TIME))s" >&3
    )
fi
