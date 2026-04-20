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
import { useNow } from '@vueuse/core'
import { computed, reactive } from 'vue'

import { useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { useServerBackupsQueue } from '../../../composables/server-backups-queue'
import { injectModrinthClient, injectModrinthServerContext } from '../../../providers'
import { commonMessages } from '../../../utils'
import Admonition from '../../base/Admonition.vue'
import ButtonStyled from '../../base/ButtonStyled.vue'
import ProgressBar from '../../base/ProgressBar.vue'

const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime()
const now = useNow({ interval: 1000 })

function formatCreatedRelative(createdAt: string | undefined) {
	void now.value // tracks the useNow tick so the string re-renders every second
	return relativeTime(createdAt)
}
const client = injectModrinthClient()
const { serverId, worldId } = injectModrinthServerContext()
const { activeOperations, backups, progressFor, invalidate } = useServerBackupsQueue(
	computed(() => serverId),
	worldId,
)

type AdmonitionDisplayState = 'ongoing' | Archon.BackupsQueue.v1.BackupQueueState

type AdmonitionEntry = {
	key: string
	backupId: string
	type: 'create' | 'restore'
	state: AdmonitionDisplayState
	progress: number
	operationId: number | null
	syntheticLegacy: boolean
	name?: string
	createdAt?: string
	error?: string | null
}

const dismissedIds = reactive(new Set<string>())
const cancellingIds = reactive(new Set<string>())

const admonitions = computed<AdmonitionEntry[]>(() => {
	const result: AdmonitionEntry[] = []
	const backupById = new Map(backups.value.map((b) => [b.id, b]))

	for (const op of activeOperations.value) {
		const key = `${op.backup_id}:${op.operation_type}:${op.operation_id ?? 'legacy'}`
		if (dismissedIds.has(key)) continue
		const backup = backupById.get(op.backup_id)
		const rawProgress = progressFor(op.backup_id, op.operation_type) ?? 0
		result.push({
			key,
			backupId: op.backup_id,
			type: op.operation_type,
			state: 'ongoing',
			progress: rawProgress,
			operationId: op.operation_id ?? null,
			syntheticLegacy: op.synthetic_legacy,
			name: backup?.name,
			createdAt: backup?.created_at,
		})
	}

	for (const backup of backups.value) {
		const last = backup.history[0]
		if (!last || !last.should_prompt) continue
		if (last.state === 'pending' || last.state === 'ongoing') continue
		const key = `${backup.id}:${last.operation_type}:${last.operation_id ?? 'legacy'}`
		if (dismissedIds.has(key)) continue
		if (result.some((r) => r.key === key)) continue
		result.push({
			key,
			backupId: backup.id,
			type: last.operation_type,
			state: last.state,
			progress: 0,
			operationId: last.operation_id ?? null,
			syntheticLegacy: last.synthetic_legacy,
			name: backup.name,
			createdAt: backup.created_at,
			error: last.error ?? null,
		})
	}

	return result
})

async function handleDismiss(item: AdmonitionEntry) {
	dismissedIds.add(item.key)
	if (item.syntheticLegacy || item.operationId == null) {
		await invalidate()
		return
	}
	try {
		if (item.type === 'create') {
			await client.archon.backups_queue_v1.ackCreate(serverId, worldId.value!, item.operationId)
		} else {
			await client.archon.backups_queue_v1.ackRestore(serverId, worldId.value!, item.operationId)
		}
	} catch (err) {
		dismissedIds.delete(item.key)
		console.error('Failed to acknowledge backup operation', err)
	} finally {
		await invalidate()
	}
}

async function handleCancel(item: AdmonitionEntry) {
	if (cancellingIds.has(item.key)) return
	cancellingIds.add(item.key)
	try {
		await client.archon.backups_v1.delete(serverId, worldId.value!, item.backupId)
		await invalidate()
	} catch (err) {
		cancellingIds.delete(item.key)
		throw err
	}
}

async function handleRetry(item: AdmonitionEntry) {
	await client.archon.backups_queue_v1.retry(serverId, worldId.value!, item.backupId)
	dismissedIds.add(item.key)
	await invalidate()
}

function getAdmonitionType(state: AdmonitionDisplayState): 'info' | 'critical' | 'success' {
	if (state === 'failed' || state === 'timed_out') return 'critical'
	if (state === 'completed') return 'success'
	return 'info'
}

function getIcon(state: AdmonitionDisplayState) {
	if (state === 'failed' || state === 'timed_out') return TriangleAlertIcon
	if (state === 'completed') return CheckCircleIcon
	return InfoIcon
}

function getButtonColor(state: AdmonitionDisplayState): 'red' | 'green' | 'blue' {
	if (state === 'failed' || state === 'timed_out') return 'red'
	if (state === 'completed') return 'green'
	return 'blue'
}

function isQueued(item: AdmonitionEntry) {
	return item.state === 'ongoing' && item.progress === 0
}

function isInProgress(item: AdmonitionEntry) {
	return item.state === 'ongoing' && item.progress > 0
}

function isTerminal(item: AdmonitionEntry) {
	return item.state !== 'ongoing'
}

function canRetry(item: AdmonitionEntry) {
	return item.state === 'failed' || item.state === 'timed_out'
}

function hasErrorDetail(item: AdmonitionEntry) {
	return !!item.error && (item.state === 'failed' || item.state === 'timed_out')
}

function getTitle(item: AdmonitionEntry) {
	if (item.type === 'create') {
		if (isQueued(item)) return formatMessage(messages.backupQueuedTitle)
		if (isInProgress(item)) return formatMessage(messages.creatingBackupTitle)
		if (item.state === 'failed') return formatMessage(messages.backupFailedTitle)
		if (item.state === 'timed_out') return formatMessage(messages.backupTimedOutTitle)
		if (item.state === 'cancelled') return formatMessage(messages.backupCancelledTitle)
		if (item.state === 'completed') return formatMessage(messages.backupCompletedTitle)
	}
	if (isQueued(item)) return formatMessage(messages.restoreQueuedTitle)
	if (isInProgress(item)) return formatMessage(messages.restoringBackupTitle)
	if (item.state === 'completed') return formatMessage(messages.restoreSuccessfulTitle)
	if (item.state === 'failed') return formatMessage(messages.restoreFailedTitle)
	if (item.state === 'timed_out') return formatMessage(messages.restoreTimedOutTitle)
	if (item.state === 'cancelled') return formatMessage(messages.restoreCancelledTitle)
	return ''
}

function getDescription(item: AdmonitionEntry) {
	const backupName = item.name ?? formatMessage(messages.fallbackName)
	if (item.type === 'create') {
		if (isQueued(item)) return formatMessage(messages.backupQueuedDescription, { backupName })
		if (isInProgress(item)) return formatMessage(messages.creatingBackupDescription, { backupName })
		if (item.state === 'failed')
			return formatMessage(messages.backupFailedDescription, { backupName })
		if (item.state === 'timed_out')
			return formatMessage(messages.backupTimedOutDescription, { backupName })
		if (item.state === 'cancelled')
			return formatMessage(messages.backupCancelledDescription, { backupName })
		if (item.state === 'completed')
			return formatMessage(messages.backupCompletedDescription, { backupName })
	}
	if (isQueued(item)) return formatMessage(messages.restoreQueuedDescription, { backupName })
	if (isInProgress(item)) return formatMessage(messages.restoringBackupDescription, { backupName })
	if (item.state === 'completed')
		return formatMessage(messages.restoreSuccessfulDescription, { backupName })
	if (item.state === 'failed')
		return formatMessage(messages.restoreFailedDescription, { backupName })
	if (item.state === 'timed_out')
		return formatMessage(messages.restoreTimedOutDescription, { backupName })
	if (item.state === 'cancelled')
		return formatMessage(messages.restoreCancelledDescription, { backupName })
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
	backupTimedOutTitle: {
		id: 'servers.backups.admonition.backup-timed-out.title',
		defaultMessage: 'Backup timed out',
	},
	backupTimedOutDescription: {
		id: 'servers.backups.admonition.backup-timed-out.description',
		defaultMessage:
			'Creating {backupName} timed out. You can try again or contact support if the issue continues.',
	},
	backupCancelledTitle: {
		id: 'servers.backups.admonition.backup-cancelled.title',
		defaultMessage: 'Backup cancelled',
	},
	backupCancelledDescription: {
		id: 'servers.backups.admonition.backup-cancelled.description',
		defaultMessage: 'Backup {backupName} was cancelled.',
	},
	backupCompletedTitle: {
		id: 'servers.backups.admonition.backup-completed.title',
		defaultMessage: 'Backup completed',
	},
	backupCompletedDescription: {
		id: 'servers.backups.admonition.backup-completed.description',
		defaultMessage: '{backupName} finished successfully.',
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
	restoreTimedOutTitle: {
		id: 'servers.backups.admonition.restore-timed-out.title',
		defaultMessage: 'Restore timed out',
	},
	restoreTimedOutDescription: {
		id: 'servers.backups.admonition.restore-timed-out.description',
		defaultMessage:
			'Restoring from {backupName} timed out. You can try again or contact support if the issue continues.',
	},
	restoreCancelledTitle: {
		id: 'servers.backups.admonition.restore-cancelled.title',
		defaultMessage: 'Restore cancelled',
	},
	restoreCancelledDescription: {
		id: 'servers.backups.admonition.restore-cancelled.description',
		defaultMessage: 'Restoring from {backupName} was cancelled.',
	},
})
</script>

<template>
	<TransitionGroup
		name="backup-admonition"
		tag="div"
		enter-active-class="transition-all duration-300 ease-out overflow-hidden"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-40"
		leave-active-class="transition-all duration-200 ease-in overflow-hidden"
		leave-from-class="opacity-100 max-h-40"
		leave-to-class="opacity-0 max-h-0"
	>
		<Admonition
			v-for="item in admonitions"
			:key="item.key"
			:type="getAdmonitionType(item.state)"
			class="mb-4"
		>
			<template #icon="{ iconClass }">
				<component :is="getIcon(item.state)" :class="iconClass" />
			</template>
			<template #header>
				<div class="flex items-center gap-2">
					<span>{{ getTitle(item) }}</span>
					<div v-if="item.createdAt" class="flex items-center gap-1.5 text-secondary">
						<ClockIcon class="size-4" />
						<span class="font-medium">{{ formatCreatedRelative(item.createdAt) }}</span>
					</div>
				</div>
			</template>
			<div class="flex flex-col gap-2">
				<span>{{ getDescription(item) }}</span>
				<span
					v-if="hasErrorDetail(item)"
					class="break-all font-mono text-sm text-secondary"
				>
					{{ item.error }}
				</span>
			</div>
			<template #top-right-actions>
				<ButtonStyled v-if="isQueued(item) || isInProgress(item)" type="outlined" color="blue">
					<button
						class="!border"
						:disabled="cancellingIds.has(item.key)"
						@click="handleCancel(item)"
					>
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="canRetry(item)" color="red">
					<button @click="handleRetry(item)">
						<RotateCounterClockwiseIcon class="size-5" />
						{{ formatMessage(commonMessages.retryButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled
					v-if="isTerminal(item)"
					circular
					type="transparent"
					hover-color-fill="background"
					:color="getButtonColor(item.state)"
				>
					<button @click="handleDismiss(item)">
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
.backup-admonition-move {
	transition: transform 300ms ease-in-out;
}
</style>
