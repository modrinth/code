import type { Archon } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, type ComputedRef, type Ref, ref, watch } from 'vue'

import type {
	ServerInstallationKey,
	ServerInstallationState,
} from '#ui/composables/server-installation-tracker'
import {
	injectModrinthClient,
	injectNotificationManager,
	SERVER_PLAY_MIGRATION_TAG,
	type ServerPlayMigration,
	type ServerPlayMigrationNext,
	type ServerPlayMigrationStep,
} from '#ui/providers'

const UNSUPPORTED_LOADERS = new Set(['vanilla', 'paper', 'purpur', 'spigot', 'bukkit', 'folia'])

type ReadableRef<T> = Ref<T> | ComputedRef<T>

/** Set `debug-server-play-migration` to `true` in localStorage to treat every server as needing migration outside production. */
function isMigrationForced() {
	if (process.env.NODE_ENV === 'production') return false
	try {
		return localStorage.getItem('debug-server-play-migration') === 'true'
	} catch {
		return false
	}
}

function waitFor<T>(source: () => T | undefined): Promise<T> {
	return new Promise((resolve) => {
		const initial = source()
		if (initial !== undefined) {
			resolve(initial)
			return
		}
		const stop = watch(source, (value) => {
			if (value === undefined) return
			stop()
			resolve(value)
		})
	})
}

export function useServerPlayMigration(options: {
	serverId: string
	worldId: ReadableRef<string | null>
	serverFull: ReadableRef<Archon.Servers.v1.ServerFull | null | undefined>
	content: ReadableRef<Archon.Content.v1.Addons | null | undefined>
	backups: ReadableRef<Archon.BackupsQueue.v1.BackupQueueBackup[]>
	invalidateBackups: () => Promise<void>
	installation: ReadableRef<ServerInstallationState | null>
	powerState: ReadableRef<Archon.Websocket.v0.PowerState>
	beginInstallation: (key: ServerInstallationKey) => void
	cancelOptimisticInstallation: () => void
	backupName: () => string
	prompt: (next: ServerPlayMigrationNext) => void
	onMigrated: (next: ServerPlayMigrationNext) => void
}): ServerPlayMigration {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const step = ref<ServerPlayMigrationStep>('idle')
	const completed = ref(false)

	const forced = isMigrationForced()
	const tagged = computed(
		() =>
			!completed.value &&
			(forced || !!options.serverFull.value?.tags.includes(SERVER_PLAY_MIGRATION_TAG)),
	)
	const ready = computed(() => !tagged.value || !!options.content.value)
	const required = computed(() => {
		const content = options.content.value
		if (!tagged.value || !content) return false
		if (content.modpack) return true
		return !!content.modloader && !UNSUPPORTED_LOADERS.has(content.modloader.toLowerCase())
	})

	const modpackName = computed(() => {
		const modpack = options.content.value?.modpack
		if (!modpack?.title) return null
		return modpack.version_number ? `${modpack.title} ${modpack.version_number}` : modpack.title
	})

	async function backup(worldId: string) {
		const { id } = await client.archon.backups_queue_v1.create(options.serverId, worldId, {
			name: options.backupName(),
		})
		await options.invalidateBackups()
		const status = await waitFor(() => {
			const current = options.backups.value.find((backup) => backup.id === id)?.status
			return current === 'done' || current === 'error' || current === 'timed_out'
				? current
				: undefined
		})
		if (status !== 'done') throw new Error('The backup could not be created.')
	}

	async function reinstall(worldId: string) {
		options.beginInstallation({ type: 'unknown' })
		try {
			await client.archon.content_v1.repair(options.serverId, worldId)
		} catch (error) {
			options.cancelOptimisticInstallation()
			throw error
		}
		const result = await waitFor(() => {
			const current = options.installation.value
			if (current?.status === 'pending' || current?.status === 'installing') return undefined
			return current ?? null
		})
		if (result?.status === 'failed') {
			throw new Error(result.error ?? 'The content could not be reinstalled.')
		}
	}

	async function start(next?: ServerPlayMigrationNext) {
		const worldId = options.worldId.value
		if (!worldId || step.value !== 'idle') return false
		try {
			step.value = 'backup'
			await backup(worldId)
			step.value = 'reinstall'
			await reinstall(worldId)
			step.value = 'restart'
			await client.archon.servers_v0.power(
				options.serverId,
				options.powerState.value === 'running' ? 'Restart' : 'Start',
			)
			await queryClient.invalidateQueries({
				queryKey: ['servers', 'v1', 'detail', options.serverId],
			})
			completed.value = true
			if (next) options.onMigrated(next)
			return true
		} catch (error) {
			handleError(error as Error)
			return false
		} finally {
			step.value = 'idle'
		}
	}

	return { ready, required, step, modpackName, prompt: options.prompt, start }
}
