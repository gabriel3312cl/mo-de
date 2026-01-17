export type TileType =
    | 'Go'
    | 'Property'
    | 'Railroad'
    | 'Utility'
    | 'Chance'
    | 'CommunityChest'
    | 'Tax'
    | 'FreeParking'
    | 'Jail'
    | 'GoToJail';

export type ColorGroup =
    | 'Brown'
    | 'LightBlue'
    | 'Pink'
    | 'Orange'
    | 'Red'
    | 'Yellow'
    | 'Green'
    | 'DarkBlue'
    | 'Railroad'
    | 'Utility';

export interface TileData {
    index: number;
    name: string;
    type: TileType;
    group?: ColorGroup;
    price?: number;
    countryCode?: string; // ISO code for flag
}

export const COLOR_GROUPS: Record<ColorGroup, string> = {
    Brown: '#8B4513',
    LightBlue: '#87CEEB',
    Pink: '#FF69B4',
    Orange: '#FFA500',
    Red: '#FF0000',
    Yellow: '#FFD700',
    Green: '#228B22',
    DarkBlue: '#00008B',
    Railroad: '#333333',
    Utility: '#CCCCCC',
};

export const BOARD_DATA: TileData[] = [
    // BOTTOM ROW (0-10) -> Right to Left
    { index: 0, name: "START", type: "Go" },
    { index: 1, name: "Salvador", type: "Property", group: "Brown", price: 60, countryCode: "BR" },
    { index: 2, name: "Community Chest", type: "CommunityChest" },
    { index: 3, name: "Rio", type: "Property", group: "Brown", price: 60, countryCode: "BR" },
    { index: 4, name: "Income Tax", type: "Tax", price: 200 },
    { index: 5, name: "TLV Airport", type: "Railroad", group: "Railroad", price: 200, countryCode: "IL" },
    { index: 6, name: "Tel Aviv", type: "Property", group: "LightBlue", price: 100, countryCode: "IL" },
    { index: 7, name: "Chance", type: "Chance" },
    { index: 8, name: "Haifa", type: "Property", group: "LightBlue", price: 100, countryCode: "IL" },
    { index: 9, name: "Jerusalem", type: "Property", group: "LightBlue", price: 120, countryCode: "IL" },
    { index: 10, name: "Jail", type: "Jail" },

    // LEFT COLUMN (11-20) -> Bottom to Top
    { index: 11, name: "Venice", type: "Property", group: "Pink", price: 140, countryCode: "IT" },
    { index: 12, name: "Electric Company", type: "Utility", group: "Utility", price: 150 },
    { index: 13, name: "Milan", type: "Property", group: "Pink", price: 140, countryCode: "IT" },
    { index: 14, name: "Rome", type: "Property", group: "Pink", price: 160, countryCode: "IT" },
    { index: 15, name: "MUC Airport", type: "Railroad", group: "Railroad", price: 200, countryCode: "DE" },
    { index: 16, name: "Frankfurt", type: "Property", group: "Orange", price: 180, countryCode: "DE" },
    { index: 17, name: "Community Chest", type: "CommunityChest" },
    { index: 18, name: "Hamburg", type: "Property", group: "Orange", price: 180, countryCode: "DE" }, // "Treasure" in Rust fixed to Hamburg
    { index: 19, name: "Munich", type: "Property", group: "Orange", price: 200, countryCode: "DE" },
    { index: 20, name: "Vacation", type: "FreeParking" },

    // TOP ROW (21-30) -> Left to Right
    { index: 21, name: "Berlin", type: "Property", group: "Red", price: 220, countryCode: "DE" },
    { index: 22, name: "Chance", type: "Chance" },
    { index: 23, name: "Manchester", type: "Property", group: "Red", price: 220, countryCode: "GB" },
    { index: 24, name: "Liverpool", type: "Property", group: "Red", price: 240, countryCode: "GB" },
    { index: 25, name: "JFK Airport", type: "Railroad", group: "Railroad", price: 200, countryCode: "US" },
    { index: 26, name: "Paris", type: "Property", group: "Yellow", price: 260, countryCode: "FR" },
    { index: 27, name: "Toulouse", type: "Property", group: "Yellow", price: 260, countryCode: "FR" },
    { index: 28, name: "Water Company", type: "Utility", group: "Utility", price: 150 },
    { index: 29, name: "Lyon", type: "Property", group: "Yellow", price: 280, countryCode: "FR" },
    { index: 30, name: "Go to Prison", type: "GoToJail" },

    // RIGHT COLUMN (31-39) -> Top to Bottom
    { index: 31, name: "CDG Airport", type: "Property", group: "Green", price: 300, countryCode: "FR" }, // Was Green group, naming as Airport is weird but ok
    { index: 32, name: "Shanghai", type: "Property", group: "Green", price: 300, countryCode: "CN" },
    { index: 33, name: "Community Chest", type: "CommunityChest" },
    { index: 34, name: "Beijing", type: "Property", group: "Green", price: 320, countryCode: "CN" },
    { index: 35, name: "Shenzhen Station", type: "Railroad", group: "Railroad", price: 200, countryCode: "CN" },
    { index: 36, name: "Chance", type: "Chance" },
    { index: 37, name: "New York", type: "Property", group: "DarkBlue", price: 350, countryCode: "US" },
    { index: 38, name: "Luxury Tax", type: "Tax", price: 100 },
    { index: 39, name: "Tokyo", type: "Property", group: "DarkBlue", price: 400, countryCode: "JP" },
];
