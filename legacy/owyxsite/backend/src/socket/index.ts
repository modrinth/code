import { Server as HttpServer } from 'http';
import { Server, Socket } from 'socket.io';
import jwt from 'jsonwebtoken';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const db = require('../database/connection');
// eslint-disable-next-line @typescript-eslint/no-var-requires
const { sessionTokenHashes } = require('../utils/authSecurity');

interface AuthedSocket extends Socket {
  user?: {
    id: number;
    nickname: string;
    role: string;
    avatar_url?: string;
  };
}

export function initSocket(httpServer: HttpServer) {
  const corsOrigin = process.env.CORS_ORIGIN
    ? process.env.CORS_ORIGIN.split(',').map((s) => s.trim())
    : process.env.NODE_ENV === 'production'
      ? [process.env.FRONTEND_URL || 'https://owyx.site']
      : ['http://localhost:3000', 'http://127.0.0.1:3000', 'http://localhost:3001'];

  const io = new Server(httpServer, {
    cors: {
      origin: corsOrigin,
      credentials: true,
    },
    path: '/socket.io',
  });

  io.use(async (socket: AuthedSocket, next) => {
    try {
      const token =
        (socket.handshake.auth && (socket.handshake.auth as { token?: string }).token) ||
        (socket.handshake.query && (socket.handshake.query as { token?: string }).token);

      if (!token || typeof token !== 'string') {
        return next(new Error('Authentication required'));
      }

      const secret = process.env.JWT_SECRET;
      if (!secret) return next(new Error('Server misconfigured'));

      const decoded = jwt.verify(token, secret) as { userId: number };
      const result = await db.query(
        `SELECT u.id, u.nickname, u.role, u.avatar_url, u.is_active, u.is_banned,
                s.id AS session_id
         FROM user_sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = ANY($1::text[])
           AND s.user_id = $2
           AND s.is_active = true
           AND s.expires_at > NOW()`,
        [sessionTokenHashes(token), decoded.userId]
      );

      if (result.rows.length === 0 || !result.rows[0].is_active || result.rows[0].is_banned) {
        return next(new Error('User not found or banned'));
      }

      socket.user = {
        id: result.rows[0].id,
        nickname: result.rows[0].nickname,
        role: result.rows[0].role || 'user',
        avatar_url: result.rows[0].avatar_url,
      };
      next();
    } catch (err) {
      next(new Error('Invalid token'));
    }
  });

  io.on('connection', (socket: AuthedSocket) => {
    console.log(`Socket connected: ${socket.user?.nickname} (${socket.id})`);

    socket.on('join_room', async (roomId: number | string, cb?: (res: unknown) => void) => {
      try {
        const id = parseInt(String(roomId), 10);
        if (!Number.isFinite(id)) {
          cb?.({ error: 'Invalid room id' });
          return;
        }

        const access = await db.query(
          `SELECT r.is_private, (rm.user_id IS NOT NULL) AS is_member
           FROM chat_rooms r
           LEFT JOIN chat_room_members rm ON rm.room_id = r.id AND rm.user_id = $2
           WHERE r.id = $1`,
          [id, socket.user!.id]
        );
        const room = access.rows[0];
        if (!room || (room.is_private && !room.is_member)) {
          cb?.({ error: 'Room not found or access denied' });
          return;
        }

        await db.query(
          `INSERT INTO chat_room_members (room_id, user_id)
           VALUES ($1, $2) ON CONFLICT DO NOTHING`,
          [id, socket.user!.id]
        );

        socket.join(`room:${id}`);
        socket.to(`room:${id}`).emit('user_joined', {
          roomId: id,
          user: socket.user,
        });
        cb?.({ success: true, roomId: id });
      } catch (error: any) {
        console.error('join_room error:', error.message);
        cb?.({ error: error.message });
      }
    });

    socket.on('leave_room', (roomId: number | string) => {
      const id = parseInt(String(roomId), 10);
      socket.leave(`room:${id}`);
      socket.to(`room:${id}`).emit('user_left', {
        roomId: id,
        user: socket.user,
      });
    });

    socket.on(
      'send_message',
      async (
        payload: { roomId: number; content: string },
        cb?: (res: unknown) => void
      ) => {
        try {
          const roomId = parseInt(String(payload.roomId), 10);
          const content = String(payload.content || '').trim();
          if (!Number.isFinite(roomId) || !content || content.length > 2000) {
            cb?.({ error: 'Invalid message' });
            return;
          }
          if (!socket.rooms.has(`room:${roomId}`)) {
            cb?.({ error: 'Join the room before sending messages' });
            return;
          }

          const insert = await db.query(
            `INSERT INTO chat_messages (room_id, user_id, content)
             VALUES ($1, $2, $3)
             RETURNING id, room_id, user_id, content, created_at`,
            [roomId, socket.user!.id, content]
          );

          const message = {
            ...insert.rows[0],
            nickname: socket.user!.nickname,
            avatar_url: socket.user!.avatar_url,
            role: socket.user!.role,
          };

          io.to(`room:${roomId}`).emit('new_message', message);
          cb?.({ success: true, message });
        } catch (error: any) {
          console.error('send_message error:', error.message);
          cb?.({ error: error.message });
        }
      }
    );

    socket.on('typing', (payload: { roomId: number; isTyping: boolean }) => {
      const roomId = parseInt(String(payload.roomId), 10);
      if (!socket.rooms.has(`room:${roomId}`)) return;
      socket.to(`room:${roomId}`).emit('typing', {
        roomId,
        user: socket.user,
        isTyping: !!payload.isTyping,
      });
    });

    socket.on('disconnect', () => {
      console.log(`Socket disconnected: ${socket.user?.nickname}`);
    });
  });

  return io;
}
