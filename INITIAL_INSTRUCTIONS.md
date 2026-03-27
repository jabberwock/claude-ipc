I am writing a coworker or skill for Claude:  This is to allow multiple claude code instances that rely on each other’s work to effectively communicate with each other so they can know about fixes from both sides.

Requirements: 
These will communicate to a hostname to query a fast api:

Languages:
- Rust 

Cross-platform (windows/Mac/linux): Yes
 - Simple command such as “collab list” from laude code.
- “collab add @other_instance <description of message>”
- “collab list” will only list messages intended for the instance requesting.
- All messages will be time stamped.
- Only return results from the last hour.
 - If the user wants to, they can tell claude code to invoke collab again (for whatever reason, if it’s been over an hour, etc.)
- Messages must contain a prefix which is a SHA1 hash of the message. This allows each  worker to address specifically which messages they are addressing from the other worker. Multiple hashes are allowed, to check multiple commits.

-  CLAUDE.md with thorough instructions
- SQLite3 database

