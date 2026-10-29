import type { Archon } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, type ComputedRef, type Ref, ref, watch } from 'vue'

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
	forced: () => boolean
	content: ReadableRef<Archon.Content.v1.Addons | null | undefined>
	backups: ReadableRef<Archon.BackupsQueue.v1.BackupQueueBackup[]>
	invalidateBackups: () => Promise<void>
	trackMigration: (backupId: string) => void
	migrationStateFor: (backupId: string) => Archon.Websocket.v0.BackupState | undefined
	migrationActive: ReadableRef<boolean>
	prompt: (next: ServerPlayMigrationNext) => void
	onMigrated: (next: ServerPlayMigrationNext) => void
}): ServerPlayMigration {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const step = ref<ServerPlayMigrationStep>('idle')
	const completed = ref(false)

	const tagged = computed(
		() =>
			!completed.value &&
			(options.forced() || !!options.serverFull.value?.tags.includes(SERVER_PLAY_MIGRATION_TAG)),
	)
	const active = computed(() => step.value !== 'idle' || options.migrationActive.value)
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

	async function migrate(worldId: string) {
		const { backup_id: backupId } = await client.archon.content_v1.restoreClientFiles(
			options.serverId,
			worldId,
		)
		if (!backupId) throw new Error('The server content could not be updated.')
		options.trackMigration(backupId)
		await options.invalidateBackups()

		const outcome = await waitFor(() => {
			const backupState = options.backups.value
				.find((backup) => backup.id === backupId)
				?.history.find((operation) => operation.operation_type === 'create')?.state
			if (backupState === 'failed' || backupState === 'timed_out') return 'failed'
			if (backupState === 'cancelled') return 'cancelled'
			const state = options.migrationStateFor(backupId)
			if (state === 'done' || state === 'cancelled') return state
			if (state === 'failed' || state === 'damaged') return 'failed'
			return undefined
		})
		if (outcome === 'failed') throw new Error('The server content could not be updated.')
		return outcome === 'done'
	}

	async function republish(worldId: string) {
		const details = await client.archon.servers_v1.get(options.serverId)
		const shared = details.worlds.find((world) => world.id === worldId)?.content?.shared_instance_id
		if (shared) await client.archon.content_v1.share(options.serverId, worldId)
	}

	async function start(next?: ServerPlayMigrationNext) {
		const worldId = options.worldId.value
		if (!worldId || active.value) return false
		const inviteEarly = next === 'invite'
		try {
			step.value = 'migrating'
			if (inviteEarly) options.onMigrated(next)
			if (!(await migrate(worldId))) return false
			await queryClient.invalidateQueries({
				queryKey: ['servers', 'v1', 'detail', options.serverId],
			})
			completed.value = true
			if (inviteEarly) await republish(worldId)
			else if (next) options.onMigrated(next)
			return true
		} catch (error) {
			handleError(error as Error)
			return false
		} finally {
			step.value = 'idle'
		}
	}

	return { ready, tagged, required, step, active, modpackName, prompt: options.prompt, start }
}
