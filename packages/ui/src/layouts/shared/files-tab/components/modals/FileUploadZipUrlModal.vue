<template>
	<NewModal
		ref="modal"
		:header="cf ? `Install a CurseForge modpack` : `Uploading .zip contents from URL`"
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
				<label v-if="cf" class="text-base font-semibold text-contrast">Enter link</label>
				<div v-else class="text-sm text-secondary">
					Copy and paste the direct download URL of a .zip file.
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
				:backup-name="'CurseForge modpack install'"
				hide-shift-click-hint
				@update:buttons-disabled="backupInProgress = $event"
			/>
		</form>

		<template #actions>
			<div class="flex w-full items-center justify-end gap-2">
				<ButtonStyled>
					<button type="button" @click="hide">
						{{ submitted ? 'Close' : 'Cancel' }}
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
						{{ submitted ? 'Installing...' : 'Install' }}
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
import { injectModrinthClient } from '#ui/providers/api-client'
import { injectNotificationManager } from '#ui/providers/web-notifications'

import InlineBackupCreator from '../../../content-tab/components/modals/InlineBackupCreator.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const steps = [
	{
		icon: SearchIcon,
		title: 'Find the modpack',
		description: 'Browse CurseForge and locate the modpack you want.',
	},
	{
		icon: FileTextIcon,
		title: 'Select a version',
		description: 'Go to the "Files" tab and pick the version to install.',
	},
	{
		icon: LinkIcon,
		title: 'Copy the URL',
		description: 'Copy the version page URL and paste it below.',
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
		return 'URL is required.'
	}
	if (cf.value && !regex.test(trimmedUrl.value)) {
		return 'URL must be a CurseForge modpack version URL.'
	} else if (!cf.value && !trimmedUrl.value.includes('/')) {
		return 'URL must be valid.'
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
				title: 'CurseForge modpack not found',
				text: `Could not find CurseForge modpack at that URL.`,
				type: 'error',
			})
		}
	} catch (err) {
		submitted.value = false
		console.error('Error installing:', err)
		addNotification({
			title: 'Installation failed',
			text: err instanceof Error ? err.message : 'An unknown error occurred',
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
