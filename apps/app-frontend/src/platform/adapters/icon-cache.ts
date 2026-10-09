import type { IconCacheContext } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'

import { cacheIconBytes, cacheRemoteIcon } from '@/platform/app-lib/icons/commands'

export function createIconCache(): IconCacheContext {
	const pending = new Map<string, Promise<string>>()
	const blobs = new WeakMap<Blob, Promise<string>>()
	return {
		cacheIcon(source) {
			if (typeof source !== 'string') {
				const existing = blobs.get(source)
				if (existing) return existing
				const request = source
					.arrayBuffer()
					.then((buffer) => cacheIconBytes(Array.from(new Uint8Array(buffer))))
					.then(convertFileSrc)
					.catch((error) => {
						blobs.delete(source)
						throw error
					})
				blobs.set(source, request)
				return request
			}
			if (!source.startsWith('https://')) return Promise.resolve(source)
			const existing = pending.get(source)
			if (existing) return existing
			const request = cacheRemoteIcon(source).finally(() => {
				pending.delete(source)
			})
			pending.set(source, request)
			return request
		},
	}
}
