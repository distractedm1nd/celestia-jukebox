use anyhow::{Context, Result};
use celestia_types::nmt::Namespace;
use std::{env, sync::Arc};

mod fullnode;
mod state;
mod tx;
mod webserver;

use crate::fullnode::FullNode;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "start-fullnode" => {
            if args.len() < 4 {
                println!("Error: start height and namespace required");
                return Ok(());
            }
            let start_height = args[2]
                .parse::<u64>()
                .context("Failed to parse start height")?;

            let namespace_bytes =
                hex::decode(&args[3]).context("Failed to decode namespace hex")?;
            let namespace = Namespace::new_v0(namespace_bytes.as_slice())
                .context("Failed to create namespace")?;

            let fullnode = Arc::new(FullNode::new(namespace, start_height).await?);
            fullnode.start().await?;
            return Ok(());
        }
        "add-song" => {
            if args.len() < 3 {
                println!("Error: URL required");
                return Ok(());
            }

            let client = reqwest::Client::new();
            let server_url =
                env::var("MUSICNODE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

            add_song(&client, &server_url, &args[2]).await?;
        }
        _ => print_usage(),
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  musicnode start-fullnode <start_height> <namespace_hex>");
    println!("  musicnode add-song <youtube_url> <duration_secs>");
}

async fn add_song(client: &reqwest::Client, server_url: &str, url: &str) -> Result<()> {
    let response = client
        .post(&format!("{}/send", server_url))
        .json(&serde_json::json!({
            "url": url,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Song added successfully.");
    } else {
        println!(
            "Failed to add song. Server responded with: {}",
            response.status()
        );
    }
    Ok(())
}
