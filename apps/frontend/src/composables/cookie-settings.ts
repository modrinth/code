import { recordSettings, type SettingsStore } from '@modrinth/ui'

import type { CookieOptions } from '#app'

/** Settings cookies last ten years and are readable by the page, which applies them. */
export const getSettingsCookieOptions = <T>() =>
	({
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: useRuntimeConfig().public.cookieSecure,
		httpOnly: false,
		path: '/',
	}) satisfies CookieOptions<T>

/**
 * Settings kept in a cookie as a record of values by setting id, so server rendered pages use
 * them too. Every caller with the same cookie name shares the same values.
 */
export function useCookieSettings(cookieName: string): SettingsStore {
	const cookie = useCookie<Record<string, unknown> | null>(cookieName, getSettingsCookieOptions())
	const values = useState<Record<string, unknown>>(`cookie-settings:${cookieName}`, () => ({
		...cookie.value,
	}))

	return recordSettings(values, () => (cookie.value = { ...values.value }))
}
