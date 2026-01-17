use super::{GameState, TradeAssets, TradeOffer, TradeStatus};
use uuid::Uuid;

pub struct TradeHandler;

impl TradeHandler {
    /// Create a new trade offer
    pub fn create_offer(
        game: &mut GameState,
        from: Uuid,
        to: Uuid,
        offering: TradeAssets,
        requesting: TradeAssets,
    ) -> Result<TradeOffer, String> {
        // Validation
        // 1. Check if 'from' player owns offered assets
        if !Self::validate_assets(game, from, &offering) {
            return Err("You do not own all the offered assets.".to_string());
        }

        // 2. Check if 'to' player owns requested assets
        if !Self::validate_assets(game, to, &requesting) {
            return Err("Target player does not own all the requested assets.".to_string());
        }

        // 3. Ensure no active trade (simplified: 1 global active trade or 1 per pair? structure allows 1 global 'active_trade' in MVP state)
        // Ideally we want a list of trades, but GameState has `active_trade: Option<TradeOffer>`.
        // Limitation: Only one trade at a time in the whole room? Or just one "viewed" trade?
        // Let's assume for MVP we allow overwriting or check if None.
        if game.active_trade.is_some() {
            // For strict MVP, maybe allow only one active trade in the room to simplify UI syncing
            return Err("There is already a pending trade in this room.".to_string());
        }

        let offer = TradeOffer {
            id: Uuid::new_v4(),
            from_player: from,
            to_player: to,
            offering,
            requesting,
            status: TradeStatus::Pending,
        };

        game.active_trade = Some(offer.clone());
        Ok(offer)
    }

    /// Validate that a player owns the specified assets
    fn validate_assets(game: &GameState, player_id: Uuid, assets: &TradeAssets) -> bool {
        let player = match game.get_player(player_id) {
            Some(p) => p,
            None => return false,
        };

        // Check Money
        if player.balance < assets.money as i32 {
            return false;
        }

        // Check Properties
        for &idx in &assets.properties {
            match game.properties.get(&idx) {
                Some(prop) => {
                    if prop.owner != Some(player_id) {
                        return false;
                    }
                    // Optional: Prevent trading mortgaged properties? Or allow? Rules say yes (mortgage stays).
                    // Optional: Prevent trading properties with buildings? Rules say must sell buildings first.
                    if prop.houses > 0 {
                        return false; // Cannot trade improved properties
                    }
                }
                None => return false,
            }
        }

        // Check Cards
        if player.get_out_cards < assets.get_out_cards {
            return false;
        }

        true
    }

    /// Accept the current active trade
    pub fn accept_trade(game: &mut GameState, trade_id: Uuid) -> Result<(), String> {
        let trade = match &game.active_trade {
            Some(t) if t.id == trade_id => t.clone(),
            _ => return Err("Trade offer not found or expired.".to_string()),
        };

        if trade.status != TradeStatus::Pending {
            return Err("Trade is no longer pending.".to_string());
        }

        // Re-validate ownership just in case state changed
        if !Self::validate_assets(game, trade.from_player, &trade.offering) {
            game.active_trade = None;
            return Err("Offer side assets no longer available.".to_string());
        }
        if !Self::validate_assets(game, trade.to_player, &trade.requesting) {
            game.active_trade = None;
            return Err("Request side assets no longer available.".to_string());
        }

        // Execute Transfer
        Self::transfer_assets(game, trade.from_player, trade.to_player, &trade.offering);
        Self::transfer_assets(game, trade.to_player, trade.from_player, &trade.requesting);

        game.active_trade = None;
        game.log("Trade completed successfully.".to_string());

        Ok(())
    }

    fn transfer_assets(game: &mut GameState, from: Uuid, to: Uuid, assets: &TradeAssets) {
        // Money
        if assets.money > 0 {
            if let Some(p) = game.get_player_mut(from) {
                p.balance -= assets.money as i32;
            }
            if let Some(p) = game.get_player_mut(to) {
                p.balance += assets.money as i32;
            }
        }

        // Properties
        for &idx in &assets.properties {
            if let Some(prop) = game.properties.get_mut(&idx) {
                prop.owner = Some(to);
            }
        }

        // Cards
        if assets.get_out_cards > 0 {
            if let Some(p) = game.get_player_mut(from) {
                p.get_out_cards -= assets.get_out_cards;
            }
            if let Some(p) = game.get_player_mut(to) {
                p.get_out_cards += assets.get_out_cards;
            }
        }
    }

    /// Reject active trade
    pub fn reject_trade(game: &mut GameState, trade_id: Uuid) -> Result<(), String> {
        let valid = match &game.active_trade {
            Some(t) => t.id == trade_id,
            None => false,
        };

        if valid {
            game.active_trade = None;
            game.log("Trade offer rejected.".to_string());
            Ok(())
        } else {
            Err("Trade not found.".to_string())
        }
    }
}
