//! Client and Server events for WebSocket communication

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{GameState, TradeOffer};

/// Events sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    /// Roll the dice
    RollDice,

    /// Buy the property we landed on
    BuyProperty,

    /// Pass on buying, trigger auction
    PassProperty,

    /// End current turn
    EndTurn,

    /// Place a bid in auction
    Bid { amount: u32 },

    /// Pass on bidding
    PassBid,

    /// Pay to get out of jail
    PayJail,

    /// Use get out of jail free card
    UseCard,

    /// Build a house/hotel on property
    Build { tile_idx: u8 },

    /// Sell a house/hotel from property
    SellBuilding { tile_idx: u8 },

    /// Mortgage a property
    Mortgage { tile_idx: u8 },

    /// Unmortgage a property
    Unmortgage { tile_idx: u8 },

    /// Create a trade offer
    TradeOffer { offer: TradeOffer },

    /// Accept a trade
    TradeAccept { trade_id: Uuid },

    /// Reject a trade
    TradeReject { trade_id: Uuid },

    /// Counter a trade with new terms
    TradeCounter { trade_id: Uuid, offer: TradeOffer },

    /// Send chat message
    Chat { message: String },
}

/// Events sent from server to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    /// Full game state update
    GameState(GameState),

    /// Dice roll result
    DiceResult {
        player_id: Uuid,
        dice: (u8, u8),
        is_doubles: bool,
    },

    /// Player moved on board
    PlayerMoved {
        player_id: Uuid,
        from: u8,
        to: u8,
        passed_go: bool,
    },

    /// Property was purchased
    PropertyBought {
        tile_idx: u8,
        player_id: Uuid,
        price: u32,
    },

    /// Rent was paid
    RentPaid {
        from: Uuid,
        to: Uuid,
        amount: u32,
        tile_idx: u8,
    },

    /// Auction started
    AuctionStart { tile_idx: u8, starting_price: u32 },

    /// New bid in auction  
    BidPlaced { player_id: Uuid, amount: u32 },

    /// Player passed on auction
    BidPassed { player_id: Uuid },

    /// Auction ended
    AuctionEnd {
        tile_idx: u8,
        winner: Option<Uuid>,
        amount: u32,
    },

    /// Card drawn
    CardDrawn {
        player_id: Uuid,
        card_type: String,
        description: String,
    },

    /// Player sent to jail
    PlayerJailed { player_id: Uuid },

    /// Player freed from jail
    PlayerFreed {
        player_id: Uuid,
        method: String, // "dice", "paid", "card"
    },

    /// Player went bankrupt
    Bankruptcy {
        player_id: Uuid,
        creditor: Option<Uuid>,
    },

    /// Game ended
    GameOver { winner: Uuid },

    /// Trade proposed
    TradeProposed { trade: TradeOffer },

    /// Trade resolved
    TradeResolved { trade_id: Uuid, accepted: bool },

    /// Building constructed
    BuildingBuilt {
        tile_idx: u8,
        player_id: Uuid,
        houses: u8,
    },

    /// Building sold
    BuildingSold {
        tile_idx: u8,
        player_id: Uuid,
        houses: u8,
    },

    /// Property mortgaged
    PropertyMortgaged { tile_idx: u8, player_id: Uuid },

    /// Property unmortgaged
    PropertyUnmortgaged { tile_idx: u8, player_id: Uuid },

    /// Chat message
    Chat {
        from: Uuid,
        from_name: String,
        message: String,
    },

    /// Log message
    Log { message: String },

    /// Error message (sent to specific player)
    Error { message: String },

    /// Turn changed
    TurnChanged { player_id: Uuid },
}
