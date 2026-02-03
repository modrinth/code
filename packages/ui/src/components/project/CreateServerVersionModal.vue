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
							v-model="internalProjectId"
							:project-types="['modpack']"
							:placeholder="formatMessage(messages.projectPlaceholder)"
							:search-placeholder="formatMessage(messages.projectSearchPlaceholder)"
							:loading-message="formatMessage(messages.loading)"
							:no-results-message="formatMessage(messages.noResults)"
						/>
					</div>

					<div v-if="internalProjectId" class="flex flex-col gap-2">
						<label class="font-semibold text-contrast">
							{{ formatMessage(messages.versionLabel) }}
						</label>
						<Combobox
							v-model="internalVersionId"
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
						v-if="!uploadedFile"
						:primary-prompt="formatMessage(messages.dropzonePrimary)"
						:secondary-prompt="formatMessage(messages.dropzoneSecondary)"
						accept=".zip,.mrpack"
						size="medium"
						@change="handleFileUpload"
					/>

					<div
						v-if="uploadedFile"
						class="flex items-center justify-between gap-2 rounded-xl bg-button-bg px-4 py-2 text-button-text"
					>
						<div class="flex items-center gap-2 overflow-hidden">
							<FileIcon />
							<span
								v-tooltip="uploadedFile.name"
								class="overflow-hidden text-ellipsis whitespace-nowrap font-medium"
							>
								{{ uploadedFile.name }}
							</span>
						</div>

						<ButtonStyled size="standard" :circular="true">
							<button
								v-tooltip="formatMessage(messages.replaceFile)"
								aria-label="Replace file"
								class="!shadow-none"
								@click="fileInput?.click()"
							>
								<ArrowLeftRightIcon aria-hidden="true" />
								<input
									ref="fileInput"
									class="hidden"
									type="file"
									accept=".zip,.mrpack"
									@change="handleFileInputChange"
								/>
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
				<ButtonStyled color="brand" :disabled="!canSubmit || isSubmitting">
					<button :disabled="!canSubmit || isSubmitting" @click="handleSubmit">
						<SpinnerIcon v-if="isSubmitting" class="animate-spin" />
						<PlusIcon v-else />
						{{ submitButtonLabel }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftRightIcon,
	FileIcon,
	InfoIcon,
	PlusIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import JSZip from 'jszip'
import { computed, nextTick, ref, toRaw, watch } from 'vue'

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
	replaceFile: {
		id: 'create-version-modal.replace-file',
		defaultMessage: 'Replace file',
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
	uploading: { id: 'create-version-modal.uploading', defaultMessage: 'Uploading' },
	creatingVersion: {
		id: 'create-version-modal.creating-version',
		defaultMessage: 'Creating version',
	},
	loading: { id: 'create-version-modal.loading', defaultMessage: 'Loading...' },
	noResults: { id: 'create-version-modal.no-results', defaultMessage: 'No results found' },
})

const { formatMessage } = useVIntl()

const props = defineProps<{
	projectId: string
	onSave?: () => void | Promise<void>
}>()

const { addNotification } = injectNotificationManager()
const { labrinth } = injectModrinthClient()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const fileInput = ref<HTMLInputElement>()

// Form state
const contentType = ref<ContentType>('published')

interface SelectedPublishedProject {
	projectId: string
	projectName: string
	versionId: string
	versionNumber: string
}

const selectedPublishedProject = ref<SelectedPublishedProject | null>(null)

// Internal refs for combobox v-models
const internalProjectId = ref<string>('')
const internalVersionId = ref<string>('')
const currentProjectName = ref<string>('')

const uploadedFile = ref<File | null>(null)
const hasLicensePermission = ref(false)
const setAsActiveVersion = ref(true)

// Submission state
const isSubmitting = ref(false)
const isUploading = ref(false)
const uploadProgress = ref({ loaded: 0, total: 0, progress: 0 })

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
	if (isSubmitting.value) return false
	if (contentType.value === 'published') {
		return !!selectedPublishedProject.value
	} else {
		return !!uploadedFile.value && hasLicensePermission.value
	}
})

const submitButtonLabel = computed(() => {
	if (isUploading.value) {
		if (uploadProgress.value.progress >= 1) {
			return formatMessage(messages.creatingVersion)
		}
		return `${formatMessage(messages.uploading)} ${Math.round(uploadProgress.value.progress * 100)}%`
	}
	return formatMessage(messages.selectModpack)
})

// Watch for project selection changes to load versions
watch(internalProjectId, async (newProjectId) => {
	internalVersionId.value = ''
	projectVersions.value = []
	selectedPublishedProject.value = null
	currentProjectName.value = ''

	if (!newProjectId) return

	versionsLoading.value = true
	try {
		// Fetch project info to get the name
		const project = await labrinth.projects_v3.get(newProjectId)
		currentProjectName.value = project.name

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

// Watch for version selection to build the complete selectedPublishedProject
watch(internalVersionId, (newVersionId) => {
	if (!newVersionId || !internalProjectId.value) {
		selectedPublishedProject.value = null
		return
	}

	const selectedVersion = projectVersions.value.find((v) => v.id === newVersionId)
	if (selectedVersion) {
		selectedPublishedProject.value = {
			projectId: internalProjectId.value,
			projectName: currentProjectName.value,
			versionId: newVersionId,
			versionNumber: selectedVersion.version_number,
		}
	}
})

function handleFileUpload(files: File[]) {
	if (files.length > 0) {
		uploadedFile.value = files[0]
	}
}

function handleFileInputChange(e: Event) {
	const target = e.target as HTMLInputElement
	const file = target.files?.[0]
	if (file) {
		uploadedFile.value = file
	}
	target.value = ''
}

function clearUploadedFile() {
	uploadedFile.value = null
}

function resetState() {
	contentType.value = 'published'
	internalProjectId.value = ''
	internalVersionId.value = ''
	selectedPublishedProject.value = null
	currentProjectName.value = ''
	uploadedFile.value = null
	hasLicensePermission.value = false
	setAsActiveVersion.value = true
	projectVersions.value = []
	isSubmitting.value = false
	isUploading.value = false
	uploadProgress.value = { loaded: 0, total: 0, progress: 0 }
}

async function handleSubmit() {
	if (!canSubmit.value) return

	isSubmitting.value = true

	try {
		if (contentType.value === 'published' && selectedPublishedProject.value) {
			// TODO: Implement published modpack linking when backend API is ready
			// For now, just mock success - the backend will handle linking to the published version
			// This will eventually create a server version that references the published modpack version
			// without re-uploading the file
			console.log('Published modpack selected:', {
				projectId: selectedPublishedProject.value.projectId,
				projectName: selectedPublishedProject.value.projectName,
				versionId: selectedPublishedProject.value.versionId,
				versionNumber: selectedPublishedProject.value.versionNumber,
				setAsActiveVersion: setAsActiveVersion.value,
			})

			addNotification({
				title: 'Published modpack selected',
				text: `Selected ${selectedPublishedProject.value.projectName} v${selectedPublishedProject.value.versionNumber}. Linking will be available soon.`,
				type: 'success',
			})
		} else if (contentType.value === 'custom' && uploadedFile.value) {
			const file = toRaw(uploadedFile.value)

			// Default to filename if we can't parse the mrpack
			let versionName = file.name.replace(/\.(zip|mrpack)$/i, '')
			let versionNumber = versionName
			let loaders: string[] = []
			let gameVersions: string[] = []

			try {
				const zip = await JSZip.loadAsync(file)
				const indexFile = zip.file('modrinth.index.json')

				if (indexFile) {
					const indexContent = await indexFile.async('text')
					const metadata = JSON.parse(indexContent) as {
						name?: string
						versionId?: string
						dependencies?: Record<string, string>
					}
					if (metadata.name) {
						versionName = metadata.name
					}
					if (metadata.versionId) {
						versionNumber = metadata.versionId
					}
					if (metadata.dependencies) {
						if ('forge' in metadata.dependencies) loaders.push('forge')
						if ('neoforge' in metadata.dependencies) loaders.push('neoforge')
						if ('fabric-loader' in metadata.dependencies) loaders.push('fabric')
						if ('quilt-loader' in metadata.dependencies) loaders.push('quilt')
						if (metadata.dependencies.minecraft) gameVersions = [metadata.dependencies.minecraft]
					}
				}
			} catch {
				console.warn('Could not parse modrinth.index.json from mrpack')
			}

			const draftVersion: Labrinth.Versions.v3.DraftVersion = {
				project_id: props.projectId,
				name: versionName,
				version_number: versionNumber,
				version_type: 'release',
				loaders,
				game_versions: gameVersions,
				featured: setAsActiveVersion.value,
				status: 'listed',
				changelog: '',
				dependencies: [],
				environment: 'client_and_server',
			}

			const files: Labrinth.Versions.v3.DraftVersionFile[] = [{ file, fileType: undefined }]

			const uploadHandle = labrinth.versions_v3.createVersion(draftVersion, files, 'modpack')

			// Track upload progress
			isUploading.value = true
			uploadProgress.value = { loaded: 0, total: 0, progress: 0 }

			uploadHandle.onProgress((progress) => {
				uploadProgress.value = progress
			})

			await uploadHandle.promise
			isUploading.value = false

			addNotification({
				title: 'Server version created',
				text: 'The version has been successfully added to your server.',
				type: 'success',
			})
		}

		await nextTick()
		modal.value?.hide()

		await props.onSave?.()
	} catch (err: unknown) {
		const error = err as { data?: { description?: string } }
		addNotification({
			title: 'Could not create server version',
			text: error.data?.description || String(err),
			type: 'error',
		})
	} finally {
		isSubmitting.value = false
		hide()
	}
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
