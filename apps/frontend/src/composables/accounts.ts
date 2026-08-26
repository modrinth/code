import type { Labrinth } from '@modrinth/api-client'
import { nextTick } from 'vue'

import { useAuthCookie } from '@/composables/auth-cookie.ts'
import type { CookieOptions } from '#app'
import { useTheme } from '~/composables/nuxt-accessors.ts'
import { getThemeType } from '~/plugins/theme/themes.ts'

export const LAST_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY = 'auth-last-sign-in-oauth-provider'
export const PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY = 'auth-pending-sign-in-oauth-provider'

export const STORED_ACCOUNT_AUTH_METHODS = [
	'password',
	'passkey',
	'github',
	'discord',
	'microsoft',
	'gitlab',
	'google',
	'steam',
	'paypal',
] as const

export type StoredAccountAuthMethod = (typeof STORED_ACCOUNT_AUTH_METHODS)[number]

export type StoredAccount = {
	id: string
	username: string
	avatarUrl: string | null
	token: string
	role?: Labrinth.Users.v2.Role
	email?: string | null
	authMethod?: StoredAccountAuthMethod
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

export const isStoredAccountAuthMethod = (value: unknown): value is StoredAccountAuthMethod =>
	typeof value === 'string' && (STORED_ACCOUNT_AUTH_METHODS as readonly string[]).includes(value)

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
		email: typeof candidate.email === 'string' ? candidate.email : undefined,
		authMethod: isStoredAccountAuthMethod(candidate.authMethod) ? candidate.authMethod : undefined,
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

const toPreview = ({
	id,
	username,
	avatarUrl,
	role,
	email,
	authMethod,
}: StoredAccount): AccountPreview => ({
	id,
	username,
	avatarUrl,
	role,
	email,
	authMethod,
})

const getErrorStatus = (error: unknown): number | undefined => {
	if (!error || typeof error !== 'object') return undefined
	const typedError = error as { statusCode?: unknown; status?: unknown }
	const status = typedError.statusCode ?? typedError.status
	return typeof status === 'number' ? status : undefined
}

const isAuthFailure = (error: unknown): boolean => {
	const status = getErrorStatus(error)
	return status === 401 || status === 403
}

const readPendingAuthMethod = (): StoredAccountAuthMethod | undefined => {
	if (!import.meta.client) return undefined

	try {
		const raw = window.localStorage.getItem(PENDING_SIGN_IN_OAUTH_PROVIDER_STORAGE_KEY)
		if (!raw) return undefined
		const parsed: unknown = JSON.parse(raw)
		return isStoredAccountAuthMethod(parsed) ? parsed : undefined
	} catch {
		return undefined
	}
}

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
	user: Pick<Labrinth.Users.v2.User, 'id' | 'username' | 'avatar_url' | 'role' | 'email'>,
	token: string,
	options?: { authMethod?: StoredAccountAuthMethod },
) => {
	if (!import.meta.client) return

	const local = readLocal()
	const accounts = local.length > 0 ? local : [...useStoredAccounts().value]
	const existingIndex = accounts.findIndex((stored) => stored.id === user.id)
	const existing = existingIndex === -1 ? undefined : accounts[existingIndex]
	const account: StoredAccount = {
		id: user.id,
		username: user.username,
		avatarUrl: user.avatar_url ?? null,
		token,
		role: isRole(user.role) ? user.role : undefined,
		email: user.email ?? existing?.email ?? null,
		authMethod:
			options && 'authMethod' in options
				? options.authMethod
				: (existing?.authMethod ?? readPendingAuthMethod()),
		appearance: existing?.appearance,
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

const getSwitchRedirectPath = (username: string) => {
	if (!import.meta.client) return `/user/${username}`

	const fullPath = `${window.location.pathname}${window.location.search}${window.location.hash}`
	if (fullPath === '/auth' || fullPath.startsWith('/auth/')) return `/user/${username}`
	return fullPath
}

async function fetchUserWithToken(
	token: string,
): Promise<Labrinth.Users.v2.User | 'invalid' | 'error'> {
	try {
		return (await useBaseFetch(
			'user',
			{
				apiVersion: 3,
				headers: {
					Authorization: token,
				},
			},
			true,
		)) as Labrinth.Users.v2.User
	} catch (error) {
		return isAuthFailure(error) ? 'invalid' : 'error'
	}
}

async function refreshStoredToken(token: string): Promise<string | 'invalid' | 'error'> {
	try {
		const session = (await useBaseFetch(
			'session/refresh',
			{
				method: 'POST',
				headers: {
					Authorization: token,
				},
			},
			true,
		)) as { session?: unknown }

		const nextToken = typeof session.session === 'string' ? session.session : ''
		return nextToken || 'invalid'
	} catch (error) {
		return isAuthFailure(error) ? 'invalid' : 'error'
	}
}

async function prepareStoredAccount(
	account: StoredAccount,
): Promise<'ready' | 'invalid' | 'error'> {
	if (!account.token) return 'invalid'

	const user = await fetchUserWithToken(account.token)
	if (user !== 'invalid' && user !== 'error') {
		rememberStoredAccount(user, account.token, { authMethod: account.authMethod })
		return 'ready'
	}
	if (user === 'error') return 'error'

	const refreshed = await refreshStoredToken(account.token)
	if (refreshed === 'error' || refreshed === 'invalid') return refreshed

	const refreshedUser = await fetchUserWithToken(refreshed)
	if (refreshedUser === 'error' || refreshedUser === 'invalid') return refreshedUser

	rememberStoredAccount(refreshedUser, refreshed, { authMethod: account.authMethod })
	return 'ready'
}

const goToReauthenticate = async (account: StoredAccount) => {
	await navigateTo({
		path: '/auth/reauthenticate',
		query: {
			account: account.id,
			redirect: getSwitchRedirectPath(account.username),
		},
	})
}

export const switchToStoredAccount = async (
	account: StoredAccount,
): Promise<'ready' | 'invalid' | 'error'> => {
	if (!import.meta.client) return 'error'

	hydrateStoredAccounts()
	const stored = readLocal().find((item) => item.id === account.id) ?? account

	if (!stored.token) {
		await goToReauthenticate(stored)
		return 'invalid'
	}

	useIsSwitchingAccount().value = true
	const status = await prepareStoredAccount(stored)
	if (status !== 'ready') {
		useIsSwitchingAccount().value = false
		if (status === 'invalid') await goToReauthenticate(stored)
		return status
	}

	const latest = readLocal().find((item) => item.id === stored.id) ?? stored
	const theme = useTheme()
	if (latest.appearance) {
		theme.applyAccountAppearance(latest.appearance)
		await nextTick()
	}

	useAuthCookie().value = latest.token
	await nextTick()
	window.location.reload()
	return 'ready'
}

export const switchToSignedOut = async () => {
	if (!import.meta.client) return

	useIsSwitchingAccount().value = true
	useAuthCookie().value = null
	await nextTick()
	window.location.reload()
}
