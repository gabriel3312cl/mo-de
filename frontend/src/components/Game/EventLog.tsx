import { useRef, useEffect } from 'react';
import { Paper, Box, Typography, List, ListItem, ListItemText } from '@mui/material';
import { useGameStore } from '@/stores/gameStore';

export default function EventLog() {
    const { logs } = useGameStore();
    const bottomRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (bottomRef.current) {
            bottomRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    return (
        <Paper
            elevation={3}
            sx={{
                width: 250,
                bgcolor: 'rgba(26, 32, 53, 0.9)',
                color: 'white',
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                mt: 2,
                maxHeight: '40vh'
            }}
        >
            <Box sx={{ p: 2, borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
                <Typography variant="h6">Game Log</Typography>
            </Box>
            <Box sx={{ flex: 1, overflowY: 'auto', p: 1 }}>
                <List dense>
                    {logs.map((log, index) => (
                        <ListItem key={index} sx={{ py: 0.5 }}>
                            <ListItemText
                                primary={log}
                                primaryTypographyProps={{
                                    variant: 'caption',
                                    color: 'text.secondary',
                                    style: { whiteSpace: 'pre-wrap' }
                                }}
                            />
                        </ListItem>
                    ))}
                    <div ref={bottomRef} />
                </List>
            </Box>
        </Paper>
    );
}
