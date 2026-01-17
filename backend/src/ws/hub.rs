//! WebSocket hub for managing connections and broadcasting

use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::api::AppState;
use crate::game::{ClientEvent, GameEngine, ServerEvent};

/// A connection to a single client
pub struct Connection {
    pub player_id: Uuid,
    pub tx: mpsc::UnboundedSender<ServerEvent>,
}

/// Hub manages all active connections grouped by room
pub struct Hub {
    rooms: HashMap<String, Vec<Connection>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    /// Add a connection to a room
    pub fn join(&mut self, room_id: &str, player_id: Uuid, tx: mpsc::UnboundedSender<ServerEvent>) {
        let room = self.rooms.entry(room_id.to_string()).or_default();
        // Remove any existing connection for this player
        room.retain(|c| c.player_id != player_id);
        room.push(Connection { player_id, tx });
    }

    /// Remove a connection from a room
    pub fn leave(&mut self, room_id: &str, player_id: Uuid) {
        if let Some(room) = self.rooms.get_mut(room_id) {
            room.retain(|c| c.player_id != player_id);
            if room.is_empty() {
                self.rooms.remove(room_id);
            }
        }
    }

    /// Broadcast event to all players in a room
    pub fn broadcast(&self, room_id: &str, event: ServerEvent) {
        if let Some(room) = self.rooms.get(room_id) {
            for conn in room {
                let _ = conn.tx.send(event.clone());
            }
        }
    }

    /// Send event to a specific player
    pub fn send_to(&self, room_id: &str, player_id: Uuid, event: ServerEvent) {
        if let Some(room) = self.rooms.get(room_id) {
            if let Some(conn) = room.iter().find(|c| c.player_id == player_id) {
                let _ = conn.tx.send(event);
            }
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle a single WebSocket connection
pub async fn handle_socket(socket: WebSocket, state: AppState, room_id: String, player_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerEvent>();

    // Register connection in hub
    {
        let mut hub = state.hub.write().await;
        hub.join(&room_id, player_id, tx);
    }

    // Send current game state on connect
    if let Ok(Some(game)) = GameEngine::get_game(&state.redis, &room_id).await {
        let state_event = ServerEvent::GameState(game);
        let msg = serde_json::to_string(&state_event).unwrap();
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    // Spawn task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let msg = serde_json::to_string(&event).unwrap();
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let recv_state = state.clone();
    let recv_room_id = room_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                    // Process the event through game engine
                    let _ = GameEngine::handle_event(
                        &recv_state.redis,
                        &recv_state.hub,
                        &recv_room_id,
                        player_id,
                        event,
                    )
                    .await;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Remove connection from hub
    {
        let mut hub = state.hub.write().await;
        hub.leave(&room_id, player_id);
    }

    tracing::debug!("Player {} disconnected from room {}", player_id, room_id);
}
