/**
 * Pure helpers for CustomSkinLoader public skin URLs (no Express).
 */

const path = require('path');
const fs = require('fs');

function websiteBase() {
  const fromEnv = (process.env.SITE_PUBLIC_URL || process.env.PUBLIC_SITE_URL || '').trim();
  if (fromEnv) return fromEnv.replace(/\/+$/, '');
  return 'https://owyx.site';
}

function allowedAssetHosts() {
  const hosts = new Set(['owyx.site', 'www.owyx.site', 'api.owyx.site']);
  for (const envKey of ['SITE_PUBLIC_URL', 'PUBLIC_SITE_URL', 'API_PUBLIC_URL']) {
    const raw = (process.env[envKey] || '').trim();
    if (!raw) continue;
    try {
      hosts.add(new URL(raw).hostname.toLowerCase());
    } catch {
      /* ignore bad env */
    }
  }
  return hosts;
}

/**
 * Resolve a stored skin/cape path to a public absolute URL on an allowlisted host.
 * Rejects open redirects / third-party texture hosts.
 */
function absoluteWebsiteAsset(value) {
  if (!value) return null;
  const allow = allowedAssetHosts();
  const site = websiteBase();

  if (/^https?:\/\//i.test(value)) {
    try {
      const u = new URL(value);
      const host = u.hostname.toLowerCase();
      if (!allow.has(host)) return null;
      if (!u.pathname.startsWith('/uploads/')) return null;
      // Prefer website origin for browser/CSL clients (no API client key).
      if (host === 'api.owyx.site' || host.startsWith('api.')) {
        return `${site}${u.pathname}${u.search}`;
      }
      return `${u.origin}${u.pathname}${u.search}`;
    } catch {
      return null;
    }
  }

  const pathname = value.startsWith('/') ? value : `/${value}`;
  if (!pathname.startsWith('/uploads/')) return null;
  return `${site}${pathname}`;
}

function uploadsRoot() {
  return path.resolve(path.join(__dirname, '../../uploads'));
}

/**
 * Map a stored upload path to a local file under uploads/, with path-traversal guard.
 */
function resolveLocalUpload(skinUrl) {
  if (!skinUrl || !skinUrl.includes('/uploads/')) return null;
  try {
    const pathname = /^https?:\/\//i.test(skinUrl) ? new URL(skinUrl).pathname : skinUrl;
    const rel = pathname.replace(/^\/+/, '');
    if (!rel.startsWith('uploads/skins/') && !rel.startsWith('uploads/capes/')) return null;
    if (rel.includes('..')) return null;
    const root = uploadsRoot();
    const full = path.resolve(path.join(__dirname, '../..', rel));
    if (full !== root && !full.startsWith(root + path.sep)) return null;
    if (!fs.existsSync(full)) return null;
    return full;
  } catch {
    return null;
  }
}

module.exports = {
  websiteBase,
  allowedAssetHosts,
  absoluteWebsiteAsset,
  uploadsRoot,
  resolveLocalUpload,
};
