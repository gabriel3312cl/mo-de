import { Box, Paper, Typography, Divider, Stack } from '@mui/material';
import { TileData, COLOR_GROUPS } from '@/utils/boardData';

interface PropertyCardProps {
    data: TileData;
    rentSchedule?: number[]; // [Base, 1H, 2H, 3H, 4H, Hotel]
    mortgageValue?: number;
    houseCost?: number;
}

// Default rent schedule if missing (fallback)
const DEFAULT_RENT_SCHEDULE = [0, 0, 0, 0, 0, 0];

export default function PropertyCard({ data, rentSchedule = DEFAULT_RENT_SCHEDULE, mortgageValue = 0, houseCost = 0 }: PropertyCardProps) {
    if (data.type !== 'Property' && data.type !== 'Railroad' && data.type !== 'Utility') {
        return null;
    }

    const colorHex = data.group ? COLOR_GROUPS[data.group] : '#333';
    const isRailroad = data.type === 'Railroad';
    const isUtility = data.type === 'Utility';

    return (
        <Paper
            elevation={5}
            sx={{
                width: 220,
                border: '1px solid black',
                bgcolor: 'white',
                color: 'black',
                overflow: 'hidden'
            }}
        >
            {/* Header */}
            <Box sx={{ bgcolor: colorHex, p: 2, textAlign: 'center', borderBottom: '1px solid black' }}>
                <Typography variant="subtitle2" sx={{ fontWeight: 'bold', color: 'white', textShadow: '1px 1px 1px black' }}>
                    {isRailroad ? 'RAILROAD' : isUtility ? 'UTILITY' : 'TITLE DEED'}
                </Typography>
                <Typography variant="h6" sx={{ fontWeight: 'bold', color: 'white', textShadow: '1px 1px 1px black', lineHeight: 1.2 }}>
                    {data.name}
                </Typography>
            </Box>

            {/* Content */}
            <Box sx={{ p: 2, textAlign: 'center' }}>

                {isRailroad && (
                    <Stack spacing={1}>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">Rent</Typography><Typography variant="body2">$25</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">If 2 R.R.'s owned</Typography><Typography variant="body2">50</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">If 3 R.R.'s owned</Typography><Typography variant="body2">100</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">If 4 R.R.'s owned</Typography><Typography variant="body2">200</Typography></Box>
                    </Stack>
                )}

                {isUtility && (
                    <Box sx={{ mt: 1 }}>
                        <Typography variant="body2" paragraph>
                            If one "Utility" is owned rent is 4 times amount shown on dice.
                        </Typography>
                        <Typography variant="body2">
                            If both "Utilities" are owned rent is 10 times amount shown on dice.
                        </Typography>
                    </Box>
                )}

                {!isRailroad && !isUtility && (
                    <Stack spacing={0.5}>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">Rent</Typography><Typography variant="body2">${rentSchedule[0]}</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">With 1 House</Typography><Typography variant="body2">${rentSchedule[1]}</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">With 2 Houses</Typography><Typography variant="body2">${rentSchedule[2]}</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">With 3 Houses</Typography><Typography variant="body2">${rentSchedule[3]}</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">With 4 Houses</Typography><Typography variant="body2">${rentSchedule[4]}</Typography></Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between' }}><Typography variant="body2">With HOTEL</Typography><Typography variant="body2">${rentSchedule[5]}</Typography></Box>
                    </Stack>
                )}

                <Divider sx={{ my: 1 }} />

                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="caption">Mortgage Value</Typography>
                    <Typography variant="caption">${mortgageValue}</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="caption">Houses Cost</Typography>
                    <Typography variant="caption">${houseCost}</Typography>
                </Box>
            </Box>
        </Paper>
    );
}
