import type { IconCacheContext } from '@modrinth/ui'

import { cacheRemoteIcon } from '@/platform/app-lib/icons/commands'

export function createIconCache(): IconCacheContext {
	const pending = new Map<string, Promise<string>>()
	const resolvedUrls = new Map<string, string>()
	return {
		cacheIcon(source) {
			const existing = pending.get(source)
			if (existing) return existing
			const request = cacheRemoteIcon(source)
				.then((url) => {
					resolvedUrls.set(source, url)
					return url
				})
				.finally(() => {
					pending.delete(source)
				})
			pending.set(source, request)
			return request
		},
		peekIcon(source) {
			return resolvedUrls.get(source)
		},
	}
}
