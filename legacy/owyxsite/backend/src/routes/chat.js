const express = require('express');
const db = require('../database/connection');
const { authenticateToken } = require('./auth');

const router = express.Router();

async function roomAccess(roomId, userId) {
  const result = await db.query(
    `SELECT r.is_private, (rm.user_id IS NOT NULL) AS is_member
     FROM chat_rooms r
     LEFT JOIN chat_room_members rm ON rm.room_id = r.id AND rm.user_id = $2
     WHERE r.id = $1`,
    [roomId, userId]
  );
  return result.rows[0] || null;
}

// GET /api/chat/rooms — list rooms the user can see
router.get('/rooms', authenticateToken, async (req, res) => {
  try {
    const result = await db.query(
      `
      SELECT r.id, r.name, r.slug, r.description, r.is_private, r.created_at,
             (
               SELECT content FROM chat_messages m
               WHERE m.room_id = r.id AND m.is_deleted = false
               ORDER BY m.created_at DESC LIMIT 1
             ) AS last_message,
             (
               SELECT created_at FROM chat_messages m
               WHERE m.room_id = r.id AND m.is_deleted = false
               ORDER BY m.created_at DESC LIMIT 1
             ) AS last_message_at
      FROM chat_rooms r
      LEFT JOIN chat_room_members rm ON rm.room_id = r.id AND rm.user_id = $1
      WHERE r.is_private = false OR rm.user_id IS NOT NULL
      ORDER BY COALESCE(
        (SELECT created_at FROM chat_messages m WHERE m.room_id = r.id ORDER BY m.created_at DESC LIMIT 1),
        r.created_at
      ) DESC
      `,
      [req.user.id]
    );
    res.json({ rooms: result.rows });
  } catch (error) {
    if (error.code === '42P01') {
      return res.status(503).json({
        error: 'Chat tables missing. Apply owyx/postgres/migrations/001_chat.sql',
      });
    }
    console.error('Chat rooms error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// GET /api/chat/rooms/:id/messages
router.get('/rooms/:id/messages', authenticateToken, async (req, res) => {
  try {
    const roomId = parseInt(req.params.id, 10);
    const limit = Math.min(100, parseInt(req.query.limit, 10) || 50);
    const before = req.query.before;
    if (!Number.isSafeInteger(roomId) || roomId < 1) {
      return res.status(400).json({ error: 'Некорректная комната' });
    }
    const access = await roomAccess(roomId, req.user.id);
    if (!access) return res.status(404).json({ error: 'Комната не найдена' });
    if (access.is_private && !access.is_member) {
      return res.status(403).json({ error: 'Нет доступа к приватной комнате' });
    }

    const params = [roomId, limit];
    let beforeClause = '';
    if (before) {
      beforeClause = 'AND m.created_at < $3';
      params.push(before);
    }

    const result = await db.query(
      `
      SELECT m.id, m.room_id, m.user_id, m.content, m.created_at, m.updated_at,
             u.nickname, u.avatar_url, u.role, u.trust_level
      FROM chat_messages m
      JOIN users u ON u.id = m.user_id
      WHERE m.room_id = $1 AND m.is_deleted = false ${beforeClause}
      ORDER BY m.created_at DESC
      LIMIT $2
      `,
      params
    );

    res.json({ messages: result.rows.reverse() });
  } catch (error) {
    if (error.code === '42P01') {
      return res.status(503).json({
        error: 'Chat tables missing. Apply owyx/postgres/migrations/001_chat.sql',
      });
    }
    console.error('Chat messages error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// POST /api/chat/rooms/:id/join
router.post('/rooms/:id/join', authenticateToken, async (req, res) => {
  try {
    const roomId = parseInt(req.params.id, 10);
    if (!Number.isSafeInteger(roomId) || roomId < 1) {
      return res.status(400).json({ error: 'Некорректная комната' });
    }
    const access = await roomAccess(roomId, req.user.id);
    if (!access) return res.status(404).json({ error: 'Комната не найдена' });
    if (access.is_private && !access.is_member) {
      return res.status(403).json({ error: 'В приватную комнату нельзя войти без приглашения' });
    }
    await db.query(
      `
      INSERT INTO chat_room_members (room_id, user_id)
      VALUES ($1, $2)
      ON CONFLICT (room_id, user_id) DO NOTHING
      `,
      [roomId, req.user.id]
    );
    res.json({ success: true });
  } catch (error) {
    console.error('Chat join error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

module.exports = router;
