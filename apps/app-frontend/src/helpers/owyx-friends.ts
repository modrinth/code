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

export type OwyxFriendPresence = 'offline' | 'online' | 'playing'

export type OwyxFriend = {
	id: string
	userId: string
	nickname: string
	displayNickname?: string
	avatarUrl?: string | null
	status: 'pending' | 'accepted'
	incoming: boolean
	createdAt?: string
	updatedAt?: string
	presence?: OwyxFriendPresence
	instanceName?: string | null
	presenceUpdatedAt?: string | null
}

export function owyxFriendLabel(friend: Pick<OwyxFriend, 'nickname' | 'displayNickname'>): string {
	return (friend.displayNickname || friend.nickname || '').trim() || friend.nickname
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

/** Map common API errors to short English (Vue i18n layers can translate later). */
function friendlyFriendsError(raw: string | undefined, status: number, fallback: string): string {
	const msg = (raw || '').toLowerCase()
	if (status === 401 || msg.includes('unauthorized') || msg.includes('no_session')) {
		return 'Sign in to your Owyx account again, then retry.'
	}
	if (status === 403 && msg.includes('not accepting')) {
		return 'This player is not accepting friend requests.'
	}
	if (status === 404 && msg.includes('not found')) {
		return 'User not found. Check the nickname and try again.'
	}
	if (status === 409 && msg.includes('already')) {
		return 'You are already friends or a request is pending.'
	}
	if (!getOwyxClientKey()) {
		return 'Missing client key. Set X-Owyx-Client-Key in Admin → API.'
	}
	return raw || `${fallback} (${status})`
}

export async function listOwyxFriends(): Promise<OwyxFriend[]> {
	const res = await owyxFetch(`${apiBase()}/api/friends`, {
		method: 'GET',
		headers: authHeaders(),
		signal: AbortSignal.timeout(12000),
	})
	const data = (await res.json().catch(() => ({}))) as { friends?: OwyxFriend[]; error?: string }
	if (!res.ok) throw new Error(friendlyFriendsError(data.error, res.status, 'Friends list failed'))
	return Array.isArray(data.friends) ? data.friends : []
}

export async function postOwyxPresence(opts: {
	status: OwyxFriendPresence
	instanceName?: string | null
}): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/friends/presence`, {
		method: 'POST',
		headers: authHeaders(),
		body: JSON.stringify({
			status: opts.status,
			instanceName: opts.instanceName || null,
		}),
		signal: AbortSignal.timeout(8000),
	})
	if (!res.ok) {
		const data = (await res.json().catch(() => ({}))) as { error?: string }
		throw new Error(data.error || `Presence failed (${res.status})`)
	}
}

export async function searchOwyxUsers(
	q: string,
): Promise<{ id: string; nickname: string; displayNickname?: string; avatarUrl?: string | null }[]> {
	const res = await owyxFetch(`${apiBase()}/api/friends/search?q=${encodeURIComponent(q)}`, {
		method: 'GET',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as {
		users?: {
			id: string
			nickname: string
			displayNickname?: string
			avatarUrl?: string | null
		}[]
		error?: string
	}
	if (!res.ok) throw new Error(friendlyFriendsError(data.error, res.status, 'Search failed'))
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
	if (!res.ok || !data.friend)
		throw new Error(friendlyFriendsError(data.error, res.status, 'Request failed'))
	return data.friend
}

export async function acceptOwyxFriend(id: string): Promise<OwyxFriend> {
	const res = await owyxFetch(`${apiBase()}/api/friends/${encodeURIComponent(id)}/accept`, {
		method: 'POST',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as { friend?: OwyxFriend; error?: string }
	if (!res.ok || !data.friend)
		throw new Error(friendlyFriendsError(data.error, res.status, 'Accept failed'))
	return data.friend
}

export async function declineOwyxFriend(id: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/friends/${encodeURIComponent(id)}/decline`, {
		method: 'POST',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	if (!res.ok) {
		const data = (await res.json().catch(() => ({}))) as { error?: string }
		throw new Error(friendlyFriendsError(data.error, res.status, 'Decline failed'))
	}
}

export async function removeOwyxFriend(id: string): Promise<void> {
	const res = await owyxFetch(`${apiBase()}/api/friends/${encodeURIComponent(id)}`, {
		method: 'DELETE',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	if (!res.ok) {
		const data = (await res.json().catch(() => ({}))) as { error?: string }
		throw new Error(friendlyFriendsError(data.error, res.status, 'Remove failed'))
	}
}

export type OwyxSocialSettings = {
	allowFriendRequests: boolean
}

export async function getOwyxSocialSettings(): Promise<OwyxSocialSettings> {
	const res = await owyxFetch(`${apiBase()}/api/friends/settings`, {
		method: 'GET',
		headers: authHeaders(),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as {
		settings?: OwyxSocialSettings
		error?: string
	}
	if (!res.ok) throw new Error(friendlyFriendsError(data.error, res.status, 'Settings failed'))
	return {
		allowFriendRequests: data.settings?.allowFriendRequests !== false,
	}
}

export async function patchOwyxSocialSettings(
	settings: Partial<OwyxSocialSettings>,
): Promise<OwyxSocialSettings> {
	const res = await owyxFetch(`${apiBase()}/api/friends/settings`, {
		method: 'PATCH',
		headers: authHeaders(),
		body: JSON.stringify(settings),
		signal: AbortSignal.timeout(10000),
	})
	const data = (await res.json().catch(() => ({}))) as {
		settings?: OwyxSocialSettings
		error?: string
	}
	if (!res.ok || !data.settings) {
		throw new Error(friendlyFriendsError(data.error, res.status, 'Save settings failed'))
	}
	return {
		allowFriendRequests: data.settings.allowFriendRequests !== false,
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
	const sizeMb = opts.file.size / (1024 * 1024)
	if (sizeMb > 2048) {
		throw new Error(
			`pack is ${sizeMb.toFixed(0)} MB — max upload is 2 GB. Host a larger archive via HTTP URL instead.`,
		)
	}
	const fd = new FormData()
	fd.append('archive', opts.file, opts.fileName)
	const ingestHeaders: Record<string, string> = { Accept: 'application/json' }
	const key = getOwyxClientKey()
	if (key) ingestHeaders['X-Owyx-Client-Key'] = key
	const token = getStoredOwyxSiteSession()?.token
	if (token) ingestHeaders.Authorization = `Bearer ${token}`
	const ingestRes = await owyxFetch(
		`${base}/api/admin/packs/${encodeURIComponent(packId)}/ingest`,
		{
			method: 'POST',
			headers: ingestHeaders,
			body: fd,
			signal: AbortSignal.timeout(Math.max(180000, Math.ceil(sizeMb) * 4000)),
		},
	)
	if (!ingestRes.ok) {
		const data = (await ingestRes.json().catch(() => ({}))) as { error?: string }
		if (ingestRes.status === 413) {
			throw new Error(
				data.error ||
					`ingest failed (413): archive too large for the API (max 2 GB, yours ~${sizeMb.toFixed(0)} MB)`,
			)
		}
		throw new Error(data.error || `Ingest failed (${ingestRes.status})`)
	}
	if (opts.serverId) {
		const bindRes = await owyxFetch(
			`${base}/api/admin/servers/${encodeURIComponent(opts.serverId)}`,
			{
				method: 'PUT',
				headers,
				body: JSON.stringify({ packId }),
				signal: AbortSignal.timeout(15000),
			},
		)
		if (!bindRes.ok) {
			const data = (await bindRes.json().catch(() => ({}))) as { error?: string }
			throw new Error(data.error || `Bind server failed (${bindRes.status})`)
		}
	}
	return { packId }
}

export async function createOwyxCatalogServer(body: {
	name: string
	address: string
	port?: number
	minecraft?: string
	loader?: string
	kind?: string
	packId?: string | null
	description?: string
	requiresAccount?: boolean
	published?: boolean
}): Promise<{ id: string }> {
	const res = await owyxFetch(`${apiBase()}/api/admin/servers`, {
		method: 'POST',
		headers: authHeaders(),
		body: JSON.stringify({
			name: body.name,
			address: body.address,
			port: body.port ?? 25565,
			minecraft: body.minecraft || '1.21.1',
			loader: body.loader || 'vanilla',
			kind: body.kind || 'owyx',
			packId: body.packId || null,
			requiresAccount: body.requiresAccount ?? false,
			published: body.published ?? true,
			accessMode: 'open',
			sortOrder: 0,
			iconUrl: null,
		}),
		signal: AbortSignal.timeout(20000),
	})
	const data = (await res.json().catch(() => ({}))) as {
		server?: { id: string }
		error?: string
	}
	if (!res.ok || !data.server?.id) {
		throw new Error(data.error || `Create server failed (${res.status})`)
	}
	return { id: data.server.id }
}
