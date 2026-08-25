import type { Labrinth } from '@modrinth/api-client'
import { nextTick } from 'vue'

import { useAuthCookie } from '@/composables/auth-cookie.ts'
import type { CookieOptions } from '#app'
import { useTheme } from '~/composables/nuxt-accessors.ts'
import { getThemeType } from '~/plugins/theme/themes.ts'

export type StoredAccount = {
	id: string
	username: string
	avatarUrl: string | null
	token: string
	role?: Labrinth.Users.v2.Role
	appearance?: Labrinth.Users.v3.AppearancePreferences
}

type AccountPreview = Omit<StoredAccount, 'token' | 'appearance'>

const STORAGE_KEY = 'auth-stored-accounts'
const COOKIE_OPTIONS = {
	maxAge: 60 * 60 * 24 * 365 * 10,
	sameSite: 'lax',
	httpOnly: false,
	path: '/',
} satisfies CookieOptions<AccountPreview[]>

const ROLES: Labrinth.Users.v2.Role[] = ['developer', 'moderator', 'admin']

const isRole = (value: unknown): value is Labrinth.Users.v2.Role =>
	ROLES.includes(value as Labrinth.Users.v2.Role)

const isAppearance = (value: unknown): value is Labrinth.Users.v3.AppearancePreferences => {
	if (!value || typeof value !== 'object') return false
	const appearance = value as Labrinth.Users.v3.AppearancePreferences
	return (
		typeof appearance.auto === 'boolean' &&
		typeof appearance.theme === 'string' &&
		getThemeType(appearance.theme) !== 'unknown'
	)
}

const parsePreview = (value: unknown): AccountPreview | null => {
	if (!value || typeof value !== 'object') return null
	const candidate = value as Partial<AccountPreview>
	if (typeof candidate.id !== 'string' || typeof candidate.username !== 'string') return null

	return {
		id: candidate.id,
		username: candidate.username,
		avatarUrl: candidate.avatarUrl ?? null,
		role: isRole(candidate.role) ? candidate.role : undefined,
	}
}

const parseAccount = (value: unknown): StoredAccount | null => {
	const preview = parsePreview(value)
	if (!preview || typeof (value as StoredAccount).token !== 'string') return null

	const candidate = value as Partial<StoredAccount>
	return {
		...preview,
		token: candidate.token ?? '',
		appearance: isAppearance(candidate.appearance) ? candidate.appearance : undefined,
	}
}

const parsePreviews = (value: unknown): AccountPreview[] =>
	Array.isArray(value) ? value.map(parsePreview).filter((account) => account !== null) : []

const toPreview = ({ id, username, avatarUrl, role }: StoredAccount): AccountPreview => ({
	id,
	username,
	avatarUrl,
	role,
})

const readLocal = (): StoredAccount[] => {
	if (!import.meta.client) return []

	try {
		const parsed: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '')
		if (!Array.isArray(parsed)) return []
		return parsed.map(parseAccount).filter((account) => account !== null)
	} catch {
		return []
	}
}

const writeLocal = (accounts: StoredAccount[]) => {
	if (!import.meta.client) return
	try {
		window.localStorage.setItem(STORAGE_KEY, JSON.stringify(accounts))
	} catch {
		// storage blocked or full — switching just won't be offered
	}
}

const accountsCookie = () => {
	const config = useRuntimeConfig()

	return useCookie<AccountPreview[]>(STORAGE_KEY, {
		...COOKIE_OPTIONS,
		secure: config.public.cookieSecure,
		default: () => [],
		decode: (value) => {
			try {
				return parsePreviews(JSON.parse(value))
			} catch {
				return []
			}
		},
		encode: (value) => JSON.stringify(parsePreviews(value)),
	})
}

export const useStoredAccounts = () =>
	useState<StoredAccount[]>('stored-accounts', () =>
		parsePreviews(accountsCookie().value).map((preview) => ({ ...preview, token: '' })),
	)

const setAccounts = (accounts: StoredAccount[]) => {
	useStoredAccounts().value = accounts
	writeLocal(accounts)
	accountsCookie().value = accounts.map(toPreview)
}

export const hydrateStoredAccounts = () => {
	if (!import.meta.client) return

	const local = readLocal()
	if (local.length > 0) {
		setAccounts(local)
		return
	}

	accountsCookie().value = useStoredAccounts().value.map(toPreview)
}

export const rememberStoredAccount = (
	user: Pick<Labrinth.Users.v2.User, 'id' | 'username' | 'avatar_url' | 'role'>,
	token: string,
) => {
	if (!import.meta.client) return

	const local = readLocal()
	const accounts = local.length > 0 ? local : [...useStoredAccounts().value]
	const existingIndex = accounts.findIndex((stored) => stored.id === user.id)
	const account: StoredAccount = {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url ?? null,
		token,
		role: isRole(user.role) ? user.role : undefined,
		appearance: accounts[existingIndex]?.appearance,
	}

	if (existingIndex === -1) {
		setAccounts([...accounts, account])
		return
	}

	const next = [...accounts]
	next[existingIndex] = account
	setAccounts(next)
}

export const rememberStoredAccountAppearance = (
	id: string,
	appearance: Labrinth.Users.v3.AppearancePreferences,
) => {
	if (!import.meta.client) return

	const accounts = readLocal()
	const existing = accounts.find((stored) => stored.id === id)
	if (
		!existing ||
		(existing.appearance?.theme === appearance.theme &&
			existing.appearance?.auto === appearance.auto)
	) {
		return
	}

	setAccounts(accounts.map((stored) => (stored.id === id ? { ...stored, appearance } : stored)))
}

export const forgetStoredAccount = (id: string) => {
	if (!import.meta.client) return
	setAccounts(readLocal().filter((stored) => stored.id !== id))
}

export const forgetStoredAccountByToken = (token: string) => {
	if (!import.meta.client) return
	setAccounts(readLocal().filter((stored) => stored.token !== token))
}

export const useIsSwitchingAccount = () => useState<boolean>('switching-account', () => false)

export const switchToStoredAccount = async (account: StoredAccount) => {
	if (!import.meta.client || !account.token) return

	const theme = useTheme()
	if (account.appearance) {
		theme.applyAccountAppearance(account.appearance)
		await nextTick()
	}

	useIsSwitchingAccount().value = true
	useAuthCookie().value = account.token
	await nextTick()
	window.location.reload()
}

export const switchToSignedOut = async () => {
	if (!import.meta.client) return

	useIsSwitchingAccount().value = true
	useAuthCookie().value = null
	await nextTick()
	window.location.reload()
}
