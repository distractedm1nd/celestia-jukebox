// use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Deserialize, Serialize)]
pub enum Transaction {
    AddToQueue { url: YoutubeLink },
}

#[derive(Clone, Deserialize, Serialize)]
pub struct YoutubeLink(String);

#[allow(dead_code)]
impl YoutubeLink {
    /// Creates a new YoutubeLink from a URL string.
    /// Validates and normalizes the URL, extracting the video ID.
    ///
    /// Accepts various YouTube URL formats:
    /// - https://www.youtube.com/watch?v=VIDEO_ID
    /// - https://youtu.be/VIDEO_ID
    /// - https://youtube.com/watch?v=VIDEO_ID
    /// - VIDEO_ID (direct video ID)
    pub fn new(url: String) -> Result<Self> {
        // Remove whitespace and backslashes
        let cleaned = url.trim().replace('\\', "");

        // Try to extract video ID from the cleaned URL
        let video_id = extract_video_id(&cleaned)?;

        // Construct canonical YouTube URL
        let canonical_url = format!("https://www.youtube.com/watch?v={}", video_id);

        Ok(Self(canonical_url))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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

/// Extract video ID from various YouTube URL formats
fn extract_video_id(url: &str) -> Result<String> {
    // Regular expressions for different YouTube URL formats
    let patterns = [
        // youtu.be URLs
        Regex::new(r"^(?:https?://)?(?:www\.)?youtu\.be/([a-zA-Z0-9_-]{11})(?:\?|&|/|$)").unwrap(),
        // youtube.com URLs
        Regex::new(r"^(?:https?://)?(?:www\.)?youtube\.com/watch\?v=([a-zA-Z0-9_-]{11})(?:&|$)")
            .unwrap(),
        // Embedded URLs
        Regex::new(r"^(?:https?://)?(?:www\.)?youtube\.com/embed/([a-zA-Z0-9_-]{11})(?:\?|&|/|$)")
            .unwrap(),
        // Direct video ID (11 characters)
        Regex::new(r"^([a-zA-Z0-9_-]{11})$").unwrap(),
    ];

    // Try each pattern until we find a match
    for pattern in &patterns {
        if let Some(captures) = pattern.captures(url) {
            if let Some(id) = captures.get(1) {
                let video_id = id.as_str().to_string();
                // Validate video ID format
                if is_valid_video_id(&video_id) {
                    return Ok(video_id);
                }
            }
        }
    }

    Err(anyhow!("Invalid YouTube URL or video ID: {}", url))
}

/// Validate YouTube video ID format
fn is_valid_video_id(id: &str) -> bool {
    // YouTube video IDs are exactly 11 characters long and contain only
    // alphanumeric characters, underscores, and hyphens
    id.len() == 11
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}
