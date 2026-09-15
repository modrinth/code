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
  const xf = req.get('x-forwarded-host');
  if (xf) {
    return xf.split(',')[0].trim().toLowerCase().split(':')[0];
  }
  return String(req.hostname || req.get('host') || '')
    .toLowerCase()
    .split(':')[0];
}

function clientKeyGate(req, res, next) {
  const path = req.path || '';
  if (path === '/health' || path.startsWith('/health/')) {
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
