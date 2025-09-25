#!/bin/bash
# End-to-end test for request_response with dual-daemon P2P architecture

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration - unique directories per test run
TEST_NAME="request-response-e2e-test"
TEST_ID=$(date +%s)_$$
ALICE_HOME="/tmp/fastn-alice-${TEST_ID}"
BOB_HOME="/tmp/fastn-bob-${TEST_ID}"

cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up dual-daemon test environment...${NC}"
    
    # Kill any running processes
    pkill -f "fastn-p2p daemon" || true
    pkill -f "request_response server" || true
    
    # Clean up test directories
    rm -rf "$ALICE_HOME" "$BOB_HOME"
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set up cleanup trap
trap cleanup EXIT

echo -e "${GREEN}🚀 Starting request_response dual-daemon P2P test${NC}"
echo -e "${BLUE}   Alice (server): $ALICE_HOME${NC}"
echo -e "${BLUE}   Bob (client):   $BOB_HOME${NC}"

# Clean up any previous test runs
cleanup

# Create test directories
mkdir -p "$ALICE_HOME" "$BOB_HOME"

echo -e "${YELLOW}📦 Building binaries...${NC}"
cargo build --bin fastn-p2p
cargo build --bin request_response

echo -e "${YELLOW}🎧 Starting Alice's daemon...${NC}"
FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- daemon &
ALICE_DAEMON_PID=$!

echo -e "${YELLOW}🎧 Starting Bob's daemon...${NC}"
FASTN_HOME="$BOB_HOME" cargo run --bin fastn-p2p -- daemon &
BOB_DAEMON_PID=$!

# Wait for daemons to start
sleep 3

echo -e "${YELLOW}🔍 Checking daemon status...${NC}"
if [ -S "$ALICE_HOME/control.sock" ]; then
    echo -e "${GREEN}✅ Alice's daemon running${NC}"
else
    echo -e "${RED}❌ Alice's daemon failed to start${NC}"
    exit 1
fi

if [ -S "$BOB_HOME/control.sock" ]; then
    echo -e "${GREEN}✅ Bob's daemon running${NC}"
else
    echo -e "${RED}❌ Bob's daemon failed to start${NC}"
    exit 1
fi

echo -e "${YELLOW}🔑 Setting up identities...${NC}"

# Create Alice identity
FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- create-identity alice
echo -e "${GREEN}✅ Alice identity created${NC}"

# Create Bob identity  
FASTN_HOME="$BOB_HOME" cargo run --bin fastn-p2p -- create-identity bob
echo -e "${GREEN}✅ Bob identity created${NC}"

# Add Echo protocol to Alice (she'll be the server)
FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- add-protocol alice --protocol Echo --config '{"max_message_length": 1000}'
echo -e "${GREEN}✅ Echo protocol added to Alice${NC}"

# Set identities online
FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- identity-online alice
FASTN_HOME="$BOB_HOME" cargo run --bin fastn-p2p -- identity-online bob
echo -e "${GREEN}✅ Identities set online${NC}"

# Check status
echo -e "${YELLOW}📊 Alice's status:${NC}"
FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- status

echo -e "${YELLOW}📊 Bob's status:${NC}"
FASTN_HOME="$BOB_HOME" cargo run --bin fastn-p2p -- status

# Get Alice's peer ID for Bob to connect to
ALICE_ID52=$(FASTN_HOME="$ALICE_HOME" cargo run --bin fastn-p2p -- status | grep -o 'alice (ONLINE) - [a-z0-9]*' | grep -o '[a-z0-9]*$')
echo -e "${BLUE}🔑 Alice's peer ID: $ALICE_ID52${NC}"

echo -e "${YELLOW}🎧 Starting Alice's Echo server...${NC}"
FASTN_HOME="$ALICE_HOME" cargo run --bin request_response -- server alice &
ALICE_SERVER_PID=$!

# Wait for server to start
sleep 2

echo -e "${YELLOW}📞 Testing Bob → Alice P2P communication...${NC}"
echo -e "${BLUE}   Bob will send Echo request to Alice via real P2P${NC}"

# Test the P2P call
FASTN_HOME="$BOB_HOME" cargo run --bin request_response -- client "$ALICE_ID52" "Hello Alice from Bob via P2P!"

echo -e "${GREEN}🎉 Request/Response dual-daemon P2P test completed!${NC}"
echo -e "${GREEN}   - Two daemons running with separate identities${NC}"
echo -e "${GREEN}   - Real P2P communication between Alice and Bob${NC}"
echo -e "${GREEN}   - Echo protocol server and client working${NC}"
echo -e "${GREEN}   - Complete end-to-end validation successful${NC}"