import type { Archon } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed, type ComputedRef, ref } from 'vue'

import { injectModrinthClient } from '#ui/providers'

type UpstreamRef = ComputedRef<Archon.Servers.v0.Server['upstream'] | null | undefined>
type ServerIdSource = string | { readonly value: string }
type WorldIdSource = string | null | undefined | { readonly value: string | null | undefined }

type UseServerImageOptions = {
	enabled?: ComputedRef<boolean> | boolean
	size?: number
	includeProjectFallback?: boolean
	worldId?: WorldIdSource
}

export async function processImageBlob(blob: Blob, size: number): Promise<string> {
	return new Promise((resolve) => {
		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')!
		const img = new Image()
		img.onload = () => {
			canvas.width = size
			canvas.height = size
			ctx.drawImage(img, 0, 0, size, size)
			const dataURL = canvas.toDataURL('image/png')
			URL.revokeObjectURL(img.src)
			resolve(dataURL)
		}
		img.src = URL.createObjectURL(blob)
	})
}

function getStatusCode(error: unknown): number | undefined {
	const err = error as { statusCode?: number; response?: { status?: number } }
	return err.statusCode ?? err.response?.status
}

function isNotFound(error: unknown): boolean {
	return getStatusCode(error) === 404
}

export function useServerImage(
	serverId: ServerIdSource,
	upstream: UpstreamRef,
	options: UseServerImageOptions = {},
) {
	const client = injectModrinthClient()
	const localImage = ref<string | null | undefined>(undefined)
	const iconSize = options.size ?? 512
	const includeProjectFallback = options.includeProjectFallback ?? false
	const resolvedServerId = computed(() => resolveServerId(serverId))
	const resolvedWorldId = computed(() => resolveWorldId(options.worldId))

	const queryKey = computed(
		() =>
			[
				'servers',
				'detail',
				resolvedServerId.value,
				'icon',
				resolvedWorldId.value ?? 'active',
				upstream.value?.project_id ?? null,
			] as const,
	)

	const isEnabled = computed(() => {
		const explicitEnabled =
			typeof options.enabled === 'boolean' ? options.enabled : options.enabled?.value
		return !!resolvedServerId.value && (explicitEnabled ?? true)
	})

	const { data: remoteImage, refetch } = useQuery({
		queryKey,
		queryFn: async (): Promise<string | null> => {
			const id = resolvedServerId.value
			if (!id) return null

			try {
				const targetWorldId = resolvedWorldId.value ?? (await getActiveWorldId(id))
				if (!targetWorldId) return null

				try {
					const blob = await client.kyros.files_v1.downloadRawFileContents(
						targetWorldId,
						'/server-icon.png',
					)
					return await processImageBlob(blob, iconSize)
				} catch (error) {
					if (!isNotFound(error)) throw error
				}

				try {
					const blob = await client.kyros.files_v1.downloadRawFileContents(
						targetWorldId,
						'/server-icon-original.png',
					)
					return await processImageBlob(blob, iconSize)
				} catch (error) {
					if (!isNotFound(error)) throw error
				}
			} catch (error) {
				console.debug('Server image fetch failed:', error)
			}

			if (!includeProjectFallback || !upstream.value?.project_id) return null

			try {
				const project = await client.labrinth.projects_v2.get(upstream.value.project_id)
				if (!project.icon_url) return null
				const response = await fetch(project.icon_url)
				if (!response.ok) return null
				const blob = await response.blob()
				return await processImageBlob(blob, iconSize)
			} catch (error) {
				console.debug('Project icon fallback failed:', error)
				return null
			}
		},
		enabled: isEnabled,
	})

	const image = computed(() => {
		if (localImage.value === null) return undefined
		const remote = remoteImage.value
		if (remote === null) return undefined
		return localImage.value ?? remote
	})

	function setImage(nextImage: string | null | undefined) {
		localImage.value = nextImage
	}

	function clearImage() {
		localImage.value = null
	}

	function resetLocalOverride() {
		localImage.value = undefined
	}

	async function getActiveWorldId(id: string): Promise<string | null> {
		const server = await client.archon.servers_v1.get(id)
		const activeWorld = server.worlds.find((world) => world.is_active)
		return activeWorld?.id ?? server.worlds[0]?.id ?? null
	}

	return {
		image,
		queryKey,
		refetch,
		setImage,
		clearImage,
		resetLocalOverride,
	}
}

function resolveServerId(serverId: ServerIdSource): string {
	return typeof serverId === 'string' ? serverId : serverId.value
}

function resolveWorldId(worldId: WorldIdSource): string | null {
	if (worldId == null) return null
	if (typeof worldId === 'string') return worldId
	return worldId.value ?? null
}
