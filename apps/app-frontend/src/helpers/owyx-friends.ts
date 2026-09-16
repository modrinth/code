/**
 * Owyx control-plane friends API (/api/friends).
 */

import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

import {
	DEFAULT_OWYX_API_BASE,
	getOwyxClientKey,
	getStoredOwyxApiBase,
	sanitizeOwyxApiBase,
} from '@/helpers/owyx-api'
import { getStoredOwyxSiteSession } from '@/helpers/owyx-site-auth'

export type OwyxFriend = {
	id: string
	userId: string
	nickname: string
	avatarUrl?: string | null
	status: 'pending' | 'accepted'
	incoming: boolean
	createdAt?: string
	updatedAt?: string
}

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

function authHeaders(): Record<string, string> {
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

export async function listOwyxFriends(): Promise<OwyxFriend[]> {
	const res = await owyxFetch(`${apiBase()}/api/friends`, {
		method: 'GET',
		headers: authHeaders(),
		signal: AbortSignal.timeout(12000),
	})
	const data = (await res.json().catch(() => ({}))) as { friends?: OwyxFriend[]; error?: string }
	if (!res.ok) throw new Error(data.error || `Friends list failed (${res.status})`)
	return Array.isArray(data.friends) ? data.friends : []
}

export async function searchOwyxUsers(q: string): Promise<{ id: string; nickname: string; avatarUrl?: string | null }[]> {
	const res = await owyxFetch(`${apiBase()}/api/friends/search?q=${encodeURIComponent(q)}`, {
		method: 'GET',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as {
		users?: { id: string; nickname: string; avatarUrl?: string | null }[]
		error?: string
	}
	if (!res.ok) throw new Error(data.error || `Search failed (${res.status})`)
	return Array.isArray(data.users) ? data.users : []
}

export async function requestOwyxFriend(nickname: string): Promise<OwyxFriend> {
	const res = await owyxFetch(`${apiBase()}/api/friends/request`, {
		method: 'POST',
		headers: authHeaders(),
		body: JSON.stringify({ nickname }),
		signal: AbortSignal.timeout(12000),
	})
	const data = (await res.json().catch(() => ({}))) as { friend?: OwyxFriend; error?: string }
	if (!res.ok || !data.friend) throw new Error(data.error || `Request failed (${res.status})`)
	return data.friend
}

export async function acceptOwyxFriend(id: string): Promise<OwyxFriend> {
	const res = await owyxFetch(`${apiBase()}/api/friends/${encodeURIComponent(id)}/accept`, {
		method: 'POST',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as { friend?: OwyxFriend; error?: string }
	if (!res.ok || !data.friend) throw new Error(data.error || `Accept failed (${res.status})`)
	return data.friend
}

export async function removeOwyxFriend(id: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/friends/${encodeURIComponent(id)}`, {
		method: 'DELETE',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	if (!res.ok) {
		const data = (await res.json().catch(() => ({}))) as { error?: string }
		throw new Error(data.error || `Remove failed (${res.status})`)
	}
}

export async function publishLibraryPackToCatalog(opts: {
	name: string
	minecraft: string
	loader: string
	description?: string
	file: Blob
	fileName: string
	serverId?: string | null
}): Promise<{ packId: string }> {
	const base = apiBase()
	const headers = authHeaders()
	const createRes = await owyxFetch(`${base}/api/admin/packs`, {
		method: 'POST',
		headers,
		body: JSON.stringify({
			name: opts.name,
			minecraft: opts.minecraft,
			loader: opts.loader,
			description: opts.description || '',
			sourceType: 'local_ingest',
			sourceConfig: {},
			published: true,
			accessMode: 'open',
		}),
		signal: AbortSignal.timeout(30000),
	})
	const createData = (await createRes.json().catch(() => ({}))) as {
		pack?: { id: string }
		error?: string
	}
	if (!createRes.ok || !createData.pack?.id) {
		throw new Error(createData.error || `Create pack failed (${createRes.status})`)
	}
	const packId = createData.pack.id
	const fd = new FormData()
	fd.append('archive', opts.file, opts.fileName)
	const ingestHeaders: Record<string, string> = { Accept: 'application/json' }
	const key = getOwyxClientKey()
	if (key) ingestHeaders['X-Owyx-Client-Key'] = key
	const token = getStoredOwyxSiteSession()?.token
	if (token) ingestHeaders.Authorization = `Bearer ${token}`
	const ingestRes = await owyxFetch(`${base}/api/admin/packs/${encodeURIComponent(packId)}/ingest`, {
		method: 'POST',
		headers: ingestHeaders,
		body: fd,
		signal: AbortSignal.timeout(120000),
	})
	if (!ingestRes.ok) {
		const data = (await ingestRes.json().catch(() => ({}))) as { error?: string }
		throw new Error(data.error || `Ingest failed (${ingestRes.status})`)
	}
	if (opts.serverId) {
		const bindRes = await owyxFetch(`${base}/api/admin/servers/${encodeURIComponent(opts.serverId)}`, {
			method: 'PUT',
			headers,
			body: JSON.stringify({ packId }),
			signal: AbortSignal.timeout(15000),
		})
		if (!bindRes.ok) {
			const data = (await bindRes.json().catch(() => ({}))) as { error?: string }
			throw new Error(data.error || `Bind server failed (${bindRes.status})`)
		}
	}
	return { packId }
}
