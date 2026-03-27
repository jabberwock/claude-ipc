use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub hash: String,
    pub sender: String,
    pub recipient: String,
    pub content: String,
    pub refs: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub instance_id: String,
    pub last_seen: DateTime<Utc>,
    pub message_count: usize,
}

pub struct CollabClient {
    base_url: String,
    instance_id: String,
    client: reqwest::Client,
}

impl CollabClient {
    pub fn new(base_url: &str, instance_id: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            instance_id: instance_id.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_messages(&self) -> Result<()> {
        let url = format!("{}/messages/{}", self.base_url, self.instance_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch messages: {}", response.status());
        }
        
        let messages: Vec<Message> = response.json().await?;
        
        if messages.is_empty() {
            println!("No messages in the last hour.");
            return Ok(());
        }
        
        println!("Messages for @{}:\n", self.instance_id);
        for msg in messages {
            println!("─────────────────────────────────────");
            println!("Hash: {}", &msg.hash[..7]); // Short hash
            println!("From: @{}", msg.sender);
            println!("Time: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
            if !msg.refs.is_empty() {
                let short_refs: Vec<String> = msg.refs.iter()
                    .map(|r| r.chars().take(7).collect())
                    .collect();
                println!("Refs: {}", short_refs.join(", "));
            }
            println!("\n{}\n", msg.content);
        }
        println!("─────────────────────────────────────");
        
        Ok(())
    }

    pub async fn add_message(
        &self,
        recipient: &str,
        content: &str,
        refs: Option<Vec<String>>,
    ) -> Result<()> {
        #[derive(Serialize)]
        struct CreateMessage {
            sender: String,
            recipient: String,
            content: String,
            refs: Vec<String>,
        }
        
        let payload = CreateMessage {
            sender: self.instance_id.clone(),
            recipient: recipient.to_string(),
            content: content.to_string(),
            refs: refs.unwrap_or_default(),
        };
        
        let url = format!("{}/messages", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to send message: {}", response.status());
        }
        
        let msg: Message = response.json().await?;
        
        println!("✓ Message sent to @{}", recipient);
        println!("  Hash: {}", &msg.hash[..7]);
        println!("  Time: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        
        Ok(())
    }
    
    pub async fn watch_messages(&self, interval_secs: u64) -> Result<()> {
        use tokio::time::{sleep, Duration};
        
        let mut seen_ids: HashSet<String> = HashSet::new();
        
        println!("👀 Watching for messages to @{} (polling every {} seconds)", 
                 self.instance_id, interval_secs);
        println!("Press Ctrl+C to stop\n");
        
        loop {
            let url = format!("{}/messages/{}", self.base_url, self.instance_id);
            
            match self.client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<Vec<Message>>().await {
                        Ok(messages) => {
                            let new_messages: Vec<_> = messages
                                .into_iter()
                                .filter(|msg| !seen_ids.contains(&msg.id))
                                .collect();
                            
                            for msg in &new_messages {
                                seen_ids.insert(msg.id.clone());
                                
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                                println!("🔔 New message!");
                                println!("Hash: {}", &msg.hash[..7]);
                                println!("From: @{}", msg.sender);
                                println!("Time: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
                                if !msg.refs.is_empty() {
                                    let short_refs: Vec<String> = msg.refs.iter()
                                        .map(|r| r.chars().take(7).collect())
                                        .collect();
                                    println!("Refs: {}", short_refs.join(", "));
                                }
                                println!("\n{}\n", msg.content);
                            }
                            
                            if !new_messages.is_empty() {
                                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  Failed to parse messages: {}", e);
                        }
                    }
                }
                Ok(response) => {
                    eprintln!("⚠️  Server error: {}", response.status());
                }
                Err(e) => {
                    eprintln!("⚠️  Connection error: {}", e);
                }
            }
            
            sleep(Duration::from_secs(interval_secs)).await;
        }
    }
    
    pub async fn show_history(&self, filter_instance: Option<&str>) -> Result<()> {
        let url = format!("{}/history/{}", self.base_url, self.instance_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch history: {}", response.status());
        }
        
        let mut messages: Vec<Message> = response.json().await?;
        
        // Filter by conversation partner if specified
        if let Some(filter_id) = filter_instance {
            messages.retain(|msg| {
                msg.sender == filter_id || msg.recipient == filter_id
            });
        }
        
        if messages.is_empty() {
            println!("No message history in the last hour.");
            if let Some(filter_id) = filter_instance {
                println!("(filtered to conversations with @{})", filter_id);
            }
            return Ok(());
        }
        
        println!("Message History for @{}:\n", self.instance_id);
        if let Some(filter_id) = filter_instance {
            println!("(showing only conversations with @{})\n", filter_id);
        }
        
        for msg in messages {
            let direction = if msg.sender == self.instance_id {
                format!("@{} → @{}", msg.sender, msg.recipient)
            } else {
                format!("@{} → @{}", msg.sender, msg.recipient)
            };
            
            println!("─────────────────────────────────────");
            println!("{}", direction);
            println!("Hash: {}", &msg.hash[..7]); // Show short hash
            println!("Time: {}", msg.timestamp.format("%Y-%m-%d %H:%M:%S"));
            if !msg.refs.is_empty() {
                let short_refs: Vec<String> = msg.refs.iter()
                    .map(|r| r.chars().take(7).collect())
                    .collect();
                println!("Refs: {}", short_refs.join(", "));
            }
            println!("\n{}\n", msg.content);
        }
        println!("─────────────────────────────────────");
        
        Ok(())
    }
    
    pub async fn show_roster(&self) -> Result<()> {
        let url = format!("{}/roster", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch roster: {}", response.status());
        }
        
        let workers: Vec<WorkerInfo> = response.json().await?;
        
        if workers.is_empty() {
            println!("No active workers in the last hour.");
            return Ok(());
        }
        
        println!("Active Workers (last hour):\n");
        for worker in workers {
            let you_marker = if worker.instance_id == self.instance_id {
                " (you)"
            } else {
                ""
            };
            println!("  @{}{}", worker.instance_id, you_marker);
            println!("    Last seen: {}", worker.last_seen.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("    Messages: {}", worker.message_count);
            println!();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = Message {
            id: "test-id".to_string(),
            hash: "abc123".to_string(),
            sender: "worker1".to_string(),
            recipient: "worker2".to_string(),
            content: "test content".to_string(),
            refs: vec!["ref1".to_string(), "ref2".to_string()],
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("worker1"));
        assert!(json.contains("worker2"));
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{
            "id": "test-id",
            "hash": "abc123",
            "sender": "worker1",
            "recipient": "worker2",
            "content": "test content",
            "refs": ["ref1"],
            "timestamp": "2024-03-27T14:30:45Z"
        }"#;

        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.id, "test-id");
        assert_eq!(message.sender, "worker1");
        assert_eq!(message.recipient, "worker2");
        assert_eq!(message.refs.len(), 1);
    }

    #[test]
    fn test_collab_client_creation() {
        let client = CollabClient::new("http://localhost:8000", "test-worker");
        assert_eq!(client.base_url, "http://localhost:8000");
        assert_eq!(client.instance_id, "test-worker");
    }
}
