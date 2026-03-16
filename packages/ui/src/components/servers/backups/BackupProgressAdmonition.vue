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
import { computed } from 'vue'

import { useRelativeTime } from '../../../composables'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import { commonMessages } from '../../../utils'
import ButtonStyled from '../../base/ButtonStyled.vue'
import ProgressBar from '../../base/ProgressBar.vue'

const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime()

const props = withDefaults(
	defineProps<{
		type: 'create' | 'restore'
		state: Archon.Backups.v1.BackupState
		progress: number
		backupName?: string
		createdAt?: string
	}>(),
	{
		backupName: undefined,
		createdAt: undefined,
	},
)

const emit = defineEmits<{
	(e: 'cancel' | 'retry' | 'dismiss'): void
}>()

const isQueued = computed(() => props.state === 'ongoing' && props.progress === 0)
const isInProgress = computed(() => props.state === 'ongoing' && props.progress > 0)
const isFailed = computed(() => props.state === 'failed')
const isSuccess = computed(() => props.state === 'done')

const showCancel = computed(() => isQueued.value || isInProgress.value)
const showRetry = computed(() => isFailed.value)
const showDismiss = computed(() => isFailed.value || isSuccess.value)
const showProgress = computed(() => isInProgress.value)

const colorClasses = computed(() => {
	if (isFailed.value) return 'border-brand-red bg-bg-red'
	if (isSuccess.value) return 'border-brand-green bg-bg-green'
	return 'border-brand-blue bg-bg-blue'
})

const icon = computed(() => {
	if (isFailed.value) return TriangleAlertIcon
	if (isSuccess.value) return CheckCircleIcon
	return InfoIcon
})

const iconClass = computed(() => {
	if (isFailed.value) return 'text-brand-red'
	if (isSuccess.value) return 'text-brand-green'
	return 'text-brand-blue'
})

const buttonColor = computed<'red' | 'green' | 'blue'>(() => {
	if (isFailed.value) return 'red'
	if (isSuccess.value) return 'green'
	return 'blue'
})

const name = computed(() => props.backupName ?? formatMessage(messages.fallbackName))

const title = computed(() => {
	if (props.type === 'create') {
		if (isQueued.value) return formatMessage(messages.backupQueuedTitle)
		if (isInProgress.value) return formatMessage(messages.creatingBackupTitle)
		if (isFailed.value) return formatMessage(messages.backupFailedTitle)
	}
	if (isQueued.value) return formatMessage(messages.restoreQueuedTitle)
	if (isInProgress.value) return formatMessage(messages.restoringBackupTitle)
	if (isSuccess.value) return formatMessage(messages.restoreSuccessfulTitle)
	if (isFailed.value) return formatMessage(messages.restoreFailedTitle)
	return ''
})

const description = computed(() => {
	if (props.type === 'create') {
		if (isQueued.value)
			return formatMessage(messages.backupQueuedDescription, { backupName: name.value })
		if (isInProgress.value)
			return formatMessage(messages.creatingBackupDescription, { backupName: name.value })
		if (isFailed.value)
			return formatMessage(messages.backupFailedDescription, { backupName: name.value })
	}
	if (isQueued.value)
		return formatMessage(messages.restoreQueuedDescription, { backupName: name.value })
	if (isInProgress.value)
		return formatMessage(messages.restoringBackupDescription, { backupName: name.value })
	if (isSuccess.value)
		return formatMessage(messages.restoreSuccessfulDescription, { backupName: name.value })
	if (isFailed.value)
		return formatMessage(messages.restoreFailedDescription, { backupName: name.value })
	return ''
})

const messages = defineMessages({
	fallbackName: {
		id: 'servers.backups.admonition.fallback-name',
		defaultMessage: 'your backup',
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
	<div :class="['flex flex-col rounded-2xl border border-solid p-4', colorClasses]">
		<div class="flex items-start gap-2">
			<div class="flex flex-1 gap-3 items-start">
				<component :is="icon" :class="['size-6 shrink-0', iconClass]" />
				<div class="flex flex-col gap-2">
					<div class="flex items-center gap-2">
						<span class="font-semibold text-contrast">{{ title }}</span>
						<div v-if="createdAt" class="flex items-center gap-1.5 text-secondary">
							<ClockIcon class="size-4" />
							<span class="font-medium">{{ relativeTime(createdAt) }}</span>
						</div>
					</div>
					<span class="text-contrast opacity-80">{{ description }}</span>
				</div>
			</div>
			<div class="flex shrink-0 items-center gap-2">
				<ButtonStyled v-if="showCancel" type="outlined" color="blue">
					<button class="!border" @click="emit('cancel')">
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled v-if="showRetry" color="red">
					<button @click="emit('retry')">
						<RotateCounterClockwiseIcon class="size-5" />
						{{ formatMessage(commonMessages.retryButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled
					v-if="showDismiss"
					circular
					type="transparent"
					hover-color-fill="background"
					:color="buttonColor"
				>
					<button @click="emit('dismiss')">
						<XIcon />
					</button>
				</ButtonStyled>
			</div>
		</div>
		<div v-if="showProgress" class="mt-4 pl-9">
			<ProgressBar :progress="progress" color="blue" :waiting="progress === 0" full-width />
		</div>
	</div>
</template>
