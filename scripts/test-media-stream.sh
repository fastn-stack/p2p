#!/bin/bash

# Test script for media streaming example
# Tests real-time audio streaming over P2P
# Usage: ./test-media-stream.sh [--quiet|-q]
#   --quiet, -q  Minimal output (only show results and performance)

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

# Redirect output if quiet mode
if [ "$QUIET_MODE" = true ]; then
    exec 3>&1 4>&2
    exec 1>/tmp/test-media-stream-$$.log 2>&1
fi

if [ "$QUIET_MODE" = false ]; then
    echo -e "${YELLOW}🧪 Testing fastn-p2p media streaming implementation${NC}"
    echo "================================================"
fi

# Clean up function
cleanup() {
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
        if [ ! -z "$PUBLISHER_PID" ]; then
            echo "Killing publisher (PID: $PUBLISHER_PID)"
        fi
    fi
    if [ ! -z "$PUBLISHER_PID" ]; then
        kill $PUBLISHER_PID 2>/dev/null || true
    fi
    # Clean up test files
    rm -f test_audio.mp3 media_server.log subscriber.log 2>/dev/null || true
    # Clean up quiet mode log if it exists
    rm -f /tmp/test-media-stream-$$.log 2>/dev/null || true
}

# Set up trap to clean up on exit
trap cleanup EXIT INT TERM

# Pre-compilation stage
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}📦 Pre-compilation stage...${NC}"
    echo "Building media_stream example (this may take a while on first run)..."
fi

# Build everything in release mode for better performance
if [ "$QUIET_MODE" = true ]; then
    cargo build --package examples --bin media_stream --release >/dev/null 2>&1 || {
        exec 1>&3 2>&4
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    }
else
    cargo build --package examples --bin media_stream --release || {
        echo -e "${RED}❌ Failed to build media_stream example${NC}"
        exit 1
    }
    echo -e "${GREEN}✅ Pre-compilation complete${NC}"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${YELLOW}Starting media streaming tests...${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

TEST_START_TIME=$(date +%s)

# Start the media publisher
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}🎵 Starting audio publisher...${NC}"
fi
./target/release/media_stream server > media_server.log 2>&1 &
PUBLISHER_PID=$!

# Give publisher time to start and generate ID52
if [ "$QUIET_MODE" = false ]; then
    echo "Waiting for publisher to start..."
fi
for i in {1..10}; do
    if grep -q "🎧 Publisher listening on:" media_server.log 2>/dev/null; then
        if [ "$QUIET_MODE" = false ]; then
            echo "Publisher started!"
        fi
        break
    fi
    sleep 1
done

# Check if publisher is still running
if ! ps -p $PUBLISHER_PID > /dev/null; then
    if [ "$QUIET_MODE" = true ]; then
        exec 1>&3 2>&4
    fi
    echo -e "${RED}❌ Publisher process died${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Check media_server.log for details:"
        tail -20 media_server.log
    fi
    exit 1
fi

# Extract the publisher ID52 from the log
PUBLISHER_ID52=$(grep "🎧 Publisher listening on:" media_server.log | sed 's/.*🎧 Publisher listening on: //' | head -1)

if [ -z "$PUBLISHER_ID52" ]; then
    if [ "$QUIET_MODE" = true ]; then
        exec 1>&3 2>&4
    fi
    echo -e "${RED}❌ Could not find publisher ID52${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Publisher log contents:"
        cat media_server.log
    fi
    exit 1
fi

if [ "$QUIET_MODE" = false ]; then
    echo -e "${GREEN}✅ Publisher started with ID52: $PUBLISHER_ID52${NC}"
    echo "Waiting for discovery services to register publisher..."
fi
sleep 3

# Test 1: Stream audio and measure performance
if [ "$QUIET_MODE" = false ]; then
    echo -e "\n${YELLOW}🎧 Test 1: Streaming audio to subscriber...${NC}"
fi

START_TIME=$(date +%s%N)
# Run subscriber for 15 seconds then kill it
./target/release/media_stream client "$PUBLISHER_ID52" > subscriber.log 2>&1 &
SUBSCRIBER_PID=$!
sleep 15
kill $SUBSCRIBER_PID 2>/dev/null || true
wait $SUBSCRIBER_PID 2>/dev/null || true
END_TIME=$(date +%s%N)
SUBSCRIBER_OUTPUT=$(cat subscriber.log)
STREAM_TIME_NS=$((END_TIME - START_TIME))
STREAM_TIME_MS=$(echo "scale=2; $STREAM_TIME_NS / 1000000" | bc)

# Parse subscriber output for metrics
CHUNKS_RECEIVED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*Final stats: \([0-9]*\) chunks.*/\1/')
BYTES_RECEIVED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*chunks, \([0-9.]*\) KB total.*/\1/')
CHUNKS_DROPPED=$(echo "$SUBSCRIBER_OUTPUT" | grep "Final stats:" | sed 's/.*total, \([0-9]*\) dropped/\1/')

if echo "$SUBSCRIBER_OUTPUT" | grep -q "Final stats:"; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "${GREEN}✅ Audio streaming completed successfully${NC}"
        echo "   Stream duration: ${STREAM_TIME_MS}ms"
        echo "   Chunks received: $CHUNKS_RECEIVED"
        echo "   Data received: ${BYTES_RECEIVED} KB"
        echo "   Packets dropped: $CHUNKS_DROPPED"
    fi
    
    # Calculate streaming metrics
    if [ ! -z "$CHUNKS_RECEIVED" ] && [ "$CHUNKS_RECEIVED" -gt 0 ]; then
        THROUGHPUT_KBPS=$(echo "scale=2; $BYTES_RECEIVED * 1024 * 8 / ($STREAM_TIME_NS / 1000000000)" | bc)
        PACKET_LOSS_RATE=$(echo "scale=4; $CHUNKS_DROPPED * 100 / $CHUNKS_RECEIVED" | bc)
        
        if [ "$QUIET_MODE" = false ]; then
            echo "   Throughput: ${THROUGHPUT_KBPS} kbps"
            echo "   Packet loss rate: ${PACKET_LOSS_RATE}%"
        fi
    fi
else
    if [ "$QUIET_MODE" = true ]; then
        exec 1>&3 2>&4
    fi
    echo -e "${RED}❌ Audio streaming failed${NC}"
    if [ "$QUIET_MODE" = false ]; then
        echo "Subscriber output:"
        echo "$SUBSCRIBER_OUTPUT"
    fi
    exit 1
fi

# Check publisher is still healthy
if ps -p $PUBLISHER_PID > /dev/null; then
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${GREEN}✅ Publisher still running after test${NC}"
    fi
else
    if [ "$QUIET_MODE" = false ]; then
        echo -e "\n${YELLOW}⚠️ Publisher finished (expected after stream completion)${NC}"
    fi
fi

# Calculate final metrics
TEST_END_TIME=$(date +%s)
TOTAL_DURATION=$((TEST_END_TIME - TEST_START_TIME))

if [ "$QUIET_MODE" = false ]; then
    # Verbose mode - show everything
    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}🎉 Audio streaming test completed!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Performance Summary
    echo -e "\n${YELLOW}📊 Performance Metrics:${NC}"
    echo "  • Stream duration: ${STREAM_TIME_MS}ms"
    echo "  • Chunks streamed: $CHUNKS_RECEIVED"
    echo "  • Data volume: ${BYTES_RECEIVED} KB"
    echo "  • Throughput: ${THROUGHPUT_KBPS} kbps"
    echo "  • Packet loss: ${PACKET_LOSS_RATE}%"
    echo "  • Total test duration: ${TOTAL_DURATION}s"
    
    echo -e "\n${YELLOW}Publisher log preview:${NC}"
    tail -5 media_server.log

    echo -e "\n${YELLOW}Test completed. Publisher will be shut down.${NC}"
else
    # Quiet mode summary
    exec 1>&3 2>&4
    echo -e "${GREEN}✅ Media Stream: PASS${NC}"
    echo "  • Duration: ${STREAM_TIME_MS}ms"
    echo "  • Chunks: $CHUNKS_RECEIVED"
    echo "  • Throughput: ${THROUGHPUT_KBPS} kbps"
    echo "  • Loss: ${PACKET_LOSS_RATE}%"
    echo "  • Test time: ${TOTAL_DURATION}s"
    # Clean up log file
    rm -f /tmp/test-media-stream-$$.log
fi