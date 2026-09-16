const express = require('express');
const db = require('../database/connection');
const { authenticateToken } = require('./auth');

const router = express.Router();
router.use(authenticateToken);

const NICK_RE = /^[A-Za-z0-9_]{3,16}$/;

async function ensureFriendsSchema() {
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.friendships (
      id BIGSERIAL PRIMARY KEY,
      user_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
      friend_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
      status VARCHAR(16) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted')),
      created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      CONSTRAINT friendships_no_self CHECK (user_id <> friend_id),
      CONSTRAINT friendships_pair_unique UNIQUE (user_id, friend_id)
    )
  `);
  await db.query(`
    CREATE INDEX IF NOT EXISTS friendships_user_status_idx
      ON public.friendships (user_id, status)
  `);
  await db.query(`
    CREATE INDEX IF NOT EXISTS friendships_friend_status_idx
      ON public.friendships (friend_id, status)
  `);
}

function publicFriend(row, meId) {
  const otherId = Number(row.user_id) === Number(meId) ? row.friend_id : row.user_id;
  const otherNick =
    Number(row.user_id) === Number(meId) ? row.friend_nickname : row.user_nickname;
  const otherAvatar =
    Number(row.user_id) === Number(meId) ? row.friend_avatar : row.user_avatar;
  const incoming = Number(row.friend_id) === Number(meId) && row.status === 'pending';
  return {
    id: String(row.id),
    userId: String(otherId),
    nickname: otherNick,
    avatarUrl: otherAvatar || null,
    status: row.status,
    incoming,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

const FRIEND_SELECT = `
  SELECT f.*,
         u.nickname AS user_nickname, u.avatar_url AS user_avatar,
         fr.nickname AS friend_nickname, fr.avatar_url AS friend_avatar
  FROM friendships f
  JOIN users u ON u.id = f.user_id
  JOIN users fr ON fr.id = f.friend_id
`;

// GET /api/friends — list mine (accepted + pending both ways)
router.get('/', async (req, res) => {
  try {
    const me = req.user.id;
    const result = await db.query(
      `${FRIEND_SELECT}
       WHERE f.user_id = $1 OR f.friend_id = $1
       ORDER BY f.updated_at DESC`,
      [me]
    );
    res.json({ friends: result.rows.map((row) => publicFriend(row, me)) });
  } catch (error) {
    console.error('friends list:', error);
    res.status(500).json({ error: 'не удалось загрузить друзей' });
  }
});

// GET /api/friends/search?q=nick — find users by nickname
router.get('/search', async (req, res) => {
  try {
    const q = String(req.query.q || '').trim();
    if (q.length < 2) {
      return res.status(400).json({ error: 'минимум 2 символа' });
    }
    const result = await db.query(
      `SELECT id, nickname, avatar_url
       FROM users
       WHERE is_active IS DISTINCT FROM false
         AND is_banned IS DISTINCT FROM true
         AND id <> $1
         AND nickname ILIKE $2
       ORDER BY nickname ASC
       LIMIT 20`,
      [req.user.id, `${q.replace(/[%_]/g, '')}%`]
    );
    res.json({
      users: result.rows.map((u) => ({
        id: String(u.id),
        nickname: u.nickname,
        avatarUrl: u.avatar_url || null,
      })),
    });
  } catch (error) {
    console.error('friends search:', error);
    res.status(500).json({ error: 'не удалось найти пользователей' });
  }
});

// POST /api/friends/request { nickname }
router.post('/request', async (req, res) => {
  try {
    const nickname = String(req.body.nickname || '').trim();
    if (!NICK_RE.test(nickname)) {
      return res.status(400).json({ error: 'ник: 3–16 латиница/цифры/_' });
    }
    const target = await db.query(
      `SELECT id, nickname, avatar_url FROM users
       WHERE LOWER(nickname) = LOWER($1)
         AND is_active IS DISTINCT FROM false
         AND is_banned IS DISTINCT FROM true`,
      [nickname]
    );
    if (!target.rows[0]) {
      return res.status(404).json({ error: 'пользователь не найден' });
    }
    const friendId = target.rows[0].id;
    if (Number(friendId) === Number(req.user.id)) {
      return res.status(400).json({ error: 'нельзя добавить себя' });
    }

    const existing = await db.query(
      `SELECT * FROM friendships
       WHERE (user_id = $1 AND friend_id = $2)
          OR (user_id = $2 AND friend_id = $1)`,
      [req.user.id, friendId]
    );
    if (existing.rows[0]) {
      const row = existing.rows[0];
      if (row.status === 'accepted') {
        return res.status(409).json({ error: 'уже друзья' });
      }
      // If they already sent us a request, auto-accept.
      if (Number(row.user_id) === Number(friendId) && Number(row.friend_id) === Number(req.user.id)) {
        const accepted = await db.query(
          `UPDATE friendships SET status = 'accepted', updated_at = NOW()
           WHERE id = $1 RETURNING *`,
          [row.id]
        );
        const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $1`, [accepted.rows[0].id]);
        return res.json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
      }
      return res.status(409).json({ error: 'заявка уже отправлена' });
    }

    const inserted = await db.query(
      `INSERT INTO friendships (user_id, friend_id, status)
       VALUES ($1, $2, 'pending') RETURNING id`,
      [req.user.id, friendId]
    );
    const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $1`, [inserted.rows[0].id]);
    res.status(201).json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
  } catch (error) {
    console.error('friends request:', error);
    res.status(500).json({ error: 'не удалось отправить заявку' });
  }
});

// POST /api/friends/:id/accept
router.post('/:id/accept', async (req, res) => {
  try {
    const result = await db.query(
      `UPDATE friendships SET status = 'accepted', updated_at = NOW()
       WHERE id = $1 AND friend_id = $2 AND status = 'pending'
       RETURNING id`,
      [req.params.id, req.user.id]
    );
    if (!result.rows[0]) {
      return res.status(404).json({ error: 'заявка не найдена' });
    }
    const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $1`, [result.rows[0].id]);
    res.json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
  } catch (error) {
    console.error('friends accept:', error);
    res.status(500).json({ error: 'не удалось принять заявку' });
  }
});

// POST /api/friends/:id/decline  OR DELETE /api/friends/:id
async function removeFriendship(req, res) {
  try {
    const result = await db.query(
      `DELETE FROM friendships
       WHERE id = $1 AND (user_id = $2 OR friend_id = $2)
       RETURNING id`,
      [req.params.id, req.user.id]
    );
    if (!result.rows[0]) {
      return res.status(404).json({ error: 'запись не найдена' });
    }
    res.json({ success: true });
  } catch (error) {
    console.error('friends remove:', error);
    res.status(500).json({ error: 'не удалось удалить' });
  }
}

router.post('/:id/decline', removeFriendship);
router.delete('/:id', removeFriendship);

module.exports = router;
module.exports.ensureFriendsSchema = ensureFriendsSchema;
