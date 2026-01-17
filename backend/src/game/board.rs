//! Board definition - 40 tiles with properties based on world cities

use serde::{Deserialize, Serialize};

/// Type of tile on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Go,
    Property,
    Railroad,
    Utility,
    Chance,
    CommunityChest,
    Tax,
    FreeParking,
    Jail,
    GoToJail,
}

/// Color group for properties
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorGroup {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    DarkBlue,
    Railroad,
    Utility,
}

impl ColorGroup {
    pub fn color_hex(&self) -> &'static str {
        match self {
            ColorGroup::Brown => "#8B4513",
            ColorGroup::LightBlue => "#87CEEB",
            ColorGroup::Pink => "#FF69B4",
            ColorGroup::Orange => "#FFA500",
            ColorGroup::Red => "#FF0000",
            ColorGroup::Yellow => "#FFD700",
            ColorGroup::Green => "#228B22",
            ColorGroup::DarkBlue => "#00008B",
            ColorGroup::Railroad => "#333333",
            ColorGroup::Utility => "#CCCCCC",
        }
    }

    pub fn property_count(&self) -> u8 {
        match self {
            ColorGroup::Brown | ColorGroup::DarkBlue => 2,
            ColorGroup::Railroad => 4,
            ColorGroup::Utility => 2,
            _ => 3,
        }
    }
}

/// A tile on the board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub index: u8,
    pub name: String,
    pub tile_type: TileType,
    pub group: Option<ColorGroup>,
    pub price: u32,
    pub rent_base: u32,
    pub rent_schedule: Vec<u32>, // [1 house, 2, 3, 4, hotel]
    pub mortgage_value: u32,
    pub build_cost: u32,
    pub country_code: Option<String>, // ISO country code for flag
}

impl Tile {
    fn go() -> Self {
        Self {
            index: 0,
            name: "START".into(),
            tile_type: TileType::Go,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn property(
        index: u8,
        name: &str,
        group: ColorGroup,
        price: u32,
        rent_base: u32,
        rent_schedule: Vec<u32>,
        build_cost: u32,
        country_code: &str,
    ) -> Self {
        Self {
            index,
            name: name.into(),
            tile_type: TileType::Property,
            group: Some(group),
            price,
            rent_base,
            rent_schedule,
            mortgage_value: price / 2,
            build_cost,
            country_code: Some(country_code.into()),
        }
    }

    fn railroad(index: u8, name: &str, country_code: &str) -> Self {
        Self {
            index,
            name: name.into(),
            tile_type: TileType::Railroad,
            group: Some(ColorGroup::Railroad),
            price: 200,
            rent_base: 25, // 1 RR: 25, 2: 50, 3: 100, 4: 200
            rent_schedule: vec![25, 50, 100, 200],
            mortgage_value: 100,
            build_cost: 0,
            country_code: Some(country_code.into()),
        }
    }

    fn utility(index: u8, name: &str) -> Self {
        Self {
            index,
            name: name.into(),
            tile_type: TileType::Utility,
            group: Some(ColorGroup::Utility),
            price: 150,
            rent_base: 4, // Multiplier: 4x dice if 1 owned, 10x if 2
            rent_schedule: vec![4, 10],
            mortgage_value: 75,
            build_cost: 0,
            country_code: None,
        }
    }

    fn chance(index: u8) -> Self {
        Self {
            index,
            name: "Surprise".into(),
            tile_type: TileType::Chance,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn community_chest(index: u8) -> Self {
        Self {
            index,
            name: "Treasure".into(),
            tile_type: TileType::CommunityChest,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn tax(index: u8, name: &str, amount: u32) -> Self {
        Self {
            index,
            name: name.into(),
            tile_type: TileType::Tax,
            group: None,
            price: 0,
            rent_base: amount,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn jail() -> Self {
        Self {
            index: 10,
            name: "In Prison".into(),
            tile_type: TileType::Jail,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn free_parking() -> Self {
        Self {
            index: 20,
            name: "Vacation".into(),
            tile_type: TileType::FreeParking,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }

    fn go_to_jail() -> Self {
        Self {
            index: 30,
            name: "Go to prison".into(),
            tile_type: TileType::GoToJail,
            group: None,
            price: 0,
            rent_base: 0,
            rent_schedule: vec![],
            mortgage_value: 0,
            build_cost: 0,
            country_code: None,
        }
    }
}

/// The complete game board - 40 tiles based on Richup.io world cities
pub static BOARD: once_cell::sync::Lazy<Vec<Tile>> = once_cell::sync::Lazy::new(|| {
    vec![
        // === BOTTOM ROW (0-10) ===
        Tile::go(),
        // Brown group
        Tile::property(
            1,
            "Salvador",
            ColorGroup::Brown,
            60,
            2,
            vec![10, 30, 90, 160, 250],
            50,
            "BR",
        ),
        Tile::community_chest(2),
        Tile::property(
            3,
            "Rio",
            ColorGroup::Brown,
            60,
            4,
            vec![20, 60, 180, 320, 450],
            50,
            "BR",
        ),
        Tile::tax(4, "Income Tax 10%", 200),
        Tile::railroad(5, "TLV Airport", "IL"),
        // Light Blue group
        Tile::property(
            6,
            "Tel Aviv",
            ColorGroup::LightBlue,
            100,
            6,
            vec![30, 90, 270, 400, 550],
            50,
            "IL",
        ),
        Tile::chance(7),
        Tile::property(
            8,
            "Haifa",
            ColorGroup::LightBlue,
            100,
            6,
            vec![30, 90, 270, 400, 550],
            50,
            "IL",
        ),
        Tile::property(
            9,
            "Jerusalem",
            ColorGroup::LightBlue,
            120,
            8,
            vec![40, 100, 300, 450, 600],
            50,
            "IL",
        ),
        Tile::jail(),
        // === LEFT COLUMN (11-20) ===
        // Pink group
        Tile::property(
            11,
            "Venice",
            ColorGroup::Pink,
            140,
            10,
            vec![50, 150, 450, 625, 750],
            100,
            "IT",
        ),
        Tile::utility(12, "Electric Company"),
        Tile::property(
            13,
            "Milan",
            ColorGroup::Pink,
            140,
            10,
            vec![50, 150, 450, 625, 750],
            100,
            "IT",
        ),
        Tile::property(
            14,
            "Rome",
            ColorGroup::Pink,
            160,
            12,
            vec![60, 180, 500, 700, 900],
            100,
            "IT",
        ),
        Tile::railroad(15, "MUC Airport", "DE"),
        // Orange group
        Tile::property(
            16,
            "Frankfurt",
            ColorGroup::Orange,
            180,
            14,
            vec![70, 200, 550, 750, 950],
            100,
            "DE",
        ),
        Tile::community_chest(17),
        Tile::property(
            18,
            "Treasure",
            ColorGroup::Orange,
            180,
            14,
            vec![70, 200, 550, 750, 950],
            100,
            "DE",
        ),
        Tile::property(
            19,
            "Munich",
            ColorGroup::Orange,
            200,
            16,
            vec![80, 220, 600, 800, 1000],
            100,
            "DE",
        ),
        Tile::free_parking(),
        // === TOP ROW (21-30) ===
        // Red group
        Tile::property(
            21,
            "Berlin",
            ColorGroup::Red,
            220,
            18,
            vec![90, 250, 700, 875, 1050],
            150,
            "DE",
        ),
        Tile::chance(22),
        Tile::property(
            23,
            "Manchester",
            ColorGroup::Red,
            220,
            18,
            vec![90, 250, 700, 875, 1050],
            150,
            "GB",
        ),
        Tile::property(
            24,
            "Liverpool",
            ColorGroup::Red,
            240,
            20,
            vec![100, 300, 750, 925, 1100],
            150,
            "GB",
        ),
        Tile::railroad(25, "JFK Airport", "US"),
        // Yellow group
        Tile::property(
            26,
            "Paris",
            ColorGroup::Yellow,
            260,
            22,
            vec![110, 330, 800, 975, 1150],
            150,
            "FR",
        ),
        Tile::property(
            27,
            "Toulouse",
            ColorGroup::Yellow,
            260,
            22,
            vec![110, 330, 800, 975, 1150],
            150,
            "FR",
        ),
        Tile::utility(28, "Water Company"),
        Tile::property(
            29,
            "Lyon",
            ColorGroup::Yellow,
            280,
            24,
            vec![120, 360, 850, 1025, 1200],
            150,
            "FR",
        ),
        Tile::go_to_jail(),
        // === RIGHT COLUMN (31-39) ===
        // Green group
        Tile::property(
            31,
            "CDG Airport",
            ColorGroup::Green,
            300,
            26,
            vec![130, 390, 900, 1100, 1275],
            200,
            "FR",
        ),
        Tile::property(
            32,
            "Shanghai",
            ColorGroup::Green,
            300,
            26,
            vec![130, 390, 900, 1100, 1275],
            200,
            "CN",
        ),
        Tile::community_chest(33),
        Tile::property(
            34,
            "Beijing",
            ColorGroup::Green,
            320,
            28,
            vec![150, 450, 1000, 1200, 1400],
            200,
            "CN",
        ),
        Tile::railroad(35, "Shenzhen", "CN"),
        Tile::chance(36),
        // Dark Blue group
        Tile::property(
            37,
            "New York",
            ColorGroup::DarkBlue,
            350,
            35,
            vec![175, 500, 1100, 1300, 1500],
            200,
            "US",
        ),
        Tile::tax(38, "Luxury Tax", 100),
        Tile::property(
            39,
            "Tokyo",
            ColorGroup::DarkBlue,
            400,
            50,
            vec![200, 600, 1400, 1700, 2000],
            200,
            "JP",
        ),
    ]
});

/// Get a tile by index
pub fn get_tile(idx: u8) -> Option<&'static Tile> {
    BOARD.get(idx as usize)
}

/// Get all tiles in a color group
pub fn get_group_tiles(group: ColorGroup) -> Vec<&'static Tile> {
    BOARD.iter().filter(|t| t.group == Some(group)).collect()
}
