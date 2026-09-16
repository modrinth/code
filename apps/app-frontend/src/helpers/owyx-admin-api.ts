/**
 * Owyx control-plane admin API helpers for the in-launcher Admin panel.
 */

import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

import {
	DEFAULT_OWYX_API_BASE,
	getOwyxClientKey,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { getStoredOwyxSiteSession } from '@/helpers/owyx-site-auth'

async function owyxFetch(input: string, init?: RequestInit): Promise<Response> {
	try {
		return await tauriFetch(input, init as Parameters<typeof tauriFetch>[1])
	} catch {
		return await fetch(input, init)
	}
}

function apiBase(): string {
	return sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
}

export function owyxAdminAuthHeaders(): Record<string, string> {
	const headers: Record<string, string> = {
		Accept: 'application/json',
		'Content-Type': 'application/json',
	}
	const key = getOwyxClientKey()
	if (key) headers['X-Owyx-Client-Key'] = key
	const token = getStoredOwyxSiteSession()?.token
	if (token) headers.Authorization = `Bearer ${token}`
	return headers
}

async function readJson<T extends Record<string, unknown>>(
	res: Response,
): Promise<T & { error?: string }> {
	return (await res.json().catch(() => ({}))) as T & { error?: string }
}

export type AdminServer = {
	id: string
	name: string
	address: string
	port: number
	kind?: string
	packId?: string | null
	minecraft?: string | null
	loader?: string | null
	requiresAccount?: boolean
	published?: boolean
	sortOrder?: number
	iconUrl?: string | null
	accessMode?: string
}

export type AdminPack = {
	id: string
	name: string
	minecraft: string
	loader: string
	description?: string
	published?: boolean
	sourceType?: string
	iconUrl?: string | null
	accessMode?: string
}

export type AdminUser = {
	id: number
	nickname?: string
	email: string
	role: string
	status?: string
	is_banned?: boolean
}

export type AdminNews = {
	id: number | string
	title: string
	tag?: string
	summary?: string
	published?: boolean
	created_at?: string
	updated_at?: string
}

export async function adminListServers(): Promise<AdminServer[]> {
	const res = await owyxFetch(`${apiBase()}/api/admin/servers`, {
		method: 'GET',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(20000),
	})
	const data = await readJson<{ servers?: AdminServer[] }>(res)
	if (!res.ok) throw new Error(data.error || `List servers failed (${res.status})`)
	return Array.isArray(data.servers) ? data.servers : []
}

export async function adminListPacks(): Promise<AdminPack[]> {
	const res = await owyxFetch(`${apiBase()}/api/admin/packs`, {
		method: 'GET',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(20000),
	})
	const data = await readJson<{ packs?: AdminPack[] }>(res)
	if (!res.ok) throw new Error(data.error || `List packs failed (${res.status})`)
	return Array.isArray(data.packs) ? data.packs : []
}

export async function adminDeleteServer(id: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/servers/${encodeURIComponent(id)}`, {
		method: 'DELETE',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Delete server failed (${res.status})`)
	}
}

export async function adminDeletePack(id: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/packs/${encodeURIComponent(id)}`, {
		method: 'DELETE',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Delete pack failed (${res.status})`)
	}
}

export async function adminSetServerPublished(id: string, published: boolean): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/servers/${encodeURIComponent(id)}`, {
		method: 'PUT',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify({ published }),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Update server failed (${res.status})`)
	}
}

export async function adminSetPackPublished(id: string, published: boolean): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/packs/${encodeURIComponent(id)}`, {
		method: 'PUT',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify({ published }),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Update pack failed (${res.status})`)
	}
}

export async function adminCreatePack(body: {
	name: string
	minecraft: string
	loader: string
	description?: string
	sourceType?: string
	sourceConfig?: Record<string, unknown>
	published?: boolean
}): Promise<AdminPack> {
	const res = await owyxFetch(`${apiBase()}/api/admin/packs`, {
		method: 'POST',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify({
			name: body.name,
			minecraft: body.minecraft,
			loader: body.loader,
			description: body.description || '',
			sourceType: body.sourceType || 'http_zip',
			sourceConfig: body.sourceConfig || {},
			published: body.published ?? true,
			accessMode: 'open',
		}),
		signal: AbortSignal.timeout(20000),
	})
	const data = await readJson<{ pack?: AdminPack }>(res)
	if (!res.ok || !data.pack) throw new Error(data.error || `Create pack failed (${res.status})`)
	return data.pack
}

export async function adminListUsers(search = ''): Promise<AdminUser[]> {
	const q = search.trim()
		? `?search=${encodeURIComponent(search.trim())}&limit=100`
		: '?limit=100'
	const res = await owyxFetch(`${apiBase()}/api/admin/users${q}`, {
		method: 'GET',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(20000),
	})
	const data = await readJson<{ users?: AdminUser[] }>(res)
	if (!res.ok) throw new Error(data.error || `List users failed (${res.status})`)
	return Array.isArray(data.users) ? data.users : []
}

export async function adminSetUserRole(id: number, role: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/users/${id}/role`, {
		method: 'PUT',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify({ role }),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Set role failed (${res.status})`)
	}
}

export async function adminBanUser(id: number, ban: boolean): Promise<void> {
	const res = await owyxFetch(
		`${apiBase()}/api/admin/users/${id}/${ban ? 'ban' : 'unban'}`,
		{
			method: 'POST',
			headers: owyxAdminAuthHeaders(),
			body: ban ? JSON.stringify({ reason: 'Banned from launcher admin' }) : undefined,
			signal: AbortSignal.timeout(15000),
		},
	)
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Ban update failed (${res.status})`)
	}
}

export async function adminListNews(): Promise<AdminNews[]> {
	const res = await owyxFetch(`${apiBase()}/api/admin/news`, {
		method: 'GET',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(15000),
	})
	const data = await readJson<{ news?: AdminNews[] }>(res)
	if (!res.ok) throw new Error(data.error || `List news failed (${res.status})`)
	return Array.isArray(data.news) ? data.news : []
}

export async function adminCreateNews(body: {
	title: string
	tag?: string
	summary?: string
	published?: boolean
}): Promise<AdminNews> {
	const res = await owyxFetch(`${apiBase()}/api/admin/news`, {
		method: 'POST',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify(body),
		signal: AbortSignal.timeout(15000),
	})
	const data = await readJson<{ news?: AdminNews }>(res)
	if (!res.ok || !data.news) throw new Error(data.error || `Create news failed (${res.status})`)
	return data.news
}

export async function adminDeleteNews(id: number | string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/news/${encodeURIComponent(String(id))}`, {
		method: 'DELETE',
		headers: owyxAdminAuthHeaders(),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Delete news failed (${res.status})`)
	}
}

export async function adminSetNewsPublished(
	id: number | string,
	published: boolean,
): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/admin/news/${encodeURIComponent(String(id))}`, {
		method: 'PUT',
		headers: owyxAdminAuthHeaders(),
		body: JSON.stringify({ published }),
		signal: AbortSignal.timeout(15000),
	})
	if (!res.ok) {
		const data = await readJson(res)
		throw new Error(data.error || `Update news failed (${res.status})`)
	}
}
