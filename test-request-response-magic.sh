#!/bin/bash
set -e

echo "🧪 Testing Magic CLI functionality with request_response binary"

# Kill any existing processes
pkill -f "request_response" || true

# Clean test directories
rm -rf /tmp/test-alice /tmp/test-bob

echo "📁 Setting up test environments"

# Initialize Alice
echo "🔧 Initializing Alice..."
FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- init

# Initialize Bob  
echo "🔧 Initializing Bob..."
FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- init

# Create identities and capture peer IDs
echo "👤 Creating Alice identity..."
ALICE_OUTPUT=$(FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- create-identity alice 2>&1)
echo "$ALICE_OUTPUT"
ALICE_ID=$(echo "$ALICE_OUTPUT" | grep "Peer ID:" | cut -d':' -f2 | xargs)

echo "👤 Creating Bob identity..."  
BOB_OUTPUT=$(FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- create-identity bob 2>&1)
echo "$BOB_OUTPUT"
BOB_ID=$(echo "$BOB_OUTPUT" | grep "Peer ID:" | cut -d':' -f2 | xargs)

# Add echo protocol to both
echo "📡 Adding echo protocol to Alice..."
FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- add-protocol alice --protocol echo.fastn.com --config '{"max_length": 1000}'

echo "📡 Adding echo protocol to Bob..."
FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- add-protocol bob --protocol echo.fastn.com --config '{"max_length": 1000}'

# Set identities online
echo "🟢 Setting Alice online..."
FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- identity-online alice

echo "🟢 Setting Bob online..." 
FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- identity-online bob

# Check status
echo "📊 Alice status:"
FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- status

echo "📊 Bob status:"
FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- status

# Start Alice server in background
echo "🚀 Starting Alice server..."
FASTN_HOME="/tmp/test-alice" cargo run --bin request_response -- run &
ALICE_PID=$!

# Start Bob server in background  
echo "🚀 Starting Bob server..."
FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- run &
BOB_PID=$!

echo "⏳ Waiting for servers to start..."
sleep 3

# Peer IDs already captured from create-identity output above

echo "🔑 Alice peer ID: $ALICE_ID"
echo "🔑 Bob peer ID: $BOB_ID"

# Try to make a call from Bob to Alice
echo "📞 Attempting call from Bob to Alice..."
echo '{"message": "Hello Alice from Bob!"}' | FASTN_HOME="/tmp/test-bob" cargo run --bin request_response -- call "$ALICE_ID" echo.fastn.com

echo "✅ Magic CLI test complete!"

# Cleanup
kill $ALICE_PID $BOB_PID 2>/dev/null || true