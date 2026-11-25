<template>
	<div data-project-field="gallery" data-project-field-scroll-only>
		<ValidationMessage :check="galleryImagesValidation" class="mb-4" />
		<ProjectIssueCard :target="['add_gallery_images', 'remove_gallery_images']" class="mb-4" />
		<SettingsSection
			v-if="filteredGallery.length"
			:title="formatMessage(commonProjectSettingsMessages.gallery)"
			:subtitle="formatMessage(messages.galleryHelp, { contentType: projectCopyType(projectV3) })"
			class="mb-4"
		>
			<template v-if="currentMember" #actions>
				<FileButton
					type="colored"
					color="brand"
					:max-size="globals?.max_gallery_image_size"
					:accept="acceptFileTypes"
					multiple
					:prompt="formatMessage(messages.upload)"
					class="ml-auto"
					:disabled="!canUploadGallery || galleryActionsBusy"
					@change="handleFiles"
				>
					<UploadIcon aria-hidden="true" />
				</FileButton>
			</template>
		</SettingsSection>
		<DropArea v-if="canUploadGallery" accept="" @change="handleFiles" />
		<AiImageWarningModal ref="aiImageWarningModal" />
		<Modal
			ref="modal_edit_item"
			:disable-close="shouldPreventActions"
			:close-label="isBatchUpload ? formatMessage(messages.skipImage) : undefined"
			:before-hide="skipCurrentUpload"
			:on-show="() => (isGalleryModalOpen = true)"
			:on-hide="() => (shouldPreventActions = true)"
			:on-after-hide="resetGalleryModal"
		>
			<template #title>
				<span class="text-2xl font-semibold text-contrast">
					{{
						editIndex === -1
							? formatMessage(messages.uploadTitle)
							: formatMessage(messages.editTitle)
					}}
				</span>
				<span
					v-if="isBatchUpload"
					class="shrink-0 text-sm font-medium text-secondary"
					aria-live="polite"
				>
					{{
						formatMessage(messages.uploadProgress, {
							current: uploadIndex + 1,
							total: uploadFiles.length,
						})
					}}
				</span>
			</template>
			<div class="flex flex-col gap-4">
				<GalleryImageFields
					v-model:title="editTitle"
					v-model:description="editDescription"
					v-model:featured="editFeatured"
					:preview-src="previewImage || filteredGallery[editIndex]?.url"
					:accept="acceptFileTypes"
				>
					<template #title-feedback>
						<ValidationMessage
							:check="galleryTitleValidation"
							:project-field="filteredGallery[editIndex]?.title ?? ''"
							:current-field="editTitle"
						/>
						<ValidationMessage :check="saveValidation.forField('gallery-text', 'name')" />
						<ProjectIssueCard
							v-if="editingImageId !== undefined"
							target="modify_gallery_image"
							:image-id="editingImageId"
						/>
					</template>
					<template #description-feedback>
						<ValidationMessage
							:check="galleryDescriptionValidation"
							:project-field="filteredGallery[editIndex]?.description ?? ''"
							:current-field="editDescription"
						/>
						<ValidationMessage :check="saveValidation.forField('gallery-text', 'description')" />
					</template>
				</GalleryImageFields>
				<div class="flex flex-wrap justify-end gap-2">
					<Button type="outlined" :disabled="shouldPreventActions" @click="modal_edit_item.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(isBatchUpload ? messages.skipImage : commonMessages.cancelButton) }}
					</Button>
					<Button
						v-if="editIndex === -1"
						type="colored"
						color="brand"
						:disabled="!canUploadGallery || shouldPreventActions || !canSaveGalleryFields"
						@click="createGalleryItem"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.uploadTitle) }}
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="!canEditGallery || shouldPreventActions || !canSaveGalleryFields"
						@click="editGalleryItem"
					>
						<SaveIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</Button>
				</div>
			</div>
		</Modal>
		<ConfirmModal
			v-if="canEditGallery"
			ref="modal_confirm"
			:title="formatMessage(messages.deleteTitle)"
			:description="formatMessage(messages.deleteDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(commonMessages.deleteLabel)"
			@proceed="deleteGalleryImage"
		/>
		<ImageViewerEditor ref="galleryViewer" :items="galleryViewerItems" editor="disabled">
			<template v-if="canEditGallery" #actions="{ index }">
				<IconButton
					v-tooltip="formatMessage(commonMessages.editButton)"
					type="quiet"
					:label="formatMessage(commonMessages.editButton)"
					:disabled="galleryActionsBusy || !filteredGallery[index]?.url"
					@click="openGalleryEditor(index)"
				>
					<EditIcon aria-hidden="true" />
				</IconButton>
				<IconButton
					v-tooltip="formatMessage(commonMessages.deleteLabel)"
					type="quiet"
					:label="formatMessage(commonMessages.deleteLabel)"
					:disabled="galleryActionsBusy || !filteredGallery[index]?.url"
					@click="confirmGalleryRemoval(filteredGallery[index]?.url)"
				>
					<TrashIcon aria-hidden="true" />
				</IconButton>
			</template>
		</ImageViewerEditor>
		<div v-if="filteredGallery.length" class="@container">
			<SortableGalleryGrid
				v-model:reordering="galleryReordering"
				:project-id="project.id"
				:images="filteredGallery"
				:editable="canEditGallery"
				:busy="galleryEditingBusy"
				class="grid grid-cols-1 gap-4 @[34rem]:grid-cols-2 @[52rem]:grid-cols-3"
				@preview="expandImage"
				@edit="openGalleryEditor"
				@remove="confirmGalleryRemoval"
			>
				<template #details="{ image }">
					<ProjectIssueCard
						v-if="galleryImageId(image.url) !== undefined"
						target="modify_gallery_image"
						:image-id="galleryImageId(image.url)"
					/>
				</template>
			</SortableGalleryGrid>
		</div>
		<EmptyState
			v-else
			:icon="ImageIcon"
			:heading="formatMessage(messages.emptyHeading)"
			:description="formatMessage(messages.emptyDescription)"
		>
			<template #actions>
				<FileButton
					type="colored"
					color="green"
					size="lg"
					:max-size="globals?.max_gallery_image_size"
					:accept="acceptFileTypes"
					multiple
					:prompt="formatMessage(messages.upload)"
					:disabled="!canUploadGallery || galleryActionsBusy"
					@change="handleFiles"
				>
					<UploadIcon aria-hidden="true" />
				</FileButton>
			</template>
		</EmptyState>
	</div>
</template>

<script setup>
import {
	EditIcon,
	ImageIcon,
	PlusIcon,
	SaveIcon,
	TrashIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	DropArea,
	EmptyState,
	FileButton,
	galleryCardMessages,
	GalleryImageFields,
	IconButton,
	ImageViewerEditor,
	injectNotificationManager,
	injectProjectPageContext,
	NewModal as Modal,
	useFormatBytes,
	useGlobals,
	useVIntl,
} from '@modrinth/ui'
import { fileIsValid, isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import SortableGalleryGrid from '~/components/ui/project-gallery/sortable-gallery-grid.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { isPermission } from '~/utils/permissions.ts'
import { projectCopyType } from '~/utils/project-messages'

import SettingsSection from '../settings-section.vue'
import { messages } from './messages'

const { data: globals } = useGlobals()
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()

const {
	projectV2: project,
	projectV3,
	currentMember,
	createGalleryItem: createGalleryItemMutation,
	editGalleryItem: editGalleryItemMutation,
	deleteGalleryItem: deleteGalleryItemMutation,
} = injectProjectPageContext()

const hasEditDetails = computed(() =>
	isPermission(currentMember.value?.permissions, TeamMemberPermission.EDIT_DETAILS),
)
const canUploadGallery = computed(
	() =>
		!!globals.value?.max_gallery_image_size &&
		(hasEditDetails.value || isAdmin(currentMember.value?.user)),
)
const canEditGallery = computed(() => hasEditDetails.value || isStaff(currentMember.value?.user))

const aiImageWarningModal = ref(null)
const modal_edit_item = ref(null)
const modal_confirm = ref(null)

const galleryViewer = useTemplateRef('galleryViewer')

const deleteImageUrl = ref(null)
const isDeletingImage = ref(false)

const editIndex = ref(-1)
function galleryImageId(url) {
	return projectV3.value?.gallery.find((image) => image.url === url)?.id
}
const editingImageId = computed(() => galleryImageId(filteredGallery.value[editIndex.value]?.url))
const editTitle = ref('')
const editDescription = ref('')
const editFeatured = ref(false)
const editFile = ref(null)
const previewImage = ref(null)
const shouldPreventActions = ref(false)
const isPreparingUpload = ref(false)
const isGalleryModalOpen = ref(false)
const uploadFiles = shallowRef([])
const uploadIndex = ref(0)
const isBatchUpload = computed(() => uploadFiles.value.length > 1)
const pendingBatchUploads = ref(0)
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const acceptFileTypes = 'image/png,image/jpeg,image/gif,image/webp,.png,.jpg,.jpeg,.gif,.webp'
const acceptedFileTypes = acceptFileTypes.split(',')

const filteredGallery = computed(
	() => project.value.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)
const galleryViewerItems = computed(() =>
	filteredGallery.value.map((image) => ({
		id: image.url || image.created,
		src: image.raw_url || image.url || 'https://cdn.modrinth.com/placeholder-banner.svg',
		alt: image.title || formatMessage(galleryCardMessages.imageAlt),
		title: image.title,
		description: image.description,
	})),
)
const nextGalleryOrdering = computed(
	() => Math.max(-1, ...filteredGallery.value.map((image) => image.ordering)) + 1,
)
const galleryEditingBusy = computed(
	() =>
		isPreparingUpload.value ||
		isGalleryModalOpen.value ||
		isDeletingImage.value ||
		pendingBatchUploads.value > 0,
)
const galleryReordering = ref(false)
const galleryActionsBusy = computed(() => galleryEditingBusy.value || galleryReordering.value)

const selectedGalleryIndex = computed(() => {
	const selectedItem = filteredGallery.value[editIndex.value]
	return selectedItem ? (project.value.gallery ?? []).indexOf(selectedItem) : -1
})
const galleryTitleValidation = useProjectNagMessages(
	'gallery-text',
	'name',
	() => selectedGalleryIndex.value,
)
const galleryDescriptionValidation = useProjectNagMessages(
	'gallery-text',
	'description',
	() => selectedGalleryIndex.value,
)
const galleryImagesValidation = useProjectNagMessages('gallery-images')
const saveValidation = useProjectSaveValidation(() => ({
	index: editIndex.value,
	title: editTitle.value,
	description: editDescription.value,
	featured: editFeatured.value,
	file: previewImage.value,
}))
const canSaveGalleryFields = computed(() => !saveValidation.hasErrors.value)

function expandImage(index) {
	galleryViewer.value?.show(index)
}

const resetEdit = () => {
	saveValidation.clear()
	editIndex.value = -1
	editTitle.value = ''
	editDescription.value = ''
	editFeatured.value = false
	editFile.value = null
	previewImage.value = null
}

function openGalleryEditor(index) {
	const item = filteredGallery.value[index]
	if (!canEditGallery.value || galleryActionsBusy.value || !item?.url) return
	galleryViewer.value?.hide()
	resetEdit()
	editIndex.value = index
	editTitle.value = item.title ?? ''
	editDescription.value = item.description ?? ''
	editFeatured.value = item.featured
	modal_edit_item.value?.show()
}

function confirmGalleryRemoval(url) {
	if (!canEditGallery.value || galleryActionsBusy.value || !url) return
	galleryViewer.value?.hide()
	deleteImageUrl.value = url
	modal_confirm.value?.show()
}

function resetGalleryModal() {
	isGalleryModalOpen.value = false
	shouldPreventActions.value = false
	uploadFiles.value = []
	uploadIndex.value = 0
	resetEdit()
}

function showUploadStep(index) {
	resetEdit()
	uploadIndex.value = index
	setEditFile(uploadFiles.value[index])
}

function skipCurrentUpload() {
	const nextIndex = uploadIndex.value + 1
	if (nextIndex >= uploadFiles.value.length) return true
	showUploadStep(nextIndex)
	return false
}

function validateGalleryFile(file) {
	const filename = file.name.toLowerCase()
	const matchesType = acceptedFileTypes.some((type) =>
		type.startsWith('.') ? filename.endsWith(type) : file.type === type,
	)
	if (!matchesType) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.invalidFileTitle),
			text: formatMessage(messages.invalidFileDescription, { filename: file.name }),
		})
		return false
	}
	return fileIsValid(
		file,
		{ maxSize: globals.value?.max_gallery_image_size, alertOnInvalid: true },
		formatBytes,
	)
}

const handleFiles = async (files) => {
	if (!canUploadGallery.value || galleryActionsBusy.value) return
	isPreparingUpload.value = true

	try {
		const validFiles = []
		let rejectedAiImage = false
		for (const file of files) {
			if (!validateGalleryFile(file)) continue
			if (await fileDeclaresAi(file)) {
				rejectedAiImage = true
			} else {
				validFiles.push(file)
			}
		}
		if (!canUploadGallery.value || !modal_edit_item.value) return
		if (validFiles.length) {
			uploadFiles.value = validFiles
			showUploadStep(0)
			modal_edit_item.value.show()
		}
		if (rejectedAiImage) aiImageWarningModal.value?.show()
	} finally {
		isPreparingUpload.value = false
	}
}

function setEditFile(file) {
	editFile.value = file
	previewImage.value = null
	const reader = new FileReader()
	reader.onload = () => {
		if (editFile.value === file) previewImage.value = reader.result
	}
	reader.readAsDataURL(file)
}

const createGalleryItem = async () => {
	if (
		!editFile.value ||
		!canUploadGallery.value ||
		!canSaveGalleryFields.value ||
		shouldPreventActions.value
	)
		return

	if (isBatchUpload.value) {
		pendingBatchUploads.value += 1
		const upload = createGalleryItemMutation(
			editFile.value,
			editTitle.value || undefined,
			editDescription.value || undefined,
			editFeatured.value,
		)
		modal_edit_item.value?.hide()
		await upload.finally(() => (pendingBatchUploads.value -= 1))
		return
	}

	shouldPreventActions.value = true

	const submittedState = saveValidation.snapshot()
	try {
		const success = await createGalleryItemMutation(
			editFile.value,
			editTitle.value || undefined,
			editDescription.value || undefined,
			editFeatured.value,
			nextGalleryOrdering.value,
			true,
		)

		if (!success) return
		saveValidation.clear()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
		return
	} finally {
		shouldPreventActions.value = false
	}
	await nextTick()
	modal_edit_item.value?.hide()
}

const editGalleryItem = async () => {
	if (!canEditGallery.value || !canSaveGalleryFields.value || shouldPreventActions.value) return
	shouldPreventActions.value = true

	const submittedState = saveValidation.snapshot()
	try {
		const success = await editGalleryItemMutation(
			filteredGallery.value[editIndex.value].url,
			editTitle.value,
			editDescription.value,
			editFeatured.value,
			undefined,
			true,
		)

		if (!success) return
		saveValidation.clear()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
		return
	} finally {
		shouldPreventActions.value = false
	}
	await nextTick()
	modal_edit_item.value?.hide()
}

const deleteGalleryImage = async () => {
	const url = deleteImageUrl.value
	if (!canEditGallery.value || !url) return

	deleteImageUrl.value = null
	isDeletingImage.value = true
	try {
		await deleteGalleryItemMutation(url)
	} finally {
		isDeletingImage.value = false
	}
}
</script>
