//! MO-DE Backend - Richup.io Clone
//!
//! A real-time multiplayer Monopoly-like game server built with:
//! - Axum for HTTP/WebSocket
//! - SQLx for PostgreSQL persistence  
//! - Redis for game session state
//!
//! Architecture:
//! - `api/` - HTTP handlers and WebSocket endpoints
//! - `game/` - Core game engine and state machine
//! - `bot/` - Deterministic AI for computer players
//! - `db/` - Database models and queries
//! - `ws/` - WebSocket hub for real-time sync

pub mod api;
pub mod bot;
pub mod db;
pub mod game;
pub mod ws;

pub mod config;
pub mod error;
