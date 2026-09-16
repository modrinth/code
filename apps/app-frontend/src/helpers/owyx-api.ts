/**
 * Owyx control-plane catalog client.
 * Base: https://api.owyx.site (override via settings).
 * Header: X-Owyx-Client-Key (placeholder only in git — never commit real secrets).
 */

import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

export const DEFAULT_OWYX_API_BASE = 'https://api.owyx.site'
export const LOCAL_OWYX_API_FALLBACK = 'http://127.0.0.1:3001'

const STORAGE_API = 'owyx.apiBaseUrl'
const STORAGE_KEY = 'owyx.clientKey'
const STORAGE_DEMO = 'owyx.demoServers'
const STORAGE_LOCAL_FALLBACK = 'owyx.allowLocalApiFallback'

export type OwyxServerEntry = {
	id: string
	name: string
	description: string
	mcVersion?: string
	loader?: string
	address: string
	iconUrl?: string
	/** HTTPS pack download URL (from pack.downloadUrl / packUrl). */
	packUrl?: string
	packId?: string
	requiresAccount?: boolean
	demo?: boolean
}

export type OwyxCatalogResult = {
	servers: OwyxServerEntry[]
	fromFallback: boolean
}

/** Hardcoded OBT smoke seed — only when demo flag is on and API is down. */
const DEMO_SERVER: OwyxServerEntry = {
	id: 'demo-obt',
	name: 'Owyx OBT (demo)',
	description: 'Offline-mode smoke server for launcher testing. Not Modrinth Hosting.',
	mcVersion: '1.20.1',
	address: '45.131.186.146:1488',
	demo: true,
}

/** Only http(s) API bases — never file:/javascript:/etc. http: only for loopback. */
export function sanitizeOwyxApiBase(url: string | null | undefined): string {
	const raw = (url ?? '').trim()
	if (!raw) return DEFAULT_OWYX_API_BASE
	try {
		const parsed = new URL(raw)
		if (parsed.protocol === 'https:') {
			return parsed.origin + (parsed.pathname === '/' ? '' : parsed.pathname.replace(/\/$/, ''))
		}
		if (
			parsed.protocol === 'http:' &&
			(parsed.hostname === '127.0.0.1' || parsed.hostname === 'localhost')
		) {
			return parsed.origin + (parsed.pathname === '/' ? '' : parsed.pathname.replace(/\/$/, ''))
		}
		return DEFAULT_OWYX_API_BASE
	} catch {
		return DEFAULT_OWYX_API_BASE
	}
}

/** Safe https URLs for pack download / icons. Relative paths OK; protocol-relative `//` is not. */
export function isSafeExternalHttpsUrl(url: string | null | undefined): boolean {
	if (!url) return false
	const trimmed = url.trim()
	if (trimmed.startsWith('//')) return false
	if (trimmed.startsWith('/')) return true
	try {
		const parsed = new URL(trimmed)
		return parsed.protocol === 'https:'
	} catch {
		return false
	}
}

export function getStoredOwyxApiBase(): string {
	try {
		return sanitizeOwyxApiBase(localStorage.getItem(STORAGE_API))
	} catch {
		return DEFAULT_OWYX_API_BASE
	}
}

export function setStoredOwyxApiBase(url: string) {
	localStorage.setItem(STORAGE_API, sanitizeOwyxApiBase(url))
}

export function getOwyxClientKey(): string {
	try {
		return localStorage.getItem(STORAGE_KEY) || import.meta.env.VITE_OWYX_CLIENT_KEY || ''
	} catch {
		return import.meta.env.VITE_OWYX_CLIENT_KEY || ''
	}
}

export function setOwyxClientKey(key: string) {
	localStorage.setItem(STORAGE_KEY, key)
}

/** Demo seed is opt-in (default off for release-ish builds). */
export function getOwyxDemoFlag(): boolean {
	try {
		const v = localStorage.getItem(STORAGE_DEMO)
		if (v === null) return false
		return v === '1' || v === 'true'
	} catch {
		return false
	}
}

export function setOwyxDemoFlag(on: boolean) {
	localStorage.setItem(STORAGE_DEMO, on ? '1' : '0')
}

/** Localhost :3001 fallback is opt-in (dev only) — never default in release. */
export function getOwyxLocalApiFallback(): boolean {
	try {
		return localStorage.getItem(STORAGE_LOCAL_FALLBACK) === '1'
	} catch {
		return false
	}
}

export function setOwyxLocalApiFallback(on: boolean) {
	localStorage.setItem(STORAGE_LOCAL_FALLBACK, on ? '1' : '0')
}

function sanitizeMediaUrl(url: string | undefined, apiBase?: string): string | undefined {
	if (!url) return undefined
	const trimmed = url.trim()
	if (!isSafeExternalHttpsUrl(trimmed)) return undefined
	if (trimmed.startsWith('/')) {
		const base = sanitizeOwyxApiBase(apiBase || getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
		return `${base.replace(/\/$/, '')}${trimmed}`
	}
	return trimmed
}

/** Resolve relative pack/media URLs to absolute https for window.open / downloads. */
export function resolveOwyxPackUrl(url: string | null | undefined, apiBase?: string): string | null {
	const resolved = sanitizeMediaUrl(url ?? undefined, apiBase)
	if (!resolved) return null
	if (resolved.startsWith('https://') || resolved.startsWith('http://127.0.0.1') || resolved.startsWith('http://localhost')) {
		return resolved
	}
	return null
}

function formatAddress(raw: Record<string, unknown>): string {
	const base = String(raw.address ?? raw.playAddress ?? raw.play_address ?? raw.host ?? '').trim()
	if (!base) return ''
	// Contract may send address + separate port; avoid double-appending if host:port already.
	if (raw.port != null && raw.port !== '' && !base.includes(':')) {
		return `${base}:${raw.port}`
	}
	return base
}

function packDownloadUrl(raw: Record<string, unknown>): string | undefined {
	const nested = raw.pack && typeof raw.pack === 'object' ? (raw.pack as Record<string, unknown>) : null
	const candidates = [
		raw.packUrl,
		raw.pack_url,
		raw.downloadUrl,
		raw.download_url,
		nested?.downloadUrl,
		nested?.download_url,
		nested?.url,
		typeof raw.pack === 'string' ? raw.pack : undefined,
	]
	for (const c of candidates) {
		if (c) return String(c)
	}
	return undefined
}

function normalizeEntry(
	raw: Record<string, unknown>,
	index: number,
	apiBase?: string,
): OwyxServerEntry | null {
	const name = String(raw.name ?? raw.title ?? '').trim()
	const address = formatAddress(raw)
	if (!name || !address) return null
	const nested = raw.pack && typeof raw.pack === 'object' ? (raw.pack as Record<string, unknown>) : null
	const packRaw = packDownloadUrl(raw)
	const iconRaw = raw.iconUrl
		? String(raw.iconUrl)
		: raw.icon_url
			? String(raw.icon_url)
			: raw.icon
				? String(raw.icon)
				: nested?.iconUrl
					? String(nested.iconUrl)
					: undefined
	const description = String(
		raw.description ?? raw.desc ?? nested?.description ?? '',
	)
	return {
		id: String(raw.id ?? raw.slug ?? `server-${index}`),
		name,
		description,
		mcVersion: raw.minecraft
			? String(raw.minecraft)
			: raw.mcVersion
				? String(raw.mcVersion)
				: raw.mc_version
					? String(raw.mc_version)
					: raw.version
						? String(raw.version)
						: nested?.minecraft
							? String(nested.minecraft)
							: undefined,
		loader: raw.loader
			? String(raw.loader)
			: nested?.loader
				? String(nested.loader)
				: undefined,
		address,
		iconUrl: sanitizeMediaUrl(iconRaw, apiBase),
		packUrl: sanitizeMediaUrl(packRaw, apiBase),
		packId: raw.packId
			? String(raw.packId)
			: raw.pack_id
				? String(raw.pack_id)
				: nested?.id
					? String(nested.id)
					: undefined,
		requiresAccount: Boolean(raw.requiresAccount ?? raw.requires_account),
	}
}

function parseCatalog(data: unknown, apiBase?: string): OwyxServerEntry[] {
	if (!data || typeof data !== 'object') return []
	const root = data as Record<string, unknown>
	const list = Array.isArray(root)
		? root
		: Array.isArray(root.servers)
			? root.servers
			: Array.isArray(root.packs)
				? root.packs
				: Array.isArray(root.data)
					? root.data
					: []

	const out: OwyxServerEntry[] = []
	list.forEach((item, i) => {
		if (item && typeof item === 'object') {
			const entry = normalizeEntry(item as Record<string, unknown>, i, apiBase)
			if (entry) out.push(entry)
		}
	})

	if (out.length === 0 && Array.isArray(root.packs)) {
		root.packs.forEach((pack, i) => {
			if (!pack || typeof pack !== 'object') return
			const p = pack as Record<string, unknown>
			const nested = Array.isArray(p.servers) ? p.servers : [p]
			nested.forEach((item, j) => {
				if (item && typeof item === 'object') {
					const entry = normalizeEntry(item as Record<string, unknown>, i * 100 + j, apiBase)
					if (entry) {
						if (!entry.packUrl && p.url) {
							entry.packUrl = sanitizeMediaUrl(String(p.url), apiBase)
						}
						out.push(entry)
					}
				}
			})
		})
	}

	return out
}

export async function fetchOwyxCatalog(opts: {
	baseUrl: string
	clientKey?: string
	/** Optional JWT so ACL whitelist/blacklist can apply. */
	authToken?: string | null
	demoFallback?: boolean
	/** Explicit opt-in for http://127.0.0.1:3001 after primary base fails */
	allowLocalFallback?: boolean
}): Promise<OwyxCatalogResult> {
	const primary = sanitizeOwyxApiBase(opts.baseUrl)
	const bases = [primary]
	if (opts.allowLocalFallback && primary !== LOCAL_OWYX_API_FALLBACK) {
		bases.push(LOCAL_OWYX_API_FALLBACK)
	}

	// Paths match owyxsite/LAUNCHER_SITE_CONTRACT.md (primary first).
	const paths = ['/api/launcher/v1/servers', '/api/launcher/servers', '/v1/launcher/catalog']

	for (const base of bases) {
		const headers: Record<string, string> = {
			Accept: 'application/json',
		}
		// Attach client key only for the configured primary base (not localhost fallback)
		if (opts.clientKey && base === primary) {
			headers['X-Owyx-Client-Key'] = opts.clientKey
		}
		if (opts.authToken) {
			headers.Authorization = `Bearer ${opts.authToken}`
		}

		for (const path of paths) {
			try {
				let res: Response
				try {
					res = await tauriFetch(`${base.replace(/\/$/, '')}${path}`, {
						method: 'GET',
						headers,
						signal: AbortSignal.timeout(8000),
					})
				} catch {
					res = await fetch(`${base.replace(/\/$/, '')}${path}`, {
						method: 'GET',
						headers,
						signal: AbortSignal.timeout(8000),
					})
				}
				if (!res.ok) continue
				const data = await res.json()
				const servers = parseCatalog(data, base)
				if (servers.length > 0 || path === '/api/launcher/v1/servers') {
					return { servers, fromFallback: false }
				}
			} catch {
				// try next path / base
			}
		}
	}

	if (opts.demoFallback === true) {
		return { servers: [DEMO_SERVER], fromFallback: true }
	}
	return { servers: [], fromFallback: true }
}
