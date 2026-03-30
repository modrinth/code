<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	ClockIcon,
	InfoIcon,
	RotateCounterClockwiseIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, watch } from 'vue'

import { useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { injectModrinthClient, injectModrinthServerContext } from '../../../providers'
import type { BackupProgressEntry } from '../../../providers/server-context'
import { commonMessages } from '../../../utils'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import ProgressBar from '../../base/ProgressBar.vue'

const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime()
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

	for (const [id, entry] of backupsState.entries()) {
		const backup = findBackup(id)
		seenIds.add(id)
		if (entry.create && entry.create.state === 'ongoing') {
			const key = `${id}:create`
			if (!dismissedIds.has(key)) {
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

function getAdmonitionType(state: Archon.Backups.v1.BackupState): 'info' | 'critical' | 'success' {
	if (state === 'failed') return 'critical'
	if (state === 'done') return 'success'
	return 'info'
}

function getIcon(state: Archon.Backups.v1.BackupState) {
	if (state === 'failed') return TriangleAlertIcon
	if (state === 'done') return CheckCircleIcon
	return InfoIcon
}

function getButtonColor(state: Archon.Backups.v1.BackupState): 'red' | 'green' | 'blue' {
	if (state === 'failed') return 'red'
	if (state === 'done') return 'green'
	return 'blue'
}

function isQueued(item: AdmonitionEntry) {
	return item.state === 'ongoing' && item.progress === 0
}

function isInProgress(item: AdmonitionEntry) {
	return item.state === 'ongoing' && item.progress > 0
}

function getTitle(item: AdmonitionEntry) {
	if (item.type === 'create') {
		if (isQueued(item)) return formatMessage(messages.backupQueuedTitle)
		if (isInProgress(item)) return formatMessage(messages.creatingBackupTitle)
		if (item.state === 'failed') return formatMessage(messages.backupFailedTitle)
	}
	if (isQueued(item)) return formatMessage(messages.restoreQueuedTitle)
	if (isInProgress(item)) return formatMessage(messages.restoringBackupTitle)
	if (item.state === 'done') return formatMessage(messages.restoreSuccessfulTitle)
	if (item.state === 'failed') return formatMessage(messages.restoreFailedTitle)
	return ''
}

function getDescription(item: AdmonitionEntry) {
	const backupName = item.name ?? formatMessage(messages.fallbackName)
	if (item.type === 'create') {
		if (isQueued(item)) return formatMessage(messages.backupQueuedDescription, { backupName })
		if (isInProgress(item)) return formatMessage(messages.creatingBackupDescription, { backupName })
		if (item.state === 'failed')
			return formatMessage(messages.backupFailedDescription, { backupName })
	}
	if (isQueued(item)) return formatMessage(messages.restoreQueuedDescription, { backupName })
	if (isInProgress(item)) return formatMessage(messages.restoringBackupDescription, { backupName })
	if (item.state === 'done')
		return formatMessage(messages.restoreSuccessfulDescription, { backupName })
	if (item.state === 'failed')
		return formatMessage(messages.restoreFailedDescription, { backupName })
	return ''
}

const messages = defineMessages({
	fallbackName: {
		id: 'servers.backups.admonition.fallback-name',
		defaultMessage: 'Your backup',
	},
	backupQueuedTitle: {
		id: 'servers.backups.admonition.backup-queued.title',
		defaultMessage: 'Backup queued',
	},
	backupQueuedDescription: {
		id: 'servers.backups.admonition.backup-queued.description',
		defaultMessage: '{backupName} is queued and will start shortly.',
	},
	creatingBackupTitle: {
		id: 'servers.backups.admonition.creating-backup.title',
		defaultMessage: 'Creating backup',
	},
	creatingBackupDescription: {
		id: 'servers.backups.admonition.creating-backup.description',
		defaultMessage:
			'Saving world data and server configuration for {backupName}. This can take a few minutes.',
	},
	backupFailedTitle: {
		id: 'servers.backups.admonition.backup-failed.title',
		defaultMessage: 'Backup failed',
	},
	backupFailedDescription: {
		id: 'servers.backups.admonition.backup-failed.description',
		defaultMessage:
			'Something went wrong while creating {backupName}. Please try again or contact support if the issue continues.',
	},
	restoreQueuedTitle: {
		id: 'servers.backups.admonition.restore-queued.title',
		defaultMessage: 'Restoring from backup queued',
	},
	restoreQueuedDescription: {
		id: 'servers.backups.admonition.restore-queued.description',
		defaultMessage: 'Restoring from {backupName} is queued and will start shortly.',
	},
	restoringBackupTitle: {
		id: 'servers.backups.admonition.restoring-backup.title',
		defaultMessage: 'Restoring from backup',
	},
	restoringBackupDescription: {
		id: 'servers.backups.admonition.restoring-backup.description',
		defaultMessage: 'Restoring your server from {backupName}. This may take a couple of minutes.',
	},
	restoreSuccessfulTitle: {
		id: 'servers.backups.admonition.restore-successful.title',
		defaultMessage: 'Restoring from backup successful',
	},
	restoreSuccessfulDescription: {
		id: 'servers.backups.admonition.restore-successful.description',
		defaultMessage: 'Your server has been restored to {backupName} and is ready to start.',
	},
	restoreFailedTitle: {
		id: 'servers.backups.admonition.restore-failed.title',
		defaultMessage: 'Restoring from backup failed',
	},
	restoreFailedDescription: {
		id: 'servers.backups.admonition.restore-failed.description',
		defaultMessage:
			'Something went wrong while restoring from {backupName}. Please try again or contact support if the issue continues.',
	},
})
</script>

<template>
	<TransitionGroup
		v-if="admonitions.length > 0"
		name="backup-admonition"
		tag="div"
		class="flex flex-col gap-3"
	>
		<Admonition v-for="item in admonitions" :key="item.key" :type="getAdmonitionType(item.state)">
			<template #icon="{ iconClass }">
				<component :is="getIcon(item.state)" :class="iconClass" />
			</template>
			<template #header>
				<div class="flex items-center gap-2">
					<span>{{ getTitle(item) }}</span>
					<div v-if="item.createdAt" class="flex items-center gap-1.5 text-secondary">
						<ClockIcon class="size-4" />
						<span class="font-medium">{{ relativeTime(item.createdAt) }}</span>
					</div>
				</div>
			</template>
			{{ getDescription(item) }}
			<template #top-right-actions>
				<ButtonStyled v-if="isQueued(item) || isInProgress(item)" type="outlined" color="blue">
					<button class="!border" @click="handleCancel(item.backupId)">
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="item.state === 'failed'" color="red">
					<button @click="handleRetry(item.backupId, item.key)">
						<RotateCounterClockwiseIcon class="size-5" />
						{{ formatMessage(commonMessages.retryButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled
					v-if="item.state === 'failed' || item.state === 'done'"
					circular
					type="transparent"
					hover-color-fill="background"
					:color="getButtonColor(item.state)"
				>
					<button @click="handleDismiss(item.key)">
						<XIcon />
					</button>
				</ButtonStyled>
			</template>
			<template v-if="isInProgress(item)" #progress>
				<div class="pl-9">
					<ProgressBar
						:progress="item.progress"
						color="blue"
						:waiting="item.progress === 0"
						full-width
					/>
				</div>
			</template>
		</Admonition>
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
