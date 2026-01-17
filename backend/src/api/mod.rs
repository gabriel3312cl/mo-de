//! API module - HTTP handlers and routes

mod handlers;
mod routes;

use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::{config::Config, ws::Hub};

pub use routes::routes;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: ConnectionManager,
    pub hub: Arc<RwLock<Hub>>,
    pub config: Config,
}
