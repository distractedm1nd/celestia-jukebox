use anyhow::Result;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::tx::Transaction;
use crate::tx::YoutubeLink;

#[derive(Clone, Serialize, Deserialize)]
pub struct QueuedSong {
    pub start_time: SystemTime,
    // TODO: Use the youtube API to get the duration of the song, otherwise this can be attacked!
    pub duration: Duration,
    pub link: YoutubeLink,
}

pub struct State {
    pub history: VecDeque<QueuedSong>,
    pub queue: VecDeque<QueuedSong>,
}

impl State {
    pub fn new() -> Self {
        State {
            history: VecDeque::new(),
            queue: VecDeque::new(),
        }
    }

    pub fn get_next_song(&self) -> Option<&QueuedSong> {
        self.queue.front()
    }

    pub fn get_history(&self) -> &VecDeque<QueuedSong> {
        &self.history
    }

    pub fn get_queue(&self) -> &VecDeque<QueuedSong> {
        &self.queue
    }

    // This method should probably be called every second or something, by the
    // fullnode. bad architecture but we ball
    pub fn cleanup_queue(&mut self) {
        while let Some(song) = self.queue.front() {
            let current_time = SystemTime::now();
            // Check if song has finished playing
            if current_time.duration_since(song.start_time).unwrap() >= song.duration {
                // Use if let to avoid simultaneous borrows
                if let Some(finished_song) = self.queue.pop_front() {
                    self.history.push_back(finished_song);
                }
            } else {
                // Rest of songs in queue are in the future, no reason to loop anymore
                break;
            }
        }
    }

    pub fn validate_tx(&self, _tx: Transaction) -> Result<()> {
        Ok(())
    }

    pub fn process_tx(&mut self, tx: Transaction) -> Result<()> {
        self.validate_tx(tx.clone())?;

        let new_start_time = self
            .queue
            .back()
            .map(|song| song.start_time + song.duration)
            .unwrap_or(SystemTime::now());

        // this can only be done because we only have one tx type rn
        let Transaction::AddToQueue { url } = tx;
        let duration = url.get_video_duration()?;

        self.queue.push_back(QueuedSong {
            start_time: new_start_time,
            duration,
            link: url,
        });

        Ok(())
    }
}
