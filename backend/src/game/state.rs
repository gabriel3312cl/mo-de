//! Game state types and structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Game configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub max_players: u8,
    pub starting_cash: i32,
    pub free_parking_jackpot: bool,
    pub auction_on_decline: bool,
    pub collect_rent_in_jail: bool,
    pub even_build_rule: bool,
    pub double_rent_on_full_set: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            max_players: 4,
            starting_cash: 1500,
            free_parking_jackpot: false,
            auction_on_decline: true,
            collect_rent_in_jail: false,
            even_build_rule: true,
            double_rent_on_full_set: true,
        }
    }
}

/// Overall game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    /// Players in lobby, waiting to start
    Lobby,
    /// Rolling to determine play order
    RollingOrder,
    /// Main game in progress
    Playing,
    /// Game has ended
    GameOver,
}

/// Current turn phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurnPhase {
    /// Waiting for player to roll
    WaitingForRoll,
    /// Dice are rolling (animation)
    Rolling,
    /// Player is moving on board
    Moving,
    /// Player must make a decision (buy/auction)
    BuyDecision,
    /// Auction in progress
    Auction,
    /// Player paying rent
    PayingRent,
    /// Player managing bankruptcy
    Bankruptcy,
    /// Turn complete, waiting for end turn
    TurnEnd,
}

/// Turn state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    pub player_id: Uuid,
    pub dice: Option<(u8, u8)>,
    pub doubles_count: u8,
    pub phase: TurnPhase,
    pub can_roll_again: bool,
}

impl TurnState {
    pub fn new(player_id: Uuid) -> Self {
        Self {
            player_id,
            dice: None,
            doubles_count: 0,
            phase: TurnPhase::WaitingForRoll,
            can_roll_again: false,
        }
    }

    pub fn dice_sum(&self) -> u8 {
        self.dice.map(|(a, b)| a + b).unwrap_or(0)
    }

    pub fn is_doubles(&self) -> bool {
        self.dice.map(|(a, b)| a == b).unwrap_or(false)
    }
}

/// Player in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub position: u8,
    pub balance: i32,
    pub in_jail: bool,
    pub jail_turns: u8,
    pub get_out_cards: u8,
    pub is_bot: bool,
    pub is_bankrupt: bool,
    pub is_host: bool,
}

impl Player {
    pub fn new(id: Uuid, name: String, color: String, is_host: bool, is_bot: bool) -> Self {
        Self {
            id,
            name,
            color,
            position: 0,
            balance: 1500,
            in_jail: false,
            jail_turns: 0,
            get_out_cards: 0,
            is_bot,
            is_bankrupt: false,
            is_host,
        }
    }
}

/// State of a property on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyState {
    pub owner: Option<Uuid>,
    pub houses: u8, // 0-4 = houses, 5 = hotel
    pub is_mortgaged: bool,
}

impl Default for PropertyState {
    fn default() -> Self {
        Self {
            owner: None,
            houses: 0,
            is_mortgaged: false,
        }
    }
}

/// Auction state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionState {
    pub tile_idx: u8,
    pub current_bid: u32,
    pub highest_bidder: Option<Uuid>,
    pub passed_players: Vec<Uuid>,
}

impl AuctionState {
    pub fn new(tile_idx: u8) -> Self {
        Self {
            tile_idx,
            current_bid: 0,
            highest_bidder: None,
            passed_players: Vec::new(),
        }
    }
}

/// Trade offer between players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOffer {
    pub id: Uuid,
    pub from_player: Uuid,
    pub to_player: Uuid,
    pub offering: TradeAssets,
    pub requesting: TradeAssets,
    pub status: TradeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeAssets {
    pub money: u32,
    pub properties: Vec<u8>,
    pub get_out_cards: u8,
}

impl Default for TradeAssets {
    fn default() -> Self {
        Self {
            money: 0,
            properties: Vec::new(),
            get_out_cards: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeStatus {
    Pending,
    Accepted,
    Rejected,
    Countered,
}

/// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub id: String,
    pub phase: GamePhase,
    pub turn: Option<TurnState>,
    pub turn_order: Vec<Uuid>,
    pub current_turn_idx: usize,
    pub players: Vec<Player>,
    pub properties: HashMap<u8, PropertyState>,
    pub auction: Option<AuctionState>,
    pub active_trade: Option<TradeOffer>,
    pub pot_money: i32, // Free parking jackpot
    pub config: GameConfig,
    pub logs: Vec<String>,
}

impl GameState {
    pub fn new(id: String, config: GameConfig) -> Self {
        // Initialize property states for ownable tiles
        let mut properties = HashMap::new();
        for idx in 0..40u8 {
            // Skip non-ownable tiles (corners, tax, chance, chest)
            if !is_ownable_tile(idx) {
                continue;
            }
            properties.insert(idx, PropertyState::default());
        }

        Self {
            id,
            phase: GamePhase::Lobby,
            turn: None,
            turn_order: Vec::new(),
            current_turn_idx: 0,
            players: Vec::new(),
            properties,
            auction: None,
            active_trade: None,
            pot_money: 0,
            config,
            logs: Vec::new(),
        }
    }

    /// Get player by ID
    pub fn get_player(&self, id: Uuid) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    /// Get mutable player by ID
    pub fn get_player_mut(&mut self, id: Uuid) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == id)
    }

    /// Get current player
    pub fn current_player(&self) -> Option<&Player> {
        self.turn
            .as_ref()
            .and_then(|t| self.get_player(t.player_id))
    }

    /// Get next active player ID
    pub fn next_player_id(&self) -> Option<Uuid> {
        let active: Vec<_> = self
            .turn_order
            .iter()
            .filter(|id| {
                self.get_player(**id)
                    .map(|p| !p.is_bankrupt)
                    .unwrap_or(false)
            })
            .collect();

        if active.is_empty() {
            return None;
        }

        let current_idx = active
            .iter()
            .position(|id| {
                self.turn
                    .as_ref()
                    .map(|t| t.player_id == **id)
                    .unwrap_or(false)
            })
            .unwrap_or(0);

        let next_idx = (current_idx + 1) % active.len();
        Some(*active[next_idx])
    }

    /// Count active (non-bankrupt) players
    pub fn active_player_count(&self) -> usize {
        self.players.iter().filter(|p| !p.is_bankrupt).count()
    }

    /// Add log entry
    pub fn log(&mut self, message: String) {
        self.logs.push(message);
        // Keep last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

/// Check if a tile can be owned
fn is_ownable_tile(idx: u8) -> bool {
    // Corners: 0 (GO), 10 (Jail), 20 (Free Parking), 30 (Go to Jail)
    // Tax: 4 (Income Tax), 38 (Luxury Tax)
    // Chance: 7, 22, 36
    // Community Chest: 2, 17, 33
    !matches!(idx, 0 | 2 | 4 | 7 | 10 | 17 | 20 | 22 | 30 | 33 | 36 | 38)
}
