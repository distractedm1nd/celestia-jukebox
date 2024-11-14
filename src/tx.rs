// use ed25519_dalek::{Signature as Ed25519Signature, Verifier, VerifyingKey};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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

    pub fn get_video_duration(&self) -> Result<Duration> {
        // Use yt-dlp to fetch video metadata
        let output = Command::new("yt-dlp")
            .args([
                "--dump-json",   // Output video metadata as JSON
                "--no-playlist", // Don't process playlists
                self.0.as_str(),
            ])
            .output()
            .context("Failed to execute yt-dlp")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("yt-dlp failed: {}", error));
        }

        let info: VideoInfo =
            serde_json::from_slice(&output.stdout).context("Failed to parse yt-dlp output")?;

        Ok(Duration::from_secs_f64(info.duration))
    }
}
