//! Bot strategy constants and helpers

/// Strategy profile for bots
#[derive(Debug, Clone, Copy)]
pub enum BotPersonality {
    /// Aggressive - buys everything, bids high
    Aggressive,
    /// Conservative - only buys good deals
    Conservative,
    /// Balanced - standard strategy
    Balanced,
}

impl BotPersonality {
    /// Get buy threshold multiplier (higher = more willing to spend)
    pub fn buy_threshold(&self) -> f32 {
        match self {
            BotPersonality::Aggressive => 0.7,
            BotPersonality::Conservative => 0.4,
            BotPersonality::Balanced => 0.55,
        }
    }

    /// Get bid multiplier (how much over base value)
    pub fn bid_multiplier(&self) -> f32 {
        match self {
            BotPersonality::Aggressive => 1.5,
            BotPersonality::Conservative => 1.1,
            BotPersonality::Balanced => 1.3,
        }
    }

    /// How quickly to build houses
    pub fn build_threshold(&self) -> i32 {
        match self {
            BotPersonality::Aggressive => 100, // Build if have $100+ after
            BotPersonality::Conservative => 500,
            BotPersonality::Balanced => 250,
        }
    }
}

impl Default for BotPersonality {
    fn default() -> Self {
        BotPersonality::Balanced
    }
}
