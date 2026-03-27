#!/usr/bin/env bash
# Integration test for collab system

set -e

echo "🧪 Running integration test..."
echo ""

CLI=./collab-cli/target/release/collab
SERVER=./collab-server/target/release/collab-server

# Start server in background
echo "Starting server..."
$SERVER &
SERVER_PID=$!
sleep 2

echo "✓ Server started (PID: $SERVER_PID)"
echo ""

# Test roster (should be empty)
echo "Testing roster..."
$CLI roster
echo ""

# Send a message
echo "Sending test message..."
$CLI --instance test-worker1 add @test-worker2 "Test message from worker1"
echo ""

# Check roster again (should show worker1)
echo "Checking roster after message..."
$CLI roster
echo ""

# List messages for worker2
echo "Listing messages for worker2..."
$CLI --instance test-worker2 list
echo ""

# View history
echo "Viewing history..."
$CLI --instance test-worker2 history
echo ""

# Cleanup
echo "Stopping server..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null || true
rm -f collab.db

echo ""
echo "✅ Integration test complete!"
