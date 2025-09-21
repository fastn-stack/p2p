#!/bin/bash

# End-to-end test suite for fastn-p2p
# Runs all integration tests to verify P2P functionality
# Usage: ./e2e-test.sh [--retry]

set -e  # Exit on error

# Check for retry flag to pass to individual tests
RETRY_FLAG=""
if [[ "$1" == "--retry" ]]; then
    RETRY_FLAG="--retry"
    echo "Note: Retry mode enabled for all tests"
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'

echo -e "${BOLD}${YELLOW}🧪 fastn-p2p End-to-End Test Suite${NC}"
echo "========================================"
echo ""

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test script
run_test() {
    local test_name=$1
    local test_script=$2
    
    echo -e "${YELLOW}📦 Running ${test_name}...${NC}"
    echo "----------------------------------------"
    
    if bash "$test_script" $RETRY_FLAG; then
        echo -e "${GREEN}✅ ${test_name} PASSED${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ ${test_name} FAILED${NC}"
        ((TESTS_FAILED++))
    fi
    
    echo ""
}

# Pre-build everything once to save time
echo -e "${YELLOW}📦 Pre-building all binaries...${NC}"
cargo build --release --all 2>&1 | tail -5
echo -e "${GREEN}✅ Build complete${NC}"
echo ""

# Run all test suites
run_test "Request-Response Pattern" "./test-request-response.sh"
run_test "File Transfer (Streaming)" "./test-file-transfer.sh"
if [ -f "./test-shell-simple.sh" ]; then
    run_test "Remote Shell (Simple)" "./test-shell-simple.sh"
fi

# Summary
echo "========================================"
echo -e "${BOLD}📊 Test Summary${NC}"
echo "----------------------------------------"
echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}${BOLD}🎉 All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}${BOLD}⚠️  Some tests failed${NC}"
    exit 1
fi