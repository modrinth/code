// Cloudflare Turnstile verification

const TURNSTILE_VERIFY_URL = 'https://challenges.cloudflare.com/turnstile/v0/siteverify';

function isUsableSecret(secret) {
  if (!secret || typeof secret !== 'string') return false;
  const s = secret.trim();
  if (!s) return false;
  // OBT / scaffold placeholders — never call Cloudflare with these
  if (/^obt[-_]?pend/i.test(s)) return false;
  if (/placeholder|changeme|your[-_]?secret|xxx+/i.test(s)) return false;
  // Cloudflare secrets are typically 32+ chars; keep a soft floor
  return s.length >= 20;
}

/**
 * Verify Cloudflare Turnstile token.
 * Set TURNSTILE_SECRET_KEY in env. When TURNSTILE_SKIP=true (local/dev), always pass.
 * Placeholder secrets (obt-pend…) are treated as "not configured" so register/login work.
 */
async function verifyTurnstile(token, remoteip = null) {
  const production = process.env.NODE_ENV === 'production';
  if (
    process.env.NODE_ENV === 'test' ||
    (process.env.TURNSTILE_SKIP === 'true' && !production)
  ) {
    return { success: true, skipped: true };
  }

  const secret = process.env.TURNSTILE_SECRET_KEY || process.env.TURNSTILE_SECRET;
  if (!isUsableSecret(secret)) {
    if (production && secret && !isUsableSecret(secret)) {
      console.warn(
        'TURNSTILE_SECRET_KEY looks like a placeholder — skipping captcha until real Cloudflare keys are set',
      );
    } else if (!secret && production) {
      console.warn('TURNSTILE_SECRET_KEY not set — skipping Turnstile in production (set real keys to enable)');
    } else if (!secret) {
      console.warn('TURNSTILE_SECRET_KEY not set — skipping Turnstile verification in development');
    }
    return { success: true, skipped: true };
  }

  if (!token) {
    return {
      success: false,
      message: 'Turnstile токен не предоставлен',
    };
  }

  try {
    const formData = new URLSearchParams();
    formData.append('secret', secret);
    formData.append('response', token);
    if (remoteip) formData.append('remoteip', remoteip);

    const response = await fetch(TURNSTILE_VERIFY_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: formData.toString(),
    });

    const data = await response.json();

    if (data.success) {
      return { success: true };
    }

    console.error('Turnstile verification failed:', data);
    return {
      success: false,
      message: 'Проверка капчи не пройдена',
      errorCodes: data['error-codes'],
    };
  } catch (error) {
    console.error('Turnstile verification error:', error);
    return {
      success: false,
      message: 'Ошибка проверки капчи на сервере',
    };
  }
}

module.exports = { verifyTurnstile, isUsableSecret };
