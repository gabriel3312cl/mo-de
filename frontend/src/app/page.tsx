'use client';
import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Box, Button, Typography, Container, Paper, TextField, Stack, Snackbar, Alert } from '@mui/material';
import CasinoIcon from '@mui/icons-material/Casino';
import { api } from '@/utils/api';

export default function Home() {
  const router = useRouter();
  const [playerName, setPlayerName] = useState('');
  const [roomId, setRoomId] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleCreateGame = async () => {
    if (!playerName.trim()) {
      setError('Please enter a player name');
      return;
    }
    setLoading(true);
    try {
      const { room_id, player_id } = await api.createRoom(playerName);
      // Save info to sessionStorage or pass via query params
      // Since this is MVP, we use query params or localStorage. 
      // Ideally we'd use a store (Zustand/Context), but let's stick to simple URL params for now or sessionStorage.
      // Actually, passing secret IDs in URL is bad practice, but for MVP it's quick.
      // Better: sessionStorage.

      sessionStorage.setItem(`player_id_${room_id}`, player_id);
      router.push(`/room/${room_id}`);
    } catch (err: any) {
      setError(err.message || 'Failed to create room');
    } finally {
      setLoading(false);
    }
  };

  const handleJoinGame = async () => {
    if (!playerName.trim()) {
      setError('Please enter a player name');
      return;
    }
    if (!roomId.trim()) {
      setError('Please enter a room ID');
      return;
    }
    setLoading(true);
    try {
      const { player_id } = await api.joinRoom(roomId, playerName);
      sessionStorage.setItem(`player_id_${roomId}`, player_id);
      router.push(`/room/${roomId}`);
    } catch (err: any) {
      setError(err.message || 'Failed to join room');
    } finally {
      setLoading(false);
    }
  };

  return (
    <Container maxWidth="md" sx={{ minHeight: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <Paper elevation={3} sx={{ p: 5, textAlign: 'center', background: 'rgba(22, 33, 62, 0.8)', minWidth: 320 }}>
        <Box sx={{ mb: 3 }}>
          <CasinoIcon sx={{ fontSize: 60, color: 'secondary.main' }} />
        </Box>
        <Typography variant="h2" component="h1" gutterBottom sx={{ fontWeight: 'bold' }}>
          MO-DE
        </Typography>
        <Typography variant="h5" color="text.secondary" paragraph>
          Richup.io Clone
        </Typography>

        <Stack spacing={3} sx={{ mt: 4 }}>
          <TextField
            label="Your Name"
            variant="outlined"
            fullWidth
            value={playerName}
            onChange={(e) => setPlayerName(e.target.value)}
          />

          <Button
            variant="contained"
            color="primary"
            size="large"
            onClick={handleCreateGame}
            disabled={loading}
          >
            Create New Game
          </Button>

          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Box sx={{ flex: 1, height: '1px', bgcolor: 'divider' }} />
            <Typography color="text.secondary">OR</Typography>
            <Box sx={{ flex: 1, height: '1px', bgcolor: 'divider' }} />
          </Box>

          <Stack direction="row" spacing={2}>
            <TextField
              label="Room ID"
              variant="outlined"
              fullWidth
              value={roomId}
              onChange={(e) => setRoomId(e.target.value)}
            />
            <Button
              variant="outlined"
              color="primary"
              size="large"
              onClick={handleJoinGame}
              disabled={loading}
            >
              Join
            </Button>
          </Stack>
        </Stack>
      </Paper>

      <Snackbar open={!!error} autoHideDuration={6000} onClose={() => setError(null)}>
        <Alert onClose={() => setError(null)} severity="error" sx={{ width: '100%' }}>
          {error}
        </Alert>
      </Snackbar>
    </Container>
  );
}
