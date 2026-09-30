import { provideIconCache } from '@modrinth/ui'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export function setupIconCacheProvider() {
	const pending = new Map<string, Promise<string>>()
	const blobs = new WeakMap<Blob, Promise<string>>()
	return provideIconCache({
		cacheIcon(source) {
			if (typeof source !== 'string') {
				const existing = blobs.get(source)
				if (existing) return existing
				const request = source
					.arrayBuffer()
					.then((buffer) =>
						invoke<string>('plugin:instance|instance_cache_icon', {
							iconBytes: Array.from(new Uint8Array(buffer)),
						}),
					)
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
			const request = invoke<string>('plugin:utils|cache_remote_icon', { source }).finally(() => {
				pending.delete(source)
			})
			pending.set(source, request)
			return request
		},
	})
}
