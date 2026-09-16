/**
 * Owyx site account (control-plane email login).
 * Contract: owyxsite/LAUNCHER_SITE_CONTRACT.md — POST /api/auth/login, GET /api/launcher/me
 */

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

const STORAGE_TOKEN = 'owyx.siteToken'
const STORAGE_USER = 'owyx.siteUser'

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

export function getStoredOwyxSiteSession(): OwyxSiteSession | null {
	try {
		const token = localStorage.getItem(STORAGE_TOKEN)
		const raw = localStorage.getItem(STORAGE_USER)
		if (!token || !raw) return null
		const user = JSON.parse(raw) as OwyxSiteUser
		if (!user?.nickname) return null
		return { token, user }
	} catch {
		return null
	}
}

export function clearOwyxSiteSession() {
	try {
		localStorage.removeItem(STORAGE_TOKEN)
		localStorage.removeItem(STORAGE_USER)
	} catch {
		/* ignore */
	}
}

function persistSession(token: string, user: OwyxSiteUser) {
	localStorage.setItem(STORAGE_TOKEN, token)
	localStorage.setItem(STORAGE_USER, JSON.stringify(user))
}

function mapUser(raw: Record<string, unknown>, cosmetics?: Record<string, unknown> | null): OwyxSiteUser {
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
		if (code === 'unauthorized_client' || res.status === 401 && code.includes('unauthorized')) {
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
	// Refresh from /me so avatar/cosmetics are absolute and up to date.
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
		// Top-level avatarUrl mirror (older shape)
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
