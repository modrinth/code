<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	InfoIcon,
	RotateCounterClockwiseIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'

import Admonition from '#ui/components/base/Admonition.vue'
import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
import type { MessageDescriptor } from '#ui/composables/i18n'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

export type AdmonitionDisplayState = 'ongoing' | Archon.BackupsQueue.v1.BackupQueueState

export type BackupAdmonitionEntry = {
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

defineProps<{
	item: BackupAdmonitionEntry
	dismissible: boolean
	cancelling: boolean
}>()

defineEmits<{
	dismiss: []
	retry: []
	cancel: []
}>()

const { formatMessage } = useVIntl()

type UiPhase = 'queued' | 'in_progress' | 'failed' | 'timed_out' | 'cancelled' | 'completed'

function resolveUiPhase(item: BackupAdmonitionEntry): UiPhase | null {
	if (item.state === 'ongoing') {
		return item.progress > 0 ? 'in_progress' : 'queued'
	}
	switch (item.state) {
		case 'failed':
		case 'timed_out':
		case 'cancelled':
		case 'completed':
			return item.state
		default:
			return null
	}
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

function isQueued(item: BackupAdmonitionEntry) {
	return resolveUiPhase(item) === 'queued'
}

function isInProgress(item: BackupAdmonitionEntry) {
	return resolveUiPhase(item) === 'in_progress'
}

function isTerminal(item: BackupAdmonitionEntry) {
	return item.state !== 'ongoing'
}

function canRetry(item: BackupAdmonitionEntry) {
	return item.state === 'failed' || item.state === 'timed_out'
}

function hasErrorDetail(item: BackupAdmonitionEntry) {
	return !!item.error && (item.state === 'failed' || item.state === 'timed_out')
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

const createTitles: Record<UiPhase, MessageDescriptor> = {
	queued: messages.backupQueuedTitle,
	in_progress: messages.creatingBackupTitle,
	failed: messages.backupFailedTitle,
	timed_out: messages.backupTimedOutTitle,
	cancelled: messages.backupCancelledTitle,
	completed: messages.backupCompletedTitle,
}

const restoreTitles: Record<UiPhase, MessageDescriptor> = {
	queued: messages.restoreQueuedTitle,
	in_progress: messages.restoringBackupTitle,
	failed: messages.restoreFailedTitle,
	timed_out: messages.restoreTimedOutTitle,
	cancelled: messages.restoreCancelledTitle,
	completed: messages.restoreSuccessfulTitle,
}

const createDescriptions: Record<UiPhase, MessageDescriptor> = {
	queued: messages.backupQueuedDescription,
	in_progress: messages.creatingBackupDescription,
	failed: messages.backupFailedDescription,
	timed_out: messages.backupTimedOutDescription,
	cancelled: messages.backupCancelledDescription,
	completed: messages.backupCompletedDescription,
}

const restoreDescriptions: Record<UiPhase, MessageDescriptor> = {
	queued: messages.restoreQueuedDescription,
	in_progress: messages.restoringBackupDescription,
	failed: messages.restoreFailedDescription,
	timed_out: messages.restoreTimedOutDescription,
	cancelled: messages.restoreCancelledDescription,
	completed: messages.restoreSuccessfulDescription,
}

function getTitle(item: BackupAdmonitionEntry): string {
	const phase = resolveUiPhase(item)
	if (phase == null) return ''
	const table = item.type === 'create' ? createTitles : restoreTitles
	return formatMessage(table[phase])
}

function getDescription(item: BackupAdmonitionEntry): string {
	const phase = resolveUiPhase(item)
	if (phase == null) return ''
	const table = item.type === 'create' ? createDescriptions : restoreDescriptions
	const backupName = item.name ?? formatMessage(messages.fallbackName)
	return formatMessage(table[phase], { backupName })
}
</script>

<template>
	<Admonition
		:type="getAdmonitionType(item.state)"
		:header="getTitle(item)"
		:timestamp="item.createdAt"
		:dismissible="dismissible && isTerminal(item)"
		@dismiss="$emit('dismiss')"
	>
		<template #icon="{ iconClass }">
			<component :is="getIcon(item.state)" :class="iconClass" />
		</template>
		<div class="flex flex-col gap-2">
			<span>{{ getDescription(item) }}</span>
			<span v-if="hasErrorDetail(item)" class="break-all font-mono text-sm text-secondary">
				{{ item.error }}
			</span>
		</div>
		<template #top-right-actions>
			<ButtonStyled v-if="isQueued(item) || isInProgress(item)" type="outlined" color="blue">
				<button class="!border" type="button" :disabled="cancelling" @click="$emit('cancel')">
					{{ formatMessage(commonMessages.cancelButton) }}
				</button>
			</ButtonStyled>
			<ButtonStyled v-if="canRetry(item)" color="red">
				<button type="button" @click="$emit('retry')">
					<RotateCounterClockwiseIcon class="size-5" />
					{{ formatMessage(commonMessages.retryButton) }}
				</button>
			</ButtonStyled>
		</template>
		<template v-if="isInProgress(item)" #progress>
			<ProgressBar
				:progress="item.progress"
				color="blue"
				:waiting="item.progress === 0"
				full-width
			/>
		</template>
	</Admonition>
</template>
