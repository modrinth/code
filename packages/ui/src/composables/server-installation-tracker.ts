import type { Archon } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watch } from 'vue'

type ReadableRef<T> = Ref<T> | ComputedRef<T>

export type ServerInstallationKey =
	| Exclude<Archon.Websocket.v0.InstallProgressKey, { type: 'file' }>
	| { type: 'unknown' }

export type ServerInstallationState = {
	id: string
	key: ServerInstallationKey
	status: 'pending' | 'installing' | 'complete' | 'failed'
	progress: number | null
	error: string | null
	source: 'optimistic' | 'websocket' | 'server'
}

type OptimisticInstallation = {
	id: string
	key: ServerInstallationKey
	startRevision: number
}

type UseServerInstallationTrackerOptions = {
	worldId: ReadableRef<string | null>
	server: ReadableRef<Archon.Servers.v0.Server | null | undefined>
	content?: ReadableRef<Archon.Content.v1.Addons | null | undefined>
}

type ServerInstallationPlatform = Extract<ServerInstallationKey, { type: 'platform' }>['platform']

function installationKeyId(key: ServerInstallationKey) {
	switch (key.type) {
		case 'platform':
			return `platform:${key.platform}:${key.platform_version}:${key.game_version}`
		case 'modrinth_modpack':
			return `modrinth-modpack:${key.project_id}:${key.version_id}`
		case 'local_modpack':
			return `local-modpack:${key.filename}`
		case 'unknown':
			return 'unknown'
	}
}

function itemStatus(
	item: Archon.Websocket.v0.InstallProgressItem,
): ServerInstallationState['status'] {
	if (item.error != null) return 'failed'
	if (item.progress === 100) return 'complete'
	return 'installing'
}

function contentPlatform(modloader: string | null): ServerInstallationPlatform | null {
	const platform = modloader === 'neo_forge' ? 'neoforge' : modloader
	switch (platform) {
		case 'forge':
		case 'neoforge':
		case 'fabric':
		case 'quilt':
		case 'paper':
		case 'purpur':
		case 'vanilla':
			return platform
		default:
			return null
	}
}

function contentInstallationKey(content: Archon.Content.v1.Addons): ServerInstallationKey {
	if (content.installing === 'modpack' || (content.error && content.modpack)) {
		const spec = content.modpack?.spec
		if (spec?.platform === 'modrinth') {
			return {
				type: 'modrinth_modpack',
				project_id: spec.project_id,
				version_id: spec.version_id,
			}
		}
		if (spec?.platform === 'local_file') {
			return {
				type: 'local_modpack',
				filename: spec.filename,
			}
		}
		return { type: 'unknown' }
	}

	const platform = contentPlatform(content.modloader)
	if (platform && content.game_version) {
		return {
			type: 'platform',
			platform,
			platform_version: content.modloader_version ?? '',
			game_version: content.game_version,
		}
	}

	return { type: 'unknown' }
}

function addonFailureId(addon: Archon.Content.v1.Addon) {
	return `addon:${addon.id}:${addon.filename}`
}

function addonFailureError(addon: Archon.Content.v1.Addon) {
	return typeof addon.status === 'object' ? addon.status.failed.error : null
}

export function useServerInstallationTracker(options: UseServerInstallationTrackerOptions) {
	const installProgressItems = ref<Archon.Websocket.v0.InstallProgressItem[]>([])
	const optimisticInstallation = ref<OptimisticInstallation | null>(null)
	const receivedProgressSnapshot = ref(false)
	const snapshotRevision = ref(0)
	const seenActiveIds = ref(new Set<string>())
	const dismissedIds = ref(new Set<string>())
	let unknownInstallationId = 0

	const currentWorldItems = computed(() =>
		installProgressItems.value.filter((item) => item.world_id === options.worldId.value),
	)

	const websocketInstallation = computed<ServerInstallationState | null>(() => {
		const optimistic = optimisticInstallation.value
		const candidates = currentWorldItems.value.filter(
			(item) => item.key.type !== 'file' && !dismissedIds.value.has(installationKeyId(item.key)),
		)

		for (const item of candidates) {
			if (item.key.type === 'file') continue
			const id = installationKeyId(item.key)
			const status = itemStatus(item)
			if (status === 'complete') {
				if (
					optimistic &&
					snapshotRevision.value <= optimistic.startRevision &&
					!seenActiveIds.value.has(id)
				) {
					continue
				}
				if (
					!optimistic &&
					!seenActiveIds.value.has(id) &&
					options.server.value?.status !== 'installing'
				) {
					continue
				}
			}

			return {
				id,
				key: item.key,
				status,
				progress: item.progress,
				error: item.error,
				source: 'websocket',
			}
		}

		return null
	})

	const persistedAddonFailure = computed<ServerInstallationState | null>(() => {
		const addon = options.content?.value?.addons?.find((addon) => {
			const id = addonFailureId(addon)
			return addonFailureError(addon) != null && !dismissedIds.value.has(id)
		})
		if (!addon) return null

		return {
			id: addonFailureId(addon),
			key: { type: 'unknown' },
			status: 'failed',
			progress: null,
			error: addonFailureError(addon),
			source: 'server',
		}
	})

	const persistedInstallation = computed<ServerInstallationState | null>(() => {
		const content = options.content?.value
		if (!content || (!content.installing && !content.error)) return null

		const key = contentInstallationKey(content)
		const id = installationKeyId(key)
		if (dismissedIds.value.has(id)) return null

		return {
			id,
			key,
			status: content.error ? 'failed' : 'installing',
			progress: null,
			error: content.error?.message ?? null,
			source: 'server',
		}
	})

	watch(
		() => options.content?.value,
		(content) => {
			if (!content) return
			if (content.error) {
				optimisticInstallation.value = null
				return
			}
			if (!content.installing) return
			const id = installationKeyId(contentInstallationKey(content))
			if (dismissedIds.value.has(id)) {
				dismissedIds.value = new Set([...dismissedIds.value].filter((item) => item !== id))
			}
			optimisticInstallation.value = null
		},
	)

	watch(
		() => options.content?.value?.addons,
		(addons) => {
			const currentFailureIds = new Set(
				(addons ?? []).filter((addon) => addonFailureError(addon) != null).map(addonFailureId),
			)
			const nextDismissedIds = new Set(
				[...dismissedIds.value].filter(
					(id) => !id.startsWith('addon:') || currentFailureIds.has(id),
				),
			)
			if (nextDismissedIds.size !== dismissedIds.value.size) {
				dismissedIds.value = nextDismissedIds
			}
		},
	)

	const installation = computed<ServerInstallationState | null>(() => {
		if (websocketInstallation.value) return websocketInstallation.value

		const optimistic = optimisticInstallation.value
		if (optimistic && !dismissedIds.value.has(optimistic.id)) {
			return {
				id: optimistic.id,
				key: optimistic.key,
				status: 'pending',
				progress: null,
				error: null,
				source: 'optimistic',
			}
		}

		if (persistedInstallation.value) return persistedInstallation.value
		if (persistedAddonFailure.value) return persistedAddonFailure.value
		if (options.content?.value?.error) return null

		if (options.server.value?.status !== 'installing' || receivedProgressSnapshot.value) return null

		return {
			id: `unknown:${unknownInstallationId}`,
			key: { type: 'unknown' },
			status: 'installing',
			progress: null,
			error: null,
			source: 'server',
		}
	})

	const isBlocking = computed(
		() => installation.value?.status === 'pending' || installation.value?.status === 'installing',
	)

	function handleProgress(items: Archon.Websocket.v0.InstallProgressItem[]) {
		snapshotRevision.value += 1
		receivedProgressSnapshot.value = true
		installProgressItems.value = items

		const optimistic = optimisticInstallation.value
		const isPostOptimisticSnapshot =
			optimistic !== null && snapshotRevision.value > optimistic.startRevision
		const nextSeenActiveIds = new Set(seenActiveIds.value)
		const nextDismissedIds = new Set(dismissedIds.value)
		let hasAuthoritativeInstallation = false
		for (const item of items) {
			if (item.world_id !== options.worldId.value || item.key.type === 'file') continue
			hasAuthoritativeInstallation = true
			const id = installationKeyId(item.key)
			if (isPostOptimisticSnapshot) {
				nextDismissedIds.delete(id)
			}
			if (item.error == null && item.progress != null && item.progress < 100) {
				nextSeenActiveIds.add(id)
				nextDismissedIds.delete(id)
			}
		}
		seenActiveIds.value = nextSeenActiveIds
		dismissedIds.value = nextDismissedIds
		if (hasAuthoritativeInstallation) {
			optimisticInstallation.value = null
		}
	}

	function begin(key: ServerInstallationKey) {
		const id =
			key.type === 'unknown' ? `unknown:${++unknownInstallationId}` : installationKeyId(key)
		const nextDismissedIds = new Set(dismissedIds.value)
		nextDismissedIds.delete(id)
		dismissedIds.value = nextDismissedIds
		optimisticInstallation.value = {
			id,
			key,
			startRevision: snapshotRevision.value,
		}
	}

	function cancelOptimistic() {
		optimisticInstallation.value = null
	}

	function dismiss(id: string) {
		dismissedIds.value = new Set([...dismissedIds.value, id])
		if (optimisticInstallation.value?.id === id) {
			optimisticInstallation.value = null
		}
	}

	function reset() {
		installProgressItems.value = []
		optimisticInstallation.value = null
		receivedProgressSnapshot.value = false
		snapshotRevision.value = 0
		seenActiveIds.value = new Set()
		dismissedIds.value = new Set()
		unknownInstallationId = 0
	}

	return {
		begin,
		cancelOptimistic,
		dismiss,
		handleProgress,
		installation,
		installProgressItems,
		isBlocking,
		reset,
	}
}
