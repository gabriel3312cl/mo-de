-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) UNIQUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Games
CREATE TABLE IF NOT EXISTS games (
    id UUID PRIMARY KEY,
    winner_id UUID REFERENCES users(id),
    config JSONB NOT NULL,
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ,
    final_state JSONB
);

-- Game Players (Participants)
CREATE TABLE IF NOT EXISTS game_players (
    game_id UUID REFERENCES games(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id),
    player_name VARCHAR(50),
    final_position INT,
    final_balance INT,
    is_bot BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (game_id, user_id)
);

-- Leaderboard View
CREATE MATERIALIZED VIEW IF NOT EXISTS leaderboard AS
SELECT 
    u.id,
    u.username,
    COUNT(*) FILTER (WHERE gp.final_position = 1) as wins,
    COUNT(*) as games_played
FROM users u
JOIN game_players gp ON u.id = gp.user_id
GROUP BY u.id, u.username
ORDER BY wins DESC;
