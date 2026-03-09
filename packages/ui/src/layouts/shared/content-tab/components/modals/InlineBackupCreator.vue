<template>
	<div class="flex flex-col gap-3">
		<span class="text-primary">
			{{ formatMessage(messages.warningBody, { type: backup.available ? 'server' : 'instance' }) }}
		</span>

		<div v-if="backup.available" class="overflow-hidden">
			<!-- Create backup button -->
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-14"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-14"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="!backup.isBackingUp.value && !backup.backupComplete.value && !backup.backupFailed.value"
				>
					<ButtonStyled type="outlined">
						<button
							class="!border !border-surface-5 !shadow-none"
							@click="backup.startBackup()"
						>
							<SaveIcon class="size-5" />
							{{ formatMessage(messages.createBackup) }}
						</button>
					</ButtonStyled>
				</div>
			</Transition>

			<!-- Progress bar -->
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-14"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-14"
				leave-to-class="opacity-0 max-h-0"
			>
				<div v-if="backup.isBackingUp.value">
					<ProgressBar
						:progress="0"
						color="brand"
						full-width
						waiting
						:label="formatMessage(messages.backingUp)"
						label-class="text-sm font-medium text-secondary"
					/>
				</div>
			</Transition>

			<!-- Success -->
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-14"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-14"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="backup.backupComplete.value"
					class="flex items-center gap-1.5 text-sm font-medium text-green"
				>
					<CheckCircleIcon class="size-5" />
					{{ formatMessage(messages.backupComplete) }}
				</div>
			</Transition>

			<!-- Failed -->
			<Transition
				enter-active-class="transition-all duration-300 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-14"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-14"
				leave-to-class="opacity-0 max-h-0"
			>
				<div v-if="backup.backupFailed.value" class="text-sm text-red">
					{{ formatMessage(messages.backupFailed) }}
				</div>
			</Transition>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CheckCircleIcon, SaveIcon } from '@modrinth/assets'
import { watch } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ProgressBar from '#ui/components/base/ProgressBar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { useInlineBackup } from '../../composables/use-inline-backup'

const props = defineProps<{
	backupName: string
}>()

const emit = defineEmits<{
	(e: 'update:disableClose' | 'update:buttonsDisabled', value: boolean): void
}>()

const { formatMessage } = useVIntl()

const backup = useInlineBackup(() => props.backupName)

watch(
	() => backup.isBackingUp.value,
	(backing) => {
		emit('update:disableClose', backing)
		emit('update:buttonsDisabled', backing)
	},
)

const messages = defineMessages({
	warningBody: {
		id: 'content.inline-backup.warning-body',
		defaultMessage:
			'We recommend creating a backup before proceeding so you can restore your {type, select, server {world} other {worlds}} if anything breaks.',
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
})
</script>
