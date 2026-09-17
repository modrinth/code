/**
 * Gate direct access to the API host (api.owyx.site).
 *
 * - Browser traffic hits https://owyx.site/api (Host: owyx.site) — no client key.
 * - Launcher hits https://api.owyx.site (Host: api.owyx.site) — requires
 *   header X-Owyx-Client-Key matching LAUNCHER_CLIENT_KEY.
 * - /health stays open for uptime checks.
 */

function parseList(raw, fallback) {
  const src = (raw && String(raw).trim()) || fallback;
  return src
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean);
}

function requestHost(req) {
  // Never trust client-supplied X-Forwarded-Host — spoofing would skip the
  // api.* client-key gate. Prefer Express hostname (respects trust proxy from
  // the real Host / nginx-rewritten forward headers only).
  const hostHeader = String(req.hostname || req.get('host') || '')
    .toLowerCase()
    .split(':')[0]
    .trim();
  return hostHeader;
}

function clientKeyGate(req, res, next) {
  const path = req.path || '';
  if (path === '/health' || path.startsWith('/health/')) {
    return next();
  }
  // Public CustomSkinLoader endpoints — Minecraft clients have no launcher key.
  if (path === '/api/csl' || path.startsWith('/api/csl/')) {
    return next();
  }

  const apiHosts = parseList(process.env.API_HOSTS, 'api.owyx.site');
  const host = requestHost(req);
  const onApiHost = apiHosts.some((h) => host === h);

  if (!onApiHost) {
    return next();
  }

  const expected = (process.env.LAUNCHER_CLIENT_KEY || '').trim();
  if (!expected) {
    if (process.env.NODE_ENV === 'production') {
      return res.status(503).json({
        error: 'launcher_client_key_missing',
        message: 'LAUNCHER_CLIENT_KEY must be set in production for api.*',
      });
    }
    return next();
  }

  const got = (req.get('x-owyx-client-key') || '').trim();
  if (got !== expected) {
    return res.status(401).json({
      error: 'unauthorized_client',
      message: 'Missing or invalid X-Owyx-Client-Key',
    });
  }
  return next();
}

module.exports = { clientKeyGate, requestHost, parseList };
