use super::GameState;
use uuid::Uuid;

pub struct BankruptcyHandler;

impl BankruptcyHandler {
    /// Check if a player is bankrupt (balance < 0)
    /// Returns true if bankrupt
    pub fn is_bankrupt(game: &GameState, player_id: Uuid) -> bool {
        if let Some(player) = game.get_player(player_id) {
            return player.balance < 0;
        }
        false
    }

    /// Handle bankruptcy processing
    /// creditor_id: None if debt is to Bank, Some(id) if debt is to another player
    pub fn handle_bankruptcy(game: &mut GameState, debtor_id: Uuid, creditor_id: Option<Uuid>) {
        // 1. Mark player as bankrupt and reset balance
        let player_name = if let Some(player) = game.get_player_mut(debtor_id) {
            player.is_bankrupt = true;
            player.balance = 0;
            player.name.clone()
        } else {
            "Unknown".to_string()
        };

        game.log(format!("Player {} has gone BANKRUPT!", player_name));

        // 2. Identify assets (properties)
        let mut debtor_properties: Vec<u8> = Vec::new();
        for (idx, prop) in game.properties.iter() {
            if prop.owner == Some(debtor_id) {
                debtor_properties.push(*idx);
            }
        }

        // 3. Transfer assets
        if let Some(creditor) = creditor_id {
            // Transfer to creditor
            // Log first
            if let Some(creditor_player) = game.get_player(creditor) {
                let msg = format!("All assets transferred to {}.", creditor_player.name);
                game.log(msg);
            }

            for idx in debtor_properties {
                if let Some(prop) = game.properties.get_mut(&idx) {
                    prop.owner = Some(creditor);
                    // Reset mortgages? Usually creditor must pay 10% interest immediately or pay off mortgage.
                    // For MVP: Transfer as is.
                }
            }

            // Transfer Get Out of Jail cards
            let cards = game
                .get_player(debtor_id)
                .map(|p| p.get_out_cards)
                .unwrap_or(0);
            if cards > 0 {
                // Remove from debtor
                if let Some(p) = game.get_player_mut(debtor_id) {
                    p.get_out_cards = 0;
                }
                // Add to creditor
                if let Some(p) = game.get_player_mut(creditor) {
                    p.get_out_cards += cards;
                }
            }
        } else {
            // Debt to Bank -> Reset properties (Auction in real rules, Reset for MVP)
            game.log("Assets returned to the Bank.".to_string());
            for idx in debtor_properties {
                if let Some(prop) = game.properties.get_mut(&idx) {
                    prop.owner = None;
                    prop.houses = 0;
                    prop.is_mortgaged = false;
                }
            }
            // Jail cards returned to deck (just delete from player)
            if let Some(p) = game.get_player_mut(debtor_id) {
                p.get_out_cards = 0;
            }
        }

        // 4. Cleanup
        // We do NOT remove the player from the vector to preserve indices/Turn order integrity for now,
        // just keep is_bankrupt = true.
    }
}
