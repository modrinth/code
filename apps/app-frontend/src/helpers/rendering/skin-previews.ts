import type { Cape, Skin } from '../skins'
import { headStorage } from '../storage/head-storage'
import { skinPreviewStorage } from '../storage/skin-preview-storage'
import type { RenderResult } from './batch-skin-renderer'

interface PreviewEntry {
	users: number
	result?: RenderResult
	promise: Promise<RenderResult | undefined>
}

const previews = new Map<string, PreviewEntry>()
let renderQueue: Promise<unknown> = Promise.resolve()
let pendingRenders = 0

export function getSkinPreviewKey(skin: Skin): string {
	return `ears-2-fixed-uvs+${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
}

export function acquireSkinPreview(skin: Skin, capes: Cape[]) {
	const key = getSkinPreviewKey(skin)
	let entry = previews.get(key)
	if (!entry) {
		const created: PreviewEntry = { users: 0, promise: Promise.resolve(undefined) }
		created.promise = skinPreviewStorage.retrieve(key).catch(() => null).then(async (cached) => {
			if (created.users === 0) return
			if (!cached) {
				pendingRenders++
				const render = renderQueue.then(async () => {
					if (created.users === 0) return
					const { renderSkinPreview } = await import('./batch-skin-renderer')
					const result = await renderSkinPreview(skin, capes)
					await skinPreviewStorage.store(key, result).catch(() => {})
					return result
				}).finally(async () => {
					if (--pendingRenders === 0) {
						const { disposeSharedRenderer } = await import('./batch-skin-renderer')
						if (pendingRenders === 0) disposeSharedRenderer()
					}
				})
				renderQueue = render.catch(() => {})
				cached = (await render) ?? null
			}
			if (!cached || created.users === 0) return
			created.result = { forwards: URL.createObjectURL(cached.forwards) }
			return created.result
		})
		previews.set(key, created)
		entry = created
	}
	const acquired = entry
	acquired.users++
	let released = false
	return {
		promise: acquired.promise,
		release: () => {
			if (released) return
			released = true
			if (--acquired.users > 0) return
			if (acquired.result) URL.revokeObjectURL(acquired.result.forwards)
			if (previews.get(key) === acquired) previews.delete(key)
		},
	}
}

export async function cleanupUnusedPreviews(skins: Skin[]): Promise<void> {
	await Promise.all([
		skinPreviewStorage.cleanupInvalidKeys(new Set(skins.map(getSkinPreviewKey))),
		headStorage.cleanupInvalidKeys(new Set(skins.map((skin) => `${skin.texture_key}-head`))),
	])
}
