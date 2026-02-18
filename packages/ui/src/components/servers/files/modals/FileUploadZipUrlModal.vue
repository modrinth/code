<template>
	<NewModal
		ref="modal"
		:header="cf ? `Installing a CurseForge modpack` : `Uploading .zip contents from URL`"
	>
		<form class="flex flex-col gap-5 md:w-[620px]" @submit.prevent="handleSubmit">
			<!-- CurseForge stepper cards -->
			<div v-if="cf" class="flex flex-col gap-2 w-full">
				<div class="grid gap-2 sm:grid-cols-3">
					<div
						v-for="(step, i) in steps"
						:key="i"
						class="flex flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-4 p-4"
					>
						<div class="flex items-center gap-2">
							<span
								class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand-highlight text-xs font-bold text-brand"
							>
								{{ i + 1 }}
							</span>
						</div>
						<div class="text-sm font-semibold leading-snug text-contrast">
							{{ step.title }}
						</div>
						<div class="text-xs leading-relaxed text-secondary">
							{{ step.description }}
						</div>
						<a
							v-if="step.link"
							:href="step.link"
							target="_blank"
							rel="noopener noreferrer"
							class="mt-auto inline-flex items-center gap-1 text-xs font-semibold text-[#F16436] transition-all hover:underline"
						>
							Browse CurseForge
							<ExternalIcon class="h-3 w-3" />
						</a>
					</div>
				</div>
			</div>

			<!-- URL input -->
			<div class="flex flex-col gap-2">
				<div v-if="!cf" class="text-sm text-secondary">
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

			<!-- Backup warning -->
			<Admonition type="warning">
				You may want to
				<AutoLink
					:to="`/hosting/manage/${serverId}/backups`"
					class="font-semibold text-orange hover:underline"
					>create a backup</AutoLink
				>
				before proceeding, as this process is irreversible and may permanently alter your world or
				the files on your server.
			</Admonition>
		</form>

		<template #actions>
			<div class="flex gap-2 justify-start">
				<ButtonStyled color="brand">
					<button
						v-tooltip="error"
						:disabled="submitted || !!error"
						type="submit"
						@click="handleSubmit"
					>
						<SpinnerIcon v-if="submitted" class="animate-spin" />
						<DownloadIcon v-else />
						{{ submitted ? 'Installing...' : 'Install' }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button type="button" @click="hide">
						<XIcon />
						{{ submitted ? 'Close' : 'Cancel' }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	ExternalIcon,
	FileTextIcon,
	LinkIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { computed, nextTick, ref } from 'vue'

import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../../providers'
import Admonition from '../../../base/Admonition.vue'
import AutoLink from '../../../base/AutoLink.vue'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import StyledInput from '../../../base/StyledInput.vue'
import NewModal from '../../../modal/NewModal.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { serverId } = injectModrinthServerContext()

const steps = [
	{
		icon: SearchIcon,
		title: 'Find the modpack',
		description: 'Browse CurseForge and locate the modpack you want.',
		link: 'https://www.curseforge.com/minecraft/search?page=1&pageSize=40&sortBy=relevancy&class=modpacks',
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
