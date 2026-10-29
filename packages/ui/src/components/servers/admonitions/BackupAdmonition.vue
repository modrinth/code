<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	CheckCircleIcon,
	InfoIcon,
	RotateCounterClockwiseIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
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
	timestamp?: string
	error?: string | null
	migration?: { restoring: boolean }
}

defineProps<{
	item: BackupAdmonitionEntry
	dismissible: boolean
	cancelling: boolean
	canManageBackups?: boolean
	permissionDeniedMessage?: string
}>()

defineEmits<{
	dismiss: []
	retry: []
	cancel: []
}>()

const { formatMessage } = useVIntl()

type UiPhase = 'queued' | 'in_progress' | 'failed' | 'timed_out' | 'cancelled' | 'completed'

function resolveUiPhase(item: BackupAdmonitionEntry): UiPhase | null {
	switch (item.state) {
		case 'pending':
			return 'queued'
		case 'ongoing':
			return 'in_progress'
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
	return item.state !== 'pending' && item.state !== 'ongoing'
}

function canRetry(item: BackupAdmonitionEntry) {
	return !item.migration && (item.state === 'failed' || item.state === 'timed_out')
}

function canCancel(item: BackupAdmonitionEntry) {
	return (isQueued(item) || isInProgress(item)) && !item.migration?.restoring
}

function hasErrorDetail(item: BackupAdmonitionEntry) {
	return !!item.error && (item.state === 'failed' || item.state === 'timed_out')
}

const messages = defineMessages({
	migrationQueuedTitle: {
		id: 'servers.backups.admonition.migration-queued.title',
		defaultMessage: 'Server update queued',
	},
	migrationQueuedDescription: {
		id: 'servers.backups.admonition.migration-queued.description',
		defaultMessage: 'Your server will be backed up and its content updated shortly.',
	},
	migrationBackingUpTitle: {
		id: 'servers.backups.admonition.migration-backing-up.title',
		defaultMessage: 'Backing up your server',
	},
	migrationBackingUpDescription: {
		id: 'servers.backups.admonition.migration-backing-up.description',
		defaultMessage: 'Creating {backupName} before updating your content.',
	},
	migrationUpdatingTitle: {
		id: 'servers.backups.admonition.migration-updating.title',
		defaultMessage: 'Updating server content',
	},
	migrationUpdatingDescription: {
		id: 'servers.backups.admonition.migration-updating.description',
		defaultMessage:
			'Reinstalling your content and enabling it for the server, players, or both. This may take a couple of minutes.',
	},
	migrationBackupFailedDescription: {
		id: 'servers.backups.admonition.migration-backup-failed.description',
		defaultMessage:
			'Something went wrong while creating {backupName}, so your content was left untouched. Please try again from the Play tab.',
	},
	migrationFailedTitle: {
		id: 'servers.backups.admonition.migration-failed.title',
		defaultMessage: 'Server update failed',
	},
	migrationFailedDescription: {
		id: 'servers.backups.admonition.migration-failed.description',
		defaultMessage:
			'Something went wrong while updating your content. You can restore {backupName} from your backups or contact support.',
	},
	migrationCancelledTitle: {
		id: 'servers.backups.admonition.migration-cancelled.title',
		defaultMessage: 'Server update cancelled',
	},
	migrationCancelledDescription: {
		id: 'servers.backups.admonition.migration-cancelled.description',
		defaultMessage: 'Your server’s content was left untouched.',
	},
	migrationCompletedTitle: {
		id: 'servers.backups.admonition.migration-completed.title',
		defaultMessage: 'Server content updated',
	},
	migrationCompletedDescription: {
		id: 'servers.backups.admonition.migration-completed.description',
		defaultMessage: 'Your server is ready to share with friends.',
	},
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
		defaultMessage: 'Backup finished',
	},
	backupCompletedDescription: {
		id: 'servers.backups.admonition.backup-completed.description',
		defaultMessage: '{backupName} finished successfully.',
	},
	restoreQueuedTitle: {
		id: 'servers.backups.admonition.restore-queued.title',
		defaultMessage: 'Restore queued',
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
		defaultMessage: 'Restore finished',
	},
	restoreSuccessfulDescription: {
		id: 'servers.backups.admonition.restore-successful.description',
		defaultMessage: 'Your server has been restored to {backupName} and is ready to start.',
	},
	restoreFailedTitle: {
		id: 'servers.backups.admonition.restore-failed.title',
		defaultMessage: 'Restore failed',
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

function migrationMessages(
	phase: UiPhase,
	restoring: boolean,
): [MessageDescriptor, MessageDescriptor] {
	switch (phase) {
		case 'queued':
			return [messages.migrationQueuedTitle, messages.migrationQueuedDescription]
		case 'in_progress':
			return restoring
				? [messages.migrationUpdatingTitle, messages.migrationUpdatingDescription]
				: [messages.migrationBackingUpTitle, messages.migrationBackingUpDescription]
		case 'failed':
		case 'timed_out':
			return restoring
				? [messages.migrationFailedTitle, messages.migrationFailedDescription]
				: [messages.migrationFailedTitle, messages.migrationBackupFailedDescription]
		case 'cancelled':
			return [messages.migrationCancelledTitle, messages.migrationCancelledDescription]
		case 'completed':
			return [messages.migrationCompletedTitle, messages.migrationCompletedDescription]
	}
}

function getTitle(item: BackupAdmonitionEntry): string {
	const phase = resolveUiPhase(item)
	if (phase == null) return ''
	if (item.migration) return formatMessage(migrationMessages(phase, item.migration.restoring)[0])
	const table = item.type === 'create' ? createTitles : restoreTitles
	return formatMessage(table[phase])
}

function getDescription(item: BackupAdmonitionEntry): string {
	const phase = resolveUiPhase(item)
	if (phase == null) return ''
	const backupName = item.name ?? formatMessage(messages.fallbackName)
	if (item.migration) {
		return formatMessage(migrationMessages(phase, item.migration.restoring)[1], { backupName })
	}
	const table = item.type === 'create' ? createDescriptions : restoreDescriptions
	return formatMessage(table[phase], { backupName })
}
</script>

<template>
	<Admonition
		:type="getAdmonitionType(item.state)"
		:header="getTitle(item)"
		:timestamp="item.timestamp"
		:dismissible="dismissible && isTerminal(item)"
		:progress="isInProgress(item) ? item.progress : undefined"
		progress-color="blue"
		:waiting="isInProgress(item) && item.progress === 0"
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
			<Button
				v-if="canCancel(item)"
				v-tooltip="canManageBackups === false ? permissionDeniedMessage : undefined"
				type="outlined"
				class="!border !text-blue [&>svg]:!text-blue !shadow-[inset_0_0_0_1px_var(--color-blue)]"
				native-type="button"
				:disabled="cancelling || canManageBackups === false"
				@click="$emit('cancel')"
			>
				{{ formatMessage(commonMessages.cancelButton) }}
			</Button>
			<Button
				v-if="canRetry(item)"
				v-tooltip="canManageBackups === false ? permissionDeniedMessage : undefined"
				type="outlined"
				class="!border !text-red [&>svg]:!text-red !shadow-[inset_0_0_0_1px_var(--color-red)]"
				native-type="button"
				:disabled="canManageBackups === false"
				@click="$emit('retry')"
			>
				<RotateCounterClockwiseIcon class="size-5" />
				{{ formatMessage(commonMessages.retryButton) }}
			</Button>
		</template>
	</Admonition>
</template>
