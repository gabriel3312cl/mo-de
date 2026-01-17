import { Box, Button, Typography, Paper, Stack } from '@mui/material';
import CasinoIcon from '@mui/icons-material/Casino';
import HomeIcon from '@mui/icons-material/Home';
import GavelIcon from '@mui/icons-material/Gavel';
import NextPlanIcon from '@mui/icons-material/NextPlan';
import { useGameStore } from '@/stores/gameStore';
import { useWsStore } from '@/stores/wsStore';
import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';

export default function GameControls() {
    const { gameState } = useGameStore();
    const { send } = useWsStore();
    const params = useParams();
    const roomId = params.roomId as string;
    const [playerId, setPlayerId] = useState<string | null>(null);

    useEffect(() => {
        setPlayerId(sessionStorage.getItem(`player_id_${roomId}`));
    }, [roomId]);

    if (!gameState || !playerId) return null;

    const currentTurn = gameState.turn;
    const isMyTurn = currentTurn.player_id === playerId;
    // TODO: Check if turn phase matches expectations (Rust Debug format usually matches Enum name)
    const phase = currentTurn.phase;

    if (!isMyTurn) {
        // Show who is playing
        const currentPlayer = gameState.players.find(p => p.id === currentTurn.player_id);
        return (
            <Paper sx={{ position: 'absolute', bottom: 20, right: 20, p: 2, bgcolor: 'rgba(0,0,0,0.8)' }}>
                <Typography variant="body1">Waiting for {currentPlayer?.name || 'Player'}...</Typography>
                <Typography variant="caption" color="text.secondary">{phase}</Typography>
            </Paper>
        );
    }

    return (
        <Paper sx={{
            position: 'absolute',
            bottom: 30,
            left: '50%',
            transform: 'translateX(-50%)',
            p: 3,
            bgcolor: 'rgba(26, 32, 53, 0.95)',
            border: '1px solid rgba(255,255,255,0.1)',
            minWidth: 300,
            textAlign: 'center',
            zIndex: 10
        }}>
            <Typography variant="h6" gutterBottom color="primary">It's Your Turn!</Typography>

            <Stack direction="row" spacing={2} justifyContent="center">
                {phase === 'WaitingForRoll' && (
                    <Button
                        variant="contained"
                        color="secondary"
                        size="large"
                        startIcon={<CasinoIcon />}
                        onClick={() => send({ type: 'ROLL_DICE' })}
                    >
                        Roll Dice
                    </Button>
                )}

                {phase === 'BuyDecision' && (
                    <>
                        <Button
                            variant="contained"
                            color="success"
                            startIcon={<HomeIcon />}
                            onClick={() => send({ type: 'BUY_PROPERTY' })}
                        >
                            Buy
                        </Button>
                        <Button
                            variant="outlined"
                            color="warning"
                            startIcon={<GavelIcon />}
                            onClick={() => send({ type: 'PASS_PROPERTY' })}
                        >
                            Auction
                        </Button>
                    </>
                )}

                {phase === 'EndTurn' && (
                    <Button
                        variant="contained"
                        color="primary"
                        startIcon={<NextPlanIcon />}
                        onClick={() => send({ type: 'END_TURN' })}
                    >
                        End Turn
                    </Button>
                )}
            </Stack>
        </Paper>
    );
}
