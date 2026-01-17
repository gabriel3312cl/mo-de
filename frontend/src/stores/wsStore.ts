import { create } from 'zustand';
import { ClientEvent, ServerEvent } from '@/types';
import { useGameStore } from './gameStore';

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3000';

interface WsStore {
    socket: WebSocket | null;
    status: 'disconnected' | 'connecting' | 'connected' | 'error';

    connect: (roomId: string, playerId: string) => void;
    disconnect: () => void;
    send: (event: ClientEvent) => void;
}

export const useWsStore = create<WsStore>((set, get) => ({
    socket: null,
    status: 'disconnected',

    connect: (roomId, playerId) => {
        const { socket } = get();
        if (socket) {
            socket.close();
        }

        set({ status: 'connecting' });
        const ws = new WebSocket(`${WS_URL}/ws/${roomId}/${playerId}`);

        ws.onopen = () => {
            set({ status: 'connected' });
            console.log('WS Connected');
        };

        ws.onclose = () => {
            set({ status: 'disconnected', socket: null });
            console.log('WS Disconnected');
        };

        ws.onerror = (err) => {
            console.error('WS Error', err);
            set({ status: 'error' });
        };

        ws.onmessage = (msg) => {
            try {
                const event: ServerEvent = JSON.parse(msg.data);
                handleServerEvent(event);
            } catch (err) {
                console.error('Failed to parse WS message', err);
            }
        };

        set({ socket: ws });
    },

    disconnect: () => {
        const { socket } = get();
        if (socket) {
            socket.close();
        }
        set({ socket: null, status: 'disconnected' });
    },

    send: (event: ClientEvent) => {
        const { socket, status } = get();
        if (socket && status === 'connected') {
            socket.send(JSON.stringify(event));
        } else {
            console.warn('Cannot send message, socket not connected');
        }
    },
}));

function handleServerEvent(event: ServerEvent) {
    const gameStore = useGameStore.getState();

    console.log('WS Event:', event);

    switch (event.type) {
        case 'GameState':
            gameStore.setGameState(event.state);
            break;

        case 'Log':
            gameStore.addLog(event.message);
            break;

        case 'Chat':
            gameStore.addChatMessage(event.from, event.message);
            break;

        case 'DiceResult':
            gameStore.addLog(`Player ${event.player_id} rolled ${event.dice[0]}+${event.dice[1]} (${event.is_doubles ? 'Doubles!' : ''})`);
            // Update local state partially if needed, or rely on GameState event
            // Often games send the full state after actions, or we update optimistically.
            // For now, let's assume we might need to fetch state or we implemented partial updates.
            // The backend 'broadcast' sends specific events. It does NOT automatically send full GameState after every move unless implemented.
            // Checking backend... engine.rs line 171 sends GameState on start.
            // But move logic broadcasts PlayerMoved. 
            // So we definitely need to implement reducers here to keep state in sync, OR poll, OR update backend to send State often.
            // Safe bet: request state update or merge changes.
            break;

        // TODO: Handle other events to update gameStore.gameState
        case 'PlayerMoved':
            gameStore.updateGameState(state => {
                if (!state) return null;
                const players = state.players.map(p =>
                    p.id === event.player_id ? { ...p, position: event.position } : p
                );
                return { ...state, players };
            });
            break;

        case 'TurnChanged':
            gameStore.updateGameState(state => {
                if (!state) return null;
                return { ...state, turn: { ...state.turn, player_id: event.player_id } };
            });
            break;

        default:
            // For unhandled events, we might log them
            console.log('Unhandled event type:', event.type);
    }
}
