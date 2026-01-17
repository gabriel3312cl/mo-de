//! Game Engine - Core game logic and state machine
//!
//! Simplified version that avoids borrow checker complexity by cloning state
//! where necessary for clarity and correctness.

use std::sync::Arc;

use rand::Rng;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::board::{get_tile, ColorGroup, TileType, BOARD};
use super::events::ServerEvent;
use super::state::*;
use crate::error::{AppError, AppResult};
use crate::ws::Hub;

/// Player colors for assignment
const PLAYER_COLORS: &[&str] = &[
    "#FF5733", "#33FF57", "#3357FF", "#FF33F5", "#F5FF33", "#33FFF5", "#FF8C33", "#8C33FF",
];

/// Bot name prefixes
const BOT_NAMES: &[&str] = &[
    "Bot Alpha",
    "Bot Beta",
    "Bot Gamma",
    "Bot Delta",
    "Bot Epsilon",
    "Bot Zeta",
    "Bot Eta",
    "Bot Theta",
];

pub struct GameEngine;

impl GameEngine {
    /// Create a new game room
    pub async fn create_room(
        redis: &ConnectionManager,
        host_name: &str,
        config: GameConfig,
    ) -> AppResult<(String, Uuid)> {
        let room_id = generate_room_id();
        let player_id = Uuid::new_v4();

        let mut game = GameState::new(room_id.clone(), config);

        let color = PLAYER_COLORS[0].to_string();
        let player = Player::new(player_id, host_name.into(), color, true, false);
        game.players.push(player);
        game.log(format!("{} created the room", host_name));

        Self::save_game(redis, &game).await?;

        Ok((room_id, player_id))
    }

    /// Join an existing room
    pub async fn join_room(
        redis: &ConnectionManager,
        room_id: &str,
        player_name: &str,
    ) -> AppResult<Uuid> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        if game.phase != GamePhase::Lobby {
            return Err(AppError::BadRequest("Game already started".into()));
        }

        if game.players.len() >= game.config.max_players as usize {
            return Err(AppError::BadRequest("Room is full".into()));
        }

        let player_id = Uuid::new_v4();
        let color = PLAYER_COLORS[game.players.len() % PLAYER_COLORS.len()].to_string();
        let player = Player::new(player_id, player_name.into(), color, false, false);

        game.log(format!("{} joined the game", player_name));
        game.players.push(player);

        Self::save_game(redis, &game).await?;

        Ok(player_id)
    }

    /// Add a bot to the room
    pub async fn add_bot(redis: &ConnectionManager, room_id: &str) -> AppResult<Uuid> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        if game.phase != GamePhase::Lobby {
            return Err(AppError::BadRequest("Game already started".into()));
        }

        if game.players.len() >= game.config.max_players as usize {
            return Err(AppError::BadRequest("Room is full".into()));
        }

        let bot_idx = game.players.iter().filter(|p| p.is_bot).count();
        let player_id = Uuid::new_v4();
        let color = PLAYER_COLORS[game.players.len() % PLAYER_COLORS.len()].to_string();
        let name = BOT_NAMES[bot_idx % BOT_NAMES.len()].to_string();
        let player = Player::new(player_id, name.clone(), color, false, true);

        game.log(format!("{} joined the game", name));
        game.players.push(player);

        Self::save_game(redis, &game).await?;

        Ok(player_id)
    }

    /// Start the game
    pub async fn start_game(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        if game.phase != GamePhase::Lobby {
            return Err(AppError::BadRequest("Game already started".into()));
        }

        if game.players.len() < 2 {
            return Err(AppError::BadRequest("Need at least 2 players".into()));
        }

        // Set starting cash
        let starting_cash = game.config.starting_cash;
        for player in &mut game.players {
            player.balance = starting_cash;
        }

        // Randomize player order (scoped to avoid RNG across await)
        let order = {
            let mut rng = rand::thread_rng();
            let mut order: Vec<Uuid> = game.players.iter().map(|p| p.id).collect();
            for i in (1..order.len()).rev() {
                let j = rng.gen_range(0..=i);
                order.swap(i, j);
            }
            order
        };
        game.turn_order = order.clone();

        // Start first turn
        let first_player = order[0];
        game.turn = Some(TurnState::new(first_player));
        game.phase = GamePhase::Playing;
        game.log("Game started!".into());

        // Check if first player is a bot
        let first_is_bot = game
            .get_player(first_player)
            .map(|p| p.is_bot)
            .unwrap_or(false);

        Self::save_game(redis, &game).await?;

        // Broadcast game start
        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(room_id, ServerEvent::GameState(game));
        }

        // Note: Bot processing will be triggered by frontend polling or separate mechanism
        // to avoid async recursion between end_turn and process_bot_turn
        let _ = first_is_bot; // Acknowledge the variable

        Ok(())
    }

    /// Handle a game event from a player
    pub async fn handle_event(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
        event: super::events::ClientEvent,
    ) -> AppResult<()> {
        use super::events::ClientEvent::*;

        let game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        // Verify it's this player's turn (for most actions)
        let is_current_player = game
            .turn
            .as_ref()
            .map(|t| t.player_id == player_id)
            .unwrap_or(false);

        match event {
            RollDice => {
                if !is_current_player {
                    return Err(AppError::Forbidden("Not your turn".into()));
                }
                Self::roll_dice(redis, hub, room_id).await?;
            }
            BuyProperty => {
                if !is_current_player {
                    return Err(AppError::Forbidden("Not your turn".into()));
                }
                Self::buy_property(redis, hub, room_id).await?;
            }
            PassProperty => {
                if !is_current_player {
                    return Err(AppError::Forbidden("Not your turn".into()));
                }
                Self::start_auction(redis, hub, room_id).await?;
            }
            EndTurn => {
                if !is_current_player {
                    return Err(AppError::Forbidden("Not your turn".into()));
                }
                Self::end_turn(redis, hub, room_id).await?;
            }
            Bid { amount } => {
                Self::place_bid(redis, hub, room_id, player_id, amount).await?;
            }
            PassBid => {
                Self::pass_bid(redis, hub, room_id, player_id).await?;
            }
            PayJail => {
                if !is_current_player {
                    return Err(AppError::Forbidden("Not your turn".into()));
                }
                Self::pay_jail(redis, hub, room_id).await?;
            }
            Build { tile_idx } => {
                Self::build_house(redis, hub, room_id, player_id, tile_idx).await?;
            }
            Mortgage { tile_idx } => {
                Self::mortgage_property(redis, hub, room_id, player_id, tile_idx).await?;
            }
            Unmortgage { tile_idx } => {
                Self::unmortgage_property(redis, hub, room_id, player_id, tile_idx).await?;
            }
            Chat { message } => {
                let player_name = game
                    .get_player(player_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".into());

                let hub_guard = hub.read().await;
                hub_guard.broadcast(
                    room_id,
                    ServerEvent::Chat {
                        from: player_id,
                        from_name: player_name,
                        message,
                    },
                );
            }
            _ => {
                tracing::warn!("Unhandled event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Roll dice and move player
    async fn roll_dice(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let turn = game
            .turn
            .as_mut()
            .ok_or_else(|| AppError::GameError("No active turn".into()))?;

        if turn.phase != TurnPhase::WaitingForRoll {
            return Err(AppError::GameError("Cannot roll now".into()));
        }

        // Roll dice (scoped to avoid RNG across await)
        let (d1, d2) = {
            let mut rng = rand::thread_rng();
            (rng.gen_range(1..=6), rng.gen_range(1..=6))
        };
        let is_doubles = d1 == d2;
        let dice_sum = d1 + d2;

        turn.dice = Some((d1, d2));
        turn.phase = TurnPhase::Moving;

        if is_doubles {
            turn.doubles_count += 1;
        }

        let player_id = turn.player_id;
        let doubles_count = turn.doubles_count;

        // Broadcast dice result
        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(
                room_id,
                ServerEvent::DiceResult {
                    player_id,
                    dice: (d1, d2),
                    is_doubles,
                },
            );
        }

        // Check for 3 doubles = jail
        if doubles_count >= 3 {
            Self::send_to_jail(&mut game, player_id);
            Self::save_game(redis, &game).await?;

            let hub_guard = hub.read().await;
            hub_guard.broadcast(room_id, ServerEvent::PlayerJailed { player_id });
            return Ok(());
        }

        // Get player data
        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        let in_jail = game.players[player_idx].in_jail;

        if in_jail {
            if is_doubles {
                // Freed by doubles
                game.players[player_idx].in_jail = false;
                game.players[player_idx].jail_turns = 0;
                let name = game.players[player_idx].name.clone();
                game.log(format!("{} rolled doubles and escaped jail!", name));

                let hub_guard = hub.read().await;
                hub_guard.broadcast(
                    room_id,
                    ServerEvent::PlayerFreed {
                        player_id,
                        method: "dice".into(),
                    },
                );
            } else {
                game.players[player_idx].jail_turns += 1;

                if game.players[player_idx].jail_turns >= 3 {
                    // Forced to pay
                    game.players[player_idx].balance -= 50;
                    game.players[player_idx].in_jail = false;
                    game.players[player_idx].jail_turns = 0;
                    let name = game.players[player_idx].name.clone();
                    game.log(format!("{} was forced to pay $50 bail", name));
                } else {
                    let name = game.players[player_idx].name.clone();
                    game.log(format!("{} failed to roll doubles in jail", name));
                    if let Some(t) = game.turn.as_mut() {
                        t.phase = TurnPhase::TurnEnd;
                        t.can_roll_again = false;
                    }
                    Self::save_game(redis, &game).await?;
                    return Ok(());
                }
            }
        }

        // Move player
        let old_pos = game.players[player_idx].position;
        let new_pos = (old_pos + dice_sum) % 40;
        let passed_go = new_pos < old_pos && old_pos != 0;

        game.players[player_idx].position = new_pos;

        if passed_go {
            game.players[player_idx].balance += 200;
            let name = game.players[player_idx].name.clone();
            game.log(format!("{} passed GO and collected $200", name));
        }

        // Broadcast movement
        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(
                room_id,
                ServerEvent::PlayerMoved {
                    player_id,
                    from: old_pos,
                    to: new_pos,
                    passed_go,
                },
            );
        }

        // Handle tile landing
        Self::handle_tile_landing(&mut game, player_id, new_pos)?;

        // Set can_roll_again if doubles (and not jailed)
        if is_doubles && !game.players[player_idx].in_jail {
            if let Some(t) = game.turn.as_mut() {
                t.can_roll_again = true;
            }
        }

        Self::save_game(redis, &game).await?;

        // Broadcast updated state
        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(room_id, ServerEvent::GameState(game));
        }

        Ok(())
    }

    /// Handle what happens when landing on a tile
    fn handle_tile_landing(game: &mut GameState, player_id: Uuid, tile_idx: u8) -> AppResult<()> {
        let tile = get_tile(tile_idx).ok_or_else(|| AppError::GameError("Invalid tile".into()))?;

        match tile.tile_type {
            TileType::Go => {
                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::Property | TileType::Railroad | TileType::Utility => {
                let owner = game.properties.get(&tile_idx).and_then(|p| p.owner);

                match owner {
                    None => {
                        if let Some(t) = game.turn.as_mut() {
                            t.phase = TurnPhase::BuyDecision;
                        }
                    }
                    Some(owner_id) if owner_id == player_id => {
                        if let Some(t) = game.turn.as_mut() {
                            t.phase = TurnPhase::TurnEnd;
                        }
                    }
                    Some(owner_id) => {
                        let is_mortgaged = game
                            .properties
                            .get(&tile_idx)
                            .map(|p| p.is_mortgaged)
                            .unwrap_or(false);

                        if !is_mortgaged {
                            let owner_in_jail = game
                                .get_player(owner_id)
                                .map(|p| p.in_jail)
                                .unwrap_or(false);

                            let collect_in_jail = game.config.collect_rent_in_jail;

                            if !owner_in_jail || collect_in_jail {
                                let rent = Self::calculate_rent(game, tile_idx);
                                Self::transfer_money(
                                    game,
                                    player_id,
                                    owner_id,
                                    rent as i32,
                                    &format!("rent on {}", tile.name),
                                );
                            }
                        }

                        if let Some(t) = game.turn.as_mut() {
                            t.phase = TurnPhase::TurnEnd;
                        }
                    }
                }
            }
            TileType::Tax => {
                let player_idx = game.players.iter().position(|p| p.id == player_id);
                if let Some(idx) = player_idx {
                    let tax = tile.rent_base as i32;
                    game.players[idx].balance -= tax;

                    if game.config.free_parking_jackpot {
                        game.pot_money += tax;
                    }

                    let name = game.players[idx].name.clone();
                    game.log(format!("{} paid ${} tax", name, tax));
                }

                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::Chance => {
                if let Some(p) = game.get_player(player_id) {
                    game.log(format!("{} drew a Surprise card", p.name));
                }
                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::CommunityChest => {
                if let Some(p) = game.get_player(player_id) {
                    game.log(format!("{} drew a Treasure card", p.name));
                }
                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::FreeParking => {
                if game.config.free_parking_jackpot && game.pot_money > 0 {
                    let pot = game.pot_money;
                    if let Some(idx) = game.players.iter().position(|p| p.id == player_id) {
                        game.players[idx].balance += pot;
                        let name = game.players[idx].name.clone();
                        game.log(format!("{} collected ${} from Free Parking!", name, pot));
                    }
                    game.pot_money = 0;
                }
                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::Jail => {
                if let Some(t) = game.turn.as_mut() {
                    t.phase = TurnPhase::TurnEnd;
                }
            }
            TileType::GoToJail => {
                Self::send_to_jail(game, player_id);
            }
        }

        Ok(())
    }

    /// Send a player to jail (internal helper)
    fn send_to_jail(game: &mut GameState, player_id: Uuid) {
        if let Some(idx) = game.players.iter().position(|p| p.id == player_id) {
            game.players[idx].position = 10;
            game.players[idx].in_jail = true;
            game.players[idx].jail_turns = 0;

            let name = game.players[idx].name.clone();
            game.log(format!("{} was sent to jail!", name));
        }

        if let Some(t) = game.turn.as_mut() {
            t.phase = TurnPhase::TurnEnd;
            t.can_roll_again = false;
            t.doubles_count = 0;
        }
    }

    /// Transfer money between players
    fn transfer_money(game: &mut GameState, from: Uuid, to: Uuid, amount: i32, reason: &str) {
        let from_idx = game.players.iter().position(|p| p.id == from);
        let to_idx = game.players.iter().position(|p| p.id == to);

        if let (Some(fi), Some(ti)) = (from_idx, to_idx) {
            let from_name = game.players[fi].name.clone();
            let to_name = game.players[ti].name.clone();

            game.players[fi].balance -= amount;
            game.players[ti].balance += amount;

            game.log(format!(
                "{} paid ${} to {} for {}",
                from_name, amount, to_name, reason
            ));
        }
    }

    /// Calculate rent for a property
    fn calculate_rent(game: &GameState, tile_idx: u8) -> u32 {
        let tile = match get_tile(tile_idx) {
            Some(t) => t,
            None => return 0,
        };

        let prop_state = match game.properties.get(&tile_idx) {
            Some(p) => p,
            None => return 0,
        };

        let owner_id = match prop_state.owner {
            Some(id) => id,
            None => return 0,
        };

        if prop_state.is_mortgaged {
            return 0;
        }

        match tile.tile_type {
            TileType::Property => {
                let houses = prop_state.houses;

                if houses > 0 {
                    tile.rent_schedule
                        .get((houses - 1) as usize)
                        .copied()
                        .unwrap_or(tile.rent_base)
                } else {
                    let group = tile.group.unwrap();
                    let has_full_set = Self::player_has_full_set(game, owner_id, group);

                    if has_full_set && game.config.double_rent_on_full_set {
                        tile.rent_base * 2
                    } else {
                        tile.rent_base
                    }
                }
            }
            TileType::Railroad => {
                let rr_count = game
                    .properties
                    .iter()
                    .filter(|(idx, state)| {
                        state.owner == Some(owner_id)
                            && get_tile(**idx)
                                .map(|t| t.tile_type == TileType::Railroad)
                                .unwrap_or(false)
                    })
                    .count();

                tile.rent_schedule
                    .get(rr_count.saturating_sub(1))
                    .copied()
                    .unwrap_or(25)
            }
            TileType::Utility => {
                let util_count = game
                    .properties
                    .iter()
                    .filter(|(idx, state)| {
                        state.owner == Some(owner_id)
                            && get_tile(**idx)
                                .map(|t| t.tile_type == TileType::Utility)
                                .unwrap_or(false)
                    })
                    .count();

                let multiplier = if util_count >= 2 { 10 } else { 4 };
                let dice_sum = game.turn.as_ref().map(|t| t.dice_sum() as u32).unwrap_or(7);

                dice_sum * multiplier
            }
            _ => 0,
        }
    }

    /// Check if player owns all properties in a color group
    fn player_has_full_set(game: &GameState, player_id: Uuid, group: ColorGroup) -> bool {
        let group_tiles: Vec<u8> = BOARD
            .iter()
            .filter(|t| t.group == Some(group))
            .map(|t| t.index)
            .collect();

        group_tiles.iter().all(|idx| {
            game.properties
                .get(idx)
                .map(|p| p.owner == Some(player_id))
                .unwrap_or(false)
        })
    }

    /// Buy the property the current player is on
    async fn buy_property(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let (player_id, position) = {
            let turn = game
                .turn
                .as_ref()
                .ok_or_else(|| AppError::GameError("No active turn".into()))?;

            if turn.phase != TurnPhase::BuyDecision {
                return Err(AppError::GameError("Cannot buy now".into()));
            }

            (
                turn.player_id,
                game.get_player(turn.player_id)
                    .map(|p| p.position)
                    .unwrap_or(0),
            )
        };

        let tile = get_tile(position).ok_or_else(|| AppError::GameError("Invalid tile".into()))?;

        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        if game.players[player_idx].balance < tile.price as i32 {
            return Err(AppError::GameError("Not enough money".into()));
        }

        // Deduct and assign
        game.players[player_idx].balance -= tile.price as i32;
        let player_name = game.players[player_idx].name.clone();

        if let Some(prop) = game.properties.get_mut(&position) {
            prop.owner = Some(player_id);
        }

        game.log(format!(
            "{} bought {} for ${}",
            player_name, tile.name, tile.price
        ));

        if let Some(t) = game.turn.as_mut() {
            t.phase = TurnPhase::TurnEnd;
        }

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::PropertyBought {
                tile_idx: position,
                player_id,
                price: tile.price,
            },
        );

        Ok(())
    }

    /// Start an auction for the current property
    async fn start_auction(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let position = {
            let turn = game
                .turn
                .as_ref()
                .ok_or_else(|| AppError::GameError("No active turn".into()))?;

            if turn.phase != TurnPhase::BuyDecision {
                return Err(AppError::GameError("Cannot start auction now".into()));
            }

            game.get_player(turn.player_id)
                .map(|p| p.position)
                .unwrap_or(0)
        };

        if !game.config.auction_on_decline {
            if let Some(t) = game.turn.as_mut() {
                t.phase = TurnPhase::TurnEnd;
            }
            Self::save_game(redis, &game).await?;
            return Ok(());
        }

        game.auction = Some(AuctionState::new(position));

        if let Some(t) = game.turn.as_mut() {
            t.phase = TurnPhase::Auction;
        }

        let tile_name = get_tile(position)
            .map(|t| t.name.clone())
            .unwrap_or_default();
        game.log(format!("Auction started for {}", tile_name));

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::AuctionStart {
                tile_idx: position,
                starting_price: 0,
            },
        );

        Ok(())
    }

    /// Place a bid in the current auction
    async fn place_bid(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
        amount: u32,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let player_balance = game.get_player(player_id).map(|p| p.balance).unwrap_or(0);

        if player_balance < amount as i32 {
            return Err(AppError::GameError("Not enough money".into()));
        }

        let current_bid = game.auction.as_ref().map(|a| a.current_bid).unwrap_or(0);

        if amount <= current_bid {
            return Err(AppError::GameError("Bid must be higher".into()));
        }

        if let Some(auction) = game.auction.as_mut() {
            auction.current_bid = amount;
            auction.highest_bidder = Some(player_id);
        }

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(room_id, ServerEvent::BidPlaced { player_id, amount });

        Ok(())
    }

    /// Pass on the current auction
    async fn pass_bid(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        if let Some(auction) = game.auction.as_mut() {
            if !auction.passed_players.contains(&player_id) {
                auction.passed_players.push(player_id);
            }
        }

        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(room_id, ServerEvent::BidPassed { player_id });
        }

        // Check if auction should end
        let active_count = game.players.iter().filter(|p| !p.is_bankrupt).count();

        let passed_count = game
            .auction
            .as_ref()
            .map(|a| a.passed_players.len())
            .unwrap_or(0);

        if passed_count >= active_count - 1 || passed_count >= active_count {
            Self::end_auction(redis, hub, room_id, &mut game).await?;
        } else {
            Self::save_game(redis, &game).await?;
        }

        Ok(())
    }

    /// End the current auction
    async fn end_auction(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        game: &mut GameState,
    ) -> AppResult<()> {
        let auction = match game.auction.take() {
            Some(a) => a,
            None => return Ok(()),
        };

        let tile_idx = auction.tile_idx;
        let tile_name = get_tile(tile_idx)
            .map(|t| t.name.clone())
            .unwrap_or_default();

        if let Some(winner_id) = auction.highest_bidder {
            let amount = auction.current_bid;

            if let Some(idx) = game.players.iter().position(|p| p.id == winner_id) {
                game.players[idx].balance -= amount as i32;
                let winner_name = game.players[idx].name.clone();

                if let Some(prop) = game.properties.get_mut(&tile_idx) {
                    prop.owner = Some(winner_id);
                }

                game.log(format!(
                    "{} won {} at auction for ${}",
                    winner_name, tile_name, amount
                ));
            }

            let hub_guard = hub.read().await;
            hub_guard.broadcast(
                room_id,
                ServerEvent::AuctionEnd {
                    tile_idx,
                    winner: Some(winner_id),
                    amount,
                },
            );
        } else {
            game.log(format!("Auction for {} ended with no bids", tile_name));

            let hub_guard = hub.read().await;
            hub_guard.broadcast(
                room_id,
                ServerEvent::AuctionEnd {
                    tile_idx,
                    winner: None,
                    amount: 0,
                },
            );
        }

        if let Some(t) = game.turn.as_mut() {
            t.phase = TurnPhase::TurnEnd;
        }

        Self::save_game(redis, game).await?;

        Ok(())
    }

    /// Pay to get out of jail
    async fn pay_jail(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let player_id = game
            .turn
            .as_ref()
            .map(|t| t.player_id)
            .ok_or_else(|| AppError::GameError("No active turn".into()))?;

        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        if !game.players[player_idx].in_jail {
            return Err(AppError::GameError("Not in jail".into()));
        }

        if game.players[player_idx].balance < 50 {
            return Err(AppError::GameError("Not enough money".into()));
        }

        game.players[player_idx].balance -= 50;
        game.players[player_idx].in_jail = false;
        game.players[player_idx].jail_turns = 0;

        let name = game.players[player_idx].name.clone();
        game.log(format!("{} paid $50 to get out of jail", name));

        if let Some(t) = game.turn.as_mut() {
            t.phase = TurnPhase::WaitingForRoll;
        }

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::PlayerFreed {
                player_id,
                method: "paid".into(),
            },
        );

        Ok(())
    }

    /// End the current turn
    async fn end_turn(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let can_roll_again = game
            .turn
            .as_ref()
            .map(|t| t.can_roll_again)
            .unwrap_or(false);

        if can_roll_again {
            if let Some(t) = game.turn.as_mut() {
                t.phase = TurnPhase::WaitingForRoll;
                t.can_roll_again = false;
            }
            Self::save_game(redis, &game).await?;

            // Note: Bot processing deferred to avoid async recursion
            return Ok(());
        }

        // Move to next player
        let next_player_id = game
            .next_player_id()
            .ok_or_else(|| AppError::GameError("No next player".into()))?;

        game.turn = Some(TurnState::new(next_player_id));

        // Check for game over
        if game.active_player_count() <= 1 {
            game.phase = GamePhase::GameOver;
            let winner_id = game
                .players
                .iter()
                .find(|p| !p.is_bankrupt)
                .map(|p| p.id)
                .unwrap();

            let winner_name = game
                .get_player(winner_id)
                .map(|p| p.name.clone())
                .unwrap_or_default();

            game.log(format!("{} wins the game!", winner_name));

            Self::save_game(redis, &game).await?;

            let hub_guard = hub.read().await;
            hub_guard.broadcast(room_id, ServerEvent::GameOver { winner: winner_id });

            return Ok(());
        }

        let next_name = game
            .get_player(next_player_id)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        let is_next_bot = game
            .get_player(next_player_id)
            .map(|p| p.is_bot)
            .unwrap_or(false);

        game.log(format!("{}'s turn", next_name));

        Self::save_game(redis, &game).await?;

        {
            let hub_guard = hub.read().await;
            hub_guard.broadcast(
                room_id,
                ServerEvent::TurnChanged {
                    player_id: next_player_id,
                },
            );
        }

        // Note: Bot processing will be triggered by frontend polling or separate mechanism
        // to avoid async recursion between end_turn and process_bot_turn
        let _ = is_next_bot; // Acknowledge the variable

        Ok(())
    }

    /// Build a house on a property
    async fn build_house(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
        tile_idx: u8,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let tile = get_tile(tile_idx).ok_or_else(|| AppError::GameError("Invalid tile".into()))?;

        if tile.tile_type != TileType::Property {
            return Err(AppError::GameError("Cannot build on this tile".into()));
        }

        let group = tile
            .group
            .ok_or_else(|| AppError::GameError("No color group".into()))?;

        if !Self::player_has_full_set(&game, player_id, group) {
            return Err(AppError::GameError("Must own full color set".into()));
        }

        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        if game.players[player_idx].balance < tile.build_cost as i32 {
            return Err(AppError::GameError("Not enough money".into()));
        }

        let current_houses = game
            .properties
            .get(&tile_idx)
            .map(|p| p.houses)
            .unwrap_or(0);

        if current_houses >= 5 {
            return Err(AppError::GameError("Already at max buildings".into()));
        }

        // Build
        game.players[player_idx].balance -= tile.build_cost as i32;

        if let Some(prop) = game.properties.get_mut(&tile_idx) {
            prop.houses += 1;
        }

        let houses = current_houses + 1;
        let building_type = if houses == 5 { "hotel" } else { "house" };
        let player_name = game.players[player_idx].name.clone();
        game.log(format!(
            "{} built a {} on {}",
            player_name, building_type, tile.name
        ));

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::BuildingBuilt {
                tile_idx,
                player_id,
                houses,
            },
        );

        Ok(())
    }

    /// Mortgage a property
    async fn mortgage_property(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
        tile_idx: u8,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let tile = get_tile(tile_idx).ok_or_else(|| AppError::GameError("Invalid tile".into()))?;

        let prop_state = game
            .properties
            .get(&tile_idx)
            .ok_or_else(|| AppError::GameError("Not a property".into()))?;

        if prop_state.owner != Some(player_id) {
            return Err(AppError::GameError("You don't own this property".into()));
        }

        if prop_state.is_mortgaged {
            return Err(AppError::GameError("Already mortgaged".into()));
        }

        if prop_state.houses > 0 {
            return Err(AppError::GameError("Must sell buildings first".into()));
        }

        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        game.players[player_idx].balance += tile.mortgage_value as i32;
        let player_name = game.players[player_idx].name.clone();

        if let Some(prop) = game.properties.get_mut(&tile_idx) {
            prop.is_mortgaged = true;
        }

        game.log(format!(
            "{} mortgaged {} for ${}",
            player_name, tile.name, tile.mortgage_value
        ));

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::PropertyMortgaged {
                tile_idx,
                player_id,
            },
        );

        Ok(())
    }

    /// Unmortgage a property
    async fn unmortgage_property(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
        player_id: Uuid,
        tile_idx: u8,
    ) -> AppResult<()> {
        let mut game = Self::get_game(redis, room_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Room not found".into()))?;

        let tile = get_tile(tile_idx).ok_or_else(|| AppError::GameError("Invalid tile".into()))?;

        let prop_state = game
            .properties
            .get(&tile_idx)
            .ok_or_else(|| AppError::GameError("Not a property".into()))?;

        if prop_state.owner != Some(player_id) {
            return Err(AppError::GameError("You don't own this property".into()));
        }

        if !prop_state.is_mortgaged {
            return Err(AppError::GameError("Not mortgaged".into()));
        }

        let unmortgage_cost = (tile.mortgage_value as f32 * 1.1) as i32;

        let player_idx = game
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| AppError::GameError("Player not found".into()))?;

        if game.players[player_idx].balance < unmortgage_cost {
            return Err(AppError::GameError("Not enough money".into()));
        }

        game.players[player_idx].balance -= unmortgage_cost;
        let player_name = game.players[player_idx].name.clone();

        if let Some(prop) = game.properties.get_mut(&tile_idx) {
            prop.is_mortgaged = false;
        }

        game.log(format!(
            "{} unmortgaged {} for ${}",
            player_name, tile.name, unmortgage_cost
        ));

        Self::save_game(redis, &game).await?;

        let hub_guard = hub.read().await;
        hub_guard.broadcast(
            room_id,
            ServerEvent::PropertyUnmortgaged {
                tile_idx,
                player_id,
            },
        );

        Ok(())
    }

    /// Process a bot's turn (iterative to avoid async recursion)
    async fn process_bot_turn(
        redis: &ConnectionManager,
        hub: &Arc<RwLock<Hub>>,
        room_id: &str,
    ) -> AppResult<()> {
        // Use a loop instead of recursion to avoid Box::pin complexity
        loop {
            // Small delay for realism
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

            let game = match Self::get_game(redis, room_id).await? {
                Some(g) => g,
                None => return Err(AppError::NotFound("Room not found".into())),
            };

            let turn = match game.turn.as_ref() {
                Some(t) => t,
                None => return Ok(()), // No active turn, exit
            };

            // Check if current player is still a bot
            let is_bot = game
                .get_player(turn.player_id)
                .map(|p| p.is_bot)
                .unwrap_or(false);

            if !is_bot {
                return Ok(()); // Not a bot's turn anymore
            }

            match turn.phase {
                TurnPhase::WaitingForRoll => {
                    Self::roll_dice(redis, hub, room_id).await?;
                    // Continue loop to handle next phase
                }
                TurnPhase::BuyDecision => {
                    let player_id = turn.player_id;
                    let position = game.get_player(player_id).map(|p| p.position).unwrap_or(0);
                    let balance = game.get_player(player_id).map(|p| p.balance).unwrap_or(0);

                    if let Some(tile) = get_tile(position) {
                        // Simple bot logic: buy if we have more than 40% extra
                        if balance as u32 > tile.price + (tile.price * 4 / 10) {
                            Self::buy_property(redis, hub, room_id).await?;
                        } else {
                            Self::start_auction(redis, hub, room_id).await?;
                        }
                    }
                    // Continue loop to handle TurnEnd
                }
                TurnPhase::TurnEnd => {
                    Self::end_turn(redis, hub, room_id).await?;
                    return Ok(()); // end_turn will call process_bot_turn if needed
                }
                TurnPhase::Auction => {
                    // Bot should bid or pass
                    let player_id = turn.player_id;
                    if let Some(auction) = &game.auction {
                        if !auction.passed_players.contains(&player_id) {
                            // Simple: just pass for now
                            Self::pass_bid(redis, hub, room_id, player_id).await?;
                        }
                    }
                    return Ok(()); // Auction handled
                }
                _ => {
                    // For other phases, wait
                    return Ok(());
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }
    }

    // === Redis Storage ===

    /// Get game state from Redis
    pub async fn get_game(
        redis: &ConnectionManager,
        room_id: &str,
    ) -> AppResult<Option<GameState>> {
        let mut conn = redis.clone();
        let key = format!("game:{}", room_id);

        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => {
                let game: GameState =
                    serde_json::from_str(&json).map_err(|e| AppError::Internal(e.into()))?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    /// Save game state to Redis
    pub async fn save_game(redis: &ConnectionManager, game: &GameState) -> AppResult<()> {
        let mut conn = redis.clone();
        let key = format!("game:{}", game.id);
        let json = serde_json::to_string(game).map_err(|e| AppError::Internal(e.into()))?;

        // Store with 24 hour expiry
        let _: () = conn.set_ex(&key, json, 86400).await?;

        Ok(())
    }
}

/// Generate a short room ID (6 chars)
fn generate_room_id() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
