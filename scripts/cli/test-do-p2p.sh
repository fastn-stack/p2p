#!/bin/bash
# Comprehensive P2P test: local machine + Digital Ocean droplet (default) or dual droplets (CI)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
MODE="single"  # Default: local + 1 droplet
TEST_ID=$(date +%s)_$$
LOCAL_HOME="/tmp/fastn-local-${TEST_ID}"
DROPLET_SIZE="s-1vcpu-1gb"
DROPLET_REGION="nyc3"
DROPLET_IMAGE="ubuntu-22-04-x64"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dual)
            MODE="dual"
            shift
            ;;
        --single)
            MODE="single"
            shift
            ;;
        *)
            echo "Usage: $0 [--single|--dual]"
            echo "  --single: Local machine + 1 Digital Ocean droplet (default)"
            echo "  --dual:   2 Digital Ocean droplets (for CI)"
            exit 1
            ;;
    esac
done

cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up test environment...${NC}"
    
    # Kill local processes
    pkill -f "fastn-p2p daemon" || true
    pkill -f "request_response" || true
    
    # Clean up local directories
    rm -rf "$LOCAL_HOME"
    
    # Clean up droplets
    if [ -n "$ALICE_DROPLET_ID" ]; then
        echo -e "${YELLOW}🗑️  Destroying alice droplet: $ALICE_DROPLET_ID${NC}"
        doctl compute droplet delete "$ALICE_DROPLET_ID" --force || true
    fi
    
    if [ -n "$BOB_DROPLET_ID" ]; then
        echo -e "${YELLOW}🗑️  Destroying bob droplet: $BOB_DROPLET_ID${NC}"
        doctl compute droplet delete "$BOB_DROPLET_ID" --force || true
    fi
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Set up cleanup trap
trap cleanup EXIT

echo -e "${GREEN}🚀 Starting P2P test in ${MODE} mode${NC}"

# Build binaries
echo -e "${YELLOW}📦 Building binaries...${NC}"
cargo build --release --bin fastn-p2p
cargo build --release --bin request_response

if [ "$MODE" = "single" ]; then
    echo -e "${BLUE}🏠 Single mode: Local machine (bob) + Digital Ocean droplet (alice)${NC}"
    
    # Set up local environment for bob
    mkdir -p "$LOCAL_HOME"
    
    echo -e "${YELLOW}🎧 Starting local daemon (bob)...${NC}"
    FASTN_HOME="$LOCAL_HOME" ./target/release/fastn-p2p daemon &
    LOCAL_DAEMON_PID=$!
    sleep 3
    
    # Create bob identity locally
    FASTN_HOME="$LOCAL_HOME" ./target/release/fastn-p2p create-identity bob
    FASTN_HOME="$LOCAL_HOME" ./target/release/fastn-p2p identity-online bob
    echo -e "${GREEN}✅ Bob identity created locally${NC}"
    
    # Create alice droplet (server)
    echo -e "${YELLOW}🌊 Creating alice droplet (server)...${NC}"
    ALICE_DROPLET_ID=$(doctl compute droplet create \
        fastn-alice-test-${TEST_ID} \
        --size $DROPLET_SIZE \
        --region $DROPLET_REGION \
        --image $DROPLET_IMAGE \
        --ssh-keys $(doctl compute ssh-key list --format FingerPrint --no-header | head -1) \
        --wait \
        --format ID \
        --no-header)
    
    ALICE_IP=$(doctl compute droplet get $ALICE_DROPLET_ID --format PublicIPv4 --no-header)
    echo -e "${GREEN}✅ Alice droplet created: $ALICE_DROPLET_ID ($ALICE_IP)${NC}"
    
    # Wait for droplet to be ready
    echo -e "${YELLOW}⏳ Waiting for alice droplet to be ready...${NC}"
    sleep 60
    
    # Deploy to alice droplet
    echo -e "${YELLOW}📦 Deploying to alice droplet...${NC}"
    scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
        target/release/fastn-p2p target/release/request_response \
        root@$ALICE_IP:/root/
    
    # Setup alice server
    ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
        root@$ALICE_IP << 'EOF'
    # Install dependencies
    apt-get update
    apt-get install -y build-essential
    
    # Start alice's daemon
    export FASTN_HOME=/root/alice
    ./fastn-p2p daemon &
    sleep 5
    
    # Create alice identity and setup Echo server
    ./fastn-p2p create-identity alice
    ./fastn-p2p add-protocol alice --protocol Echo --config '{"max_message_length": 1000}'
    ./fastn-p2p identity-online alice
    
    # Get alice's peer ID
    ALICE_ID52=$(./fastn-p2p status | grep -o 'alice (ONLINE) - [a-z0-9]*' | grep -o '[a-z0-9]*$')
    echo "ALICE_PEER_ID=$ALICE_ID52" > alice_peer_id.txt
    
    # Start Echo server
    ./request_response server alice &
    
    echo "🟢 Alice server ready, peer ID: $ALICE_ID52"
EOF
    
    # Get alice's peer ID
    ALICE_PEER_ID=$(ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
        root@$ALICE_IP "cat alice_peer_id.txt | cut -d'=' -f2")
    echo -e "${BLUE}🔑 Alice's peer ID: $ALICE_PEER_ID${NC}"
    
    # Test P2P communication: local bob → remote alice
    echo -e "${PURPLE}📞 Testing local → remote P2P communication...${NC}"
    echo -e "${BLUE}   Local bob will send request to remote alice via real internet P2P${NC}"
    
    FASTN_HOME="$LOCAL_HOME" ./target/release/request_response client "$ALICE_PEER_ID" "Hello Alice from local Bob via real internet P2P!"
    
    echo -e "${GREEN}🎉 Single-mode P2P test completed successfully!${NC}"
    echo -e "${GREEN}   - Local daemon (bob) ↔ Remote droplet (alice)${NC}"
    echo -e "${GREEN}   - Real internet P2P communication validated${NC}"
    
else
    echo -e "${BLUE}☁️  Dual mode: Two Digital Ocean droplets (alice + bob)${NC}"
    
    # Create both droplets for CI mode
    echo -e "${YELLOW}🌊 Creating alice droplet (server)...${NC}"
    ALICE_DROPLET_ID=$(doctl compute droplet create \
        fastn-alice-test-${TEST_ID} \
        --size $DROPLET_SIZE \
        --region $DROPLET_REGION \
        --image $DROPLET_IMAGE \
        --ssh-keys $(doctl compute ssh-key list --format FingerPrint --no-header | head -1) \
        --wait \
        --format ID \
        --no-header)
    
    echo -e "${YELLOW}🌊 Creating bob droplet (client)...${NC}"  
    BOB_DROPLET_ID=$(doctl compute droplet create \
        fastn-bob-test-${TEST_ID} \
        --size $DROPLET_SIZE \
        --region $DROPLET_REGION \
        --image $DROPLET_IMAGE \
        --ssh-keys $(doctl compute ssh-key list --format FingerPrint --no-header | head -1) \
        --wait \
        --format ID \
        --no-header)
    
    ALICE_IP=$(doctl compute droplet get $ALICE_DROPLET_ID --format PublicIPv4 --no-header)
    BOB_IP=$(doctl compute droplet get $BOB_DROPLET_ID --format PublicIPv4 --no-header)
    
    echo -e "${GREEN}✅ Droplets created:${NC}"
    echo -e "${GREEN}   Alice: $ALICE_DROPLET_ID ($ALICE_IP)${NC}"
    echo -e "${GREEN}   Bob:   $BOB_DROPLET_ID ($BOB_IP)${NC}"
    
    # Continue with dual-droplet setup...
    echo -e "${PURPLE}📞 Dual-droplet P2P testing not fully implemented yet${NC}"
    echo -e "${BLUE}   Use --single mode for now${NC}"
fi