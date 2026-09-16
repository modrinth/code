/**
 * Resolve Owyx avatar URLs (absolute for launcher; relative OK on site).
 * Fallback: brand favicon SVG when user has no upload.
 */

import owyxAvatarFallback from '@/assets/owyx-avatar-fallback.svg?url'

import { DEFAULT_OWYX_API_BASE, getStoredOwyxApiBase, sanitizeOwyxApiBase } from '@/helpers/owyx-api'

const SITE_ORIGIN = 'https://owyx.site'

export const OWYX_AVATAR_FALLBACK = owyxAvatarFallback

/** Absolute URL for uploaded avatars, or brand fallback. */
export function resolveOwyxAvatarUrl(avatarUrl?: string | null): string {
	const raw = (avatarUrl ?? '').trim()
	if (!raw) return OWYX_AVATAR_FALLBACK
	if (raw.startsWith('data:') || raw.startsWith('blob:')) return raw
	if (raw.startsWith('https://') || raw.startsWith('http://')) return raw
	if (raw.startsWith('/')) {
		// Prefer website origin for /uploads (nginx). API host also serves /uploads.
		const base = sanitizeOwyxApiBase(getStoredOwyxApiBase() || DEFAULT_OWYX_API_BASE)
		try {
			const apiHost = new URL(base).hostname
			if (apiHost === 'api.owyx.site' || apiHost.endsWith('.owyx.site')) {
				return `${SITE_ORIGIN}${raw}`
			}
			return `${base.replace(/\/$/, '')}${raw}`
		} catch {
			return `${SITE_ORIGIN}${raw}`
		}
	}
	return OWYX_AVATAR_FALLBACK
}
