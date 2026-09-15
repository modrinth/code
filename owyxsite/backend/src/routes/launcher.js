const express = require('express');
const router = express.Router();
const db = require('../database/connection');
const { authenticateToken } = require('./auth');
const catalog = require('./catalog');

// Owyx launcher API.
//
// Product model: playing is OPEN. Applications ("заявки") are no longer a gate.
// A normal, active, non-banned account can play. Ban still blocks everything.
//
// Versioned surface lives under /api/launcher/v1/*. Catalogs (servers/packs)
// are real rows from the site control-plane — not empty stubs.

const LAUNCHER_API_VERSION = '1.1.0';

/** Absolute base URL the launcher can use to download static assets (skins).
 *  Built from the request the launcher made (its own API base), so it works in
 *  dev (direct :3001) and behind a reverse proxy in prod (Host = public domain).
 *  Set API_PUBLIC_URL only to force a specific base. */
function publicBase(req) {
  const fromEnv = (process.env.API_PUBLIC_URL || '').trim();
  if (fromEnv) return fromEnv.replace(/\/+$/, '');
  return `${req.protocol}://${req.get('host')}`;
}

/** Turn a stored `/uploads/...` path into an absolute URL for the launcher. */
function absoluteAsset(req, value) {
  if (!value) return null;
  if (/^https?:\/\//i.test(value)) return value;
  return `${publicBase(req)}${value.startsWith('/') ? '' : '/'}${value}`;
}

/** Build the launcher-facing view of a user row. */
function buildMe(req, user) {
  const banned = Boolean(user.is_banned);
  const active = user.is_active !== false;
  const emailVerified = Boolean(user.is_email_verified);

  return {
    user: {
      id: user.id,
      nickname: user.nickname,
      email: user.email,
      role: user.role || 'user',
      trustLevel: user.trust_level ?? 0,
      banned,
      emailVerified,
      registeredAt: user.registered_at,
    },
    // Open access: any active, non-banned account may play. No application needed.
    serverAccess: active && !banned,
    // Why access is or isn't granted — human-readable, launcher shows it as-is.
    accessReason: banned
      ? 'banned'
      : !active
        ? 'inactive'
        : 'ok',
    cosmetics: {
      skinUrl: absoluteAsset(req, user.skin_url),
      skinModel: user.skin_model || 'classic',
      capeUrl: absoluteAsset(req, user.cape_url),
    },
    // Convenience mirror of the skin for older launcher builds.
    skinUrl: absoluteAsset(req, user.skin_url),
    // DEPRECATED: applications ("заявки") are no longer part of the product.
    // Always null now; kept in the response shape for one transition release.
    application: null,
  };
}

// GET /api/launcher/me — profile for the signed-in launcher user (open access).
router.get('/me', authenticateToken, async (req, res) => {
  try {
    res.json(buildMe(req, req.user));
  } catch (error) {
    console.error('launcher/me error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// Versioned alias so the launcher can pin /v1/me.
router.get('/v1/me', authenticateToken, async (req, res) => {
  try {
    res.json(buildMe(req, req.user));
  } catch (error) {
    console.error('launcher/v1/me error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

// GET /api/launcher/v1/status — public: launcher discovers the API and access model.
router.get('/v1/status', (_req, res) => {
  res.json({
    api: 'owyx-launcher',
    version: LAUNCHER_API_VERSION, // launcher-facing API surface version
    siteVersion: '0.1.0',          // Owyx site UI version
    // "open" = play without an approved application; ban still blocks.
    serverAccessModel: 'open',
    auth: {
      login: '/api/auth/login',
      me: '/api/launcher/me',
    },
    modules: ['servers', 'packs', 'news', 'cosmetics', 'adminCatalog'],
  });
});

// GET /api/launcher/v1/servers — published Owyx + community servers.
// Bearer optional: requiresAccount is a Play gate in the launcher, not a list filter.
router.get('/v1/servers', async (req, res) => {
  try {
    const servers = await catalog.listPublishedServers(req);
    res.json({ servers });
  } catch (error) {
    console.error('launcher/v1/servers error:', error);
    res.status(500).json({ error: 'Не удалось загрузить серверы' });
  }
});

// GET /api/launcher/v1/packs — published builds (safe player fields only).
router.get('/v1/packs', async (req, res) => {
  try {
    const packs = await catalog.listPublishedPacks(req);
    res.json({ packs });
  } catch (error) {
    console.error('launcher/v1/packs error:', error);
    res.status(500).json({ error: 'Не удалось загрузить сборки' });
  }
});

// GET /api/launcher/v1/packs/:id — one published pack.
router.get('/v1/packs/:id', async (req, res) => {
  try {
    const pack = await catalog.getPublishedPack(req, req.params.id);
    if (!pack) return res.status(404).json({ error: 'Пак не найден' });
    res.json({ pack });
  } catch (error) {
    console.error('launcher/v1/packs/:id error:', error);
    res.status(500).json({ error: 'Не удалось загрузить пак' });
  }
});

// GET /api/launcher/v1/packs/:id/manifest — file list + sha (no SFTP secrets).
router.get('/v1/packs/:id/manifest', async (req, res) => {
  try {
    const result = await db.query(
      `SELECT * FROM packs WHERE id = $1 AND published = true`,
      [req.params.id]
    );
    if (!result.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    res.json(catalog.packManifest(req, result.rows[0]));
  } catch (error) {
    console.error('launcher/v1/packs/:id/manifest error:', error);
    res.status(500).json({ error: 'Не удалось загрузить манифест' });
  }
});

// GET /api/launcher/v1/news — same published news as the site home.
router.get('/v1/news', async (req, res) => {
  try {
    const limit = Math.min(parseInt(req.query.limit, 10) || 12, 50);
    const result = await db.query(
      `SELECT id, title, tag, summary, published, created_at, updated_at
       FROM news WHERE published = true
       ORDER BY created_at DESC LIMIT $1`,
      [limit]
    );
    res.json({
      news: result.rows.map((r) => ({
        id: r.id,
        title: r.title,
        tag: r.tag,
        summary: r.summary,
        publishedAt: r.created_at,
        updatedAt: r.updated_at,
      })),
    });
  } catch (error) {
    console.error('launcher/v1/news error:', error);
    res.status(500).json({ error: 'Не удалось загрузить новости' });
  }
});

// GET /api/launcher/v1/cosmetics — cosmetics of the signed-in user.
router.get('/v1/cosmetics', authenticateToken, async (req, res) => {
  try {
    const u = req.user;
    res.json({
      skinUrl: absoluteAsset(req, u.skin_url),
      skinModel: u.skin_model || 'classic',
      capeUrl: absoluteAsset(req, u.cape_url),
      updatedAt: u.cosmetics_updated_at || null,
    });
  } catch (error) {
    console.error('launcher/v1/cosmetics error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

module.exports = router;
