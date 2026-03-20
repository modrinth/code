<template>
	<NewModal
		ref="modal"
		:header="cf ? formatMessage(messages.cfHeader) : formatMessage(messages.zipHeader)"
	>
		<form class="flex flex-col gap-6 md:w-[620px]" @submit.prevent="handleSubmit">
			<!-- CurseForge stepper cards -->
			<div v-if="cf" class="flex gap-4">
				<div
					v-for="(step, i) in steps"
					:key="i"
					class="flex flex-1 flex-col gap-2 overflow-clip rounded-[20px] bg-surface-2 p-3"
				>
					<span
						class="flex size-6 shrink-0 items-center justify-center rounded-full border border-solid border-surface-5 bg-surface-4 text-sm font-medium text-contrast"
					>
						{{ i + 1 }}
					</span>
					<div class="text-sm font-semibold leading-snug text-contrast">
						{{ step.title }}
					</div>
					<div class="text-xs leading-relaxed text-secondary">
						{{ step.description }}
					</div>
				</div>
			</div>

			<!-- URL input -->
			<div class="flex flex-col gap-2.5">
				<label v-if="cf" class="text-base font-semibold text-contrast">{{
					formatMessage(messages.enterLink)
				}}</label>
				<div v-else class="text-sm text-secondary">
					{{ formatMessage(messages.zipDescription) }}
				</div>
				<StyledInput
					v-model="url"
					:icon="LinkIcon"
					type="url"
					:placeholder="
						cf
							? 'https://www.curseforge.com/minecraft/modpacks/.../files/6412259'
							: 'https://www.example.com/.../modpack-name-1.0.2.zip'
					"
					:disabled="submitted"
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
				<ButtonStyled type="outlined">
					<button type="button" class="!border !border-surface-4" @click="hide">
						<XIcon />
						{{
							submitted
								? formatMessage(commonMessages.closeButton)
								: formatMessage(commonMessages.cancelButton)
						}}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						v-tooltip="error"
						:disabled="submitted || !!error || backupInProgress"
						type="submit"
						@click="handleSubmit"
					>
						<SpinnerIcon v-if="submitted" class="animate-spin" />
						<DownloadIcon v-else />
						{{
							submitted
								? formatMessage(commonMessages.installingLabel)
								: formatMessage(messages.installButton)
						}}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	FileTextIcon,
	LinkIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import StyledInput from '#ui/components/base/StyledInput.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'
import { commonMessages } from '#ui/utils/common-messages'

import InlineBackupCreator from '../../../content-tab/components/modals/InlineBackupCreator.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	cfHeader: {
		id: 'files.zip-url-modal.cf-header',
		defaultMessage: 'Install a CurseForge modpack',
	},
	zipHeader: {
		id: 'files.zip-url-modal.zip-header',
		defaultMessage: 'Uploading .zip contents from URL',
	},
	enterLink: {
		id: 'files.zip-url-modal.enter-link',
		defaultMessage: 'Enter link',
	},
	zipDescription: {
		id: 'files.zip-url-modal.zip-description',
		defaultMessage: 'Copy and paste the direct download URL of a .zip file.',
	},
	installButton: {
		id: 'files.zip-url-modal.install-button',
		defaultMessage: 'Install',
	},
	stepFindTitle: {
		id: 'files.zip-url-modal.step-find-title',
		defaultMessage: 'Find the modpack',
	},
	stepFindDescription: {
		id: 'files.zip-url-modal.step-find-description',
		defaultMessage: 'Browse CurseForge and locate the modpack you want.',
	},
	stepSelectTitle: {
		id: 'files.zip-url-modal.step-select-title',
		defaultMessage: 'Select a version',
	},
	stepSelectDescription: {
		id: 'files.zip-url-modal.step-select-description',
		defaultMessage: 'Go to the "Files" tab and pick the version to install.',
	},
	stepCopyTitle: {
		id: 'files.zip-url-modal.step-copy-title',
		defaultMessage: 'Copy the URL',
	},
	stepCopyDescription: {
		id: 'files.zip-url-modal.step-copy-description',
		defaultMessage: 'Copy the version page URL and paste it below.',
	},
	errorUrlRequired: {
		id: 'files.zip-url-modal.error-url-required',
		defaultMessage: 'URL is required.',
	},
	errorCfUrl: {
		id: 'files.zip-url-modal.error-cf-url',
		defaultMessage: 'URL must be a CurseForge modpack version URL.',
	},
	errorUrlInvalid: {
		id: 'files.zip-url-modal.error-url-invalid',
		defaultMessage: 'URL must be valid.',
	},
	cfNotFoundTitle: {
		id: 'files.zip-url-modal.cf-not-found-title',
		defaultMessage: 'CurseForge modpack not found',
	},
	cfNotFoundText: {
		id: 'files.zip-url-modal.cf-not-found-text',
		defaultMessage: 'Could not find CurseForge modpack at that URL.',
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
		defaultMessage: 'CurseForge modpack install',
	},
})

const steps = [
	{
		icon: SearchIcon,
		title: formatMessage(messages.stepFindTitle),
		description: formatMessage(messages.stepFindDescription),
	},
	{
		icon: FileTextIcon,
		title: formatMessage(messages.stepSelectTitle),
		description: formatMessage(messages.stepSelectDescription),
	},
	{
		icon: LinkIcon,
		title: formatMessage(messages.stepCopyTitle),
		description: formatMessage(messages.stepCopyDescription),
	},
]

const cf = ref(false)

const modal = ref<InstanceType<typeof NewModal>>()
const url = ref('')
const submitted = ref(false)
const touched = ref(false)
const backupInProgress = ref(false)

const trimmedUrl = computed(() => url.value.trim())

const regex = /https:\/\/(www\.)?curseforge\.com\/minecraft\/modpacks\/[^/]+\/files\/\d+/

const error = computed(() => {
	if (trimmedUrl.value.length === 0) {
		return formatMessage(messages.errorUrlRequired)
	}
	if (cf.value && !regex.test(trimmedUrl.value)) {
		return formatMessage(messages.errorCfUrl)
	} else if (!cf.value && !trimmedUrl.value.includes('/')) {
		return formatMessage(messages.errorUrlInvalid)
	}
	return ''
})

const handleSubmit = async () => {
	touched.value = true
	if (error.value) return

	submitted.value = true
	try {
		const dry = await client.kyros.files_v0.extractFile(trimmedUrl.value, true, true)

		if (!cf.value || dry.modpack_name) {
			await client.kyros.files_v0.extractFile(trimmedUrl.value, true, false)
			hide()
		} else {
			submitted.value = false
			addNotification({
				title: formatMessage(messages.cfNotFoundTitle),
				text: formatMessage(messages.cfNotFoundText),
				type: 'error',
			})
		}
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

const show = (isCf: boolean) => {
	cf.value = isCf
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
