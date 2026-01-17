import { Box, Paper, Typography, Button, Stack } from '@mui/material';
import { useGameStore } from '@/stores/gameStore';
import { useWsStore } from '@/stores/wsStore';
import { BOARD_DATA } from '@/utils/boardData';
import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import GavelIcon from '@mui/icons-material/Gavel';

export default function AuctionModal() {
    const { gameState } = useGameStore();
    const { send } = useWsStore();
    const params = useParams();
    const roomId = params.roomId as string;
    const [playerId, setPlayerId] = useState<string | null>(null);

    useEffect(() => {
        setPlayerId(sessionStorage.getItem(`player_id_${roomId}`));
    }, [roomId]);

    if (!gameState || !gameState.auction || !playerId) return null;

    const { property_index, current_bid, current_bidder } = gameState.auction;
    const tile = BOARD_DATA.find(t => t.index === property_index);
    const isMyTurn = gameState.turn.player_id === playerId; // OR current_bidder === playerId? They should be synced.
    const currentBidderName = gameState.players.find(p => p.id === current_bidder)?.name || 'Unknown';

    const handleBid = (amount: number) => {
        send({ type: 'BID', amount }); // Backend expects absolute amount? Or increment?
        // Engine says: if amount <= current_bid => Error.
        // So we send New Total.
    };

    const handlePass = () => {
        send({ type: 'PASS_BID' });
    };

    return (
        <Box sx={{
            position: 'fixed', top: 0, left: 0, width: '100vw', height: '100vh',
            bgcolor: 'rgba(0,0,0,0.7)', zIndex: 200, display: 'flex', alignItems: 'center', justifyContent: 'center'
        }}>
            <Paper elevation={24} sx={{ p: 4, width: 400, textAlign: 'center', bgcolor: '#1e293b', color: 'white' }}>
                <GavelIcon sx={{ fontSize: 60, color: 'warning.main', mb: 2 }} />
                <Typography variant="h4" gutterBottom>Auction!</Typography>

                <Box sx={{ my: 3, p: 2, bgcolor: 'rgba(255,255,255,0.1)', borderRadius: 2 }}>
                    <Typography variant="h6">{tile?.name || `Property #${property_index}`}</Typography>
                    <Typography variant="body2" color="text.secondary">Base Price: ${tile?.price}</Typography>
                </Box>

                <Typography variant="h2" color="success.main" sx={{ mb: 1 }}>${current_bid}</Typography>
                <Typography variant="subtitle1" gutterBottom>
                    Current Highest Bid
                </Typography>

                <Box sx={{ mt: 4 }}>
                    {!isMyTurn ? (
                        <Typography variant="h6" className="animate-pulse">
                            Waiting for {currentBidderName} to bid...
                        </Typography>
                    ) : (
                        <Stack spacing={2}>
                            <Typography variant="h6" color="primary">It's your turn to bid!</Typography>
                            <Stack direction="row" spacing={2} justifyContent="center">
                                <Button
                                    variant="contained"
                                    color="success"
                                    onClick={() => handleBid(current_bid + 10)}
                                >
                                    Bid ${current_bid + 10}
                                </Button>
                                <Button
                                    variant="contained"
                                    color="success"
                                    onClick={() => handleBid(current_bid + 100)}
                                >
                                    Bid ${current_bid + 100}
                                </Button>
                            </Stack>
                            <Button
                                variant="outlined"
                                color="error"
                                onClick={handlePass}
                            >
                                Pass (Fold)
                            </Button>
                        </Stack>
                    )}
                </Box>
            </Paper>
        </Box>
    );
}
