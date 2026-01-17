import { create } from 'zustand';
import { ServerEvent } from '@/types';

// TODO: Define better types from backend
interface GameState {
    id: string; // Room ID
    phase: string;
    turn: {
        player_id: string;
        dice?: [number, number];
        phase: string;
    }
    players: any[];
    properties: Record<number, any>; // tile_idx -> state
    // ... other fields
}

interface GameStore {
    gameState: GameState | null;
    logs: string[];
    chatMessages: Array<{ from: string; message: string }>;

    setGameState: (state: GameState) => void;
    updateGameState: (updater: (state: GameState | null) => GameState | null) => void;
    addLog: (message: string) => void;
    addChatMessage: (from: string, message: string) => void;
    reset: () => void;
}

export const useGameStore = create<GameStore>((set) => ({
    gameState: null,
    logs: [],
    chatMessages: [],

    setGameState: (state) => set({ gameState: state }),

    updateGameState: (updater) => set((store) => ({
        gameState: updater(store.gameState)
    })),

    addLog: (message) => set((store) => ({
        logs: [...store.logs, message]
    })),

    addChatMessage: (from, message) => set((store) => ({
        chatMessages: [...store.chatMessages, { from, message }]
    })),

    reset: () => set({
        gameState: null,
        logs: [],
        chatMessages: []
    }),
}));
