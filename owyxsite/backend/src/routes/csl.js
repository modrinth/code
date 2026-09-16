/**
 * CustomSkinLoader-compatible public skin API (no auth / no client key needed
 * when reached via owyx.site; also exempted on api.* for in-game clients).
 *
 * Legacy PNG: GET /api/csl/skins/:nickname.png
 * CustomSkinAPI profile: GET /api/csl/:nickname.json
 *
 * Mod: https://github.com/xfl03/MCCustomSkinLoader (GPL-3.0)
 */

const express = require('express');
const db = require('../database/connection');
const {
  absoluteWebsiteAsset,
  resolveLocalUpload,
  allowedAssetHosts,
} = require('./csl-helpers');

const router = express.Router();
const NICK_RE = /^[A-Za-z0-9_]{3,16}$/;

async function findUserByNick(nickname) {
  const result = await db.query(
    `SELECT nickname, skin_url, skin_model, cape_url, is_active, is_banned
     FROM users
     WHERE LOWER(nickname) = LOWER($1)
     LIMIT 1`,
    [nickname]
  );
  return result.rows[0] || null;
}

// GET /api/csl/skins/:nickname.png — Legacy skin texture
router.get('/skins/:nickname.png', async (req, res) => {
  try {
    const raw = String(req.params.nickname || '').replace(/\.png$/i, '');
    if (!NICK_RE.test(raw)) {
      return res.status(400).json({ error: 'invalid nickname' });
    }
    const user = await findUserByNick(raw);
    if (!user || user.is_active === false || user.is_banned) {
      return res.status(404).json({ error: 'skin not found' });
    }
    if (!user.skin_url) {
      return res.status(404).json({ error: 'skin not found' });
    }
    const local = resolveLocalUpload(user.skin_url);
    if (local) {
      res.setHeader('Cache-Control', 'public, max-age=120');
      return res.sendFile(local);
    }
    const abs = absoluteWebsiteAsset(user.skin_url);
    if (!abs) return res.status(404).json({ error: 'skin not found' });
    res.setHeader('Cache-Control', 'public, max-age=120');
    return res.redirect(302, abs);
  } catch (error) {
    console.error('csl skin png:', error);
    res.status(500).json({ error: 'skin lookup failed' });
  }
});

// GET /api/csl/:nickname.json — CustomSkinAPI profile
router.get('/:nickname.json', async (req, res) => {
  try {
    const raw = String(req.params.nickname || '').replace(/\.json$/i, '');
    if (!NICK_RE.test(raw)) {
      return res.status(400).json({ error: 'invalid nickname' });
    }
    const user = await findUserByNick(raw);
    if (!user || user.is_active === false || user.is_banned || !user.skin_url) {
      return res.status(404).json({ error: 'profile not found' });
    }
    const skinAbs = absoluteWebsiteAsset(user.skin_url);
    if (!skinAbs) {
      return res.status(404).json({ error: 'profile not found' });
    }
    const capeAbs = absoluteWebsiteAsset(user.cape_url);
    const model = user.skin_model === 'slim' ? 'slim' : 'default';
    const skins = {};
    skins[model] = skinAbs;
    if (model === 'slim') skins.default = skinAbs;
    else skins.slim = skinAbs;
    res.setHeader('Cache-Control', 'public, max-age=60');
    res.json({
      username: user.nickname,
      skins,
      cape: capeAbs || undefined,
    });
  } catch (error) {
    console.error('csl profile json:', error);
    res.status(500).json({ error: 'profile lookup failed' });
  }
});

module.exports = router;
module.exports.absoluteWebsiteAsset = absoluteWebsiteAsset;
module.exports.resolveLocalUpload = resolveLocalUpload;
module.exports.allowedAssetHosts = allowedAssetHosts;
