use clap::{Parser, Subcommand};
use anyhow::Result;

mod client;

use client::CollabClient;

/// CLI for inter-instance communication between Claude Code workers
#[derive(Parser)]
#[command(name = "collab")]
#[command(about = "Collaboration tool for Claude Code instances", long_about = None)]
struct Cli {
    /// Server URL (default: http://localhost:8000)
    #[arg(short, long, default_value = "http://localhost:8000")]
    server: String,

    /// Instance identifier for this Claude Code worker
    #[arg(short, long)]
    instance: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List messages intended for this instance (last hour only)
    List,
    
    /// Send a message to another instance
    Add {
        /// Target instance (e.g., @other_instance)
        #[arg(value_name = "@INSTANCE")]
        recipient: String,
        
        /// Message content/description
        #[arg(value_name = "MESSAGE")]
        message: String,
        
        /// Reference message hash(es) - comma-separated SHA1 hashes
        #[arg(short, long, value_name = "HASH1,HASH2")]
        refs: Option<String>,
    },
    
    /// Poll for new messages every 10 seconds (runs continuously)
    Watch {
        /// Polling interval in seconds (default: 10)
        #[arg(short, long, default_value = "10")]
        interval: u64,
    },
    
    /// View message history including your own sent messages
    History {
        /// Filter by conversation partner (e.g., @other_instance)
        #[arg(value_name = "@INSTANCE")]
        filter: Option<String>,
    },
    
    /// Show active workers (who's been sending messages recently)
    Roster,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Roster command doesn't need instance ID
    if matches!(cli.command, Commands::Roster) {
        let client = CollabClient::new(&cli.server, "");
        client.show_roster().await?;
        return Ok(());
    }
    
    let instance_id = cli.instance.ok_or_else(|| {
        anyhow::anyhow!("Instance ID required. Set via --instance or COLLAB_INSTANCE env var")
    })?;
    
    let client = CollabClient::new(&cli.server, &instance_id);
    
    match cli.command {
        Commands::List => {
            client.list_messages().await?;
        }
        Commands::Add { recipient, message, refs } => {
            let recipient = recipient.trim_start_matches('@');
            let ref_hashes = refs.map(|r| {
                r.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            });
            
            client.add_message(recipient, &message, ref_hashes).await?;
        }
        Commands::Watch { interval } => {
            client.watch_messages(interval).await?;
        }
        Commands::History { filter } => {
            let filter_id = filter.as_ref().map(|s| s.trim_start_matches('@'));
            client.show_history(filter_id).await?;
        }
        Commands::Roster => {
            unreachable!("Roster handled before instance ID check")
        }
    }
    
    Ok(())
}
