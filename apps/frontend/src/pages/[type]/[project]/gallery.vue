<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<Modal
			v-if="currentMember"
			ref="modal_edit_item"
			:on-show="() => (isGalleryModalOpen = true)"
			:on-hide="() => (isGalleryModalOpen = false)"
			:header="editIndex === -1 ? 'Upload gallery image' : 'Edit gallery item'"
		>
			<div class="flex flex-col gap-4">
				<GalleryImageFields
					v-model:title="editTitle"
					v-model:description="editDescription"
					v-model:featured="editFeatured"
					:preview-src="previewImage || filteredGallery[editIndex]?.url"
					:filename="editFile?.name"
					:accept="acceptFileTypes"
					:max-size="globals?.max_gallery_image_size"
					:replace-disabled="!canUploadGallery"
					@replace="replaceEditFile"
				>
					<template #title-feedback>
						<ValidationMessage
							:check="galleryTitleValidation"
							:project-field="filteredGallery[editIndex]?.title ?? ''"
							:current-field="editTitle"
						/>
					</template>
					<template #description-feedback>
						<ValidationMessage
							:check="galleryDescriptionValidation"
							:project-field="filteredGallery[editIndex]?.description ?? ''"
							:current-field="editDescription"
						/>
					</template>
				</GalleryImageFields>
				<div class="flex flex-wrap justify-end gap-2">
					<Button type="outlined" @click="modalEditItem?.hide()">
						<XIcon aria-hidden="true" />
						Cancel
					</Button>
					<Button
						v-if="editIndex === -1"
						type="colored"
						color="brand"
						:disabled="shouldPreventActions || !canSaveGalleryFields"
						@click="createGalleryItem"
					>
						<PlusIcon aria-hidden="true" />
						Add gallery image
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="shouldPreventActions || !canSaveGalleryFields"
						@click="editGalleryItem"
					>
						<SaveIcon aria-hidden="true" />
						Save changes
					</Button>
				</div>
			</div>
		</Modal>
		<ConfirmModal
			v-if="canEditGallery"
			ref="modal_confirm"
			title="Are you sure you want to delete this gallery image?"
			description="This will remove this gallery image forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteGalleryImage"
		/>
		<ImageViewerEditor ref="galleryViewer" :items="galleryViewerItems" editor="disabled">
			<template #actions="{ item }">
				<Button
					type="quiet"
					class="!w-9 !rounded-full !p-0"
					aria-label="Open image in new tab"
					@click="openImageInNewTab(item.src)"
				>
					<ExternalIcon aria-hidden="true" />
				</Button>
			</template>
		</ImageViewerEditor>

		<div v-if="currentMember && filteredGallery.length" class="card header-buttons">
			<FileButton
				type="colored"
				color="brand"
				:max-size="globals?.max_gallery_image_size"
				:accept="acceptFileTypes"
				prompt="Upload an image"
				aria-label="Upload an image"
				class="button-like"
				:disabled="!canUploadGallery || galleryEditingBusy || galleryReordering"
				@change="handleFiles"
			>
				<UploadIcon aria-hidden="true" />
			</FileButton>
			<span class="indicator">
				<InfoIcon aria-hidden="true" /> Click to choose an image or drag one onto this page
			</span>
			<DropArea v-if="canUploadGallery" :accept="acceptFileTypes" @change="handleFiles" />
		</div>
		<SortableGalleryGrid
			v-if="filteredGallery.length"
			v-model:reordering="galleryReordering"
			:project-id="project.id"
			:images="filteredGallery"
			:editable="canEditGallery"
			:busy="galleryEditingBusy"
			class="items"
			heading="h2"
			@preview="expandImage"
			@edit="openGalleryEditor"
			@remove="confirmGalleryRemoval"
		/>
		<template v-else>
			<p class="ml-2">
				No images in gallery. Visit
				<NuxtLink to="settings/gallery">
					<span class="font-medium text-green hover:underline">project settings</span> to
				</NuxtLink>
				upload images.
			</p>
		</template>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon, InfoIcon, PlusIcon, SaveIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	DropArea,
	FileButton,
	GalleryImageFields,
	ImageViewerEditor,
	injectProjectPageContext,
	NewModal as Modal,
	useFormatBytes,
	useGlobals,
} from '@modrinth/ui'
import { fileIsValid, isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import SortableGalleryGrid from '~/components/ui/project-gallery/sortable-gallery-grid.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { isPermission } from '~/utils/permissions.ts'

const { data: globals } = useGlobals()
const formatBytes = useFormatBytes()

// Single DI injection
const {
	projectV2: project,
	currentMember,
	createGalleryItem: contextCreateGalleryItem,
	editGalleryItem: contextEditGalleryItem,
	deleteGalleryItem: contextDeleteGalleryItem,
} = injectProjectPageContext()

// Template refs
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')
const modalEditItem = useTemplateRef('modal_edit_item')
const modalConfirm = useTemplateRef('modal_confirm')
const galleryViewer = useTemplateRef('galleryViewer')

// SEO
const title = computed(() => `${project.value.title} - Gallery`)
const description = computed(
	() => `View ${project.value.gallery?.length ?? 0} images of ${project.value.title} on Modrinth.`,
)

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

// Delete state
const deleteImageUrl = ref<string>()

// Edit state
const editIndex = ref(-1)
const editTitle = ref('')
const editDescription = ref('')
const editFeatured = ref(false)
const editFile = ref<File | null>(null)
const previewImage = ref<string | null>(null)

// UI state
const shouldPreventActions = ref(false)
const isGalleryModalOpen = ref(false)
const isPreparingUpload = ref(false)
const galleryReordering = ref(false)
const galleryEditingBusy = computed(
	() => shouldPreventActions.value || isGalleryModalOpen.value || isPreparingUpload.value,
)
const hasEditDetails = computed(() =>
	isPermission(currentMember.value?.permissions, TeamMemberPermission.EDIT_DETAILS),
)
const canEditGallery = computed(() => hasEditDetails.value || isStaff(currentMember.value?.user))
const canUploadGallery = computed(
	() =>
		!!globals.value?.max_gallery_image_size &&
		(hasEditDetails.value || isAdmin(currentMember.value?.user)),
)
// Constant for accepted file types
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const acceptFileTypes = 'image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp'

const filteredGallery = computed(
	() => project.value.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)
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
const canSaveGalleryFields = computed(() => true)

const galleryViewerItems = computed(() =>
	filteredGallery.value.map((image) => ({
		id: image.url,
		src: image.raw_url ?? 'https://cdn.modrinth.com/placeholder-banner.svg',
		alt: image.title || 'Gallery image',
		title: image.title,
		description: image.description,
	})),
)

function expandImage(index: number) {
	galleryViewer.value?.show(index)
}

function openImageInNewTab(url: string) {
	window.open(url, '_blank', 'noopener,noreferrer')
}

// Edit state management
function openGalleryEditor(index: number) {
	if (!canEditGallery.value || galleryEditingBusy.value || galleryReordering.value) return
	const item = filteredGallery.value[index]
	resetEdit()
	editIndex.value = index
	editTitle.value = item.title ?? ''
	editDescription.value = item.description ?? ''
	editFeatured.value = item.featured
	modalEditItem.value?.show()
}

function confirmGalleryRemoval(url: string) {
	if (!canEditGallery.value || galleryEditingBusy.value || galleryReordering.value) return
	deleteImageUrl.value = url
	modalConfirm.value?.show()
}

function resetEdit() {
	editIndex.value = -1
	editTitle.value = ''
	editDescription.value = ''
	editFeatured.value = false
	editFile.value = null
	previewImage.value = null
}

async function handleFiles(files: File[]) {
	if (!canUploadGallery.value || galleryEditingBusy.value || galleryReordering.value) return
	const file = files[0]
	if (!file) {
		return
	}
	if (
		!globals.value?.max_gallery_image_size ||
		!fileIsValid(
			file,
			{ maxSize: globals.value.max_gallery_image_size, alertOnInvalid: true },
			formatBytes,
		)
	) {
		return
	}
	isPreparingUpload.value = true
	try {
		if (await fileDeclaresAi(file)) {
			aiImageWarningModal.value?.show()
			return
		}
		if (!canUploadGallery.value) return
		resetEdit()
		editFile.value = file
		showPreviewImage()
		modalEditItem.value?.show()
	} finally {
		isPreparingUpload.value = false
	}
}

async function replaceEditFile(files: File[]) {
	const file = files[0]
	if (!file) {
		return
	}
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	editFile.value = file
	showPreviewImage()
}

function showPreviewImage() {
	const reader = new FileReader()
	if (editFile.value instanceof Blob) {
		reader.readAsDataURL(editFile.value)
		reader.onload = (event) => {
			previewImage.value = event.target?.result as string | null
		}
	}
}

// CRUD operations
async function createGalleryItem() {
	if (
		!canUploadGallery.value ||
		galleryReordering.value ||
		!canSaveGalleryFields.value ||
		shouldPreventActions.value
	)
		return
	shouldPreventActions.value = true
	startLoading()

	const success = await contextCreateGalleryItem(
		editFile.value!,
		editTitle.value || undefined,
		editDescription.value || undefined,
		editFeatured.value,
		Math.max(-1, ...filteredGallery.value.map((image) => image.ordering)) + 1,
	)

	if (success) {
		modalEditItem.value?.hide()
	}

	stopLoading()
	shouldPreventActions.value = false
}

async function editGalleryItem() {
	if (
		!canEditGallery.value ||
		galleryReordering.value ||
		!canSaveGalleryFields.value ||
		shouldPreventActions.value
	)
		return
	shouldPreventActions.value = true
	startLoading()

	const imageUrl = filteredGallery.value[editIndex.value].url
	const success = await contextEditGalleryItem(
		imageUrl,
		editTitle.value,
		editDescription.value,
		editFeatured.value,
	)

	if (success) {
		modalEditItem.value?.hide()
	}

	stopLoading()
	shouldPreventActions.value = false
}

async function deleteGalleryImage() {
	const url = deleteImageUrl.value
	if (!canEditGallery.value || galleryReordering.value || !url) return
	deleteImageUrl.value = undefined
	shouldPreventActions.value = true
	try {
		await contextDeleteGalleryItem(url)
	} finally {
		shouldPreventActions.value = false
	}
}
</script>

<style lang="scss" scoped>
.header-buttons {
	display: flex;
	align-items: center;
	gap: 1rem;

	.indicator {
		display: flex;
		gap: 0.5ch;
		align-items: center;
		color: var(--color-text-inactive);
	}
}

.items {
	display: grid;
	grid-template-rows: 1fr;
	grid-template-columns: 1fr;
	grid-gap: var(--spacing-card-md);

	@media screen and (min-width: 1024px) {
		grid-template-columns: 1fr 1fr 1fr;
	}
}
</style>
