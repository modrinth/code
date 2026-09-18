const express = require('express');
const router = express.Router();
const db = require('../database/connection');
const { authenticateToken, optionalAuthenticate } = require('./auth');
const catalog = require('./catalog');
const { absoluteWebsiteAsset } = require('./csl-helpers');
const {
  clientIp,
  normalizeTelemetryEvent,
} = require('../utils/activityLog');

// Owyx launcher API.
//
// Product model: playing is OPEN. Applications ("заявки") are no longer a gate.
// A normal, active, non-banned account can play. Ban still blocks everything.
//
// Versioned surface lives under /api/launcher/v1/*. Catalogs (servers/packs)
// are real rows from the site control-plane — not empty stubs.

const LAUNCHER_API_VERSION = '1.4.0';

/** Build the launcher-facing view of a user row. */
function buildMe(req, user) {
  const banned = Boolean(user.is_banned);
  const active = user.is_active !== false;
  const emailVerified = Boolean(user.is_email_verified);
  const avatarUrl = absoluteWebsiteAsset(user.avatar_url);

  return {
    user: {
      id: user.id,
      nickname: user.nickname,
      displayNickname: user.display_nickname || user.nickname,
      email: user.email,
      role: user.role || 'user',
      trustLevel: user.trust_level ?? 0,
      banned,
      emailVerified,
      registeredAt: user.registered_at,
      avatarUrl,
      avatar_url: avatarUrl,
      nicknameChangedAt: user.nickname_changed_at || null,
      emailChangedAt: user.email_changed_at || null,
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
      avatarUrl,
      skinUrl: absoluteWebsiteAsset(user.skin_url),
      skinModel: user.skin_model || 'classic',
      capeUrl: absoluteWebsiteAsset(user.cape_url),
    },
    // Convenience mirrors for older launcher builds.
    avatarUrl,
    skinUrl: absoluteWebsiteAsset(user.skin_url),
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
    modules: ['servers', 'packs', 'news', 'cosmetics', 'adminCatalog', 'friends', 'csl', 'telemetry'],
    skins: {
      customSkinLoader: {
        mod: 'https://modrinth.com/mod/customskinloader',
        source: 'https://github.com/xfl03/MCCustomSkinLoader',
        license: 'GPL-3.0',
        apiRoot: '/api/csl/',
        legacySkin: '/api/csl/skins/{USERNAME}.png',
        profileJson: '/api/csl/{USERNAME}.json',
      },
    },
  });
});

// GET /api/launcher/v1/servers — published Owyx + community servers (ACL-filtered).
router.get('/v1/servers', optionalAuthenticate, async (req, res) => {
  try {
    const servers = await catalog.listPublishedServers(req);
    res.json({ servers });
  } catch (error) {
    console.error('launcher/v1/servers error:', error);
    res.status(500).json({ error: 'Не удалось загрузить серверы' });
  }
});

// GET /api/launcher/v1/packs — published builds (safe player fields only).
router.get('/v1/packs', optionalAuthenticate, async (req, res) => {
  try {
    const packs = await catalog.listPublishedPacks(req);
    res.json({ packs });
  } catch (error) {
    console.error('launcher/v1/packs error:', error);
    res.status(500).json({ error: 'Не удалось загрузить сборки' });
  }
});

// GET /api/launcher/v1/packs/:id — one published pack.
router.get('/v1/packs/:id', optionalAuthenticate, async (req, res) => {
  try {
    const pack = await catalog.getPublishedPack(req, req.params.id);
    if (!pack) return res.status(404).json({ error: 'Пак не найден' });
    const allowed = await catalog.listPublishedPacks(req);
    if (!allowed.some((p) => p.id === pack.id)) {
      return res.status(404).json({ error: 'Пак не найден' });
    }
    res.json({ pack });
  } catch (error) {
    console.error('launcher/v1/packs/:id error:', error);
    res.status(500).json({ error: 'Не удалось загрузить пак' });
  }
});

// GET /api/launcher/v1/packs/:id/manifest — file list + sha (no SFTP secrets).
router.get('/v1/packs/:id/manifest', optionalAuthenticate, async (req, res) => {
  try {
    const result = await db.query(
      `SELECT * FROM packs WHERE id = $1 AND published = true`,
      [req.params.id]
    );
    if (!result.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    const allowed = await catalog.listPublishedPacks(req);
    if (!allowed.some((p) => p.id === req.params.id)) {
      return res.status(404).json({ error: 'Пак не найден' });
    }
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
      skinUrl: absoluteWebsiteAsset(u.skin_url),
      skinModel: u.skin_model || 'classic',
      capeUrl: absoluteWebsiteAsset(u.cape_url),
      updatedAt: u.cosmetics_updated_at || null,
    });
  } catch (error) {
    console.error('launcher/v1/cosmetics error:', error);
    res.status(500).json({ error: 'Внутренняя ошибка сервера' });
  }
});

/**
 * POST /api/launcher/v1/telemetry
 * Anonymous launcher stats / errors. Optional JWT links user_id only.
 * Body: { installId: uuid, events: [...] }  (max 20 events)
 * No passwords, emails, paths with usernames — sanitized server-side.
 */
const TELEMETRY_WINDOW_MS = 60_000;
const TELEMETRY_MAX_PER_IP = 30;
const TELEMETRY_MAX_PER_INSTALL = 60;
/** @type {Map<string, { windowStart: number, count: number }>} */
const telemetryIpBuckets = new Map();
/** @type {Map<string, { windowStart: number, count: number }>} */
const telemetryInstallBuckets = new Map();
let lastTelemetryBucketSweep = Date.now();

function pruneTelemetryBuckets(map, now) {
  for (const [key, bucket] of map) {
    if (now - bucket.windowStart >= TELEMETRY_WINDOW_MS) map.delete(key);
  }
}

function takeTelemetryToken(map, key, max) {
  if (!key) return true;
  const now = Date.now();
  if (now - lastTelemetryBucketSweep > TELEMETRY_WINDOW_MS) {
    pruneTelemetryBuckets(telemetryIpBuckets, now);
    pruneTelemetryBuckets(telemetryInstallBuckets, now);
    lastTelemetryBucketSweep = now;
  }
  let bucket = map.get(key);
  if (!bucket || now - bucket.windowStart >= TELEMETRY_WINDOW_MS) {
    bucket = { windowStart: now, count: 0 };
    map.set(key, bucket);
  }
  if (bucket.count >= max) return false;
  bucket.count += 1;
  return true;
}

router.post('/v1/telemetry', optionalAuthenticate, async (req, res) => {
  try {
    const installId = String(req.body?.installId || req.body?.install_id || '').trim();
    const uuidRe =
      /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    if (!uuidRe.test(installId)) {
      return res.status(400).json({ error: 'installId must be a UUID' });
    }

    const ip = clientIp(req) || 'unknown';
    if (
      !takeTelemetryToken(telemetryIpBuckets, ip, TELEMETRY_MAX_PER_IP) ||
      !takeTelemetryToken(telemetryInstallBuckets, installId.toLowerCase(), TELEMETRY_MAX_PER_INSTALL)
    ) {
      return res.status(429).json({ error: 'Too many telemetry requests; try again later' });
    }

    const rawEvents = Array.isArray(req.body?.events)
      ? req.body.events
      : req.body?.event
        ? [req.body.event]
        : [];
    if (!rawEvents.length) {
      return res.status(400).json({ error: 'events required' });
    }
    if (rawEvents.length > 20) {
      return res.status(400).json({ error: 'max 20 events per request' });
    }

    const normalized = rawEvents.map(normalizeTelemetryEvent).filter(Boolean);
    if (!normalized.length) {
      return res.status(400).json({ error: 'no valid events' });
    }

    const userId = req.user?.id || null;
    let inserted = 0;

    for (const ev of normalized) {
      await db.query(
        `INSERT INTO launcher_telemetry
           (install_id, user_id, event_kind, message, app_version,
            os_name, os_version, arch, cpu_cores, ram_mb, locale, metadata, ip_address)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12::jsonb,$13)`,
        [
          installId,
          userId,
          ev.kind,
          ev.message,
          ev.appVersion,
          ev.osName,
          ev.osVersion,
          ev.arch,
          ev.cpuCores,
          ev.ramMb,
          ev.locale,
          JSON.stringify(ev.metadata || {}),
          ip === 'unknown' ? null : ip,
        ]
      );
      inserted += 1;
    }

    res.status(202).json({ ok: true, accepted: inserted });
  } catch (error) {
    console.error('launcher/v1/telemetry error:', error);
    if (error.code === '42P01') {
      return res.status(503).json({
        error: 'Telemetry table missing. Apply migrations/012_logs_telemetry.sql',
      });
    }
    res.status(500).json({ error: 'Не удалось принять телеметрию' });
  }
});

module.exports = router;
