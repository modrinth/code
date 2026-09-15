-- Chat tables for Socket.io realtime (apply manually if DB already initialized)
--   psql -U owyx_user -d owyx_db -f owyx/postgres/migrations/001_chat.sql
-- Or from docker:
--   docker exec -i owyx-postgres psql -U owyx_user -d owyx_db < owyx/postgres/migrations/001_chat.sql

CREATE TABLE IF NOT EXISTS public.chat_rooms (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_private BOOLEAN DEFAULT false,
    created_by INTEGER REFERENCES public.users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS public.chat_room_members (
    room_id INTEGER NOT NULL REFERENCES public.chat_rooms(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (room_id, user_id)
);

CREATE TABLE IF NOT EXISTS public.chat_messages (
    id SERIAL PRIMARY KEY,
    room_id INTEGER NOT NULL REFERENCES public.chat_rooms(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    is_deleted BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_room_created
    ON public.chat_messages (room_id, created_at DESC);

-- Default public lobby
INSERT INTO public.chat_rooms (name, slug, description, is_private)
VALUES ('Общий чат', 'general', 'Публичный чат сервера Owyx', false)
ON CONFLICT (slug) DO NOTHING;
