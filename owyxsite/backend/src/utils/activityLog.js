/**
 * Shared helpers for account activity + launcher telemetry inserts.
 * Never store passwords, tokens, or full emails in metadata.
 */

const db = require('../database/connection');

function clientIp(req) {
  // Prefer Express trust-proxy IP set by middleware (req.clientIp / req.ip).
  // Do not trust raw X-Forwarded-For over that path.
  const trusted =
    (typeof req.clientIp === 'string' && req.clientIp) ||
    (typeof req.ip === 'string' && req.ip) ||
    (req.socket?.remoteAddress ? String(req.socket.remoteAddress) : '');
  let ip = trusted.toString().slice(0, 45);
  if (ip.startsWith('::ffff:')) ip = ip.substring(7);
  return ip || null;
}

function clientUa(req) {
  const ua = req.headers['user-agent'];
  return typeof ua === 'string' ? ua.slice(0, 512) : null;
}

/** Redact obvious secrets from free-form strings before storage. */
function sanitizeMessage(raw, max = 500) {
  if (raw == null) return null;
  let s = String(raw).slice(0, max);
  s = s.replace(/Bearer\s+[A-Za-z0-9\-._~+/]+=*/gi, 'Bearer [redacted]');
  s = s.replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi, '[email]');
  s = s.replace(/[\\/]Users[\\/][^\\/\s]+/gi, '/Users/[user]');
  s = s.replace(/[\\/]home[\\/][^\\/\s]+/gi, '/home/[user]');
  return s;
}

async function logUserActivity(userId, activityType, description, opts = {}) {
  const {
    req = null,
    metadata = null,
    ip = null,
    userAgent = null,
  } = opts;
  try {
    await db.query(
      `INSERT INTO user_activity
         (user_id, activity_type, description, metadata, ip_address, user_agent)
       VALUES ($1, $2, $3, $4, $5, $6)`,
      [
        userId,
        String(activityType).slice(0, 50),
        String(description || activityType).slice(0, 500),
        metadata ? JSON.stringify(metadata) : null,
        ip || (req ? clientIp(req) : null),
        userAgent || (req ? clientUa(req) : null),
      ]
    );
  } catch (err) {
    console.error('logUserActivity failed:', err.message);
  }
}

const ALLOWED_TELEMETRY_KINDS = new Set([
  'session_start',
  'heartbeat',
  'error',
  'crash',
  'perf',
  'feature',
]);

function normalizeTelemetryEvent(raw) {
  if (!raw || typeof raw !== 'object') return null;
  const kind = String(raw.kind || raw.event_kind || '').toLowerCase().slice(0, 40);
  if (!ALLOWED_TELEMETRY_KINDS.has(kind)) return null;

  const cpu = Number(raw.cpuCores ?? raw.cpu_cores);
  const ram = Number(raw.ramMb ?? raw.ram_mb);

  let metadata = {};
  if (raw.metadata && typeof raw.metadata === 'object' && !Array.isArray(raw.metadata)) {
    // Drop nested blobs that look like PII dumps
    const keys = Object.keys(raw.metadata).slice(0, 24);
    for (const k of keys) {
      const v = raw.metadata[k];
      if (typeof v === 'string') metadata[k] = sanitizeMessage(v, 200);
      else if (typeof v === 'number' || typeof v === 'boolean') metadata[k] = v;
      else if (v == null) metadata[k] = null;
    }
  }

  return {
    kind,
    message: sanitizeMessage(raw.message, 500),
    appVersion: raw.appVersion || raw.app_version
      ? String(raw.appVersion || raw.app_version).slice(0, 32)
      : null,
    osName: raw.os || raw.osName || raw.os_name
      ? String(raw.os || raw.osName || raw.os_name).slice(0, 32)
      : null,
    osVersion: raw.osVersion || raw.os_version
      ? String(raw.osVersion || raw.os_version).slice(0, 64)
      : null,
    arch: raw.arch ? String(raw.arch).slice(0, 16) : null,
    cpuCores: Number.isFinite(cpu) && cpu > 0 && cpu < 512 ? Math.round(cpu) : null,
    ramMb: Number.isFinite(ram) && ram > 0 && ram < 2_000_000 ? Math.round(ram) : null,
    locale: raw.locale ? String(raw.locale).slice(0, 16) : null,
    metadata,
  };
}

module.exports = {
  clientIp,
  clientUa,
  sanitizeMessage,
  logUserActivity,
  ALLOWED_TELEMETRY_KINDS,
  normalizeTelemetryEvent,
};
