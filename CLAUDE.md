# Claude IPC - Collaboration Tool

## Overview

`collab` is a command-line tool that enables multiple Claude Code instances to communicate and collaborate with each other. It allows workers to send messages, reference previous communications, and stay synchronized on fixes and updates.

## Installation

### Prerequisites

- **Rust toolchain** (1.70+): Install from [rustup.rs](https://rustup.rs)
- **SQLite3**: Pre-installed on most systems

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd claude-ipc

# Build the server
cd collab-server
cargo build --release

# Build the CLI
cd ../collab-cli
cargo build --release
```

### Install Binaries

```bash
# Server binary
cp collab-server/target/release/collab-server /usr/local/bin/

# CLI binary
cp collab-cli/target/release/collab /usr/local/bin/
```

## Quick Start

### 1. Start the Server

```bash
collab-server
```

The server starts on `http://localhost:8000` and creates a `collab.db` SQLite database.

### 2. Send Your First Message

```bash
# Set your worker instance ID
export COLLAB_INSTANCE=worker1

# Send a message to another worker
collab add @worker2 "Fixed the authentication bug in login.rs"
```

### 3. Check Messages

```bash
# List messages for your instance
collab list

# Watch for new messages (polls every 30 seconds)
collab watch
```

## CLI Usage

### Commands

#### `collab list`

List all messages intended for your instance from the last hour.

```bash
collab --instance worker1 list
```

**Output:**
```
Messages for @worker1:

─────────────────────────────────────
Hash: a94a8fe5ccb19ba61c4c0873d391e987982fbbd3
From: @worker2
Time: 2024-03-27 14:30:45 UTC

Applied your authentication fix and tested successfully
─────────────────────────────────────
```

#### `collab add @<instance> <message>`

Send a message to another worker instance.

```bash
# Basic message
collab add @worker2 "Completed database migration"

# Message with references to previous messages
collab add @worker2 "Applied your suggestions" \
  --refs a94a8fe5ccb19ba61c4c0873d391e987982fbbd3
```

**Options:**
- `--refs <HASH1,HASH2>`: Reference previous message hashes (comma-separated)

#### `collab watch`

Continuously poll for new messages every 30 seconds. This is the recommended way to monitor incoming messages during active collaboration.

```bash
# Watch with default 30-second interval
collab watch

# Watch with custom interval (e.g., every 10 seconds)
collab watch --interval 10
```

**Output:**
```
👀 Watching for messages to @worker1 (polling every 30 seconds)
Press Ctrl+C to stop

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔔 New message!
Hash: b3d5c3a...
From: @worker2
Time: 2024-03-27 14:35:12 UTC

Database schema updated, please pull changes
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Global Options

- `--server <URL>`: Server URL (default: `http://localhost:8000`)
- `--instance <ID>`: Your worker instance identifier (required)

### Environment Variables

```bash
# Set these in your shell profile for convenience
export COLLAB_INSTANCE=worker1
export COLLAB_SERVER=http://localhost:8000
```

## Message Format

Every message includes:

- **Hash**: SHA1 hash (40 hex characters) of the message content - use this to reference messages
- **Sender**: Instance ID of the sender (e.g., `worker1`)
- **Recipient**: Instance ID of the recipient (e.g., `worker2`)
- **Content**: The actual message text
- **Refs**: Array of referenced message hashes (for conversation threading)
- **Timestamp**: UTC timestamp in RFC3339 format

### Message Addressing

Messages are identified by their SHA1 hash, which allows workers to reference specific communications:

```bash
# Worker 1 sends a message
collab add @worker2 "Found bug in payment processor"
# Output: Hash: abc123...

# Worker 2 responds, referencing the original message
collab add @worker1 "Fixed the bug you mentioned" --refs abc123...
```

## Use Cases

### Scenario 1: Bug Fix Coordination

**Worker 1** (Frontend):
```bash
collab add @backend "Authentication redirects to 404 after login"
```

**Worker 2** (Backend):
```bash
collab list
# Sees the message, fixes the route

collab add @frontend "Fixed route in auth.rs - commit a7b3c2" \
  --refs <hash-from-frontend-message>
```

**Worker 1** (Frontend):
```bash
collab watch
# Sees the fix notification, pulls changes
```

### Scenario 2: Schema Migration

**Worker 1** (Database):
```bash
collab add @api "Running migration - users table gets email_verified column"
```

**Worker 2** (API):
```bash
collab watch
# Sees notification
# Waits for migration to complete

collab add @database "Ready to deploy API changes"
```

### Scenario 3: Continuous Collaboration

Start watching in the background while working:

```bash
# Terminal 1: Your Claude Code session working on features
# Terminal 2: Watch for messages
collab watch

# Other workers can now notify you of important changes
# You'll see messages appear in real-time
```

## Message Retention

- Messages are retained for **1 hour** from creation time
- After 1 hour, messages are filtered out from `list` and `watch` results
- Use the server's `/messages/cleanup` endpoint to manually purge old messages

```bash
curl -X DELETE http://localhost:8000/messages/cleanup
```

## Server API

The server exposes a REST API for integration:

### `GET /messages/{instance_id}`

Retrieve messages for a specific instance (last hour only).

**Response:**
```json
[
  {
    "id": "uuid",
    "hash": "sha1-hash-of-content",
    "sender": "worker1",
    "recipient": "worker2",
    "content": "Message text",
    "refs": ["hash1", "hash2"],
    "timestamp": "2024-03-27T14:30:45Z"
  }
]
```

### `POST /messages`

Create a new message.

**Request:**
```json
{
  "sender": "worker1",
  "recipient": "worker2",
  "content": "Message text",
  "refs": ["hash1", "hash2"]
}
```

### `DELETE /messages/cleanup`

Remove messages older than 1 hour.

## Best Practices

### 1. Use Descriptive Instance IDs

Choose instance IDs that reflect the worker's role:
```bash
export COLLAB_INSTANCE=frontend-worker
export COLLAB_INSTANCE=api-worker
export COLLAB_INSTANCE=database-worker
```

### 2. Use `watch` for Active Collaboration

During intensive collaboration sessions, run `collab watch` in a dedicated terminal to get real-time notifications.

### 3. Reference Previous Messages

Always use `--refs` when responding to maintain conversation context:
```bash
collab add @other "Applied your fix" --refs abc123def456
```

### 4. Keep Messages Concise but Informative

Good:
```bash
collab add @api "Added user.avatar_url field - migration pending"
```

Bad:
```bash
collab add @api "did some stuff"
```

### 5. Include Relevant Context

Mention files, commits, or specific changes:
```bash
collab add @frontend "Updated API endpoint in users.ts:45 - now returns 201 instead of 200"
```

## Troubleshooting

### Connection Refused

**Problem:** `Connection error: connection refused`

**Solution:** Ensure the server is running:
```bash
collab-server
```

### No Messages Appearing

**Problem:** Running `collab list` shows no messages.

**Solution:**
1. Verify you're using the correct instance ID
2. Check messages were sent in the last hour
3. Confirm messages were sent TO your instance ID

### Server Not Starting

**Problem:** Server fails to bind to port 8000.

**Solution:** Check if another process is using port 8000:
```bash
lsof -i :8000
# Kill the process or change server port
```

## Architecture

### Components

1. **CLI** (`collab`): Rust binary using `clap` for argument parsing and `reqwest` for HTTP
2. **Server** (`collab-server`): Rust web service using `axum` and `sqlx`
3. **Database**: SQLite3 with indexed queries for fast recipient lookup

### Data Flow

```
Worker 1                    Server                    Worker 2
   |                          |                          |
   |-- POST /messages ------->|                          |
   |<------- 201 Created -----|                          |
   |                          |<---- GET /messages -----|
   |                          |------ messages[] ------->|
```

### Security Notes

This is designed for **trusted local networks** or **VPN environments**. For production use over public networks, add:

- HTTPS/TLS encryption
- Authentication tokens
- Rate limiting
- Input validation/sanitization

## Database Schema

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,          -- UUID v4
    hash TEXT NOT NULL,            -- SHA1 hash of content
    sender TEXT NOT NULL,          -- Sender instance ID
    recipient TEXT NOT NULL,       -- Recipient instance ID
    content TEXT NOT NULL,         -- Message body
    refs TEXT NOT NULL,            -- Comma-separated hash references
    timestamp TEXT NOT NULL        -- RFC3339 timestamp
);

CREATE INDEX idx_recipient_timestamp 
ON messages(recipient, timestamp DESC);
```

## License

[Your License Here]

## Contributing

[Contribution Guidelines]

## Support

For issues or questions:
- GitHub Issues: [repository-url]/issues
- Documentation: [repository-url]/docs
