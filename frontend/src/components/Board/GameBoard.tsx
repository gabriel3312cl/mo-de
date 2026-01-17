import { Box, Paper } from '@mui/material';
import { BOARD_DATA } from '@/utils/boardData';
import Tile from './Tile';
import { useGameStore } from '@/stores/gameStore';

export default function GameBoard() {
    const { gameState } = useGameStore();

    // Helper to get players on a specific tile
    const getPlayersOnTile = (tileIdx: number) => {
        if (!gameState) return [];
        return gameState.players.filter(p => p.position === tileIdx);
    };

    // Helper to get owner
    const getOwner = (tileIdx: number) => {
        // TODO: Map from gameState.properties[tileIdx].owner (which is UUID) to Player color/ID
        // For now returning undefined
        return undefined;
    }

    // Map 0-39 indices to Grid Area
    // Grid is 11x11.
    // Row 1 (Top): 20..30 (Left to Right) -> 20=(1,1)..30=(1,11)
    // Row 11 (Bottom): 10..0 (Left to Right) -> 10=(11,1)..0=(11,11)
    // Col 1 (Left): 20..10 (Top to Bottom) -> 20=(1,1)..10=(11,1)
    // Col 11 (Right): 30..0 (Top to Bottom) -> 30=(1,11)..0=(11,11) WRONG? 30 is Top Right. 0 is Bottom Right. So 31..39 go down

    // Let's explicitly calculate grid position for each index 0..39
    const getGridArea = (index: number) => {
        // Bottom Row: 10,9,8,7,6,5,4,3,2,1,0 (Left to Right visualization, but indices decrease?)
        // Wait. Monoploy: Go is usually Bottom Right. Then you move Left.
        // So 0 (Go) is at (11, 11).
        // 1 is at (11, 10).
        // 10 (Jail) is at (11, 1).

        if (index >= 0 && index <= 10) {
            // Bottom Row
            const col = 11 - index;
            return `11 / ${col} / 12 / ${col + 1}`;
        }

        if (index >= 11 && index <= 20) {
            // Left Column (Upwards)
            // 11 is (10, 1)
            // 20 is (1, 1)
            const row = 11 - (index - 10);
            return `${row} / 1 / ${row + 1} / 2`;
        }

        if (index >= 21 && index <= 30) {
            // Top Row (Rightwards)
            // 21 is (1, 2)
            // 30 is (1, 11)
            const col = index - 20 + 1;
            return `1 / ${col} / 2 / ${col + 1}`;
        }

        if (index >= 31 && index <= 39) {
            // Right Column (Downwards)
            // 31 is (2, 11)
            // 39 is (10, 11)
            const row = index - 30 + 1;
            return `${row} / 11 / ${row + 1} / 12`;
        }

        return 'auto';
    };

    return (
        <Box sx={{
            width: '100%',
            height: '100%',
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            perspective: '1000px', // For 3D tilt effect later
            bgcolor: '#0f0f1a'
        }}>
            <Box sx={{
                display: 'grid',
                gridTemplateColumns: 'repeat(11, 1fr)',
                gridTemplateRows: 'repeat(11, 1fr)',
                gap: '2px',
                width: '90vmin',
                height: '90vmin',
                transform: 'rotateX(20deg) rotateZ(0deg)', // Isometric-ish tilt
                transformStyle: 'preserve-3d',
                transition: 'transform 0.5s',
            }}>
                {BOARD_DATA.map((tile) => (
                    <Box
                        key={tile.index}
                        sx={{
                            gridArea: getGridArea(tile.index),
                            position: 'relative',
                            transformStyle: 'preserve-3d'
                        }}
                    >
                        <Tile
                            data={tile}
                            players={getPlayersOnTile(tile.index)}
                            owner={getOwner(tile.index)}
                        />
                    </Box>
                ))}

                {/* Center Area (Logo/Dice) */}
                <Box sx={{
                    gridColumn: '2 / 11',
                    gridRow: '2 / 11',
                    bgcolor: 'rgba(20,20,30,0.5)',
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    justifyContent: 'center',
                    border: '2px solid rgba(255,255,255,0.05)'
                }}>
                    <Box sx={{
                        width: '60%',
                        height: '20%',
                        bgcolor: '#7c3aed',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        transform: 'rotate(-45deg)', // Logo across center
                        mb: 5
                    }}>
                        <Box component="span" sx={{ color: 'white', fontWeight: 'bold', fontSize: '2rem' }}>MO-DE</Box>
                    </Box>
                </Box>
            </Box>
        </Box>
    );
}
