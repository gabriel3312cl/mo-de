# MO-DE: Richup.io Clone

A real-time multiplayer Monopoly-like game built with modern technologies.

## Tech Stack

- **Backend**: Rust + Axum + SQLx + Redis
- **Frontend**: Next.js + MUI (coming soon)
- **Database**: PostgreSQL
- **Cache**: Redis

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- Node.js 20+ (for frontend)

### Development

1. **Start services**:
   ```bash
   docker-compose up -d
   ```

2. **Run backend**:
   ```bash
   cd backend
   cp .env.example .env
   cargo run
   ```

3. **API will be available at**: http://localhost:3000

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/api/rooms` | Create new room |
| GET | `/api/rooms/:id` | Get room info |
| POST | `/api/rooms/:id/join` | Join room |
| POST | `/api/rooms/:id/bot` | Add bot |
| POST | `/api/rooms/:id/start` | Start game |
| WS | `/ws/:room_id/:player_id` | Game WebSocket |

## Project Structure

```
mo-de/
├── backend/           # Rust backend
│   ├── src/
│   │   ├── api/       # HTTP handlers
│   │   ├── game/      # Game engine
│   │   ├── bot/       # Bot AI
│   │   ├── db/        # Database
│   │   └── ws/        # WebSocket
│   └── Cargo.toml
├── frontend/          # Next.js frontend (coming soon)
├── captures/          # Reference screenshots
└── docker-compose.yml
```

## License

MIT
