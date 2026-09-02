import type { CookieOptions } from '#app'

const AUTH_COOKIE_NAME = 'auth-token'

const AUTH_COOKIE_OPTIONS = {
	maxAge: 60 * 60 * 24 * 365 * 10,
	sameSite: 'lax',
	httpOnly: false,
	path: '/',
} satisfies CookieOptions<string | null>

export const useAuthCookie = () => {
	const config = useRuntimeConfig()

	return useCookie<string | null>(AUTH_COOKIE_NAME, {
		...AUTH_COOKIE_OPTIONS,
		secure: config.public.cookieSecure,
	})
}
