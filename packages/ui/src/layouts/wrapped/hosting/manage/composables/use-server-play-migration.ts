import type { Archon, UploadProgress } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type ComputedRef, type Ref, ref, watch } from 'vue'

import {
	hasCurseForgeUploads,
	restoreCurseForgeClientFiles,
} from '#ui/components/flows/creation-flow-modal/curseforge'
import type {
	ServerPlayMigrationKind,
	ServerPlayMigrationSource,
} from '#ui/components/servers/play-migration/types'
import { serverKeys } from '#ui/composables/server-queries'
import { injectModrinthClient, injectNotificationManager } from '#ui/providers'

import {
	SERVER_PLAY_MIGRATION_TAG,
	type ServerPlayMigration,
	type ServerPlayMigrationNext,
	type ServerPlayMigrationStep,
} from '../providers/server-play-migration'

const UNSUPPORTED_LOADERS = new Set(['vanilla', 'paper', 'purpur', 'spigot', 'bukkit', 'folia'])

type ReadableRef<T> = Ref<T> | ComputedRef<T>

export type TrackUpload = <T>(
	run: (onProgress: (progress: UploadProgress) => void) => Promise<T>,
) => Promise<T>

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
	trackUpload: TrackUpload
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

	const modpackPlatform = computed(() => options.content.value?.modpack?.spec.platform ?? null)
	const needsDetection = computed(
		() =>
			tagged.value &&
			!!options.content.value &&
			(modpackPlatform.value === null || modpackPlatform.value === 'local_file'),
	)
	const detection = useQuery({
		queryKey: computed(() => ['kyros', 'flameanvil', 'detection', options.worldId.value]),
		queryFn: () => client.kyros.flameanvil_v1.detect(options.worldId.value!),
		enabled: computed(() => needsDetection.value && !!options.worldId.value),
		retry: false,
	})

	const kind = computed<ServerPlayMigrationKind | null>(() => {
		if (!options.content.value) return null
		switch (modpackPlatform.value) {
			case 'modrinth':
				return 'modrinth'
			case 'local_file':
				return 'mrpack'
			case 'curseforge':
				return 'curseforge'
		}
		return detection.data.value?.likely_curseforge ? 'curseforge' : 'custom'
	})
	const hasUploadedMrpack = computed(
		() => kind.value === 'mrpack' && !!detection.data.value?.has_manually_uploaded_mrpack,
	)

	const ready = computed(
		() =>
			!tagged.value ||
			(!!options.content.value && (!needsDetection.value || detection.isFetched.value)),
	)
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

	async function requestReinstall(
		worldId: string,
		source: ServerPlayMigrationSource,
	): Promise<string | null> {
		switch (source.kind) {
			case 'restore': {
				const response = await client.archon.content_v1.restoreClientFiles(
					options.serverId,
					worldId,
				)
				return response.backup_id
			}
			case 'mrpack': {
				const upload = (onProgress?: (progress: UploadProgress) => void) =>
					client.kyros.content_v1.uploadModpackFile(
						worldId,
						source.file,
						{ known: {} },
						{ restoreClientFiles: true, useExistingMrpack: !source.file, onProgress },
					).promise
				const response = source.file ? await options.trackUpload(upload) : await upload()
				return response.backup_id
			}
			case 'curseforge': {
				const restore = (onUploadProgress?: (progress: UploadProgress) => void) =>
					restoreCurseForgeClientFiles(client, worldId, source.selection, { onUploadProgress })
				return hasCurseForgeUploads(source.selection) ? options.trackUpload(restore) : restore()
			}
		}
	}

	async function waitForMigration(backupId: string) {
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

	async function start(source: ServerPlayMigrationSource, next?: ServerPlayMigrationNext) {
		const worldId = options.worldId.value
		if (!worldId || active.value) return false
		const inviteEarly = next === 'invite'
		try {
			step.value = 'migrating'
			const backupId = await requestReinstall(worldId, source)
			if (!backupId) throw new Error('The server content could not be updated.')
			if (inviteEarly) options.onMigrated(next)
			if (!(await waitForMigration(backupId))) return false
			await queryClient.invalidateQueries({ queryKey: serverKeys.detailV1(options.serverId) })
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

	return {
		ready,
		tagged,
		required,
		kind,
		hasUploadedMrpack,
		step,
		active,
		modpackName,
		prompt: options.prompt,
		start,
	}
}
