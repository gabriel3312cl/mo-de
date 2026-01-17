//! WebSocket module for real-time game communication

mod hub;

pub use hub::Hub;

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use uuid::Uuid;

use crate::api::AppState;

/// WebSocket handler - upgrades HTTP to WebSocket connection
pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((room_id, player_id)): Path<(String, Uuid)>,
) -> Response {
    ws.on_upgrade(move |socket| hub::handle_socket(socket, state, room_id, player_id))
}
