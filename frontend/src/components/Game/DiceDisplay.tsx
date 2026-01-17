import { Box, Paper, Typography } from '@mui/material';
import { useGameStore } from '@/stores/gameStore';

export default function DiceDisplay() {
    const { gameState } = useGameStore();

    if (!gameState || !gameState.turn || !gameState.turn.dice) return null;

    const [d1, d2] = gameState.turn.dice;

    return (
        <Paper
            elevation={10}
            sx={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                p: 4,
                bgcolor: 'rgba(255, 255, 255, 0.9)',
                color: 'black',
                textAlign: 'center',
                zIndex: 50,
                borderRadius: 4
            }}
        >
            <Typography variant="h6" gutterBottom>Dice Roll</Typography>
            <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center' }}>
                <DiceBox value={d1} />
                <DiceBox value={d2} />
            </Box>
            <Typography variant="h4" sx={{ mt: 2, fontWeight: 'bold' }}>{d1 + d2}</Typography>
            {d1 === d2 && <Typography color="secondary" sx={{ fontWeight: 'bold' }}>DOUBLES!</Typography>}
        </Paper>
    );
}

function DiceBox({ value }: { value: number }) {
    return (
        <Box sx={{
            width: 60,
            height: 60,
            border: '2px solid black',
            borderRadius: 2,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '1.5rem',
            fontWeight: 'bold',
            bgcolor: 'white'
        }}>
            {value}
        </Box>
    )
}
