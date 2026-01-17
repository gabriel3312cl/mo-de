//! Bot AI decision making (deterministic, no LLM)

use uuid::Uuid;

use crate::game::board::{get_tile, ColorGroup, TileType, BOARD};
use crate::game::state::GameState;

/// Bot AI decision engine
pub struct BotAI;

/// Property priority based on Monopoly statistics
#[derive(Debug, Clone, Copy)]
pub struct PropertyPriority {
    pub group: ColorGroup,
    pub priority: u8,
}

impl BotAI {
    /// Get property priorities (based on landing statistics)
    pub fn get_priorities() -> Vec<PropertyPriority> {
        vec![
            PropertyPriority {
                group: ColorGroup::Orange,
                priority: 5,
            },
            PropertyPriority {
                group: ColorGroup::Red,
                priority: 5,
            },
            PropertyPriority {
                group: ColorGroup::Yellow,
                priority: 4,
            },
            PropertyPriority {
                group: ColorGroup::Railroad,
                priority: 4,
            },
            PropertyPriority {
                group: ColorGroup::Green,
                priority: 3,
            },
            PropertyPriority {
                group: ColorGroup::Pink,
                priority: 3,
            },
            PropertyPriority {
                group: ColorGroup::LightBlue,
                priority: 2,
            },
            PropertyPriority {
                group: ColorGroup::DarkBlue,
                priority: 2,
            },
            PropertyPriority {
                group: ColorGroup::Brown,
                priority: 2,
            },
            PropertyPriority {
                group: ColorGroup::Utility,
                priority: 1,
            },
        ]
    }

    /// Decide whether to buy a property
    pub fn should_buy(game: &GameState, bot_id: Uuid, tile_idx: u8) -> bool {
        let bot = match game.get_player(bot_id) {
            Some(p) => p,
            None => return false,
        };

        let tile = match get_tile(tile_idx) {
            Some(t) => t,
            None => return false,
        };

        let group = match tile.group {
            Some(g) => g,
            None => return false,
        };

        let priority = Self::get_priorities()
            .iter()
            .find(|p| p.group == group)
            .map(|p| p.priority)
            .unwrap_or(1);

        let owned_in_group: usize = game
            .properties
            .iter()
            .filter(|(idx, state)| {
                state.owner == Some(bot_id) && get_tile(**idx).and_then(|t| t.group) == Some(group)
            })
            .count();

        let group_size = group.property_count() as usize;

        let max_percent: u32 = match (priority, owned_in_group) {
            (5, n) if n >= group_size - 1 => 80,
            (5, _) => 60,
            (4, n) if n >= group_size - 1 => 70,
            (4, _) => 50,
            (3, n) if n >= group_size - 1 => 60,
            (3, _) => 40,
            (_, n) if n >= group_size - 1 => 50,
            (_, _) => 30,
        };

        let max_spend = (bot.balance as u32 * max_percent) / 100;

        tile.price <= max_spend
    }

    /// Calculate max bid for auction
    pub fn calculate_max_bid(game: &GameState, bot_id: Uuid, tile_idx: u8) -> u32 {
        let bot = match game.get_player(bot_id) {
            Some(p) => p,
            None => return 0,
        };

        let tile = match get_tile(tile_idx) {
            Some(t) => t,
            None => return 0,
        };

        let group = match tile.group {
            Some(g) => g,
            None => return 0,
        };

        let priority = Self::get_priorities()
            .iter()
            .find(|p| p.group == group)
            .map(|p| p.priority)
            .unwrap_or(1);

        let owned_in_group: usize = game
            .properties
            .iter()
            .filter(|(idx, state)| {
                state.owner == Some(bot_id) && get_tile(**idx).and_then(|t| t.group) == Some(group)
            })
            .count();

        let group_size = group.property_count() as usize;
        let would_complete_set = owned_in_group >= group_size - 1;

        let blocks_opponent = game
            .players
            .iter()
            .filter(|p| p.id != bot_id && !p.is_bankrupt)
            .any(|p| {
                let their_count: usize = game
                    .properties
                    .iter()
                    .filter(|(idx, state)| {
                        state.owner == Some(p.id)
                            && get_tile(**idx).and_then(|t| t.group) == Some(group)
                    })
                    .count();
                their_count >= group_size - 1
            });

        let mut value = tile.price as f32;

        if would_complete_set {
            value *= 1.8;
        }
        if blocks_opponent {
            value *= 1.5;
        }
        value *= 1.0 + (priority as f32 * 0.1);

        let max_spend = (bot.balance as f32 * 0.5) as u32;

        (value as u32).min(max_spend)
    }

    /// Get properties the bot should build on
    pub fn get_build_targets(game: &GameState, bot_id: Uuid) -> Vec<u8> {
        let mut targets: Vec<u8> = Vec::new();

        let bot = match game.get_player(bot_id) {
            Some(p) => p,
            None => return targets,
        };

        for priority in Self::get_priorities() {
            let group = priority.group;
            let group_tiles: Vec<u8> = BOARD
                .iter()
                .filter(|t| t.group == Some(group) && t.tile_type == TileType::Property)
                .map(|t| t.index)
                .collect();

            let owns_all = group_tiles.iter().all(|idx| {
                game.properties
                    .get(idx)
                    .map(|p| p.owner == Some(bot_id) && !p.is_mortgaged)
                    .unwrap_or(false)
            });

            if !owns_all {
                continue;
            }

            let tile = match get_tile(group_tiles[0]) {
                Some(t) => t,
                None => continue,
            };
            let build_cost = tile.build_cost;

            if bot.balance < build_cost as i32 {
                continue;
            }

            let min_houses = group_tiles
                .iter()
                .filter_map(|idx| game.properties.get(idx))
                .map(|p| p.houses)
                .min()
                .unwrap_or(0);

            if min_houses >= 5 {
                continue;
            }

            for idx in &group_tiles {
                if let Some(prop) = game.properties.get(idx) {
                    if prop.houses == min_houses {
                        targets.push(*idx);
                        break;
                    }
                }
            }
        }

        targets
    }

    /// Decide whether to pay to leave jail
    pub fn should_pay_jail(game: &GameState, bot_id: Uuid) -> bool {
        let bot = match game.get_player(bot_id) {
            Some(p) => p,
            None => return false,
        };

        let unowned_properties: usize = game
            .properties
            .iter()
            .filter(|(_, state)| state.owner.is_none())
            .count();

        let total_properties = game.properties.len();
        let game_progress = 1.0 - (unowned_properties as f32 / total_properties as f32);

        if game_progress < 0.5 && bot.balance >= 50 {
            return true;
        }

        if bot.balance < 200 {
            return false;
        }

        if bot.get_out_cards > 0 {
            return false;
        }

        bot.balance >= 100
    }

    /// Evaluate a trade offer
    pub fn evaluate_trade(offering_value: i32, requesting_value: i32) -> TradeDecision {
        if offering_value as f32 > requesting_value as f32 * 1.2 {
            TradeDecision::Accept
        } else if offering_value as f32 > requesting_value as f32 * 0.8 {
            TradeDecision::Counter
        } else {
            TradeDecision::Reject
        }
    }

    /// Calculate value of a property for trade evaluation
    pub fn calculate_property_value(game: &GameState, player_id: Uuid, tile_idx: u8) -> i32 {
        let tile = match get_tile(tile_idx) {
            Some(t) => t,
            None => return 0,
        };

        let group = match tile.group {
            Some(g) => g,
            None => return tile.price as i32,
        };

        let base = tile.price as i32;

        let owned_in_group: usize = game
            .properties
            .iter()
            .filter(|(idx, state)| {
                state.owner == Some(player_id)
                    && get_tile(**idx).and_then(|t| t.group) == Some(group)
            })
            .count();

        let group_size = group.property_count() as usize;

        let multiplier: f32 = match group_size.saturating_sub(owned_in_group) {
            0 => 0.5,
            1 => 2.5,
            2 => 1.5,
            _ => 1.0,
        };

        (base as f32 * multiplier) as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeDecision {
    Accept,
    Reject,
    Counter,
}
