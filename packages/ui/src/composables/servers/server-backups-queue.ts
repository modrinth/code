import type { Archon } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, type Ref } from 'vue'

import { type BusyReason, injectModrinthClient } from '#ui/providers'

import { defineMessage } from '../i18n'

type ProgressKey = `${string}:${'create' | 'restore'}`

export function useServerBackupsQueue(serverId: Ref<string>, worldId: Ref<string | null>) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	const queryKey = computed(() => ['backups', 'queue', serverId.value, worldId.value] as const)

	const progressOverlay = reactive(new Map<ProgressKey, number>())
	const lastSeenState = new Map<ProgressKey, Archon.Websocket.v0.BackupState>()

	const query = useQuery({
		queryKey,
		queryFn: () => client.archon.backups_queue_v1.list(serverId.value, worldId.value!),
		enabled: computed(() => !!worldId.value),
		refetchInterval: (q) => {
			const data = q.state.data as Archon.BackupsQueue.v1.BackupsQueueResponse | undefined
			return data?.active_operations?.length ? 3000 : false
		},
	})

	const data = computed(() => query.data.value)
	const activeOperations = computed(() => data.value?.active_operations ?? [])
	const backups = computed(() =>
		[...(data.value?.backups ?? [])].sort(
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

	function handleWsBackupProgress(evt: Archon.Websocket.v0.WSBackupProgressEvent) {
		if (evt.task === 'file') return
		const key = `${evt.id}:${evt.task}` as ProgressKey

		if (evt.state === 'ongoing') {
			progressOverlay.set(key, evt.progress)
		} else {
			progressOverlay.delete(key)
		}

		const prev = lastSeenState.get(key)
		if (prev !== evt.state) {
			lastSeenState.set(key, evt.state)
			queryClient.invalidateQueries({ queryKey: queryKey.value })
		}
	}

	function progressFor(backupId: string, kind: 'create' | 'restore'): number | undefined {
		return progressOverlay.get(`${backupId}:${kind}`)
	}

	const busyReasons = computed<BusyReason[]>(() => {
		const reasons: BusyReason[] = []
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
		activeOperations,
		activeOperationByBackupId,
		backups,
		hasActiveCreate,
		hasActiveRestore,
		hasRunningCreate,
		hasRunningRestore,
		progressFor,
		handleWsBackupProgress,
		busyReasons,
		invalidate,
	}
}
