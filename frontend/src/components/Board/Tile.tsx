import { Box, Typography } from '@mui/material';
import { TileData, COLOR_GROUPS } from '@/utils/boardData';
import PersonIcon from '@mui/icons-material/Person'; // Placeholder for tokens

interface TileProps {
    data: TileData;
    players?: any[]; // Array of players currently on this tile
    owner?: string; // Player ID of owner
}

export default function Tile({ data, players = [], owner }: TileProps) {
    const isCorner = ['Go', 'Jail', 'FreeParking', 'GoToJail'].includes(data.type);
    const colorHex = data.group ? COLOR_GROUPS[data.group] : null;

    return (
        <Box
            sx={{
                width: '100%',
                height: '100%',
                bgcolor: '#2a2a40',
                border: '1px solid rgba(255,255,255,0.1)',
                display: 'flex',
                flexDirection: 'column',
                position: 'relative',
                overflow: 'hidden',
                boxShadow: owner ? `inset 0 0 0 4px ${colorHex || '#fff'}` : 'none', // Show ownership
            }}
        >
            {/* Color Strip */}
            {colorHex && !isCorner && (
                <Box sx={{ height: '20%', bgcolor: colorHex, width: '100%' }} />
            )}

            {/* Content */}
            <Box sx={{ p: 0.5, flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
                {/* Name */}
                <Typography variant="caption" sx={{ fontSize: '0.65rem', lineHeight: 1.1, textAlign: 'center', fontWeight: 'bold' }}>
                    {data.name}
                </Typography>

                {/* Price (if applicable) */}
                {data.price && (
                    <Typography variant="caption" sx={{ fontSize: '0.6rem', color: 'text.secondary', mt: 0.5 }}>
                        ${data.price}
                    </Typography>
                )}

                {/* Flag (MVP: Text code) */}
                {data.countryCode && (
                    <Typography variant="caption" sx={{ fontSize: '0.5rem', opacity: 0.5 }}>{data.countryCode}</Typography>
                )}
            </Box>

            {/* Player Tokens */}
            {players.length > 0 && (
                <Box sx={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: '100%',
                    display: 'flex',
                    flexWrap: 'wrap',
                    justifyContent: 'center',
                    alignItems: 'center',
                    bgcolor: 'rgba(0,0,0,0.3)'
                }}>
                    {players.map(p => (
                        <Box key={p.id} sx={{
                            bgcolor: p.color,
                            width: 12,
                            height: 12,
                            borderRadius: '50%',
                            border: '1px solid white',
                            mx: 0.2
                        }} />
                    ))}
                </Box>
            )}
        </Box>
    );
}
