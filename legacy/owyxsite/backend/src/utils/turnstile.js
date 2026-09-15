// Cloudflare Turnstile verification

const TURNSTILE_VERIFY_URL = 'https://challenges.cloudflare.com/turnstile/v0/siteverify';

/**
 * Verify Cloudflare Turnstile token.
 * Set TURNSTILE_SECRET_KEY in env. When TURNSTILE_SKIP=true (local/dev), always pass.
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
  if (!secret) {
    if (production) {
      console.error('TURNSTILE_SECRET_KEY is required in production');
      return {
        success: false,
        message: 'Проверка капчи временно недоступна',
        misconfigured: true,
      };
    }
    console.warn('TURNSTILE_SECRET_KEY not set — skipping Turnstile verification in development');
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

module.exports = { verifyTurnstile };
