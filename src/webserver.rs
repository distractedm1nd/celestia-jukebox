use crate::fullnode::FullNode;
use crate::state::QueuedSong;
use crate::tx::{Transaction, YoutubeLink};
use axum::{extract::State as AxumState, http::StatusCode, Json};
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Deserialize)]
pub(crate) struct AddSongRequest {
    url: YoutubeLink,
}

pub(crate) async fn get_queue(
    AxumState(node): AxumState<Arc<FullNode>>,
) -> Json<VecDeque<QueuedSong>> {
    let state = node.state.lock().await;
    Json(state.get_queue().clone())
}

pub(crate) async fn get_history(
    AxumState(node): AxumState<Arc<FullNode>>,
) -> Json<VecDeque<QueuedSong>> {
    let state = node.state.lock().await;
    Json(state.get_history().clone())
}

pub(crate) async fn send_tx(
    AxumState(node): AxumState<Arc<FullNode>>,
    Json(payload): Json<AddSongRequest>,
) -> Result<(), (StatusCode, String)> {
    let tx = Transaction::AddToQueue { url: payload.url };
    node.queue_transaction(tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
