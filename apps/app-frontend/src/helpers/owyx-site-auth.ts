/**
 * Owyx site account (control-plane email login).
 * Contract: owyxsite/LAUNCHER_SITE_CONTRACT.md — POST /api/auth/login, GET /api/launcher/me
 *
 * JWT lives in OS app-data (`~/owyx/site_session.json` via Tauri), not webview localStorage.
 */

import { invoke } from '@tauri-apps/api/core'
import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

import {
	DEFAULT_OWYX_API_BASE,
	getOwyxClientKey,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { syncOwyxCosmeticsToDisk } from '@/helpers/owyx-cosmetics'

/** Prefer Tauri HTTP plugin (no CORS); fall back to browser fetch for vitest/SSR. */
async function owyxFetch(input: string, init?: RequestInit): Promise<Response> {
	try {
		return await tauriFetch(input, init as Parameters<typeof tauriFetch>[1])
	} catch {
		return await fetch(input, init)
	}
}

/** Legacy keys — migrated once into OS storage then cleared. */
const LEGACY_STORAGE_TOKEN = 'owyx.siteToken'
const LEGACY_STORAGE_USER = 'owyx.siteUser'

export type OwyxSiteUser = {
	id: number | string
	nickname: string
	email?: string
	role?: string
	avatarUrl?: string | null
}

export type OwyxSiteSession = {
	token: string
	user: OwyxSiteUser
}

/** In-memory cache; never write JWT to localStorage. */
let memorySession: OwyxSiteSession | null = null
let hydratePromise: Promise<OwyxSiteSession | null> | null = null

function authHeaders(token?: string): Record<string, string> {
	const headers: Record<string, string> = {
		Accept: 'application/json',
		'Content-Type': 'application/json',
	}
	const key = getOwyxClientKey()
	if (key) headers['X-Owyx-Client-Key'] = key
	if (token) headers.Authorization = `Bearer ${token}`
	return headers
}

function apiBase(): string {
	return sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
}

function parseSessionPayload(raw: string): OwyxSiteSession | null {
	try {
		const data = JSON.parse(raw) as { token?: string; user?: OwyxSiteUser }
		if (!data?.token || !data?.user?.nickname) return null
		return { token: String(data.token), user: data.user }
	} catch {
		return null
	}
}

function readLegacyLocalStorage(): OwyxSiteSession | null {
	try {
		const token = localStorage.getItem(LEGACY_STORAGE_TOKEN)
		const raw = localStorage.getItem(LEGACY_STORAGE_USER)
		if (!token || !raw) return null
		const user = JSON.parse(raw) as OwyxSiteUser
		if (!user?.nickname) return null
		return { token, user }
	} catch {
		return null
	}
}

function clearLegacyLocalStorage() {
	try {
		localStorage.removeItem(LEGACY_STORAGE_TOKEN)
		localStorage.removeItem(LEGACY_STORAGE_USER)
	} catch {
		/* ignore */
	}
}

async function writeOsSession(session: OwyxSiteSession): Promise<void> {
	try {
		await invoke('plugin:utils|owyx_site_session_set', {
			payload: JSON.stringify({ token: session.token, user: session.user }),
		})
	} catch (err) {
		console.warn('Failed to persist Owyx site session to OS storage', err)
	}
}

async function clearOsSession(): Promise<void> {
	try {
		await invoke('plugin:utils|owyx_site_session_clear')
	} catch {
		/* ignore */
	}
}

/** Sync peek of in-memory session (call {@link hydrateOwyxSiteSession} at startup). */
export function getStoredOwyxSiteSession(): OwyxSiteSession | null {
	return memorySession
}

/** Load JWT from OS app-data; migrate legacy localStorage once. */
export async function hydrateOwyxSiteSession(): Promise<OwyxSiteSession | null> {
	if (!hydratePromise) {
		hydratePromise = (async () => {
			try {
				const raw = await invoke<string | null>('plugin:utils|owyx_site_session_get')
				if (raw) {
					const parsed = parseSessionPayload(raw)
					if (parsed) {
						memorySession = parsed
						clearLegacyLocalStorage()
						return memorySession
					}
				}
			} catch {
				/* Tauri unavailable (tests) — fall through */
			}

			const legacy = readLegacyLocalStorage()
			if (legacy) {
				memorySession = legacy
				clearLegacyLocalStorage()
				await writeOsSession(legacy)
				return memorySession
			}

			memorySession = null
			return null
		})()
	}
	return hydratePromise
}

export function clearOwyxSiteSession() {
	memorySession = null
	hydratePromise = Promise.resolve(null)
	clearLegacyLocalStorage()
	void clearOsSession()
}

function persistSession(token: string, user: OwyxSiteUser) {
	memorySession = { token, user }
	hydratePromise = Promise.resolve(memorySession)
	void writeOsSession(memorySession)
}

function mapUser(
	raw: Record<string, unknown>,
	cosmetics?: Record<string, unknown> | null,
): OwyxSiteUser {
	const avatarRaw = raw.avatarUrl
		? String(raw.avatarUrl)
		: raw.avatar_url
			? String(raw.avatar_url)
			: cosmetics?.avatarUrl
				? String(cosmetics.avatarUrl)
				: cosmetics?.avatar_url
					? String(cosmetics.avatar_url)
					: null
	let avatarUrl: string | null = avatarRaw
	if (avatarRaw?.startsWith('/')) {
		avatarUrl = `https://owyx.site${avatarRaw}`
	}
	return {
		id: (raw.id as number | string) ?? 0,
		nickname: String(raw.nickname ?? raw.username ?? raw.email ?? 'Owyx'),
		email: raw.email ? String(raw.email) : undefined,
		role: raw.role ? String(raw.role) : undefined,
		avatarUrl,
	}
}

export async function loginOwyxSite(login: string, password: string): Promise<OwyxSiteSession> {
	const base = apiBase()
	const key = getOwyxClientKey()
	if (!key.trim()) {
		throw new Error(
			'Launcher is missing X-Owyx-Client-Key. Reinstall from a current GitHub release or enable Developer mode to set the key.',
		)
	}
	const res = await owyxFetch(`${base.replace(/\/$/, '')}/api/auth/login`, {
		method: 'POST',
		headers: authHeaders(),
		body: JSON.stringify({ login: login.trim(), password, remember: true }),
		signal: AbortSignal.timeout(12000),
	})
	const data = (await res.json().catch(() => ({}))) as Record<string, unknown>
	if (!res.ok || !data.token) {
		const code = String(data.error ?? '')
		if (code === 'unauthorized_client' || (res.status === 401 && code.includes('unauthorized'))) {
			throw new Error(
				'Invalid or missing client key (unauthorized_client). Set X-Owyx-Client-Key under Owyx Servers.',
			)
		}
		const err = String(data.error ?? data.message ?? `Login failed (${res.status})`)
		throw new Error(err)
	}
	const userRaw = (data.user && typeof data.user === 'object' ? data.user : {}) as Record<
		string,
		unknown
	>
	const user = mapUser(userRaw)
	const token = String(data.token)
	persistSession(token, user)
	const refreshed = await fetchOwyxSiteMe(token)
	return refreshed ?? { token, user }
}

export async function fetchOwyxSiteMe(token?: string): Promise<OwyxSiteSession | null> {
	const session = token
		? { token, user: getStoredOwyxSiteSession()?.user ?? { id: 0, nickname: 'Owyx' } }
		: getStoredOwyxSiteSession()
	if (!session?.token) return null

	const base = apiBase()
	try {
		const res = await owyxFetch(`${base.replace(/\/$/, '')}/api/launcher/me`, {
			method: 'GET',
			headers: authHeaders(session.token),
			signal: AbortSignal.timeout(10000),
		})
		if (res.status === 401 || res.status === 403) {
			clearOwyxSiteSession()
			return null
		}
		if (!res.ok) return session
		const data = (await res.json()) as Record<string, unknown>
		const userRaw = (data.user && typeof data.user === 'object' ? data.user : data) as Record<
			string,
			unknown
		>
		const cosmetics =
			data.cosmetics && typeof data.cosmetics === 'object'
				? (data.cosmetics as Record<string, unknown>)
				: null
		if (!userRaw.avatarUrl && data.avatarUrl) userRaw.avatarUrl = data.avatarUrl
		const user = mapUser(userRaw, cosmetics)
		persistSession(session.token, user)
		void syncOwyxCosmeticsToDisk(user.nickname, cosmetics).catch(() => undefined)
		return { token: session.token, user }
	} catch {
		return session
	}
}

export async function logoutOwyxSite() {
	const session = getStoredOwyxSiteSession()
	if (session?.token) {
		try {
			const base = apiBase()
			await owyxFetch(`${base.replace(/\/$/, '')}/api/auth/logout`, {
				method: 'POST',
				headers: authHeaders(session.token),
				signal: AbortSignal.timeout(5000),
			}).catch(() => undefined)
		} catch {
			/* ignore */
		}
	}
	clearOwyxSiteSession()
}

export const OWYX_SITE_REGISTER_URL = 'https://owyx.site/register'
export const OWYX_SITE_LOGIN_URL = 'https://owyx.site/login'
export const OWYX_SITE_PROFILE_URL = 'https://owyx.site/profile'
export const OWYX_SITE_SUPPORT_URL = 'https://owyx.site'
export const OWYX_SITE_CHANGELOG_URL = 'https://github.com/ebluffy/Owyx/releases'
