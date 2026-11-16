<template>
	<NewModal ref="modal" :header="formatMessage(messages.zipHeader)">
		<form class="flex flex-col gap-6 md:w-[700px]" @submit.prevent="handleSubmit">
			<!-- URL input -->
			<div class="flex flex-col gap-2.5">
				<div class="text-sm text-secondary">
					{{ formatMessage(messages.zipDescription) }}
				</div>
				<Input
					v-model="url"
					v-tooltip="props.disabled ? props.disabledTooltip : undefined"
					:icon="LinkIcon"
					type="url"
					placeholder="https://www.example.com/.../modpack-name-1.0.2.zip"
					:disabled="submitted || props.disabled"
					:error="touched && !!error"
					autocomplete="off"
					@focus="touched = true"
				/>
				<div v-if="touched && error" class="text-xs text-red">{{ error }}</div>
			</div>

			<!-- Inline backup creator -->
			<InlineBackupCreator
				:backup-name="formatMessage(messages.backupName)"
				hide-shift-click-hint
				@update:buttons-disabled="backupInProgress = $event"
			/>
		</form>

		<template #actions>
			<div class="flex w-full items-center justify-end gap-2">
				<Button type="outlined" native-type="button" @click="hide">
					<XIcon />
					{{
						submitted
							? formatMessage(commonMessages.closeButton)
							: formatMessage(commonMessages.cancelButton)
					}}
				</Button>
				<Button
					v-tooltip="submitTooltip"
					type="colored"
					color="brand"
					:disabled="submitDisabled"
					native-type="submit"
					@click="handleSubmit"
				>
					<SpinnerIcon v-if="submitted" class="animate-spin" />
					<DownloadIcon v-else />
					{{
						submitted
							? formatMessage(commonMessages.installingLabel)
							: formatMessage(messages.installButton)
					}}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { DownloadIcon, LinkIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Input from '#ui/components/base/inputs/Input.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from '../../../content-tab/components/modals/InlineBackupCreator.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()

const props = withDefaults(
	defineProps<{
		disabled?: boolean
		disabledTooltip?: string
	}>(),
	{
		disabled: false,
		disabledTooltip: undefined,
	},
)

const messages = defineMessages({
	zipHeader: {
		id: 'files.zip-url-modal.zip-header',
		defaultMessage: 'Uploading .zip contents from URL',
	},
	zipDescription: {
		id: 'files.zip-url-modal.zip-description',
		defaultMessage: 'Copy and paste the direct download URL of a .zip file.',
	},
	installButton: {
		id: 'files.zip-url-modal.install-button',
		defaultMessage: 'Install',
	},
	errorUrlRequired: {
		id: 'files.zip-url-modal.error-url-required',
		defaultMessage: 'URL is required.',
	},
	errorUrlInvalid: {
		id: 'files.zip-url-modal.error-url-invalid',
		defaultMessage: 'URL must be valid.',
	},
	installFailedTitle: {
		id: 'files.zip-url-modal.install-failed-title',
		defaultMessage: 'Installation failed',
	},
	unknownError: {
		id: 'files.zip-url-modal.unknown-error',
		defaultMessage: 'An unknown error occurred',
	},
	backupName: {
		id: 'files.zip-url-modal.backup-name',
		defaultMessage: 'Before .zip URL upload',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const url = ref('')
const submitted = ref(false)
const touched = ref(false)
const backupInProgress = ref(false)

const trimmedUrl = computed(() => url.value.trim())

const error = computed(() => {
	if (trimmedUrl.value.length === 0) {
		return formatMessage(messages.errorUrlRequired)
	}
	if (!trimmedUrl.value.includes('/')) {
		return formatMessage(messages.errorUrlInvalid)
	}
	return ''
})

const submitDisabled = computed(
	() => submitted.value || props.disabled || !!error.value || backupInProgress.value,
)
const submitTooltip = computed(() => {
	if (props.disabled) return props.disabledTooltip
	return error.value || undefined
})

const handleSubmit = async () => {
	touched.value = true
	if (submitDisabled.value) return

	submitted.value = true
	try {
		await client.kyros.files_v0.extractFile(trimmedUrl.value, true, true)
		await client.kyros.files_v0.extractFile(trimmedUrl.value, true, false)
		hide()
	} catch (err) {
		submitted.value = false
		console.error('Error installing:', err)
		addNotification({
			title: formatMessage(messages.installFailedTitle),
			text: err instanceof Error ? err.message : formatMessage(messages.unknownError),
			type: 'error',
		})
	}
}

const show = () => {
	if (props.disabled) return

	url.value = ''
	submitted.value = false
	touched.value = false
	backupInProgress.value = false
	modal.value?.show()
	nextTick(() => {
		setTimeout(() => {
			modal.value?.$el?.querySelector('input')?.focus()
		}, 100)
	})
}

const hide = () => {
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
