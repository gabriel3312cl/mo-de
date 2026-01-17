import { Paper, List, ListItem, ListItemAvatar, Avatar, ListItemText, Typography, Chip, Box } from '@mui/material';
import PersonIcon from '@mui/icons-material/Person';
import SmartToyIcon from '@mui/icons-material/SmartToy';
import { useGameStore } from '@/stores/gameStore';
import { GamePlayer } from '@/types';

export default function PlayerList() {
    const { gameState } = useGameStore();

    if (!gameState) return null;

    const players = gameState.players as GamePlayer[];
    const currentPlayerId = gameState.turn.player_id;

    return (
        <Paper
            elevation={3}
            sx={{
                width: 250,
                bgcolor: 'rgba(26, 32, 53, 0.9)',
                color: 'white',
                maxHeight: '40vh',
                overflowY: 'auto'
            }}
        >
            <Box sx={{ p: 2, borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                <Typography variant="h6">Players</Typography>
            </Box>
            <List dense>
                {players.map((player) => {
                    const isActive = player.id === currentPlayerId;
                    return (
                        <ListItem
                            key={player.id}
                            sx={{
                                bgcolor: isActive ? 'rgba(124, 58, 237, 0.2)' : 'transparent',
                                borderLeft: isActive ? '4px solid #7c3aed' : '4px solid transparent'
                            }}
                        >
                            <ListItemAvatar>
                                <Avatar sx={{ bgcolor: player.color, width: 32, height: 32 }}>
                                    {player.is_bot ? <SmartToyIcon fontSize="small" /> : <PersonIcon fontSize="small" />}
                                </Avatar>
                            </ListItemAvatar>
                            <ListItemText
                                primary={
                                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                                        <Typography variant="body2" fontWeight="bold">{player.name}</Typography>
                                        {player.is_in_jail && <Chip label="JAIL" size="small" color="error" sx={{ height: 16, fontSize: '0.6rem' }} />}
                                    </Box>
                                }
                                secondary={
                                    <Typography variant="body2" color="success.main" fontWeight="bold">
                                        ${player.money.toLocaleString()}
                                    </Typography>
                                }
                            />
                        </ListItem>
                    );
                })}
            </List>
        </Paper>
    );
}
