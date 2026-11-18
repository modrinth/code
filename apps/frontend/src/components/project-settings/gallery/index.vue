<template>
	<div data-project-field="gallery" data-project-field-scroll-only>
		<h2 class="settings-section-title">
			{{ formatMessage(commonProjectSettingsMessages.gallery) }}
		</h2>
		<ValidationMessage :check="galleryImagesValidation" class="mb-4" />
		<ProjectIssueCard :target="['add_gallery_images', 'remove_gallery_images']" class="mb-4" />
		<AiImageWarningModal ref="aiImageWarningModal" />
		<Modal
			v-if="currentMember"
			ref="modal_edit_item"
			:header="
				editIndex === -1 ? formatMessage(messages.uploadTitle) : formatMessage(messages.editTitle)
			"
		>
			<div class="modal-gallery universal-labels">
				<div class="gallery-file-input">
					<div class="file-header">
						<ImageIcon aria-hidden="true" />
						<strong>{{ editFile ? editFile.name : formatMessage(messages.currentImage) }}</strong>
						<FileButton
							v-if="editIndex === -1"
							:disabled="!globals?.max_gallery_image_size"
							type="outlined"
							class="button-like"
							:prompt="formatMessage(messages.replace)"
							:accept="acceptFileTypes"
							:max-size="globals?.max_gallery_image_size"
							:aria-label="formatMessage(messages.replaceImage)"
							@change="replaceEditFile"
						>
							<TransferIcon aria-hidden="true" />
						</FileButton>
					</div>
					<img
						:src="
							previewImage
								? previewImage
								: filteredGallery[editIndex] && filteredGallery[editIndex].url
									? filteredGallery[editIndex].url
									: 'https://cdn.modrinth.com/placeholder-banner.svg'
						"
						:alt="formatMessage(messages.previewAlt)"
					/>
				</div>
				<label for="gallery-image-title" class="w-fit">
					<span class="label__title">{{ formatMessage(messages.title) }}</span>
				</label>
				<Input
					id="gallery-image-title"
					v-model="editTitle"
					:maxlength="64"
					:placeholder="formatMessage(messages.titlePlaceholder)"
				/>
				<ValidationMessage
					class="mt-2 max-w-[550px]"
					:check="galleryTitleValidation"
					:project-field="filteredGallery[editIndex]?.title ?? ''"
					:current-field="editTitle"
				/>
				<ValidationMessage :check="saveValidation.forField('gallery-text', 'name')" class="mt-2" />
				<ProjectIssueCard
					v-if="editingImageId !== undefined"
					target="modify_gallery_image"
					:image-id="editingImageId"
					class="mt-2"
				/>
				<label for="gallery-image-desc" class="w-fit">
					<span class="label__title">{{ formatMessage(messages.description) }}</span>
				</label>
				<Textarea
					id="gallery-image-desc"
					v-model="editDescription"
					:maxlength="255"
					:placeholder="formatMessage(messages.descriptionPlaceholder)"
				/>
				<ValidationMessage
					class="mt-2 max-w-[550px]"
					:check="galleryDescriptionValidation"
					:project-field="filteredGallery[editIndex]?.description ?? ''"
					:current-field="editDescription"
				/>
				<ValidationMessage
					:check="saveValidation.forField('gallery-text', 'description')"
					class="mt-2"
				/>
				<label for="gallery-image-ordering" class="w-fit">
					<span class="label__title">{{ formatMessage(messages.order) }}</span>
				</label>
				<Input
					id="gallery-image-ordering"
					v-model="editOrder"
					type="number"
					:placeholder="formatMessage(messages.orderPlaceholder)"
				/>
				<label for="gallery-image-featured" class="w-fit">
					<span class="label__title">{{ formatMessage(messages.banner) }}</span>
					<span class="label__description">
						{{ formatMessage(messages.bannerDescription) }}
					</span>
				</label>
				<Button
					v-if="!editFeatured"
					id="gallery-image-featured"
					class="w-fit"
					@click="editFeatured = true"
				>
					<StarIcon aria-hidden="true" />
					{{ formatMessage(messages.setBanner) }}
				</Button>
				<Button v-else id="gallery-image-featured" class="w-fit" @click="editFeatured = false">
					<StarIcon fill="currentColor" aria-hidden="true" />
					{{ formatMessage(messages.unsetBanner) }}
				</Button>
				<div class="mt-3 flex flex-wrap justify-end gap-2">
					<Button type="outlined" @click="modal_edit_item.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						v-if="editIndex === -1"
						type="colored"
						color="brand"
						:disabled="shouldPreventActions || !canSaveGalleryFields"
						@click="createGalleryItem"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.addImage) }}
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="shouldPreventActions || !canSaveGalleryFields"
						@click="editGalleryItem"
					>
						<SaveIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.saveChangesButton) }}
					</Button>
				</div>
			</div>
		</Modal>
		<ConfirmModal
			v-if="currentMember"
			ref="modal_confirm"
			:title="formatMessage(messages.deleteTitle)"
			:description="formatMessage(messages.deleteDescription)"
			:has-to-type="false"
			:proceed-label="formatMessage(commonMessages.deleteLabel)"
			@proceed="deleteGalleryImage"
		/>
		<GalleryImageViewer
			v-if="expandedGalleryItem != null"
			v-model:zoomed-in="zoomedIn"
			:item="expandedGalleryItem"
			:has-multiple-images="filteredGallery.length > 1"
			@close="expandedGalleryItem = null"
			@previous="previousImage"
			@next="nextImage"
		/>
		<div v-if="currentMember" class="card header-buttons">
			<FileButton
				type="colored"
				color="brand"
				:max-size="globals?.max_gallery_image_size"
				:accept="acceptFileTypes"
				:prompt="formatMessage(messages.upload)"
				:aria-label="formatMessage(messages.upload)"
				class="button-like"
				:disabled="
					!globals?.max_gallery_image_size || !isPermission(currentMember?.permissions, 1 << 2)
				"
				@change="handleFiles"
			>
				<UploadIcon aria-hidden="true" />
			</FileButton>
			<span class="indicator">
				<InfoIcon aria-hidden="true" /> {{ formatMessage(messages.uploadHint) }}
			</span>
			<DropArea
				:accept="acceptFileTypes"
				:disabled="!isPermission(currentMember?.permissions, 1 << 2)"
				@change="handleFiles"
			/>
		</div>
		<div class="items">
			<div v-for="(item, index) in filteredGallery" :key="index" class="card gallery-item">
				<a class="gallery-thumbnail" @click="expandImage(item, index)">
					<FullImage
						:src="item.url ? item.url : 'https://cdn.modrinth.com/placeholder-banner.svg'"
						:raw-src="item.raw_url"
						:alt="item.title ? item.title : formatMessage(messages.imageAlt)"
					/>
				</a>
				<div class="gallery-body">
					<div class="gallery-info">
						<h2 v-if="item.title">
							{{ item.title }}
						</h2>
						<p v-if="item.description">
							{{ item.description }}
						</p>
					</div>
					<ProjectIssueCard
						v-if="galleryImageId(item.url) !== undefined"
						target="modify_gallery_image"
						:image-id="galleryImageId(item.url)"
						class="mt-2"
					/>
				</div>
				<div class="gallery-bottom">
					<div class="gallery-created">
						<CalendarIcon aria-hidden="true" :aria-label="formatMessage(messages.dateCreated)" />
						{{ formatDate(item.created) }}
					</div>
					<div v-if="currentMember" class="gallery-buttons input-group">
						<Button
							@click="
								() => {
									resetEdit()
									editIndex = index
									editTitle = item.title
									editDescription = item.description
									editFeatured = item.featured
									editOrder = item.ordering
									modal_edit_item.show()
								}
							"
						>
							<EditIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.editButton) }}
						</Button>
						<Button
							@click="
								() => {
									deleteIndex = index
									modal_confirm.show()
								}
							"
						>
							<TrashIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.removeButton) }}
						</Button>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import {
	CalendarIcon,
	EditIcon,
	ImageIcon,
	InfoIcon,
	PlusIcon,
	SaveIcon,
	StarIcon,
	TransferIcon,
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
	FileButton,
	FullImage,
	injectProjectPageContext,
	Input,
	NewModal as Modal,
	Textarea,
	useFormatBytes,
	useFormatDateTime,
	useGlobals,
	useVIntl,
} from '@modrinth/ui'
import { fileIsValid } from '@modrinth/utils'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { isPermission } from '~/utils/permissions.ts'

import GalleryImageViewer from './image-viewer.vue'
import { messages } from './messages'

const { data: globals } = useGlobals()
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()

const formatDate = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})
const {
	projectV2: project,
	projectV3,
	currentMember,
	createGalleryItem: createGalleryItemMutation,
	editGalleryItem: editGalleryItemMutation,
	deleteGalleryItem: deleteGalleryItemMutation,
} = injectProjectPageContext()

const aiImageWarningModal = ref(null)
const modal_edit_item = ref(null)
const modal_confirm = ref(null)

const expandedGalleryItem = ref(null)
const expandedGalleryIndex = ref(0)
const zoomedIn = ref(false)

const deleteIndex = ref(-1)

const editIndex = ref(-1)
function galleryImageId(url) {
	return projectV3.value?.gallery.find((image) => image.url === url)?.id
}
const editingImageId = computed(() => galleryImageId(filteredGallery.value[editIndex.value]?.url))
const editTitle = ref('')
const editDescription = ref('')
const editFeatured = ref(false)
const editOrder = ref(null)
const editFile = ref(null)
const previewImage = ref(null)
const shouldPreventActions = ref(false)
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
const galleryImagesValidation = useProjectNagMessages('gallery-images')
const saveValidation = useProjectSaveValidation(() => ({
	index: editIndex.value,
	title: editTitle.value,
	description: editDescription.value,
	featured: editFeatured.value,
	ordering: editOrder.value,
	file: previewImage.value,
}))
const canSaveGalleryFields = computed(() => !saveValidation.hasErrors.value)

const nextImage = () => {
	expandedGalleryIndex.value++
	if (expandedGalleryIndex.value >= filteredGallery.value.length) {
		expandedGalleryIndex.value = 0
	}
	expandedGalleryItem.value = filteredGallery.value[expandedGalleryIndex.value]
}

const previousImage = () => {
	expandedGalleryIndex.value--
	if (expandedGalleryIndex.value < 0) {
		expandedGalleryIndex.value = filteredGallery.value.length - 1
	}
	expandedGalleryItem.value = filteredGallery.value[expandedGalleryIndex.value]
}

const expandImage = (item, index) => {
	expandedGalleryItem.value = item
	expandedGalleryIndex.value = index
	zoomedIn.value = false
}

const resetEdit = () => {
	saveValidation.clear()
	editIndex.value = -1
	editTitle.value = ''
	editDescription.value = ''
	editFeatured.value = false
	editOrder.value = null
	editFile.value = null
	previewImage.value = null
}

const handleFiles = async (files) => {
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
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	resetEdit()
	editFile.value = file

	showPreviewImage()
	modal_edit_item.value.show()
}

const replaceEditFile = async (files) => {
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

const showPreviewImage = () => {
	const reader = new FileReader()
	if (editFile.value instanceof Blob) {
		reader.readAsDataURL(editFile.value)
		reader.onload = (event) => {
			previewImage.value = event.target.result
		}
	}
}

const createGalleryItem = async () => {
	if (!canSaveGalleryFields.value || shouldPreventActions.value) return
	shouldPreventActions.value = true

	const submittedState = saveValidation.snapshot()
	try {
		const success = await createGalleryItemMutation(
			editFile.value,
			editTitle.value || undefined,
			editDescription.value || undefined,
			editFeatured.value,
			editOrder.value ?? undefined,
			true,
		)

		if (success) {
			saveValidation.clear()
			modal_edit_item.value.hide()
		}
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	} finally {
		shouldPreventActions.value = false
	}
}

const editGalleryItem = async () => {
	if (!canSaveGalleryFields.value || shouldPreventActions.value) return
	shouldPreventActions.value = true

	const submittedState = saveValidation.snapshot()
	try {
		const success = await editGalleryItemMutation(
			filteredGallery.value[editIndex.value].url,
			editTitle.value,
			editDescription.value,
			editFeatured.value,
			editOrder.value ?? undefined,
			true,
		)

		if (success) {
			saveValidation.clear()
			modal_edit_item.value.hide()
		}
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	} finally {
		shouldPreventActions.value = false
	}
}

const deleteGalleryImage = async () => {
	await deleteGalleryItemMutation(filteredGallery.value[deleteIndex.value].url)
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

.gallery-item {
	display: flex;
	flex-direction: column;
	padding: 0;

	img {
		width: 100%;
		margin-top: 0;
		margin-bottom: 0;
		border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;

		aspect-ratio: 16 / 9;
		object-fit: cover;
	}

	.gallery-body {
		width: calc(100% - 2 * var(--spacing-card-md));
		padding: var(--spacing-card-sm) var(--spacing-card-md);
		overflow-wrap: anywhere;

		.gallery-info {
			h2 {
				margin-bottom: 0.5rem;
			}

			p {
				margin: 0 0 0.5rem 0;
			}
		}
	}

	.gallery-thumbnail {
		cursor: pointer;

		img {
			transition: filter 0.25s ease-in-out;

			&:hover {
				filter: brightness(0.7);
			}
		}
	}

	.gallery-bottom {
		width: calc(100% - 2 * var(--spacing-card-md));
		padding: 0 var(--spacing-card-md) var(--spacing-card-sm) var(--spacing-card-md);

		.gallery-created {
			display: flex;
			align-items: center;
			margin-bottom: 0.5rem;
			color: var(--color-icon);

			svg {
				width: 1rem;
				height: 1rem;
				margin-right: 0.25rem;
			}
		}

		.gallery-buttons {
			display: flex;
		}

		.columns {
			margin-bottom: 0.5rem;
		}
	}
}

.modal-gallery {
	display: flex;
	flex-direction: column;

	.gallery-file-input {
		.file-header {
			border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;

			display: flex;
			align-items: center;
			gap: 0.5rem;
			background-color: var(--color-button-bg);
			padding: var(--spacing-card-md);

			svg {
				min-width: 1rem;
			}
			strong {
				word-wrap: anywhere;
			}

			label.button-like {
				margin-left: auto;
			}
		}

		img {
			border-radius: 0 0 var(--size-rounded-card) var(--size-rounded-card);
			width: 100%;
			height: auto;
			max-height: 15rem;
			object-fit: contain;
			background-color: #000000;
		}
	}
}
</style>
