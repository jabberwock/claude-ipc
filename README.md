# Claude IPC

Inter-Process Communication system for multiple Claude Code instances to collaborate and share messages.

## Project Structure

```
claude-ipc/
├── collab-cli/          # Rust CLI tool
│   ├── src/
│   │   ├── main.rs      # CLI entry point with clap
│   │   ├── client.rs    # HTTP client for API communication
│   │   └── hash.rs      # SHA1 hashing utilities
│   └── Cargo.toml
│
├── collab-server/       # Rust API server
│   ├── src/
│   │   ├── main.rs      # Axum web server
│   │   └── db.rs        # SQLite database initialization
│   └── Cargo.toml
│
└── README.md
```

## Technology Stack

- **Language**: Rust (100%)
- **CLI**: Clap for argument parsing
- **Server**: Axum web framework
- **Database**: SQLite3 with sqlx
- **HTTP Client**: reqwest (async)
- **Hashing**: SHA1 for message identification

## Features

- ✅ Cross-platform CLI (Windows, macOS, Linux)
- ✅ Message timestamping
- ✅ 1-hour message retention
- ✅ SHA1 hash prefixes for message addressing
- ✅ Multi-hash references support
- ✅ Instance-specific message filtering
- ✅ RESTful API with JSON responses

## Quick Start

### 1. Start the Server

```bash
cd collab-server
cargo run --release
# Server starts on http://localhost:8000
```

### 2. Discover Active Workers

```bash
# Set your instance ID
export COLLAB_INSTANCE=worker1

# See who's active
collab roster
```

### 3. Send Your First Message

```bash
# Send a message to another worker
collab add @worker2 "Fixed the authentication bug in login.rs"
```

### 4. Watch for Messages

```bash
# Continuously watch for new messages (polls every 10 seconds)
collab watch

# Or check once
collab list
```

## CLI Usage

```
collab [OPTIONS] <COMMAND>

Commands:
  list     List messages intended for this instance (last hour only)
  add      Send a message to another instance
  watch    Poll for new messages every 10 seconds (runs continuously)
  history  View message history including your own sent messages
  roster   Show active workers (who's been sending messages recently)

Options:
  -s, --server <SERVER>      Server URL [env: COLLAB_SERVER] [default: http://localhost:8000]
  -i, --instance <INSTANCE>  Instance identifier [env: COLLAB_INSTANCE]
  -h, --help                Print help
```

## API Endpoints

### GET /
Health check and version info

### GET /messages/{instance_id}
List messages for a specific instance (last hour only)

### POST /messages
Create a new message

**Request body:**
```json
{
  "sender": "worker1",
  "recipient": "worker2",
  "content": "Message content",
  "refs": ["hash1", "hash2"]
}
```

### DELETE /messages/cleanup
Manually cleanup old messages (maintenance endpoint)

## Environment Variables

- `COLLAB_SERVER`: Server URL (default: `http://localhost:8000`)
- `COLLAB_INSTANCE`: Instance identifier (required for CLI)

## Message Format

Each message includes:
- **id**: Unique UUID
- **hash**: SHA1 hash of message content (40 hex chars)
- **sender**: Sending instance ID
- **recipient**: Receiving instance ID
- **content**: Message body
- **refs**: Array of referenced message hashes
- **timestamp**: UTC timestamp

## Development Status

✅ **Complete**: Project structure with working Rust CLI and server

**Next Steps**:
- [ ] Add comprehensive tests
- [ ] Create CLAUDE.md documentation
- [ ] Add authentication/authorization
- [ ] Implement message search/filtering
- [ ] Add CLI output formatting options
