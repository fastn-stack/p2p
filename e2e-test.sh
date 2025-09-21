#!/bin/bash

# End-to-end test suite for fastn-p2p
# Runs all integration tests to verify P2P functionality
# Usage: ./e2e-test.sh [--quiet] [--retry]
#   --quiet: Minimal output (only show results and performance)
#   --retry: Enable retry on discovery failures

set -e  # Exit on error

# Parse command line arguments
QUIET_FLAG=""
RETRY_FLAG=""

for arg in "$@"; do
    case $arg in
        --quiet|-q)
            QUIET_FLAG="--quiet"
            ;;
        --retry)
            RETRY_FLAG="--retry"
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

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color
BOLD='\033[1m'

if [ -z "$QUIET_FLAG" ]; then
    echo -e "${BOLD}${YELLOW}🧪 fastn-p2p End-to-End Test Suite${NC}"
    echo "========================================"
    echo ""
fi

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0

# Function to run a test script
run_test() {
    local test_name=$1
    local test_script=$2
    
    if [ -z "$QUIET_FLAG" ]; then
        echo -e "${YELLOW}📦 Running ${test_name}...${NC}"
        echo "----------------------------------------"
    fi
    
    if bash "$test_script" $QUIET_FLAG $RETRY_FLAG; then
        if [ -z "$QUIET_FLAG" ]; then
            echo -e "${GREEN}✅ ${test_name} PASSED${NC}"
        fi
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ ${test_name} FAILED${NC}"
        ((TESTS_FAILED++))
    fi
    
    if [ -z "$QUIET_FLAG" ]; then
        echo ""
    fi
}

# Pre-build everything once to save time
if [ -z "$QUIET_FLAG" ]; then
    echo -e "${YELLOW}📦 Pre-building all binaries...${NC}"
    cargo build --release --all 2>&1 | tail -5
    echo -e "${GREEN}✅ Build complete${NC}"
    echo ""
else
    cargo build --release --all >/dev/null 2>&1
fi

# Run all test suites
run_test "Request-Response Pattern" "./test-request-response.sh"
run_test "File Transfer (Streaming)" "./test-file-transfer.sh"
if [ -f "./test-shell-simple.sh" ]; then
    run_test "Remote Shell (Simple)" "./test-shell-simple.sh"
fi

# Summary
if [ -n "$QUIET_FLAG" ]; then
    # Quiet mode - just show pass/fail
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed (${TESTS_PASSED}/${TESTS_PASSED})${NC}"
    else
        echo -e "${RED}❌ Tests failed (${TESTS_PASSED} passed, ${TESTS_FAILED} failed)${NC}"
    fi
else
    echo "========================================"
    echo -e "${BOLD}📊 Test Summary${NC}"
    echo "----------------------------------------"
    echo -e "Passed: ${GREEN}${TESTS_PASSED}${NC}"
    echo -e "Failed: ${RED}${TESTS_FAILED}${NC}"
fi

if [ $TESTS_FAILED -eq 0 ]; then
    if [ -z "$QUIET_FLAG" ]; then
        echo ""
        echo -e "${GREEN}${BOLD}🎉 All tests passed!${NC}"
        echo ""
        echo -e "${YELLOW}${BOLD}Note:${NC} Each test includes detailed performance metrics at the end."
        echo "Look for the 📊 Performance Metrics section in each test output above."
    fi
    exit 0
else
    if [ -z "$QUIET_FLAG" ]; then
        echo ""
    fi
    echo -e "${RED}${BOLD}⚠️  Some tests failed${NC}"
    exit 1
fi