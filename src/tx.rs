// use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;
use std::time::Duration;

#[derive(Clone, Deserialize, Serialize)]
pub enum Transaction {
    AddToQueue { url: YoutubeLink },
}

#[derive(Clone, Deserialize, Serialize)]
pub struct YoutubeLink(String);

#[derive(Debug, Deserialize)]
struct VideoInfo {
    duration: f64,
}

impl YoutubeLink {
    // TODO: Validation that its a valid youtube URL
    pub fn new(url: String) -> Self {
        Self(url)
    }

    pub async fn get_video_duration(&self) -> Result<Duration> {
        // Create HTTP client
        let client = reqwest::Client::new();

        // Fetch video page
        let response = client
            .get(&self.0)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await?
            .text()
            .await?;

        let re = Regex::new(r#"ytInitialPlayerResponse\s*=\s*(\{.+?\}\});"#).unwrap();
        let json_str = re
            .captures(&response)
            .and_then(|caps| caps.get(1))
            .ok_or_else(|| anyhow!("Could not find ytInitialPlayerResponse"))?
            .as_str();

        // Parse JSON and extract duration
        let json: Value = serde_json::from_str(json_str)?;
        let seconds_str = json["videoDetails"]["lengthSeconds"]
            .as_str()
            .ok_or_else(|| anyhow!("Could not find duration"))?;

        // Parse seconds and create Duration
        let seconds: u64 = seconds_str.parse()?;
        Ok(Duration::from_secs(seconds))
    }
}
