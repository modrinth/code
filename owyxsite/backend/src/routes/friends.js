const express = require('express');
const db = require('../database/connection');
const { authenticateToken } = require('./auth');

const router = express.Router();
router.use(authenticateToken);

const NICK_RE = /^[A-Za-z0-9_]{3,16}$/;
/** Presence older than this is treated as offline. */
const PRESENCE_TTL_MS = 90_000;

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
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.user_presence (
      user_id INTEGER PRIMARY KEY REFERENCES public.users(id) ON DELETE CASCADE,
      status VARCHAR(16) NOT NULL DEFAULT 'offline'
        CHECK (status IN ('offline', 'online', 'playing')),
      instance_name TEXT,
      updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )
  `);
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.user_social_settings (
      user_id INTEGER PRIMARY KEY REFERENCES public.users(id) ON DELETE CASCADE,
      allow_friend_requests BOOLEAN NOT NULL DEFAULT true,
      updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP
    )
  `);
}

async function getSocialSettings(userId) {
  const result = await db.query(
    `SELECT allow_friend_requests FROM user_social_settings WHERE user_id = $1`,
    [userId]
  );
  if (!result.rows[0]) {
    return { allowFriendRequests: true };
  }
  return { allowFriendRequests: result.rows[0].allow_friend_requests !== false };
}

function presenceFromRow(row) {
  if (!row || !row.updated_at) {
    return { status: 'offline', instanceName: null, updatedAt: null };
  }
  const age = Date.now() - new Date(row.updated_at).getTime();
  if (Number.isNaN(age) || age > PRESENCE_TTL_MS) {
    return { status: 'offline', instanceName: null, updatedAt: row.updated_at };
  }
  return {
    status: row.status === 'playing' ? 'playing' : row.status === 'online' ? 'online' : 'offline',
    instanceName: row.status === 'playing' ? row.instance_name || null : null,
    updatedAt: row.updated_at,
  };
}

function publicFriend(row, meId) {
  const otherId = Number(row.user_id) === Number(meId) ? row.friend_id : row.user_id;
  const otherLogin =
    Number(row.user_id) === Number(meId) ? row.friend_nickname : row.user_nickname;
  const otherDisplay =
    Number(row.user_id) === Number(meId)
      ? row.friend_display_nickname || row.friend_nickname
      : row.user_display_nickname || row.user_nickname;
  const otherAvatar =
    Number(row.user_id) === Number(meId) ? row.friend_avatar : row.user_avatar;
  const incoming = Number(row.friend_id) === Number(meId) && row.status === 'pending';
  const presence = presenceFromRow({
    status: row.presence_status,
    instance_name: row.presence_instance,
    updated_at: row.presence_updated_at,
  });
  return {
    id: String(row.id),
    userId: String(otherId),
    nickname: otherLogin,
    displayNickname: otherDisplay,
    avatarUrl: otherAvatar || null,
    status: row.status,
    incoming,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
    presence: presence.status,
    instanceName: presence.instanceName,
    presenceUpdatedAt: presence.updatedAt,
  };
}

const FRIEND_SELECT = `
  SELECT f.*,
         u.nickname AS user_nickname,
         COALESCE(u.display_nickname, u.nickname) AS user_display_nickname,
         u.avatar_url AS user_avatar,
         fr.nickname AS friend_nickname,
         COALESCE(fr.display_nickname, fr.nickname) AS friend_display_nickname,
         fr.avatar_url AS friend_avatar,
         p.status AS presence_status,
         p.instance_name AS presence_instance,
         p.updated_at AS presence_updated_at
  FROM friendships f
  JOIN users u ON u.id = f.user_id
  JOIN users fr ON fr.id = f.friend_id
  LEFT JOIN user_presence p ON p.user_id = CASE
    WHEN f.user_id = $1 THEN f.friend_id
    ELSE f.user_id
  END
`;

// GET /api/friends/settings
router.get('/settings', async (req, res) => {
  try {
    const settings = await getSocialSettings(req.user.id);
    res.json({ settings });
  } catch (error) {
    console.error('friends settings get:', error);
    res.status(500).json({ error: 'could not load social settings' });
  }
});

// PATCH /api/friends/settings  { allowFriendRequests?: boolean }
router.patch('/settings', async (req, res) => {
  try {
    const allow =
      typeof req.body.allowFriendRequests === 'boolean'
        ? req.body.allowFriendRequests
        : typeof req.body.allow_friend_requests === 'boolean'
          ? req.body.allow_friend_requests
          : null;
    if (allow === null) {
      return res.status(400).json({ error: 'allowFriendRequests boolean required' });
    }
    await db.query(
      `INSERT INTO user_social_settings (user_id, allow_friend_requests, updated_at)
       VALUES ($1, $2, NOW())
       ON CONFLICT (user_id) DO UPDATE SET
         allow_friend_requests = EXCLUDED.allow_friend_requests,
         updated_at = NOW()`,
      [req.user.id, allow]
    );
    res.json({ settings: { allowFriendRequests: allow } });
  } catch (error) {
    console.error('friends settings patch:', error);
    res.status(500).json({ error: 'could not save social settings' });
  }
});

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
    const friends = result.rows.map((row) => publicFriend(row, me));
    res.json({
      friends,
      incomingCount: friends.filter((f) => f.incoming && f.status === 'pending').length,
    });
  } catch (error) {
    console.error('friends list:', error);
    res.status(500).json({ error: 'could not load friends' });
  }
});

// POST /api/friends/presence — heartbeat / playing status
router.post('/presence', async (req, res) => {
  try {
    const raw = String(req.body.status || 'online').toLowerCase();
    const status = ['offline', 'online', 'playing'].includes(raw) ? raw : 'online';
    const instanceName =
      status === 'playing'
        ? String(req.body.instanceName || req.body.instance_name || '')
            .trim()
            .slice(0, 120) || null
        : null;
    await db.query(
      `INSERT INTO user_presence (user_id, status, instance_name, updated_at)
       VALUES ($1, $2, $3, NOW())
       ON CONFLICT (user_id) DO UPDATE SET
         status = EXCLUDED.status,
         instance_name = EXCLUDED.instance_name,
         updated_at = NOW()`,
      [req.user.id, status, instanceName]
    );
    res.json({
      success: true,
      presence: {
        status,
        instanceName,
        updatedAt: new Date().toISOString(),
      },
    });
  } catch (error) {
    console.error('friends presence:', error);
    res.status(500).json({ error: 'could not update presence' });
  }
});

// GET /api/friends/search?q=nick — find users by login or display nickname
router.get('/search', async (req, res) => {
  try {
    const q = String(req.query.q || '').trim();
    if (q.length < 2) {
      return res.status(400).json({ error: 'at least 2 characters' });
    }
    const like = `${q.replace(/[%_]/g, '')}%`;
    const result = await db.query(
      `SELECT id, nickname, COALESCE(display_nickname, nickname) AS display_nickname, avatar_url
       FROM users
       WHERE is_active IS DISTINCT FROM false
         AND is_banned IS DISTINCT FROM true
         AND id <> $1
         AND (nickname ILIKE $2 OR COALESCE(display_nickname, nickname) ILIKE $2)
       ORDER BY nickname ASC
       LIMIT 20`,
      [req.user.id, like]
    );
    res.json({
      users: result.rows.map((u) => ({
        id: String(u.id),
        nickname: u.nickname,
        displayNickname: u.display_nickname || u.nickname,
        avatarUrl: u.avatar_url || null,
      })),
    });
  } catch (error) {
    console.error('friends search:', error);
    res.status(500).json({ error: 'could not search users' });
  }
});

// POST /api/friends/request { nickname }
router.post('/request', async (req, res) => {
  try {
    const nickname = String(req.body.nickname || '').trim();
    if (!NICK_RE.test(nickname)) {
      return res.status(400).json({ error: 'nickname: 3–16 letters, digits, or _' });
    }
    const target = await db.query(
      `SELECT id, nickname, avatar_url FROM users
       WHERE LOWER(nickname) = LOWER($1)
         AND is_active IS DISTINCT FROM false
         AND is_banned IS DISTINCT FROM true`,
      [nickname]
    );
    if (!target.rows[0]) {
      return res.status(404).json({ error: 'user not found' });
    }
    const friendId = target.rows[0].id;
    if (Number(friendId) === Number(req.user.id)) {
      return res.status(400).json({ error: 'cannot add yourself' });
    }

    const targetSettings = await getSocialSettings(friendId);
    if (!targetSettings.allowFriendRequests) {
      return res.status(403).json({ error: 'this player is not accepting friend requests' });
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
        return res.status(409).json({ error: 'already friends' });
      }
      // If they already sent us a request, auto-accept.
      if (Number(row.user_id) === Number(friendId) && Number(row.friend_id) === Number(req.user.id)) {
        const accepted = await db.query(
          `UPDATE friendships SET status = 'accepted', updated_at = NOW()
           WHERE id = $1 RETURNING id`,
          [row.id]
        );
        const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $2`, [
          req.user.id,
          accepted.rows[0].id,
        ]);
        return res.json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
      }
      return res.status(409).json({ error: 'request already sent' });
    }

    const inserted = await db.query(
      `INSERT INTO friendships (user_id, friend_id, status)
       VALUES ($1, $2, 'pending') RETURNING id`,
      [req.user.id, friendId]
    );
    const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $2`, [
      req.user.id,
      inserted.rows[0].id,
    ]);
    res.status(201).json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
  } catch (error) {
    console.error('friends request:', error);
    res.status(500).json({ error: 'could not send friend request' });
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
      return res.status(404).json({ error: 'request not found' });
    }
    const full = await db.query(`${FRIEND_SELECT} WHERE f.id = $2`, [
      req.user.id,
      result.rows[0].id,
    ]);
    res.json({ success: true, friend: publicFriend(full.rows[0], req.user.id) });
  } catch (error) {
    console.error('friends accept:', error);
    res.status(500).json({ error: 'could not accept request' });
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
      return res.status(404).json({ error: 'friendship not found' });
    }
    res.json({ success: true });
  } catch (error) {
    console.error('friends remove:', error);
    res.status(500).json({ error: 'could not remove friendship' });
  }
}

router.post('/:id/decline', removeFriendship);
router.delete('/:id', removeFriendship);

module.exports = router;
module.exports.ensureFriendsSchema = ensureFriendsSchema;
