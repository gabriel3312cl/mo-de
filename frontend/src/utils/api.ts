import {
    CreateRoomRequest,
    CreateRoomResponse,
    JoinRoomRequest,
    JoinRoomResponse,
    RoomState,
} from '@/types';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000';

class ApiClient {
    async createRoom(hostName: string, config?: Partial<CreateRoomRequest['config']>): Promise<CreateRoomResponse> {
        const res = await fetch(`${API_BASE_URL}/api/rooms`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ host_name: hostName, config }),
        });
        if (!res.ok) throw new Error(await res.text());
        return res.json();
    }

    async joinRoom(roomId: string, playerName: string): Promise<JoinRoomResponse> {
        const res = await fetch(`${API_BASE_URL}/api/rooms/${roomId}/join`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ player_name: playerName }),
        });
        if (!res.ok) throw new Error(await res.text());
        return res.json();
    }

    async getRoom(roomId: string): Promise<RoomState> {
        const res = await fetch(`${API_BASE_URL}/api/rooms/${roomId}`);
        if (!res.ok) throw new Error(await res.text());
        return res.json();
    }

    async startGame(roomId: string): Promise<void> {
        const res = await fetch(`${API_BASE_URL}/api/rooms/${roomId}/start`, {
            method: 'POST',
        });
        if (!res.ok) throw new Error(await res.text());
    }

    async addBot(roomId: string): Promise<JoinRoomResponse> {
        const res = await fetch(`${API_BASE_URL}/api/rooms/${roomId}/bot`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({}),
        });
        if (!res.ok) throw new Error(await res.text());
        return res.json();
    }
}

export const api = new ApiClient();
