const crypto = require('crypto');

const PUBLIC_USER_FIELDS = [
  'id',
  'nickname',
  'display_nickname',
  'email',
  'first_name',
  'role',
  'trust_level',
  'status',
  'is_active',
  'is_email_verified',
  'is_banned',
  'registered_at',
  'last_login',
  'age',
  'bio',
  'avatar_url',
  'ban_reason',
  'ban_until',
  'discord_username',
  'total_minutes',
  'skin_url',
  'skin_model',
  'cape_url',
  'nickname_changed_at',
  'display_nickname_changed_at',
  'email_changed_at',
];

function hashSessionToken(token) {
  return crypto.createHash('sha256').update(String(token)).digest('hex');
}

function legacySessionTokenHash(token) {
  return Buffer.from(String(token)).toString('base64');
}

function sessionTokenHashes(token) {
  return [hashSessionToken(token), legacySessionTokenHash(token)];
}

function publicUser(user) {
  const safe = {};
  for (const field of PUBLIC_USER_FIELDS) {
    if (user[field] !== undefined) safe[field] = user[field];
  }
  // Preserve the names consumed by the current Next frontend.
  safe.created_at = user.created_at ?? user.registered_at;
  safe.discord = user.discord ?? user.discord_username;
  safe.display_nickname = user.display_nickname ?? user.nickname;
  return safe;
}

module.exports = {
  hashSessionToken,
  legacySessionTokenHash,
  sessionTokenHashes,
  publicUser,
};
