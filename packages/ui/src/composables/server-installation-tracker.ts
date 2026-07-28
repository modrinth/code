import type { Archon } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref } from 'vue'

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
}

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

		const nextSeenActiveIds = new Set(seenActiveIds.value)
		const nextDismissedIds = new Set(dismissedIds.value)
		let hasAuthoritativeInstallation = false
		for (const item of items) {
			if (item.world_id !== options.worldId.value || item.key.type === 'file') continue
			hasAuthoritativeInstallation = true
			const id = installationKeyId(item.key)
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
