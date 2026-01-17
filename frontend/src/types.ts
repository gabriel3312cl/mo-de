export interface GameConfig {
    starting_cash: number;
    salary: number;
    double_rent_on_full_set: boolean;
    auction_on_decline: boolean;
    turn_timer_seconds: number;
}

export interface PlayerInfo {
    id: string;
    name: string;
    color: string;
    is_host: boolean;
    is_bot: boolean;
}

export interface GamePlayer extends PlayerInfo {
    money: number;
    position: number;
    is_in_jail: boolean;
    jail_turns: number;
    jail_cards: number;
    properties: number[]; // Indices of owned properties
}

export interface AuctionState {
    property_index: number;
    current_bid: number;
    current_bidder: string;
    remaining_bidders: string[];
}

export interface RoomState {
    room_id: string;
    players: PlayerInfo[];
    phase: string;
    config: GameConfig;
}

export interface CreateRoomRequest {
    host_name: string;
    config?: Partial<GameConfig>;
}

export interface CreateRoomResponse {
    room_id: string;
    player_id: string;
}

export interface JoinRoomRequest {
    player_name: string;
}

export interface JoinRoomResponse {
    player_id: string;
}

// -- WebSocket Events --

export type ClientEvent =
    | { type: "ROLL_DICE" }
    | { type: "BUY_PROPERTY" }
    | { type: "PASS_PROPERTY" }  // triggers auction
    | { type: "END_TURN" }
    | { type: "BID"; amount: number }
    | { type: "PASS_BID" }
    | { type: "PAY_JAIL" }
    | { type: "USE_CARD" }
    | { type: "BUILD"; tile_idx: number }
    | { type: "MORTGAGE"; tile_idx: number }
    | { type: "UNMORTGAGE"; tile_idx: number }
    | { type: "TRADE_OFFER"; offer: any } // TODO: Define TradeOffer
    | { type: "TRADE_ACCEPT"; trade_id: string }
    | { type: "TRADE_REJECT"; trade_id: string }
    | { type: "CHAT"; message: string };

export type ServerEvent =
    | { type: "GameState"; state: any } // TODO: Full GameState definition
    | { type: "DiceResult"; player_id: string; dice: [number, number]; is_doubles: boolean }
    | { type: "PlayerMoved"; player_id: string; position: number }
    | { type: "PropertyBought"; tile_idx: number; player_id: string; price: number }
    | { type: "RentPaid"; from_player: string; to_player: string; amount: number }
    | { type: "AuctionStarted"; tile_idx: number; current_bid: number }
    | { type: "BidPlaced"; player_id: string; amount: number }
    | { type: "AuctionEnded"; tile_idx: number; winner_id: string | null; amount: number }
    | { type: "TurnChanged"; player_id: string }
    | { type: "PlayerJailed"; player_id: string }
    | { type: "PlayerFreed"; player_id: string }
    | { type: "BuildingBuilt"; tile_idx: number; player_id: string; houses: number }
    | { type: "PropertyMortgaged"; tile_idx: number; player_id: string }
    | { type: "PropertyUnmortgaged"; tile_idx: number; player_id: string }
    | { type: "GameOver"; winner: string }
    | { type: "Chat"; from: string; message: string }
    | { type: "Log"; message: string };
