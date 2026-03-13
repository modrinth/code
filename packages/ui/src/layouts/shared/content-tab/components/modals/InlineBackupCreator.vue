<template>
	<div class="flex flex-col gap-3">
		<span class="text-primary">
			{{ formatMessage(messages.warningBody, { type: backup.isServer ? 'server' : 'instance' }) }}
		</span>

		<div v-if="backup.available" class="flex items-center gap-2">
			<!-- Button / Loading state -->
			<ButtonStyled v-if="!backup.backupComplete.value && !backup.backupFailed.value">
				<button
					v-tooltip="
						backup.externalBackupInProgress.value
							? formatMessage(messages.backupInProgress)
							: undefined
					"
					class="!shadow-none"
					:disabled="backup.isBackingUp.value || backup.externalBackupInProgress.value"
					@click="backup.startBackup()"
				>
					<SpinnerIcon v-if="backup.isBackingUp.value" class="size-5 animate-spin" />
					<PlusIcon v-else class="size-5" />
					{{ formatMessage(backup.isBackingUp.value ? messages.backingUp : messages.createBackup) }}
				</button>
			</ButtonStyled>

			<!-- Success -->
			<div
				v-else-if="backup.backupComplete.value"
				class="flex items-center gap-1.5 text-sm font-medium text-green"
			>
				<CheckCircleIcon class="size-5" />
				{{ formatMessage(messages.backupComplete) }}
			</div>

			<!-- Failed -->
			<div v-else-if="backup.backupFailed.value" class="text-sm text-red">
				{{ formatMessage(messages.backupFailed) }}
			</div>

			<TriangleAlertIcon
				v-if="backup.isServer"
				v-tooltip="formatMessage(messages.backupTakesAWhile)"
				class="size-5 shrink-0 text-brand-orange hover:brightness-110"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, PlusIcon, SpinnerIcon, TriangleAlertIcon } from '@modrinth/assets'
import { watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { useInlineBackup } from '../../composables/use-inline-backup'

const props = defineProps<{
	backupName: string
}>()

const emit = defineEmits<{
	(e: 'update:buttonsDisabled', value: boolean): void
}>()

const { formatMessage } = useVIntl()

const backup = useInlineBackup(() => props.backupName)

watch(
	() => backup.isBackingUp.value,
	(backing) => {
		emit('update:buttonsDisabled', backing)
	},
)

defineExpose({
	cancelBackup: backup.cancelBackup,
	isBackingUp: backup.isBackingUp,
})

const messages = defineMessages({
	warningBody: {
		id: 'content.inline-backup.warning-body',
		defaultMessage:
			'We recommend creating a backup before proceeding so you can restore your {type, select, server {world} other {instance}} if anything breaks.',
	},
	createBackup: {
		id: 'content.inline-backup.create-backup',
		defaultMessage: 'Create backup',
	},
	backingUp: {
		id: 'content.inline-backup.backing-up',
		defaultMessage: 'Creating backup...',
	},
	backupComplete: {
		id: 'content.inline-backup.backup-complete',
		defaultMessage: 'Backup created successfully',
	},
	backupFailed: {
		id: 'content.inline-backup.backup-failed',
		defaultMessage: 'Backup creation failed. You can still proceed.',
	},
	backupTakesAWhile: {
		id: 'content.inline-backup.backup-takes-a-while',
		defaultMessage:
			'Creating a backup may take several minutes depending on the size of your server.',
	},
	backupInProgress: {
		id: 'content.inline-backup.backup-in-progress',
		defaultMessage:
			"A backup is in progress, it's recommended to wait for it to finish before performing this action.",
	},
})
</script>
