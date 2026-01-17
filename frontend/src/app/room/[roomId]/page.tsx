'use client';
import { useEffect, useState } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { Box, Typography, Container, Paper, List, ListItem, ListItemText, ListItemAvatar, Avatar, Chip, Button, Stack, CircularProgress } from '@mui/material';
import PersonIcon from '@mui/icons-material/Person';
import SmartToyIcon from '@mui/icons-material/SmartToy';
import StarIcon from '@mui/icons-material/Star';
import { api } from '@/utils/api';
import { RoomState } from '@/types';
import { useWsStore } from '@/stores/wsStore';
import { useGameStore } from '@/stores/gameStore';
import GameBoard from '@/components/Board/GameBoard';
import GameControls from '@/components/Game/GameControls';
import DiceDisplay from '@/components/Game/DiceDisplay';
import PlayerList from '@/components/Game/PlayerList';
import EventLog from '@/components/Game/EventLog';
import AuctionModal from '@/components/Game/AuctionModal';

export default function RoomPage() {
    const params = useParams();
    const roomId = params.roomId as string;
    const router = useRouter();

    const [roomState, setRoomState] = useState<RoomState | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [playerId, setPlayerId] = useState<string | null>(null);

    const { connect: connectWs, disconnect: disconnectWs, status: wsStatus } = useWsStore();
    const { gameState } = useGameStore();

    useEffect(() => {
        // Check authentication
        const storedId = sessionStorage.getItem(`player_id_${roomId}`);
        if (!storedId) {
            router.push('/');
            return;
        }
        setPlayerId(storedId);

        // Connect WebSocket
        connectWs(roomId, storedId);

        // Initial fetch REST (for Lobby state before GameStart)
        fetchRoom();

        const interval = setInterval(fetchRoom, 2000);
        return () => {
            clearInterval(interval);
            disconnectWs();
        };
    }, [roomId, router]);

    const fetchRoom = async () => {
        try {
            const state = await api.getRoom(roomId);
            setRoomState(state);
            setLoading(false);
        } catch (err: any) {
            setError(err.message);
            setLoading(false);
        }
    };

    const handleStartGame = async () => {
        try {
            await api.startGame(roomId);
        } catch (err: any) {
            alert(err.message);
        }
    };

    const handleAddBot = async () => {
        try {
            await api.addBot(roomId);
            fetchRoom(); // refresh list immediately
        } catch (err: any) {
            alert(err.message);
        }
    };

    const copyRoomLink = () => {
        navigator.clipboard.writeText(roomId);
        alert("Room ID copied!");
    }

    if (loading) {
        return (
            <Container sx={{ display: 'flex', justifyContent: 'center', mt: 10 }}>
                <CircularProgress />
            </Container>
        );
    }

    if (error || !roomState) {
        return (
            <Container sx={{ mt: 10, textAlign: 'center' }}>
                <Typography color="error" variant="h5">Error: {error}</Typography>
                <Button onClick={() => router.push('/')} sx={{ mt: 2 }}>Go Home</Button>
            </Container>
        );
    }

    // If game is running or we have WS game state, switch to Game View
    // For MVP, if phase is 'Playing', we show Game UI
    if (roomState.phase === 'Playing' || gameState) {
        return (
            <Container maxWidth={false} sx={{ mt: 0, p: 0, height: '100vh', overflow: 'hidden', bgcolor: '#0f0f1a', position: 'relative', display: 'flex' }}>
                {/* Controls Overlay */}
                <Box sx={{ position: 'absolute', top: 10, left: 10, zIndex: 100 }}>
                    <Chip label={`Room: ${roomId}`} />
                    <Chip label={`Status: ${wsStatus}`} color={wsStatus === 'connected' ? 'success' : 'error'} sx={{ ml: 1 }} />
                </Box>

                {/* Main Board Area */}
                <Box sx={{ flex: 1, position: 'relative', display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                    <GameBoard />
                    <DiceDisplay />
                    <GameControls />
                    <AuctionModal />
                </Box>

                {/* Right Sidebar */}
                <Box sx={{ width: 280, p: 2, display: 'flex', flexDirection: 'column', bgcolor: 'rgba(0,0,0,0.2)', zIndex: 20 }}>
                    <PlayerList />
                    <EventLog />
                </Box>
            </Container>
        )
    }

    const currentPlayer = roomState.players.find(p => p.id === playerId);
    const isHost = currentPlayer?.is_host;

    return (
        <Container maxWidth="md" sx={{ mt: 4 }}>
            <Paper sx={{ p: 4, background: 'rgba(22, 33, 62, 0.9)' }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 4 }}>
                    <Box>
                        <Typography variant="h4" gutterBottom>Room: {roomId}</Typography>
                        <Typography variant="subtitle1" color="text.secondary" onClick={copyRoomLink} sx={{ cursor: 'pointer', textDecoration: 'underline' }}>
                            Copy Room ID
                        </Typography>
                    </Box>
                    <Stack alignItems="flex-end">
                        <Chip label={roomState.phase} color={roomState.phase === 'Playing' ? "success" : "default"} />
                        <Typography variant="caption" color={wsStatus === 'connected' ? 'success.main' : 'error.main'}>
                            WS: {wsStatus}
                        </Typography>
                    </Stack>
                </Box>

                <Box sx={{ mb: 4 }}>
                    <Typography variant="h6" gutterBottom>Players ({roomState.players.length})</Typography>
                    <List>
                        {roomState.players.map((player) => (
                            <ListItem key={player.id} sx={{ bgcolor: 'rgba(255,255,255,0.05)', mb: 1, borderRadius: 1 }}>
                                <ListItemAvatar>
                                    <Avatar sx={{ bgcolor: player.color }}>
                                        {player.is_bot ? <SmartToyIcon /> : <PersonIcon />}
                                    </Avatar>
                                </ListItemAvatar>
                                <ListItemText
                                    primary={
                                        <Stack direction="row" spacing={1} alignItems="center">
                                            <Typography variant="body1">{player.name}</Typography>
                                            {player.is_host && <StarIcon fontSize="small" color="warning" />}
                                            {player.id === playerId && <Chip label="You" size="small" color="primary" />}
                                        </Stack>
                                    }
                                    secondary={player.is_bot ? "Bot" : "Human"}
                                />
                            </ListItem>
                        ))}
                    </List>
                </Box>

                {isHost && roomState.phase === 'Lobby' && (
                    <Stack direction="row" spacing={2} justifyContent="flex-end">
                        <Button variant="outlined" startIcon={<SmartToyIcon />} onClick={handleAddBot}>
                            Add Bot
                        </Button>
                        <Button variant="contained" color="success" size="large" onClick={handleStartGame}>
                            Start Game
                        </Button>
                    </Stack>
                )}
            </Paper>
        </Container>
    );
}
