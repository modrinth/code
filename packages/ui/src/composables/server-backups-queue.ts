import type { AbstractModrinthClient, Archon } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, type Ref } from 'vue'

import { type BusyReason, injectModrinthClient } from '#ui/providers'

import { defineMessage } from './i18n'

type ProgressKey = `${string}:${'create' | 'restore'}`

type ProgressStore = {
	overlay: Map<ProgressKey, number>
	lastSeenState: Map<ProgressKey, Archon.Websocket.v0.BackupState>
	migrations: Map<string, Archon.Websocket.v0.BackupState>
}

const progressStores = new Map<string, ProgressStore>()

function progressStoreFor(serverId: string): ProgressStore {
	let store = progressStores.get(serverId)
	if (!store) {
		store = {
			overlay: reactive(new Map()),
			lastSeenState: new Map(),
			migrations: reactive(new Map()),
		}
		progressStores.set(serverId, store)
	}
	return store
}

export function serverBackupsQueueQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
) {
	return {
		queryKey: ['backups', 'queue', serverId] as const,
		queryFn: () => client.archon.backups_queue_v1.listWithOptions(serverId, worldId),
	}
}

export function useServerBackupsQueue(serverId: Ref<string>, worldId: Ref<string | null>) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	const queryKey = computed(() => ['backups', 'queue', serverId.value] as const)

	const store = computed(() => progressStoreFor(serverId.value))

	const query = useQuery({
		queryKey,
		queryFn: () => client.archon.backups_queue_v1.listWithOptions(serverId.value, worldId.value!),
		enabled: computed(() => !!worldId.value),
		refetchInterval: (q) => {
			const data = q.state.data as
				| Archon.BackupsQueue.v1.BackupsQueueWithOptionsResponse
				| undefined
			return data?.active_operations?.length ? 30_000 : false
		},
	})

	const data = computed(() => query.data.value)
	const options = computed(() => data.value?.options ?? null)
	const nextAutoBackupAt = computed(() => data.value?.next_auto_backup_at ?? null)
	const activeOperations = computed(() => data.value?.active_operations ?? [])
	const backups = computed(() =>
		[...(data.value?.backups ?? [])].sort(
			(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
		),
	)
	const deletedBackups = computed(() =>
		[...(data.value?.deleted_backups ?? [])].sort(
			(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
		),
	)

	const activeOperationByBackupId = computed(() => {
		const map = new Map<string, Archon.BackupsQueue.v1.ActiveOperation>()
		for (const op of activeOperations.value) map.set(op.backup_id, op)
		return map
	})
	const backupById = computed(() => {
		const map = new Map<string, Archon.BackupsQueue.v1.BackupQueueBackup>()
		for (const backup of backups.value) map.set(backup.id, backup)
		return map
	})

	const hasActiveCreate = computed(() =>
		activeOperations.value.some((o) => o.operation_type === 'create' && !o.has_parent),
	)
	const hasActiveRestore = computed(() =>
		activeOperations.value.some((o) => o.operation_type === 'restore'),
	)
	const hasRunningCreate = computed(() =>
		activeOperations.value.some(
			(o) =>
				o.operation_type === 'create' &&
				!o.has_parent &&
				backupById.value.get(o.backup_id)?.status === 'in_progress',
		),
	)
	const hasRunningRestore = computed(() =>
		activeOperations.value.some(
			(o) =>
				o.operation_type === 'restore' &&
				backupById.value.get(o.backup_id)?.status === 'in_progress',
		),
	)

	const hasRunningMigration = computed(() =>
		[...store.value.migrations.values()].some((state) => state === 'ongoing'),
	)
	const hasActiveMigration = computed(() =>
		[...store.value.migrations].some(([backupId, state]) => {
			if (state !== 'pending' && state !== 'ongoing') return false
			const create = backupById.value
				.get(backupId)
				?.history.find((h) => h.operation_type === 'create')?.state
			return create !== 'failed' && create !== 'timed_out' && create !== 'cancelled'
		}),
	)

	function handleWsBackupProgress(evt: Archon.Websocket.v0.WSBackupProgressEvent) {
		if (evt.task === 'file') return
		const { overlay, lastSeenState, migrations } = store.value
		const migration = evt.task === 'create-with-restore-files'
		const key = `${evt.id}:${migration ? 'create' : evt.task}` as ProgressKey

		if (migration) migrations.set(evt.id, evt.state)

		if (evt.state === 'ongoing') {
			overlay.set(key, evt.progress)
		} else {
			overlay.delete(key)
		}

		const prev = lastSeenState.get(key)
		if (prev !== evt.state) {
			lastSeenState.set(key, evt.state)
			queryClient.invalidateQueries({ queryKey: queryKey.value })
		}
	}

	function progressFor(backupId: string, kind: 'create' | 'restore'): number | undefined {
		return store.value.overlay.get(`${backupId}:${kind}`)
	}

	function trackMigration(backupId: string) {
		const { migrations } = store.value
		if (!migrations.has(backupId)) migrations.set(backupId, 'pending')
	}

	function migrationStateFor(backupId: string): Archon.Websocket.v0.BackupState | undefined {
		return store.value.migrations.get(backupId)
	}

	function forgetMigration(backupId: string) {
		store.value.migrations.delete(backupId)
	}

	const migrationIds = computed(() => [...store.value.migrations.keys()])

	const busyReasons = computed<BusyReason[]>(() => {
		const reasons: BusyReason[] = []
		if (hasRunningMigration.value) {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.migrating',
					defaultMessage: 'Server content update in progress',
				}),
			})
		}
		if (hasRunningCreate.value) {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.backup-creating',
					defaultMessage: 'Backup creation in progress',
				}),
			})
		}
		if (hasRunningRestore.value) {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.backup-restoring',
					defaultMessage: 'Backup restore in progress',
				}),
			})
		}
		return reasons
	})

	async function invalidate() {
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	}

	return {
		query,
		queryKey,
		data,
		options,
		nextAutoBackupAt,
		activeOperations,
		activeOperationByBackupId,
		backups,
		deletedBackups,
		hasActiveCreate,
		hasActiveRestore,
		hasRunningCreate,
		hasRunningRestore,
		hasRunningMigration,
		hasActiveMigration,
		migrationIds,
		progressFor,
		trackMigration,
		migrationStateFor,
		forgetMigration,
		handleWsBackupProgress,
		busyReasons,
		invalidate,
	}
}
