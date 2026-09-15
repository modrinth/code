# Owyx Backend API

Express + Socket.io TypeScript service ported from `chiwawasite`.

## Endpoints

| Path | Description |
|------|-------------|
| `/health` | Healthcheck |
| `/api/auth/*` | Auth, game tokens, Discord, password reset |
| `/api/applications/*` | Whitelist applications |
| `/api/profile/*` | Profile, avatar, stats |
| `/api/admin/*` | Admin users, logs, settings, tokens |
| `/api/settings/*` | Public/server settings |
| `/api/reputation/*` | Reputation votes |
| `/api/trust-level/*` | Trust level applications |
| `/api/forum/*` | Forum categories/topics/posts |
| `/api/chat/*` | Chat rooms/messages REST |
| `/socket.io` | Realtime chat |

## Local development

```bash
# from owyx/
cp .env.example .env
# fill DB_* JWT_SECRET TURNSTILE_SKIP=true

cd backend
npm install
npm run dev   # http://127.0.0.1:3001
```

Requires PostgreSQL with schema from `../postgres/init.sql`.
Chat tables are auto-created on boot; see also `../postgres/migrations/001_chat.sql`.

## Production

Runs as the `backend` service in `../docker-compose.yml` on port **3001**.
