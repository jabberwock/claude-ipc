#!/usr/bin/env bash
# Integration test for collab system

set -e

echo "🧪 Running integration test..."
echo ""

CLI=~/.cargo/bin/collab
SERVER=./collab-server/target/release/collab-server
LOG_FILE="integration_test.log"
touch $LOG_FILE

log_error() {
    local message="$1"
    echo "$(date '+%Y-%m-%d %H:%M:%S') ERROR: $message" | tee -a $LOG_FILE
}

run_command_and_log() {
    local command="$1"
    local description="$2"
    if ! eval "$command"; then
        log_error "Command '$description' failed."
        exit 1
    fi
}

# Start server in background
echo "Starting server..."
$SERVER &
SERVER_PID=$!

# Wait for server to start (replace with a more robust check if needed)
sleep 2

echo "✓ Server started (PID: $SERVER_PID)"
echo ""

# Test roster (should be empty)
echo "Testing roster..."
run_command_and_log "$CLI roster" "Testing roster"

# Send a message
echo "Sending test message..."
run_command_and_log "$CLI --instance test-worker1 add @test-worker2 'Test message from worker1'" "Sending test message"

# Check roster again (should show worker1)
echo "Checking roster after message..."
run_command_and_log "$CLI roster" "Checking roster after message"

# List messages for worker2
echo "Listing messages for worker2..."
$CLI --instance test-worker2 list

# View history
echo "Viewing history..."
$CLI --instance test-worker2 history

echo ""
echo "✅ Integration test complete!"

# Cleanup
echo "Stopping server..."
if kill $SERVER_PID; then
    echo "✓ Server stopped."
else
    log_error "Failed to stop the server (PID: $SERVER_PID)."
fi

wait $SERVER_PID 2>/dev/null || true

rm -f collab.db
if [ $? -eq 0 ]; then
    echo "✓ Database file removed."
else
    log_error "Failed to remove database file 'collab.db'."
fi

echo ""