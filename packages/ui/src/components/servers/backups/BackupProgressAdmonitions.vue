<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, watch } from 'vue'

import { injectModrinthClient, injectModrinthServerContext } from '../../../providers'
import type { BackupProgressEntry } from '../../../providers/server-context'
import BackupProgressAdmonition from './BackupProgressAdmonition.vue'

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { serverId, worldId, backupsState, markBackupCancelled } = injectModrinthServerContext()

const backupsQueryKey = ['backups', 'list', serverId]

const { data: backupsList } = useQuery({
	queryKey: backupsQueryKey,
	queryFn: () => client.archon.backups_v1.list(serverId, worldId.value!),
	enabled: computed(() => !!worldId.value),
})

interface TerminalEntry {
	type: 'create' | 'restore'
	state: Archon.Backups.v1.BackupState
	backupName?: string
	createdAt?: string
}

interface AdmonitionEntry {
	key: string
	backupId: string
	type: 'create' | 'restore'
	state: Archon.Backups.v1.BackupState
	progress: number
	name?: string
	createdAt?: string
}

const terminalEntries = reactive(new Map<string, TerminalEntry>())
const dismissedIds = reactive(new Set<string>())

function findBackup(backupId: string) {
	return backupsList.value?.find((b) => b.id === backupId)
}

watch(
	() => [...backupsState.entries()] as [string, BackupProgressEntry][],
	(entries) => {
		for (const [id, entry] of entries) {
			const backup = findBackup(id)
			if (entry.create?.state === 'failed') {
				terminalEntries.set(`${id}:create`, {
					type: 'create',
					state: 'failed',
					backupName: backup?.name,
					createdAt: backup?.created_at,
				})
			}
			if (entry.restore?.state === 'done') {
				terminalEntries.set(`${id}:restore`, {
					type: 'restore',
					state: 'done',
					backupName: backup?.name,
					createdAt: backup?.created_at,
				})
			}
			if (entry.restore?.state === 'failed') {
				terminalEntries.set(`${id}:restore`, {
					type: 'restore',
					state: 'failed',
					backupName: backup?.name,
					createdAt: backup?.created_at,
				})
			}
		}
	},
	{ deep: true },
)

const admonitions = computed<AdmonitionEntry[]>(() => {
	const result: AdmonitionEntry[] = []
	const seenIds = new Set<string>()

	// 1. Active WS entries (real-time progress from backupsState)
	for (const [id, entry] of backupsState.entries()) {
		const backup = findBackup(id)
		if (entry.create && entry.create.state === 'ongoing') {
			const key = `${id}:create`
			if (!dismissedIds.has(key)) {
				seenIds.add(id)
				result.push({
					key,
					backupId: id,
					type: 'create',
					state: entry.create.state,
					progress: entry.create.progress,
					name: backup?.name,
					createdAt: backup?.created_at,
				})
			}
		}
		if (entry.restore && entry.restore.state === 'ongoing') {
			const key = `${id}:restore`
			if (!dismissedIds.has(key)) {
				seenIds.add(id)
				result.push({
					key,
					backupId: id,
					type: 'restore',
					state: entry.restore.state,
					progress: entry.restore.progress,
					name: backup?.name,
					createdAt: backup?.created_at,
				})
			}
		}
	}

	// 2. REST-based entries for pending/in_progress backups without WS data yet
	if (backupsList.value) {
		for (const backup of backupsList.value) {
			if (seenIds.has(backup.id)) continue
			if (backup.status === 'pending' || backup.status === 'in_progress') {
				const key = `${backup.id}:create`
				if (!dismissedIds.has(key)) {
					result.push({
						key,
						backupId: backup.id,
						type: 'create',
						state: 'ongoing',
						progress: 0,
						name: backup.name,
						createdAt: backup.created_at,
					})
				}
			}
		}
	}

	// 3. Terminal entries (snapshotted before cleanup)
	for (const [key, entry] of terminalEntries.entries()) {
		if (dismissedIds.has(key)) continue
		if (result.some((r) => r.key === key)) continue

		const backupId = key.split(':')[0]
		const backup = findBackup(backupId)
		result.push({
			key,
			backupId,
			type: entry.type,
			state: entry.state,
			progress: entry.state === 'done' ? 1 : 0,
			name: backup?.name ?? entry.backupName,
			createdAt: backup?.created_at ?? entry.createdAt,
		})
	}

	return result
})

function handleCancel(backupId: string) {
	client.archon.backups_v1.delete(serverId, worldId.value!, backupId).then(() => {
		markBackupCancelled(backupId)
		backupsState.delete(backupId)
		queryClient.invalidateQueries({ queryKey: backupsQueryKey })
	})
}

function handleRetry(backupId: string, key: string) {
	client.archon.backups_v1.retry(serverId, worldId.value!, backupId).then(() => {
		terminalEntries.delete(key)
		dismissedIds.delete(key)
		queryClient.invalidateQueries({ queryKey: backupsQueryKey })
	})
}

function handleDismiss(key: string) {
	dismissedIds.add(key)
	terminalEntries.delete(key)
}
</script>

<template>
	<TransitionGroup
		v-if="admonitions.length > 0"
		name="backup-admonition"
		tag="div"
		class="flex flex-col gap-3"
	>
		<BackupProgressAdmonition
			v-for="item in admonitions"
			:key="item.key"
			:type="item.type"
			:state="item.state"
			:progress="item.progress"
			:backup-name="item.name"
			:created-at="item.createdAt"
			@cancel="handleCancel(item.backupId)"
			@retry="handleRetry(item.backupId, item.key)"
			@dismiss="handleDismiss(item.key)"
		/>
	</TransitionGroup>
</template>

<style scoped>
.backup-admonition-enter-active,
.backup-admonition-leave-active {
	transition:
		opacity 300ms ease-in-out,
		transform 300ms ease-in-out;
}

.backup-admonition-enter-from {
	opacity: 0;
	transform: translateY(-10px);
}

.backup-admonition-leave-to {
	opacity: 0;
	transform: translateY(-10px);
}

.backup-admonition-move {
	transition: transform 300ms ease-in-out;
}
</style>
