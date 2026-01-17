//! Route definitions

use axum::{
    routing::{get, post},
    Router,
};

use super::{handlers, AppState};
use crate::ws;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Room management
        .route("/api/rooms", post(handlers::create_room))
        .route("/api/rooms/:room_id", get(handlers::get_room))
        .route("/api/rooms/:room_id/join", post(handlers::join_room))
        .route("/api/rooms/:room_id/bot", post(handlers::add_bot))
        .route("/api/rooms/:room_id/start", post(handlers::start_game))
        // WebSocket
        .route("/ws/:room_id/:player_id", get(ws::handler))
}
