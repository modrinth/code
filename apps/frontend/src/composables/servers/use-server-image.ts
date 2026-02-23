import type { Archon } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { type ComputedRef, ref, watch } from 'vue'

// TODO: Remove and use V1 when available
export function useServerImage(
	serverId: string,
	upstream: ComputedRef<Archon.Servers.v0.Server['upstream'] | null>,
) {
	const client = injectModrinthClient()
	const image = ref<string | undefined>()

	const sharedImage = useState<string | undefined>(`server-icon-${serverId}`)
	if (sharedImage.value) {
		image.value = sharedImage.value
	}

	async function loadImage() {
		if (sharedImage.value) {
			image.value = sharedImage.value
			return
		}

		if (import.meta.server) return

		const cached = localStorage.getItem(`server-icon-${serverId}`)
		if (cached) {
			sharedImage.value = cached
			image.value = cached
			return
		}

		let projectIconUrl: string | undefined
		const upstreamVal = upstream.value
		if (upstreamVal?.project_id) {
			try {
				const project = await $fetch<{ icon_url?: string }>(
					`https://api.modrinth.com/v2/project/${upstreamVal.project_id}`,
				)
				projectIconUrl = project.icon_url
			} catch {
				// project fetch failed, continue without icon url
			}
		}

		try {
			const fileData = await client.kyros.files_v0.downloadFile('/server-icon-original.png')

			if (fileData instanceof Blob) {
				const dataURL = await resizeImage(fileData, 512)
				sharedImage.value = dataURL
				localStorage.setItem(`server-icon-${serverId}`, dataURL)
				image.value = dataURL
				return
			}
		} catch (error: any) {
			if (error?.statusCode >= 500) {
				image.value = undefined
				return
			}

			if (error?.statusCode === 404 && projectIconUrl) {
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
							sharedImage.value = result
							localStorage.setItem(`server-icon-${serverId}`, result)
							resolve(result)
							URL.revokeObjectURL(img.src)
						}
						img.src = URL.createObjectURL(file)
					})
					image.value = dataURL
					return
				} catch (externalError: any) {
					console.debug('Could not process external icon:', externalError.message)
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
