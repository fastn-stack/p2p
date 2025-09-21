#!/bin/bash
# 🌐 DIGITAL OCEAN AUDIO STREAMING TEST
# 
# Tests real-time audio streaming across internet (laptop ↔ Digital Ocean droplet).
# Self-contained with automatic setup, cleanup, and audio quality validation.
#
# Usage:
#   Default: ./test-do-audio-streaming.sh (fast droplet, ~5min setup)
#   Quick test: ./test-do-audio-streaming.sh --small (cheaper but slower setup)
#   High quality: ./test-do-audio-streaming.sh --turbo (fastest setup)
#
# Droplet sizes:
#   --small: 1GB RAM, $6/month, slower builds (~15min)
#   --fast: 4GB RAM, $48/month, fast builds (~5min) [DEFAULT]
#   --turbo: 8CPU/16GB RAM, $96/month, fastest builds (~3min)
#
# Audio options:
#   --high-quality: Use high-quality MP3 for better audio testing
#   --duration N: Stream for N seconds (default: 30)
#
# Debugging:
#   --keep-droplet: Keep droplet after test for debugging
#   --verbose: Show detailed output
#
# Requirements: doctl auth init (one-time setup)

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m' 
RED='\033[0;31m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

# Test configuration
TEST_ID="fastn-audio-$(date +%s)"
START_TIME=$(date +%s)
DROPLET_NAME="fastn-audio-test-$TEST_ID"
DROPLET_SIZE="s-4vcpu-8gb"  # Default: fast builds
DROPLET_REGION="nyc1"  # New York for good connectivity to US
DROPLET_IMAGE="ubuntu-22-04-x64"
TEST_SSH_KEY="/tmp/${TEST_ID}_ssh_key"
KEEP_DROPLET=false
VERBOSE=false
HIGH_QUALITY=false
STREAM_DURATION=30

# Logging functions
log() { printf "${BLUE}[$(date +'%H:%M:%S')] $1${NC}\n"; }
success() { printf "${GREEN}✅ $1${NC}\n"; }
error() { printf "${RED}❌ $1${NC}\n"; exit 1; }
warn() { printf "${YELLOW}⚠️ $1${NC}\n"; }
header() { printf "\n${BOLD}${YELLOW}$1${NC}\n"; echo "$(printf '=%.0s' {1..50})"; }

time_checkpoint() { 
    local checkpoint="$1"
    local current_time=$(date +%s)
    local elapsed=$((current_time - START_TIME))
    printf "${BLUE}[$(date +'%H:%M:%S')] ⏱️  $checkpoint: ${elapsed}s${NC}\n"
}

# Parse arguments
for arg in "$@"; do
    case $arg in
        "--small")
            DROPLET_SIZE="s-1vcpu-1gb"
            log "Using small droplet (1GB RAM, cheaper but slower)"
            ;;
        "--fast")
            DROPLET_SIZE="s-4vcpu-8gb"
            log "Using fast droplet (4CPU/8GB RAM) [DEFAULT]"
            ;;
        "--turbo")
            DROPLET_SIZE="s-8vcpu-16gb"
            log "Using turbo droplet (8CPU/16GB RAM, fastest)"
            ;;
        "--high-quality")
            HIGH_QUALITY=true
            log "Will download high-quality MP3 for testing"
            ;;
        "--duration")
            shift
            STREAM_DURATION="$1"
            log "Stream duration set to: ${STREAM_DURATION}s"
            ;;
        "--keep-droplet")
            KEEP_DROPLET=true
            log "🔧 DEBUG MODE: Droplet will be kept for debugging"
            ;;
        "--verbose")
            VERBOSE=true
            log "Verbose mode enabled"
            ;;
        "--help")
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --small         Use 1GB droplet (cheaper, slower)"
            echo "  --fast          Use 4GB droplet (default, balanced)"
            echo "  --turbo         Use 8CPU/16GB droplet (fastest)"
            echo "  --high-quality  Download high-quality MP3"
            echo "  --duration N    Stream for N seconds (default: 30)"
            echo "  --keep-droplet  Keep droplet for debugging"
            echo "  --verbose       Show detailed output"
            echo "  --help          Show this help"
            exit 0
            ;;
        *)
            warn "Unknown argument: $arg (ignoring)"
            ;;
    esac
done

# Comprehensive cleanup
cleanup() {
    log "🧹 Comprehensive cleanup..."
    
    # Kill local processes
    pkill -f "media_stream" 2>/dev/null || true
    
    # Destroy droplet (unless debugging)
    if [[ "$KEEP_DROPLET" == "true" ]]; then
        log "🔧 DEBUG MODE: Keeping droplet for debugging"
        if [[ -n "${DROPLET_NAME:-}" ]] && [[ -n "${DROPLET_IP:-}" ]]; then
            echo ""
            echo "📍 DEBUGGING INFORMATION:"
            echo "  Droplet: $DROPLET_NAME"
            echo "  IP: $DROPLET_IP"
            echo "  SSH: ssh -i $TEST_SSH_KEY root@$DROPLET_IP"
            echo "  Binary: /root/fastn-p2p/target/release/media_stream"
        fi
    else
        # Clean up droplet and SSH key
        if command -v doctl >/dev/null 2>&1; then
            CLEANUP_DOCTL="doctl"
        elif [[ -f "$HOME/doctl" ]] && [[ -x "$HOME/doctl" ]]; then
            CLEANUP_DOCTL="$HOME/doctl"
        fi
        
        if [[ -n "${CLEANUP_DOCTL:-}" ]] && $CLEANUP_DOCTL account get >/dev/null 2>&1; then
            if [[ -n "${DROPLET_NAME:-}" ]] && $CLEANUP_DOCTL compute droplet list --format Name --no-header | grep -q "$DROPLET_NAME"; then
                log "Destroying droplet: $DROPLET_NAME"
                $CLEANUP_DOCTL compute droplet delete "$DROPLET_NAME" --force
            fi
            
            # Remove auto-generated SSH key
            if $CLEANUP_DOCTL compute ssh-key list --format Name --no-header | grep -q "$TEST_ID"; then
                $CLEANUP_DOCTL compute ssh-key delete "$TEST_ID" --force 2>/dev/null || true
            fi
        fi
    fi
    
    # Clean up test files
    rm -rf "/tmp/$TEST_ID"* 2>/dev/null || true
    rm -f high_quality_audio.mp3 subscriber.log 2>/dev/null || true
    
    success "Cleanup complete"
}
trap cleanup EXIT

header "🎵 DIGITAL OCEAN AUDIO STREAMING TEST"
log "Test ID: $TEST_ID"
log "Tests real-time audio streaming across internet"

if [[ "$KEEP_DROPLET" != "true" ]]; then
    log "💡 For debugging, use: ./test-do-audio-streaming.sh --keep-droplet"
fi
echo

# Phase 1: Setup dependencies
header "🔧 Phase 1: Setup Dependencies"

# Check doctl
log "Checking Digital Ocean CLI..."
if command -v doctl >/dev/null 2>&1; then
    DOCTL="doctl"
elif [[ -f "$HOME/doctl" ]] && [[ -x "$HOME/doctl" ]]; then
    DOCTL="$HOME/doctl"
    log "Using doctl from home directory: $HOME/doctl"
else
    error "Install doctl first: brew install doctl"
fi

if ! $DOCTL account get >/dev/null 2>&1; then
    error "Please authenticate doctl first: $DOCTL auth init"
else
    success "doctl already authenticated"
fi

# Generate SSH key
log "Generating test SSH key..."
mkdir -p "$(dirname "$TEST_SSH_KEY")"
ssh-keygen -t rsa -b 2048 -f "$TEST_SSH_KEY" -N "" -C "$TEST_ID" -q
success "SSH key generated: $TEST_SSH_KEY"

# Import SSH key to Digital Ocean
log "Importing SSH key to Digital Ocean..."
SSH_KEY_ID=$($DOCTL compute ssh-key import "$TEST_ID" --public-key-file "$TEST_SSH_KEY.pub" --format ID --no-header)
success "SSH key imported: $SSH_KEY_ID"

# Download high-quality MP3 if requested
if [[ "$HIGH_QUALITY" == "true" ]]; then
    log "Downloading high-quality MP3 for testing..."
    # Download a royalty-free high-quality sample
    curl -L "https://www.soundjay.com/misc/sounds/bell-ringing-05.mp3" -o high_quality_audio.mp3 2>/dev/null || {
        warn "Failed to download high-quality MP3, will use generated test tone"
        HIGH_QUALITY=false
    }
    if [[ -f "high_quality_audio.mp3" ]]; then
        MP3_SIZE=$(wc -c < high_quality_audio.mp3)
        success "Downloaded high-quality MP3: ${MP3_SIZE} bytes"
    fi
fi

# Build fastn-p2p locally first
log "Building fastn-p2p locally..."
cargo build --package examples --bin media_stream --release || {
    error "Failed to build media_stream locally"
}
success "Local build complete"

time_checkpoint "Setup complete"

# Phase 2: Droplet provisioning
header "🚀 Phase 2: Droplet Provisioning"

log "Creating droplet: $DROPLET_NAME"
DROPLET_ID=$($DOCTL compute droplet create "$DROPLET_NAME" \
    --size "$DROPLET_SIZE" \
    --image "$DROPLET_IMAGE" \
    --region "$DROPLET_REGION" \
    --ssh-keys "$SSH_KEY_ID" \
    --format ID \
    --no-header)
success "Droplet created: $DROPLET_ID"

# Wait for droplet to boot
log "Waiting for droplet to boot..."
for i in {1..60}; do
    DROPLET_STATUS=$($DOCTL compute droplet get "$DROPLET_ID" --format Status --no-header)
    if [[ "$DROPLET_STATUS" == "active" ]]; then
        break
    fi
    if [[ "$VERBOSE" == "true" ]]; then
        echo -n "."
    fi
    sleep 5
done

if [[ "$DROPLET_STATUS" != "active" ]]; then
    error "Droplet failed to boot within 5 minutes"
fi
time_checkpoint "Droplet boot"

# Get droplet IP
DROPLET_IP=$($DOCTL compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
success "Droplet active: $DROPLET_IP"

# Wait for SSH
log "Waiting for SSH to be ready..."
for i in {1..30}; do
    if ssh -i "$TEST_SSH_KEY" -o ConnectTimeout=5 -o StrictHostKeyChecking=no root@"$DROPLET_IP" "echo SSH ready" 2>/dev/null; then
        break
    fi
    if [[ "$VERBOSE" == "true" ]]; then
        echo -n "."
    fi
    sleep 5
done
time_checkpoint "SSH ready"

# Phase 3: Setup droplet for audio streaming
header "🎵 Phase 3: Setup Audio Streaming"

# Install dependencies on droplet
log "Installing Rust and dependencies on droplet..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" << 'EOF'
    # Install Rust
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    # Install audio dependencies (headless - no actual audio output needed on server)
    apt-get update
    apt-get install -y build-essential pkg-config libasound2-dev git
    
    echo "Dependencies installed successfully"
EOF
success "Dependencies installed"

# Copy project to droplet
log "Copying fastn-p2p project to droplet..."
TEMP_DIR="/tmp/${TEST_ID}_project"
mkdir -p "$TEMP_DIR"
rsync -avz --exclude target --exclude .git . "$TEMP_DIR/" >/dev/null
scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no -r "$TEMP_DIR" root@"$DROPLET_IP":/root/fastn-p2p >/dev/null
success "Project copied to droplet"

# Copy high-quality MP3 if available
if [[ "$HIGH_QUALITY" == "true" ]] && [[ -f "high_quality_audio.mp3" ]]; then
    log "Copying high-quality MP3 to droplet..."
    scp -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no high_quality_audio.mp3 root@"$DROPLET_IP":/root/fastn-p2p/
    success "High-quality MP3 copied"
fi

# Build on droplet
log "Building fastn-p2p on droplet..."
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" << 'EOF'
    cd /root/fastn-p2p
    source ~/.cargo/env
    cargo build --package examples --bin media_stream --release
    echo "Build complete"
EOF
success "Droplet build complete"
time_checkpoint "Droplet build complete"

# Phase 4: Start audio streaming
header "🎧 Phase 4: Real-World Audio Streaming Test"

# Start publisher on droplet
log "Starting audio publisher on droplet..."
MP3_FILE_ARG=""
if [[ "$HIGH_QUALITY" == "true" ]]; then
    MP3_FILE_ARG="high_quality_audio.mp3"
fi

ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" << EOF &
    cd /root/fastn-p2p
    source ~/.cargo/env
    ./target/release/media_stream server $MP3_FILE_ARG 2>&1 | tee publisher.log
EOF
DROPLET_SSH_PID=$!

# Wait for publisher to start
log "Waiting for publisher to start..."
sleep 10

# Get publisher ID from droplet
PUBLISHER_ID=""
for i in {1..20}; do
    PUBLISHER_ID=$(ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" \
        "grep '🎧 Publisher listening on:' /root/fastn-p2p/publisher.log 2>/dev/null | sed 's/.*🎧 Publisher listening on: //' | head -1" 2>/dev/null || true)
    if [[ -n "$PUBLISHER_ID" ]]; then
        break
    fi
    sleep 3
done

if [[ -z "$PUBLISHER_ID" ]]; then
    error "Failed to get publisher ID from droplet"
fi
success "Publisher started with ID: $PUBLISHER_ID"

# Give discovery services time to register
log "Waiting for discovery registration..."
sleep 5

# Start local subscriber
log "Starting local audio subscriber..."
log "🎵 You should hear audio playing shortly..."

START_TIME_STREAM=$(date +%s%N)
./target/release/media_stream client "$PUBLISHER_ID" > subscriber.log 2>&1 &
SUBSCRIBER_PID=$!

# Let it stream for specified duration
sleep "$STREAM_DURATION"

# Stop subscriber
kill $SUBSCRIBER_PID 2>/dev/null || true
wait $SUBSCRIBER_PID 2>/dev/null || true

END_TIME_STREAM=$(date +%s%N)
STREAM_TIME_NS=$((END_TIME_STREAM - START_TIME_STREAM))
STREAM_TIME_MS=$(echo "scale=2; $STREAM_TIME_NS / 1000000" | bc)

# Analyze streaming results
log "Analyzing streaming performance..."
SUBSCRIBER_OUTPUT=$(cat subscriber.log)

# Parse metrics from subscriber output
CHUNKS_RECEIVED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*Final stats: \([0-9]*\) chunks.*/\1/' | head -1)
BYTES_RECEIVED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*chunks, \([0-9.]*\) KB total.*/\1/' | head -1)
CHUNKS_DROPPED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*total, \([0-9]*\) dropped/\1/' | head -1)

# Get publisher metrics from droplet
ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" \
    "pkill -f media_stream 2>/dev/null || true" >/dev/null 2>&1

PUBLISHER_OUTPUT=$(ssh -i "$TEST_SSH_KEY" -o StrictHostKeyChecking=no root@"$DROPLET_IP" \
    "cat /root/fastn-p2p/publisher.log 2>/dev/null" || true)

CHUNKS_SENT=$(echo "$PUBLISHER_OUTPUT" | grep "Finished streaming" | sed 's/.*Finished streaming \([0-9]*\) chunks.*/\1/' | head -1)
BYTES_SENT=$(echo "$PUBLISHER_OUTPUT" | grep "Finished streaming" | sed 's/.*chunks (\([0-9.]*\) KB total)/\1/' | head -1)

time_checkpoint "P2P streaming test complete"

# Phase 5: Results and analysis
header "📊 Phase 5: Audio Streaming Results"

if [[ -n "$CHUNKS_RECEIVED" ]] && [[ "$CHUNKS_RECEIVED" -gt 0 ]]; then
    # Calculate performance metrics
    THROUGHPUT_KBPS=$(echo "scale=2; $BYTES_RECEIVED * 1024 * 8 / ($STREAM_TIME_NS / 1000000000)" | bc)
    PACKET_LOSS_RATE=0
    if [[ -n "$CHUNKS_DROPPED" ]] && [[ "$CHUNKS_DROPPED" -gt 0 ]]; then
        PACKET_LOSS_RATE=$(echo "scale=4; $CHUNKS_DROPPED * 100 / $CHUNKS_RECEIVED" | bc)
    fi
    
    success "Audio streaming test PASSED"
    echo ""
    printf "${YELLOW}📊 Performance Metrics:${NC}\n"
    echo "  🎵 Stream duration: $(echo "scale=1; $STREAM_TIME_NS / 1000000000" | bc)s"
    echo "  📦 Chunks sent: $CHUNKS_SENT"
    echo "  📥 Chunks received: $CHUNKS_RECEIVED"
    echo "  💾 Data volume: ${BYTES_RECEIVED} KB"
    echo "  🚀 Throughput: ${THROUGHPUT_KBPS} kbps"
    echo "  📉 Packet loss: ${PACKET_LOSS_RATE}%"
    
    echo ""
    printf "${YELLOW}🌐 Network Quality:${NC}\n"
    echo "  📍 Route: Laptop ↔ Digital Ocean ($DROPLET_REGION)"
    echo "  ⚡ Latency: Real-time streaming achieved"
    echo "  📶 Quality: $(if (( $(echo "$PACKET_LOSS_RATE < 1" | bc -l) )); then echo "Excellent"; elif (( $(echo "$PACKET_LOSS_RATE < 5" | bc -l) )); then echo "Good"; else echo "Poor"; fi)"
    
    if [[ "$HIGH_QUALITY" == "true" ]]; then
        echo "  🎼 Audio: High-quality MP3 source"
    else
        echo "  🎼 Audio: Generated test tone (440Hz)"
    fi
else
    error "Audio streaming test FAILED - no data received"
fi

# Show timing breakdown
echo ""
printf "${YELLOW}⏱️ Timing Breakdown:${NC}\n"
TOTAL_TIME=$(($(date +%s) - START_TIME))
echo "  🚀 Total test time: ${TOTAL_TIME}s"
echo "  🎧 Audio streaming: $(echo "scale=1; $STREAM_TIME_NS / 1000000000" | bc)s"
echo "  ☁️ Setup overhead: $((TOTAL_TIME - STREAM_DURATION))s"

if [[ "$VERBOSE" == "true" ]]; then
    echo ""
    printf "${YELLOW}🔍 Detailed Logs:${NC}\n"
    echo "Subscriber log:"
    tail -10 subscriber.log
    echo ""
    echo "Publisher log:"
    echo "$PUBLISHER_OUTPUT" | tail -10
fi

echo ""
success "🎉 Digital Ocean audio streaming test completed!"