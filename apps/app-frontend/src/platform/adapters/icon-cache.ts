import type { IconCacheContext } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'

import { cacheIconBytes, cacheRemoteIcon } from '@/platform/app-lib/icons/commands'

export function createIconCache(): IconCacheContext {
	const pending = new Map<string, Promise<string>>()
	const blobs = new WeakMap<Blob, Promise<string>>()
	const resolvedUrls = new Map<string, string>()
	const resolvedBlobs = new WeakMap<Blob, string>()
	return {
		cacheIcon(source) {
			if (typeof source !== 'string') {
				const existing = blobs.get(source)
				if (existing) return existing
				const request = source
					.arrayBuffer()
					.then((buffer) => cacheIconBytes(Array.from(new Uint8Array(buffer))))
					.then(convertFileSrc)
					.then((url) => {
						resolvedBlobs.set(source, url)
						return url
					})
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
			return typeof source === 'string' ? resolvedUrls.get(source) : resolvedBlobs.get(source)
		},
	}
}
