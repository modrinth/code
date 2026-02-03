<template>
	<NewModal ref="modal" :header="formatMessage(messages.title)" @hide="resetState">
		<div class="flex flex-col gap-6 sm:w-[500px] max-w-[500px]">
			<!-- Modded Content -->
			<div class="flex flex-col gap-2.5">
				<div class="flex flex-col gap-2">
					<label class="font-semibold text-contrast">
						{{ formatMessage(messages.moddedContentLabel) }}
					</label>
					<Chips
						v-model="contentType"
						:items="contentTypeOptions"
						:format-label="formatContentType"
					/>
				</div>

				<!-- Published Modpack Selection -->
				<div
					v-if="contentType === 'published'"
					class="flex flex-col gap-6 p-4 border border-surface-5 border-solid rounded-2xl"
				>
					<div class="flex flex-col gap-2">
						<label class="font-semibold text-contrast">
							{{ formatMessage(messages.projectLabel) }}
						</label>
						<ProjectCombobox
							ref="projectCombobox"
							v-model="selectedProjectId"
							:project-types="['modpack']"
							:placeholder="formatMessage(messages.projectPlaceholder)"
							:search-placeholder="formatMessage(messages.projectSearchPlaceholder)"
							:loading-message="formatMessage(messages.loading)"
							:no-results-message="formatMessage(messages.noResults)"
						/>
					</div>

					<div v-if="selectedProjectId" class="flex flex-col gap-2">
						<label class="font-semibold text-contrast">
							{{ formatMessage(messages.versionLabel) }}
						</label>
						<Combobox
							v-model="selectedVersionId"
							:placeholder="formatMessage(messages.versionPlaceholder)"
							:options="versionOptions"
							:searchable="true"
							:search-placeholder="formatMessage(messages.versionSearchPlaceholder)"
							:no-options-message="
								versionsLoading
									? formatMessage(messages.loading)
									: formatMessage(messages.noResults)
							"
						/>
					</div>
				</div>

				<!-- Custom Modpack Upload -->
				<div v-else class="flex flex-col gap-4">
					<DropzoneFileInput
						:primary-prompt="formatMessage(messages.dropzonePrimary)"
						:secondary-prompt="formatMessage(messages.dropzoneSecondary)"
						accept=".zip,.mrpack"
						size="medium"
						@change="handleFileUpload"
					/>

					<div v-if="uploadedFile" class="flex items-center gap-2 text-primary">
						<FileIcon class="h-4 w-4" />
						<span>{{ uploadedFile.name }}</span>
						<ButtonStyled type="transparent" size="small">
							<button @click="clearUploadedFile">
								<XIcon class="h-4 w-4" />
							</button>
						</ButtonStyled>
					</div>

					<Checkbox v-model="hasLicensePermission">
						<span class="text-left text-primary max-w-[90%]">
							{{ formatMessage(messages.licenseCheckbox) }}
							<NuxtLink
								to="https://support.modrinth.com/en/articles/8797527-obtaining-modpack-permissions"
								external
								target="_blank"
								class="text-blue underline font-medium"
							>
								{{ formatMessage(messages.learnMore) }}
							</NuxtLink>
						</span>
					</Checkbox>
				</div>
			</div>

			<!-- Set as active/primary version -->
			<div class="flex items-center gap-3">
				<Toggle id="set-active-version" v-model="setAsActiveVersion" />
				<label for="set-active-version" class="text-contrast cursor-pointer">
					{{
						contentType === 'published'
							? formatMessage(messages.setActiveVersion)
							: formatMessage(messages.setPrimaryVersion)
					}}
				</label>
				<span
					v-tooltip="formatMessage(messages.activeVersionTooltip)"
					class="text-secondary cursor-help"
				>
					<InfoIcon class="h-4 w-4" />
				</span>
			</div>
		</div>

		<template #actions>
			<div class="flex items-center justify-end gap-3">
				<ButtonStyled>
					<button @click="hide">
						<XIcon />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand" :disabled="!canSubmit">
					<button :disabled="!canSubmit" @click="handleSubmit">
						<PlusIcon />
						{{ formatMessage(messages.selectModpack) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { FileIcon, InfoIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { injectModrinthClient, injectNotificationManager } from '../../providers'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import Chips from '../base/Chips.vue'
import type { ComboboxOption } from '../base/Combobox.vue'
import Combobox from '../base/Combobox.vue'
import DropzoneFileInput from '../base/DropzoneFileInput.vue'
import Toggle from '../base/Toggle.vue'
import NewModal from '../modal/NewModal.vue'
import ProjectCombobox from './ProjectCombobox.vue'

type ContentType = 'published' | 'custom'

interface VersionInfo {
	id: string
	version_number: string
	name: string
}

export interface CreateServerVersionData {
	versionName: string
	contentType: ContentType
	// Published modpack data
	projectId?: string
	versionId?: string
	// Custom modpack data
	file?: File
	hasLicensePermission?: boolean
	// Common
	setAsActiveVersion: boolean
}

const messages = defineMessages({
	title: { id: 'create-version-modal.title', defaultMessage: 'Create new version' },
	versionNameLabel: {
		id: 'create-version-modal.version-name-label',
		defaultMessage: 'Name',
	},
	versionNumberPlaceholder: {
		id: 'create-version-modal.version-number-placeholder',
		defaultMessage: 'Enter version number',
	},
	moddedContentLabel: {
		id: 'create-version-modal.modded-content-label',
		defaultMessage: 'Modded content',
	},
	publishedModpack: {
		id: 'create-version-modal.published-modpack',
		defaultMessage: 'Published modpack',
	},
	customModpack: { id: 'create-version-modal.custom-modpack', defaultMessage: 'Custom modpack' },
	projectLabel: { id: 'create-version-modal.project-label', defaultMessage: 'Project' },
	projectPlaceholder: {
		id: 'create-version-modal.project-placeholder',
		defaultMessage: 'Select modpack',
	},
	projectSearchPlaceholder: {
		id: 'create-version-modal.project-search-placeholder',
		defaultMessage: 'Search by name or paste ID...',
	},
	versionLabel: { id: 'create-version-modal.version-label', defaultMessage: 'Version' },
	versionPlaceholder: {
		id: 'create-version-modal.version-placeholder',
		defaultMessage: 'Select version',
	},
	versionSearchPlaceholder: {
		id: 'create-version-modal.version-search-placeholder',
		defaultMessage: 'Search versions...',
	},
	dropzonePrimary: {
		id: 'create-version-modal.dropzone-primary',
		defaultMessage: 'Drag and drop files or click to browse',
	},
	dropzoneSecondary: {
		id: 'create-version-modal.dropzone-secondary',
		defaultMessage: 'You can try to drag files or folder or click this area to select it',
	},
	licenseCheckbox: {
		id: 'create-version-modal.license-checkbox',
		defaultMessage:
			'Do you have the appropriate licenses to redistribute all content in this Modpack?',
	},
	learnMore: { id: 'create-version-modal.learn-more', defaultMessage: 'Learn more' },
	setActiveVersion: {
		id: 'create-version-modal.set-active-version',
		defaultMessage: 'Set as the active version.',
	},
	setPrimaryVersion: {
		id: 'create-version-modal.set-primary-version',
		defaultMessage: 'Set as the primary version.',
	},
	activeVersionTooltip: {
		id: 'create-version-modal.active-version-tooltip',
		defaultMessage: 'This version will be used when players join the server.',
	},
	cancel: { id: 'create-version-modal.cancel', defaultMessage: 'Cancel' },
	selectModpack: { id: 'create-version-modal.select-modpack', defaultMessage: 'Select modpack' },
	loading: { id: 'create-version-modal.loading', defaultMessage: 'Loading...' },
	noResults: { id: 'create-version-modal.no-results', defaultMessage: 'No results found' },
})

const { formatMessage } = useVIntl()

const emit = defineEmits<{
	submit: [data: CreateServerVersionData]
}>()

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const projectCombobox = ref<InstanceType<typeof ProjectCombobox> | null>(null)

// Form state
const versionName = ref('')
const contentType = ref<ContentType>('published')
const selectedProjectId = ref<string>('')
const selectedVersionId = ref<string>('')
const uploadedFile = ref<File | null>(null)
const hasLicensePermission = ref(false)
const setAsActiveVersion = ref(true)

// Version loading state
const versionsLoading = ref(false)
const projectVersions = ref<VersionInfo[]>([])

const contentTypeOptions: ContentType[] = ['published', 'custom']

function formatContentType(type: ContentType): string {
	return type === 'published'
		? formatMessage(messages.publishedModpack)
		: formatMessage(messages.customModpack)
}

const versionOptions = computed<ComboboxOption<string>[]>(() => {
	return projectVersions.value.map((version) => ({
		label: version.version_number,
		value: version.id,
	}))
})

const canSubmit = computed(() => {
	if (!versionName.value.trim()) return false

	if (contentType.value === 'published') {
		return !!selectedProjectId.value && !!selectedVersionId.value
	} else {
		return !!uploadedFile.value && hasLicensePermission.value
	}
})

// Watch for project selection changes to load versions
watch(selectedProjectId, async (newProjectId) => {
	selectedVersionId.value = ''
	projectVersions.value = []

	if (!newProjectId) return

	versionsLoading.value = true
	try {
		const versions = await labrinth.versions_v3.getProjectVersions(newProjectId)
		projectVersions.value = versions.map((v) => ({
			id: v.id,
			version_number: v.version_number,
			name: v.name,
		}))
	} catch (error: unknown) {
		const err = error as { data?: { description?: string } }
		addNotification({
			title: 'Failed to load versions',
			text: err.data?.description || String(error),
			type: 'error',
		})
	} finally {
		versionsLoading.value = false
	}
})

function handleFileUpload(files: File[]) {
	if (files.length > 0) {
		uploadedFile.value = files[0]
	}
}

function clearUploadedFile() {
	uploadedFile.value = null
}

function resetState() {
	versionName.value = ''
	contentType.value = 'published'
	selectedProjectId.value = ''
	selectedVersionId.value = ''
	uploadedFile.value = null
	hasLicensePermission.value = false
	setAsActiveVersion.value = true
	projectVersions.value = []
}

function handleSubmit() {
	if (!canSubmit.value) return

	const data: CreateServerVersionData = {
		versionName: versionName.value.trim(),
		contentType: contentType.value,
		setAsActiveVersion: setAsActiveVersion.value,
	}

	if (contentType.value === 'published') {
		data.projectId = selectedProjectId.value
		data.versionId = selectedVersionId.value
	} else {
		data.file = uploadedFile.value || undefined
		data.hasLicensePermission = hasLicensePermission.value
	}

	emit('submit', data)
}

function show(event?: MouseEvent) {
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})
</script>
