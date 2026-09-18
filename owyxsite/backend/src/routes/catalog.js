const express = require('express');
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const multer = require('multer');
const db = require('../database/connection');
const { authenticateToken, requireRole } = require('./auth');

// Site control-plane: packs (what to download) + servers (where to connect).
// Players never receive SFTP credentials. Bytes live as HTTP URLs, not in Postgres.

const SOURCE_TYPES = ['http_zip', 'http_manifest', 'google_drive', 'mrpack', 'sftp', 'local_ingest'];
const LOADERS = ['vanilla', 'fabric', 'forge', 'neoforge', 'quilt'];
const KINDS = ['owyx', 'community'];
const ACCESS_MODES = ['open', 'whitelist', 'blacklist'];
const ID_RE = /^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/;
const ingestDir = path.join(__dirname, '../../uploads/packs');

const packsAdmin = express.Router();
const serversAdmin = express.Router();
packsAdmin.use(authenticateToken, requireRole(['admin', 'moderator']));
packsAdmin.use((req, res, next) => {
  if (req.method === 'GET' || req.method === 'HEAD' || req.method === 'OPTIONS') return next();
  return requireRole(['admin'])(req, res, next);
});
serversAdmin.use(authenticateToken, requireRole(['admin', 'moderator']));
serversAdmin.use((req, res, next) => {
  if (req.method === 'GET' || req.method === 'HEAD' || req.method === 'OPTIONS') return next();
  return requireRole(['admin'])(req, res, next);
});

function publicBase(req) {
  const fromEnv = (process.env.API_PUBLIC_URL || '').trim();
  if (fromEnv) return fromEnv.replace(/\/+$/, '');
  return `${req.protocol}://${req.get('host')}`;
}

function absoluteAsset(req, value) {
  if (!value) return null;
  if (/^https?:\/\//i.test(value)) return value;
  return `${publicBase(req)}${value.startsWith('/') ? '' : '/'}${value}`;
}

function parseConfig(raw) {
  if (!raw) return {};
  if (typeof raw === 'object') return raw;
  try {
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

function slugify(name, fallback) {
  const base = String(name || '')
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 48);
  return base || fallback;
}

async function uniqueId(table, base) {
  let candidate = base;
  let n = 2;
  while (true) {
    const result = await db.query(`SELECT 1 FROM ${table} WHERE id = $1`, [candidate]);
    if (result.rows.length === 0) return candidate;
    candidate = `${base}-${n}`.slice(0, 64);
    n += 1;
    if (n > 99) return `${base}-${Date.now().toString(36)}`.slice(0, 64);
  }
}

function normalizeId(raw, name, fallback) {
  const given = String(raw || '').trim().toLowerCase();
  if (given) {
    if (!ID_RE.test(given)) {
      const err = new Error('id: латиница, цифры и дефис, 2–64 символа');
      err.status = 400;
      throw err;
    }
    return given;
  }
  return slugify(name, fallback);
}

function asBool(value, fallback = true) {
  if (value === undefined || value === null || value === '') return fallback;
  if (typeof value === 'boolean') return value;
  if (value === 'true' || value === '1' || value === 1) return true;
  if (value === 'false' || value === '0' || value === 0) return false;
  return fallback;
}

function validateLoader(loader) {
  const v = String(loader || 'vanilla').trim().toLowerCase();
  if (!LOADERS.includes(v)) {
    const err = new Error(`loader: ${LOADERS.join(', ')}`);
    err.status = 400;
    throw err;
  }
  return v;
}

function validateKind(kind) {
  const v = String(kind || 'owyx').trim().toLowerCase();
  if (!KINDS.includes(v)) {
    const err = new Error('kind: owyx или community');
    err.status = 400;
    throw err;
  }
  return v;
}

function validateAccessMode(mode) {
  const v = String(mode || 'open').trim().toLowerCase();
  if (!ACCESS_MODES.includes(v)) {
    const err = new Error(`accessMode: ${ACCESS_MODES.join(', ')}`);
    err.status = 400;
    throw err;
  }
  return v;
}

/** Guest sees only open. Signed-in applies whitelist/blacklist via catalog_acl. */
async function filterByAcl(req, rows, resourceType) {
  if (!rows.length) return [];
  const userId = req.user?.id != null ? Number(req.user.id) : null;
  const ids = rows.map((r) => r.id);
  const acl = await db.query(
    `SELECT resource_id, user_id, effect
     FROM catalog_acl
     WHERE resource_type = $1 AND resource_id = ANY($2::text[])`,
    [resourceType, ids]
  );
  const byResource = new Map();
  for (const row of acl.rows) {
    if (!byResource.has(row.resource_id)) byResource.set(row.resource_id, []);
    byResource.get(row.resource_id).push(row);
  }
  return rows.filter((row) => {
    const mode = row.access_mode || 'open';
    if (mode === 'open') return true;
    if (userId == null) return false;
    const entries = byResource.get(row.id) || [];
    if (mode === 'whitelist') {
      return entries.some((e) => Number(e.user_id) === userId && e.effect === 'allow');
    }
    if (mode === 'blacklist') {
      return !entries.some((e) => Number(e.user_id) === userId && e.effect === 'deny');
    }
    return true;
  });
}

function validateHttpUrl(raw, field) {
  const url = String(raw || '').trim();
  if (!url) {
    const err = new Error(`${field} обязателен`);
    err.status = 400;
    throw err;
  }
  if (url.startsWith('/')) return url;
  let parsed;
  try {
    parsed = new URL(url);
  } catch {
    const err = new Error(`${field}: некорректный URL`);
    err.status = 400;
    throw err;
  }
  if (parsed.protocol !== 'http:' && parsed.protocol !== 'https:') {
    const err = new Error(`${field}: только http(s)`);
    err.status = 400;
    throw err;
  }
  if (parsed.username || parsed.password) {
    const err = new Error(`${field}: URL не должен содержать логин/пароль`);
    err.status = 400;
    throw err;
  }
  return url;
}

function validateSha256(raw) {
  if (raw === undefined || raw === null || raw === '') return null;
  const v = String(raw).trim().toLowerCase();
  if (!/^[a-f0-9]{64}$/.test(v)) {
    const err = new Error('sha256: 64 hex-символа');
    err.status = 400;
    throw err;
  }
  return v;
}

function normalizeSource(typeRaw, configRaw) {
  const type = String(typeRaw || 'http_zip').trim().toLowerCase();
  if (!SOURCE_TYPES.includes(type)) {
    const err = new Error(`source.type: ${SOURCE_TYPES.join(', ')}`);
    err.status = 400;
    throw err;
  }
  const cfg = parseConfig(configRaw) || {};
  if (type === 'http_zip' || type === 'local_ingest') {
    const url = cfg.url ? validateHttpUrl(cfg.url, 'source.url') : null;
    if (type === 'http_zip' && !url) {
      const err = new Error('http_zip: нужен source.url');
      err.status = 400;
      throw err;
    }
    return { type, config: { url, sha256: validateSha256(cfg.sha256) } };
  }
  if (type === 'http_manifest') {
    const manifestUrl = validateHttpUrl(cfg.manifestUrl || cfg.url, 'source.manifestUrl');
    return { type, config: { manifestUrl, files: Array.isArray(cfg.files) ? cfg.files : undefined } };
  }
  if (type === 'google_drive') {
    const url = validateHttpUrl(cfg.url || cfg.directDownloadUrl, 'source.url');
    const direct = cfg.directDownloadUrl ? validateHttpUrl(cfg.directDownloadUrl, 'source.directDownloadUrl') : null;
    return {
      type,
      config: {
        url,
        directDownloadUrl: direct,
        note: String(cfg.note || '').slice(0, 400) || null,
      },
    };
  }
  if (type === 'mrpack') {
    const url = validateHttpUrl(cfg.url || cfg.downloadUrl, 'source.url');
    return { type, config: { url, ingest: 'planned' } };
  }
  // sftp — admin-only warehouse. Players never see these fields.
  const host = String(cfg.host || '').trim();
  const user = String(cfg.user || '').trim();
  const remotePath = String(cfg.path || '').trim();
  if (!host || !user || !remotePath) {
    const err = new Error('sftp: нужны host, user, path');
    err.status = 400;
    throw err;
  }
  const port = parseInt(cfg.port, 10) || 22;
  if (port < 1 || port > 65535) {
    const err = new Error('sftp.port: 1–65535');
    err.status = 400;
    throw err;
  }
  return {
    type,
    config: {
      host,
      port,
      user,
      path: remotePath,
      password: cfg.password ? String(cfg.password) : undefined,
    },
  };
}

function adminSourceView(type, config) {
  const cfg = parseConfig(config);
  if (type === 'sftp') {
    return {
      type,
      config: {
        host: cfg.host || '',
        port: cfg.port || 22,
        user: cfg.user || '',
        path: cfg.path || '',
        hasPassword: Boolean(cfg.password),
      },
    };
  }
  return { type, config: cfg };
}

function publicPack(req, row) {
  const type = row.source_type;
  const cfg = parseConfig(row.source_config);
  const out = {
    id: row.id,
    name: row.name,
    minecraft: row.minecraft,
    loader: row.loader,
    iconUrl: absoluteAsset(req, row.icon_url),
    description: row.description || '',
    sourceType: type,
    manifestUrl: row.manifest_url ? absoluteAsset(req, row.manifest_url) : null,
    accessMode: row.access_mode || 'open',
  };
  if (type === 'http_zip' || type === 'local_ingest') {
    out.downloadUrl = cfg.url ? absoluteAsset(req, cfg.url) : null;
    if (cfg.sha256) out.sha256 = cfg.sha256;
  } else if (type === 'http_manifest') {
    out.manifestUrl = absoluteAsset(req, cfg.manifestUrl || row.manifest_url);
  } else if (type === 'google_drive') {
    if (cfg.directDownloadUrl) {
      out.downloadUrl = cfg.directDownloadUrl;
    } else {
      out.downloadAvailable = false;
    }
  } else if (type === 'mrpack') {
    out.downloadUrl = cfg.url ? absoluteAsset(req, cfg.url) : null;
    out.downloadAvailable = false;
    out.ingest = 'planned';
  } else if (type === 'sftp') {
    out.downloadAvailable = false;
  }
  return out;
}

function publicServer(req, row, packRow) {
  return {
    id: row.id,
    name: row.name,
    iconUrl: absoluteAsset(req, row.icon_url),
    address: row.address,
    port: row.port,
    kind: row.kind,
    packId: row.pack_id,
    minecraft: row.minecraft || packRow?.minecraft || null,
    loader: row.loader || packRow?.loader || null,
    requiresAccount: Boolean(row.requires_account),
    accessMode: row.access_mode || 'open',
    status: { online: null, players: null, max: null },
    pack: packRow ? publicPack(req, packRow) : null,
  };
}

function adminPack(row) {
  const source = adminSourceView(row.source_type, row.source_config);
  return {
    id: row.id,
    name: row.name,
    minecraft: row.minecraft,
    loader: row.loader,
    iconUrl: row.icon_url,
    description: row.description || '',
    sourceType: source.type,
    source: source,
    manifestUrl: row.manifest_url,
    published: Boolean(row.published),
    accessMode: row.access_mode || 'open',
    createdAt: row.created_at,
    updatedAt: row.updated_at,
    createdBy: row.created_by,
  };
}

function adminServer(row) {
  return {
    id: row.id,
    name: row.name,
    iconUrl: row.icon_url,
    address: row.address,
    port: row.port,
    kind: row.kind,
    packId: row.pack_id,
    minecraft: row.minecraft,
    loader: row.loader,
    requiresAccount: Boolean(row.requires_account),
    published: Boolean(row.published),
    accessMode: row.access_mode || 'open',
    sortOrder: row.sort_order,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
    createdBy: row.created_by,
  };
}

function readPackBody(body) {
  const name = String(body.name || '').trim();
  if (name.length < 2 || name.length > 120) {
    const err = new Error('Название пака: 2–120 символов');
    err.status = 400;
    throw err;
  }
  const minecraft = String(body.minecraft || '').trim();
  if (!minecraft || minecraft.length > 32) {
    const err = new Error('Укажите версию Minecraft');
    err.status = 400;
    throw err;
  }
  const source = normalizeSource(body.sourceType || body.source?.type, body.sourceConfig || body.source?.config);
  return {
    name,
    minecraft,
    loader: validateLoader(body.loader),
    iconUrl: body.iconUrl != null ? String(body.iconUrl).trim().slice(0, 500) || null : null,
    description: String(body.description || '').slice(0, 2000),
    source,
    manifestUrl: body.manifestUrl != null ? String(body.manifestUrl).trim() || null : null,
    published: asBool(body.published, true),
    accessMode: validateAccessMode(body.accessMode),
  };
}

function readServerBody(body) {
  const name = String(body.name || '').trim();
  if (name.length < 2 || name.length > 120) {
    const err = new Error('Название сервера: 2–120 символов');
    err.status = 400;
    throw err;
  }
  const address = String(body.address || '').trim();
  if (!address || address.length > 255 || /[\s@]/.test(address)) {
    const err = new Error('Адрес: хост без пробелов и без логина');
    err.status = 400;
    throw err;
  }
  const port = parseInt(body.port, 10);
  if (!Number.isFinite(port) || port < 1 || port > 65535) {
    const err = new Error('Порт: 1–65535');
    err.status = 400;
    throw err;
  }
  return {
    name,
    iconUrl: body.iconUrl != null ? String(body.iconUrl).trim().slice(0, 500) || null : null,
    address,
    port,
    kind: validateKind(body.kind),
    packId: body.packId ? String(body.packId).trim() : null,
    minecraft: body.minecraft != null ? String(body.minecraft).trim().slice(0, 32) || null : null,
    loader: body.loader != null ? validateLoader(body.loader) : null,
    requiresAccount: asBool(body.requiresAccount, false),
    published: asBool(body.published, true),
    accessMode: validateAccessMode(body.accessMode),
    sortOrder: Number.isFinite(parseInt(body.sortOrder, 10)) ? parseInt(body.sortOrder, 10) : 0,
  };
}

function sendError(res, error, fallback) {
  const status = error.status || 500;
  if (status >= 500) console.error(fallback, error);
  res.status(status).json({ error: status >= 500 ? fallback : error.message });
}

async function listPublishedPacks(req) {
  const result = await db.query(
    `SELECT * FROM packs WHERE published = true ORDER BY name ASC`
  );
  const allowed = await filterByAcl(req, result.rows, 'pack');
  return allowed.map((row) => publicPack(req, row));
}

async function listPublishedServers(req) {
  const result = await db.query(
    `SELECT s.*,
            p.id AS p_id, p.name AS p_name, p.minecraft AS p_minecraft, p.loader AS p_loader,
            p.icon_url AS p_icon_url, p.description AS p_description,
            p.source_type AS p_source_type, p.source_config AS p_source_config,
            p.manifest_url AS p_manifest_url, p.published AS p_published,
            p.access_mode AS p_access_mode
     FROM servers s
     LEFT JOIN packs p ON p.id = s.pack_id
     WHERE s.published = true
     ORDER BY s.sort_order ASC, s.name ASC`
  );
  const allowed = await filterByAcl(req, result.rows, 'server');
  const out = [];
  for (const row of allowed) {
    const packRow = row.p_id
      ? {
          id: row.p_id,
          name: row.p_name,
          minecraft: row.p_minecraft,
          loader: row.p_loader,
          icon_url: row.p_icon_url,
          description: row.p_description,
          source_type: row.p_source_type,
          source_config: row.p_source_config,
          manifest_url: row.p_manifest_url,
          published: row.p_published,
          access_mode: row.p_access_mode,
        }
      : null;
    if (packRow) {
      const packOk = await filterByAcl(req, [packRow], 'pack');
      if (!packOk.length) continue;
    }
    out.push(publicServer(req, row, packRow));
  }
  return out;
}

async function getPublishedPack(req, id) {
  const result = await db.query(`SELECT * FROM packs WHERE id = $1 AND published = true`, [id]);
  return result.rows[0] ? publicPack(req, result.rows[0]) : null;
}

function packManifest(req, row) {
  const type = row.source_type;
  const cfg = parseConfig(row.source_config);
  const files = [];
  if (type === 'http_manifest' && Array.isArray(cfg.files)) {
    for (const file of cfg.files) {
      files.push({
        path: String(file.path || file.name || 'file'),
        url: file.url ? absoluteAsset(req, file.url) : null,
        sha256: file.sha256 || null,
        size: file.size || null,
      });
    }
  } else if (type === 'http_zip' || type === 'local_ingest') {
    if (cfg.url) {
      files.push({
        path: `${row.id}.zip`,
        url: absoluteAsset(req, cfg.url),
        sha256: cfg.sha256 || null,
        size: cfg.size || null,
      });
    }
  } else if (type === 'google_drive' && cfg.directDownloadUrl) {
    files.push({
      path: `${row.id}.zip`,
      url: cfg.directDownloadUrl,
      sha256: cfg.sha256 || null,
    });
  }
  return {
    id: row.id,
    name: row.name,
    minecraft: row.minecraft,
    loader: row.loader,
    sourceType: type,
    files,
  };
}

packsAdmin.get('/', async (_req, res) => {
  try {
    const result = await db.query(`SELECT * FROM packs ORDER BY updated_at DESC, name ASC`);
    res.json({ packs: result.rows.map(adminPack) });
  } catch (error) {
    sendError(res, error, 'Не удалось загрузить паки');
  }
});

packsAdmin.post('/', async (req, res) => {
  try {
    const body = readPackBody(req.body);
    const id = await uniqueId('packs', normalizeId(req.body.id, body.name, 'pack'));
    const manifestUrl = body.manifestUrl || `/api/launcher/v1/packs/${id}/manifest`;
    const result = await db.query(
      `INSERT INTO packs (id, name, minecraft, loader, icon_url, description, source_type, source_config, manifest_url, published, access_mode, created_by)
       VALUES ($1,$2,$3,$4,$5,$6,$7,$8::jsonb,$9,$10,$11,$12) RETURNING *`,
      [
        id,
        body.name,
        body.minecraft,
        body.loader,
        body.iconUrl,
        body.description,
        body.source.type,
        JSON.stringify(body.source.config),
        manifestUrl,
        body.published,
        body.accessMode,
        req.user.id,
      ]
    );
    res.status(201).json({ success: true, pack: adminPack(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось создать пак');
  }
});

packsAdmin.get('/:id', async (req, res) => {
  try {
    const result = await db.query(`SELECT * FROM packs WHERE id = $1`, [req.params.id]);
    if (!result.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    res.json({ pack: adminPack(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось загрузить пак');
  }
});

packsAdmin.put('/:id', async (req, res) => {
  try {
    const existing = await db.query(`SELECT * FROM packs WHERE id = $1`, [req.params.id]);
    if (!existing.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    const prev = existing.rows[0];
    const merged = {
      name: req.body.name !== undefined ? req.body.name : prev.name,
      minecraft: req.body.minecraft !== undefined ? req.body.minecraft : prev.minecraft,
      loader: req.body.loader !== undefined ? req.body.loader : prev.loader,
      iconUrl: req.body.iconUrl !== undefined ? req.body.iconUrl : prev.icon_url,
      description: req.body.description !== undefined ? req.body.description : prev.description,
      sourceType: req.body.sourceType || req.body.source?.type || prev.source_type,
      sourceConfig: req.body.sourceConfig || req.body.source?.config || prev.source_config,
      manifestUrl: req.body.manifestUrl !== undefined ? req.body.manifestUrl : prev.manifest_url,
      published: req.body.published !== undefined ? req.body.published : prev.published,
      accessMode: req.body.accessMode !== undefined ? req.body.accessMode : prev.access_mode || 'open',
    };
    if (prev.source_type === 'sftp' && merged.sourceType === 'sftp') {
      const nextCfg = parseConfig(merged.sourceConfig);
      const prevCfg = parseConfig(prev.source_config);
      if (!nextCfg.password && prevCfg.password) nextCfg.password = prevCfg.password;
      merged.sourceConfig = nextCfg;
    }
    const body = readPackBody(merged);
    const result = await db.query(
      `UPDATE packs SET name=$2, minecraft=$3, loader=$4, icon_url=$5, description=$6,
        source_type=$7, source_config=$8::jsonb, manifest_url=$9, published=$10, access_mode=$11, updated_at=NOW()
       WHERE id=$1 RETURNING *`,
      [
        req.params.id,
        body.name,
        body.minecraft,
        body.loader,
        body.iconUrl,
        body.description,
        body.source.type,
        JSON.stringify(body.source.config),
        body.manifestUrl,
        body.published,
        body.accessMode,
      ]
    );
    res.json({ success: true, pack: adminPack(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось обновить пак');
  }
});

packsAdmin.delete('/:id', async (req, res) => {
  try {
    const result = await db.query(
      `DELETE FROM packs WHERE id = $1 RETURNING id, source_type, source_config`,
      [req.params.id]
    );
    if (!result.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    const config = parseConfig(result.rows[0].source_config);
    if (result.rows[0].source_type === 'local_ingest' || /^\/uploads\/packs\//.test(config.url || '')) {
      const fileName = path.basename(String(config.url || ''));
      if (fileName) {
        await fs.promises.unlink(path.join(ingestDir, fileName)).catch(() => {});
      }
    }
    res.json({ success: true });
  } catch (error) {
    sendError(res, error, 'Не удалось удалить пак');
  }
});

const ingestUpload = multer({
  storage: multer.diskStorage({
    destination: async (_req, _file, cb) => {
      try {
        await fs.promises.mkdir(ingestDir, { recursive: true });
        cb(null, ingestDir);
      } catch (error) {
        cb(error);
      }
    },
    filename: (req, file, cb) => {
      const safe = String(req.params.id || 'pack').replace(/[^a-z0-9-]/gi, '');
      cb(null, `${safe}-${Date.now()}.zip`);
    },
  }),
  limits: { fileSize: 2 * 1024 * 1024 * 1024 },
  fileFilter: (_req, file, cb) => {
    const name = String(file.originalname || '').toLowerCase();
    if (!name.endsWith('.zip') && !name.endsWith('.mrpack')) {
      cb(new Error('Нужен .zip или .mrpack'));
      return;
    }
    cb(null, true);
  },
});

packsAdmin.post(
  '/:id/ingest',
  (req, res, next) => {
    if (!ID_RE.test(String(req.params.id || ''))) {
      return res.status(400).json({ error: 'Некорректный id пака' });
    }
    next();
  },
  (req, res, next) => {
    ingestUpload.single('archive')(req, res, (err) => {
      if (!err) return next();
      if (err.code === 'LIMIT_FILE_SIZE') {
        return res.status(413).json({
          error: 'archive too large (max 2 GB)',
        });
      }
      return res.status(400).json({ error: err.message || 'upload failed' });
    });
  },
  async (req, res) => {
  try {
    const existing = await db.query(`SELECT * FROM packs WHERE id = $1`, [req.params.id]);
    if (!existing.rows[0]) return res.status(404).json({ error: 'Пак не найден' });
    if (!req.file) return res.status(400).json({ error: 'Приложите zip (поле archive), до 2 ГБ' });
    const bytes = await fs.promises.readFile(req.file.path);
    const sha256 = crypto.createHash('sha256').update(bytes).digest('hex');
    const isMrpack = String(req.file.originalname || '').toLowerCase().endsWith('.mrpack');
    const finalName = `${req.params.id}.${isMrpack ? 'mrpack' : 'zip'}`;
    const finalPath = path.join(ingestDir, finalName);
    await fs.promises.rename(req.file.path, finalPath);
    const url = `/uploads/packs/${finalName}`;
    const result = await db.query(
      `UPDATE packs SET source_type=$2, source_config=$3::jsonb, manifest_url=$4, updated_at=NOW()
       WHERE id=$1 RETURNING *`,
      [
        req.params.id,
        isMrpack ? 'mrpack' : 'http_zip',
        JSON.stringify({ url, sha256, size: bytes.length, ingest: isMrpack ? 'planned' : 'local' }),
        `/api/launcher/v1/packs/${req.params.id}/manifest`,
      ]
    );
    res.json({
      success: true,
      pack: adminPack(result.rows[0]),
      downloadUrl: url,
      sha256,
      note: 'Файл на этом сайте. Для очень больших складов задайте http_zip URL мини-ПК вручную.',
    });
  } catch (error) {
    if (req.file?.path) fs.promises.unlink(req.file.path).catch(() => {});
    sendError(res, error, 'Не удалось принять архив');
  }
});

serversAdmin.get('/', async (_req, res) => {
  try {
    const result = await db.query(`SELECT * FROM servers ORDER BY sort_order ASC, name ASC`);
    res.json({ servers: result.rows.map(adminServer) });
  } catch (error) {
    sendError(res, error, 'Не удалось загрузить серверы');
  }
});

serversAdmin.post('/', async (req, res) => {
  try {
    const body = readServerBody(req.body);
    const id = await uniqueId('servers', normalizeId(req.body.id, body.name, 'server'));
    if (body.packId) {
      const pack = await db.query(`SELECT id FROM packs WHERE id = $1`, [body.packId]);
      if (!pack.rows[0]) return res.status(400).json({ error: 'Пак не найден' });
    }
    const result = await db.query(
      `INSERT INTO servers (id, name, icon_url, address, port, kind, pack_id, minecraft, loader, requires_account, published, access_mode, sort_order, created_by)
       VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14) RETURNING *`,
      [
        id,
        body.name,
        body.iconUrl,
        body.address,
        body.port,
        body.kind,
        body.packId,
        body.minecraft,
        body.loader,
        body.requiresAccount,
        body.published,
        body.accessMode,
        body.sortOrder,
        req.user.id,
      ]
    );
    res.status(201).json({ success: true, server: adminServer(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось создать сервер');
  }
});

serversAdmin.get('/:id', async (req, res) => {
  try {
    const result = await db.query(`SELECT * FROM servers WHERE id = $1`, [req.params.id]);
    if (!result.rows[0]) return res.status(404).json({ error: 'Сервер не найден' });
    res.json({ server: adminServer(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось загрузить сервер');
  }
});

serversAdmin.put('/:id', async (req, res) => {
  try {
    const existing = await db.query(`SELECT * FROM servers WHERE id = $1`, [req.params.id]);
    if (!existing.rows[0]) return res.status(404).json({ error: 'Сервер не найден' });
    const prev = existing.rows[0];
    const body = readServerBody({
      name: req.body.name !== undefined ? req.body.name : prev.name,
      iconUrl: req.body.iconUrl !== undefined ? req.body.iconUrl : prev.icon_url,
      address: req.body.address !== undefined ? req.body.address : prev.address,
      port: req.body.port !== undefined ? req.body.port : prev.port,
      kind: req.body.kind !== undefined ? req.body.kind : prev.kind,
      packId: req.body.packId !== undefined ? req.body.packId : prev.pack_id,
      minecraft: req.body.minecraft !== undefined ? req.body.minecraft : prev.minecraft,
      loader: req.body.loader !== undefined ? req.body.loader : prev.loader,
      requiresAccount: req.body.requiresAccount !== undefined ? req.body.requiresAccount : prev.requires_account,
      published: req.body.published !== undefined ? req.body.published : prev.published,
      accessMode: req.body.accessMode !== undefined ? req.body.accessMode : prev.access_mode || 'open',
      sortOrder: req.body.sortOrder !== undefined ? req.body.sortOrder : prev.sort_order,
    });
    if (body.packId) {
      const pack = await db.query(`SELECT id FROM packs WHERE id = $1`, [body.packId]);
      if (!pack.rows[0]) return res.status(400).json({ error: 'Пак не найден' });
    }
    const result = await db.query(
      `UPDATE servers SET name=$2, icon_url=$3, address=$4, port=$5, kind=$6, pack_id=$7,
        minecraft=$8, loader=$9, requires_account=$10, published=$11, access_mode=$12, sort_order=$13, updated_at=NOW()
       WHERE id=$1 RETURNING *`,
      [
        req.params.id,
        body.name,
        body.iconUrl,
        body.address,
        body.port,
        body.kind,
        body.packId,
        body.minecraft,
        body.loader,
        body.requiresAccount,
        body.published,
        body.accessMode,
        body.sortOrder,
      ]
    );
    res.json({ success: true, server: adminServer(result.rows[0]) });
  } catch (error) {
    sendError(res, error, 'Не удалось обновить сервер');
  }
});

serversAdmin.delete('/:id', async (req, res) => {
  try {
    const result = await db.query(`DELETE FROM servers WHERE id = $1 RETURNING id`, [req.params.id]);
    if (!result.rows[0]) return res.status(404).json({ error: 'Сервер не найден' });
    res.json({ success: true });
  } catch (error) {
    sendError(res, error, 'Не удалось удалить сервер');
  }
});

async function ensureCatalogSchema() {
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.packs (
      id TEXT PRIMARY KEY,
      name VARCHAR(120) NOT NULL,
      minecraft VARCHAR(32) NOT NULL,
      loader VARCHAR(32) NOT NULL DEFAULT 'vanilla',
      icon_url TEXT,
      description TEXT NOT NULL DEFAULT '',
      source_type VARCHAR(32) NOT NULL DEFAULT 'http_zip',
      source_config JSONB NOT NULL DEFAULT '{}'::jsonb,
      manifest_url TEXT,
      published BOOLEAN NOT NULL DEFAULT true,
      access_mode VARCHAR(16) NOT NULL DEFAULT 'open',
      created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      created_by INTEGER
    )
  `);
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.servers (
      id TEXT PRIMARY KEY,
      name VARCHAR(120) NOT NULL,
      icon_url TEXT,
      address VARCHAR(255) NOT NULL,
      port INTEGER NOT NULL DEFAULT 25565,
      kind VARCHAR(32) NOT NULL DEFAULT 'owyx',
      pack_id TEXT REFERENCES public.packs(id) ON DELETE SET NULL,
      minecraft VARCHAR(32),
      loader VARCHAR(32),
      requires_account BOOLEAN NOT NULL DEFAULT false,
      published BOOLEAN NOT NULL DEFAULT true,
      access_mode VARCHAR(16) NOT NULL DEFAULT 'open',
      sort_order INTEGER NOT NULL DEFAULT 0,
      created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      created_by INTEGER
    )
  `);
  await db.query(`ALTER TABLE public.packs ADD COLUMN IF NOT EXISTS access_mode VARCHAR(16) NOT NULL DEFAULT 'open'`);
  await db.query(`ALTER TABLE public.servers ADD COLUMN IF NOT EXISTS access_mode VARCHAR(16) NOT NULL DEFAULT 'open'`);
  await db.query(`
    CREATE TABLE IF NOT EXISTS public.catalog_acl (
      id BIGSERIAL PRIMARY KEY,
      resource_type VARCHAR(16) NOT NULL CHECK (resource_type IN ('pack', 'server')),
      resource_id TEXT NOT NULL,
      user_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
      effect VARCHAR(8) NOT NULL CHECK (effect IN ('allow', 'deny')),
      created_at TIMESTAMP WITHOUT TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      CONSTRAINT catalog_acl_unique UNIQUE (resource_type, resource_id, user_id, effect)
    )
  `);
  await db.query(`CREATE INDEX IF NOT EXISTS packs_published_idx ON public.packs (published)`);
  await db.query(`CREATE INDEX IF NOT EXISTS servers_published_sort_idx ON public.servers (published, sort_order, name)`);
  await db.query(`CREATE INDEX IF NOT EXISTS catalog_acl_resource_idx ON public.catalog_acl (resource_type, resource_id)`);
  // Demo seed removed for OBT — admins publish real packs/servers from the launcher or site.
}

async function loadAcl(resourceType, resourceId) {
  const result = await db.query(
    `SELECT a.id, a.user_id, a.effect, u.nickname
     FROM catalog_acl a
     JOIN users u ON u.id = a.user_id
     WHERE a.resource_type = $1 AND a.resource_id = $2
     ORDER BY u.nickname ASC`,
    [resourceType, resourceId]
  );
  return result.rows.map((row) => ({
    id: String(row.id),
    userId: String(row.user_id),
    nickname: row.nickname,
    effect: row.effect,
  }));
}

function aclRouter(resourceType, table) {
  const r = express.Router({ mergeParams: true });
  r.get('/:id/acl', async (req, res) => {
    try {
      const exists = await db.query(`SELECT id FROM ${table} WHERE id = $1`, [req.params.id]);
      if (!exists.rows[0]) return res.status(404).json({ error: 'не найдено' });
      res.json({ entries: await loadAcl(resourceType, req.params.id) });
    } catch (error) {
      sendError(res, error, 'не удалось загрузить ACL');
    }
  });
  r.put('/:id/acl', async (req, res) => {
    try {
      const exists = await db.query(`SELECT id FROM ${table} WHERE id = $1`, [req.params.id]);
      if (!exists.rows[0]) return res.status(404).json({ error: 'не найдено' });
      const accessMode = validateAccessMode(req.body.accessMode);
      await db.query(`UPDATE ${table} SET access_mode = $2, updated_at = NOW() WHERE id = $1`, [
        req.params.id,
        accessMode,
      ]);
      const nicknames = Array.isArray(req.body.nicknames)
        ? req.body.nicknames.map((n) => String(n).trim()).filter(Boolean)
        : [];
      const effect = accessMode === 'blacklist' ? 'deny' : 'allow';
      await db.query(`DELETE FROM catalog_acl WHERE resource_type = $1 AND resource_id = $2`, [
        resourceType,
        req.params.id,
      ]);
      for (const nick of nicknames) {
        const user = await db.query(
          `SELECT id FROM users WHERE LOWER(nickname) = LOWER($1)`,
          [nick]
        );
        if (!user.rows[0]) {
          const err = new Error(`пользователь не найден: ${nick}`);
          err.status = 400;
          throw err;
        }
        await db.query(
          `INSERT INTO catalog_acl (resource_type, resource_id, user_id, effect)
           VALUES ($1,$2,$3,$4)
           ON CONFLICT (resource_type, resource_id, user_id, effect) DO NOTHING`,
          [resourceType, req.params.id, user.rows[0].id, effect]
        );
      }
      const row = await db.query(`SELECT * FROM ${table} WHERE id = $1`, [req.params.id]);
      res.json({
        success: true,
        accessMode,
        entries: await loadAcl(resourceType, req.params.id),
        pack: table === 'packs' ? adminPack(row.rows[0]) : undefined,
        server: table === 'servers' ? adminServer(row.rows[0]) : undefined,
      });
    } catch (error) {
      sendError(res, error, 'не удалось сохранить ACL');
    }
  });
  return r;
}

packsAdmin.use(aclRouter('pack', 'packs'));
serversAdmin.use(aclRouter('server', 'servers'));

module.exports = {
  packsAdmin,
  serversAdmin,
  publicPack,
  publicServer,
  packManifest,
  listPublishedPacks,
  listPublishedServers,
  getPublishedPack,
  ensureCatalogSchema,
  publicBase,
  absoluteAsset,
};
