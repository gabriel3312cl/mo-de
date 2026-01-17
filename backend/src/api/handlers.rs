//! HTTP handlers for REST API

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::error::{AppError, AppResult};
use crate::game::{GameConfig, GameEngine};

/// Create a new game room
#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    pub host_name: String,
    pub config: Option<GameConfig>,
}

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    pub room_id: String,
    pub player_id: Uuid,
}

pub async fn create_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> AppResult<Json<CreateRoomResponse>> {
    let config = req.config.unwrap_or_default();
    let (room_id, player_id) =
        GameEngine::create_room(&state.redis, &req.host_name, config).await?;

    Ok(Json(CreateRoomResponse { room_id, player_id }))
}

/// Join an existing room
#[derive(Debug, Deserialize)]
pub struct JoinRoomRequest {
    pub player_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinRoomResponse {
    pub player_id: Uuid,
}

pub async fn join_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(req): Json<JoinRoomRequest>,
) -> AppResult<Json<JoinRoomResponse>> {
    let player_id = GameEngine::join_room(&state.redis, &room_id, &req.player_name).await?;

    Ok(Json(JoinRoomResponse { player_id }))
}

/// Get room state
#[derive(Debug, Serialize)]
pub struct RoomStateResponse {
    pub room_id: String,
    pub players: Vec<PlayerInfo>,
    pub phase: String,
    pub config: GameConfig,
}

#[derive(Debug, Serialize)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub is_host: bool,
    pub is_bot: bool,
}

pub async fn get_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> AppResult<Json<RoomStateResponse>> {
    let game = GameEngine::get_game(&state.redis, &room_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

    let players = game
        .players
        .iter()
        .map(|p| PlayerInfo {
            id: p.id,
            name: p.name.clone(),
            color: p.color.clone(),
            is_host: p.is_host,
            is_bot: p.is_bot,
        })
        .collect();

    Ok(Json(RoomStateResponse {
        room_id,
        players,
        phase: format!("{:?}", game.phase),
        config: game.config,
    }))
}

/// Add a bot to the room
#[derive(Debug, Deserialize)]
pub struct AddBotRequest {
    pub difficulty: Option<String>,
}

pub async fn add_bot(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(_req): Json<AddBotRequest>,
) -> AppResult<Json<JoinRoomResponse>> {
    let player_id = GameEngine::add_bot(&state.redis, &room_id).await?;
    Ok(Json(JoinRoomResponse { player_id }))
}

/// Start the game
pub async fn start_game(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    GameEngine::start_game(&state.redis, &state.hub, &room_id).await?;
    Ok(Json(serde_json::json!({ "status": "started" })))
}

/// Health check
pub async fn health() -> &'static str {
    "OK"
}
