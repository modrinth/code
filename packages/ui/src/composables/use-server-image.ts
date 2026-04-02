import type { Archon } from '@modrinth/api-client'
import { type ComputedRef, ref, watch } from 'vue'

import { injectModrinthClient } from '#ui/providers'

const imageCache = new Map<string, string>()

// TODO: Remove and use V1 when available
export function useServerImage(
	serverId: string,
	upstream: ComputedRef<Archon.Servers.v0.Server['upstream'] | null>,
) {
	const client = injectModrinthClient()
	const image = ref<string | undefined>()

	const cached = imageCache.get(serverId)
	if (cached) {
		image.value = cached
	}

	async function loadImage() {
		if (typeof window === 'undefined') return

		if (imageCache.has(serverId)) {
			image.value = imageCache.get(serverId)
			return
		}

		const localCached = localStorage.getItem(`server-icon-${serverId}`)
		if (localCached) {
			imageCache.set(serverId, localCached)
			image.value = localCached
			return
		}

		let projectIconUrl: string | undefined
		const upstreamVal = upstream.value
		if (upstreamVal?.project_id) {
			try {
				const project = await client.labrinth.projects_v2.get(upstreamVal.project_id)
				projectIconUrl = project.icon_url
			} catch {
				// project fetch failed, continue without icon url
			}
		}

		try {
			const fileData = await client.kyros.files_v0.downloadFile('/server-icon-original.png')

			if (fileData instanceof Blob) {
				const dataURL = await resizeImage(fileData, 512)
				imageCache.set(serverId, dataURL)
				localStorage.setItem(`server-icon-${serverId}`, dataURL)
				image.value = dataURL
				return
			}
		} catch (error: unknown) {
			const statusCode = (error as { statusCode?: number })?.statusCode
			if (statusCode != null && statusCode >= 500) {
				image.value = undefined
				return
			}

			if (statusCode === 404 && projectIconUrl) {
				try {
					const response = await fetch(projectIconUrl)
					if (!response.ok) throw new Error('Failed to fetch icon')
					const file = await response.blob()
					const originalFile = new File([file], 'server-icon-original.png', {
						type: 'image/png',
					})

					const dataURL = await new Promise<string>((resolve) => {
						const canvas = document.createElement('canvas')
						const ctx = canvas.getContext('2d')
						const img = new Image()
						img.onload = () => {
							canvas.width = 64
							canvas.height = 64
							ctx?.drawImage(img, 0, 0, 64, 64)
							canvas.toBlob(async (blob) => {
								if (blob) {
									const scaledFile = new File([blob], 'server-icon.png', {
										type: 'image/png',
									})
									client.kyros.files_v0
										.uploadFile('/server-icon.png', scaledFile)
										.promise.catch(() => {})
									client.kyros.files_v0
										.uploadFile('/server-icon-original.png', originalFile)
										.promise.catch(() => {})
								}
							}, 'image/png')
							const result = canvas.toDataURL('image/png')
							imageCache.set(serverId, result)
							localStorage.setItem(`server-icon-${serverId}`, result)
							resolve(result)
							URL.revokeObjectURL(img.src)
						}
						img.src = URL.createObjectURL(file)
					})
					image.value = dataURL
					return
				} catch (externalError: unknown) {
					console.debug('Could not process external icon:', (externalError as Error).message)
				}
			}
		}

		image.value = undefined
	}

	watch(upstream, () => loadImage(), { immediate: true })

	return image
}

function resizeImage(blob: Blob, size: number): Promise<string> {
	return new Promise<string>((resolve) => {
		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')
		const img = new Image()
		img.onload = () => {
			canvas.width = size
			canvas.height = size
			ctx?.drawImage(img, 0, 0, size, size)
			const dataURL = canvas.toDataURL('image/png')
			resolve(dataURL)
			URL.revokeObjectURL(img.src)
		}
		img.src = URL.createObjectURL(blob)
	})
}
