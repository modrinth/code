<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<Modal
			v-if="currentMember"
			ref="modal_edit_item"
			:header="editIndex === -1 ? 'Upload gallery image' : 'Edit gallery item'"
		>
			<div class="modal-gallery universal-labels">
				<div class="gallery-file-input">
					<div class="file-header">
						<ImageIcon aria-hidden="true" />
						<strong>{{ editFile ? editFile.name : 'Current image' }}</strong>
						<FileButton
							v-if="editIndex === -1"
							type="outlined"
							class="button-like"
							prompt="Replace"
							:accept="acceptFileTypes"
							:max-size="5242880"
							aria-label="Replace image"
							@change="replaceEditFile"
						>
							<TransferIcon aria-hidden="true" />
						</FileButton>
					</div>
					<img
						:src="
							previewImage
								? previewImage
								: filteredGallery[editIndex]?.url
									? filteredGallery[editIndex].url
									: 'https://cdn.modrinth.com/placeholder-banner.svg'
						"
						alt="gallery-preview"
					/>
				</div>
				<label for="gallery-image-title">
					<span class="label__title">Title</span>
				</label>
				<Input
					id="gallery-image-title"
					v-model="editTitle"
					:maxlength="64"
					placeholder="Enter title..."
				/>
				<label for="gallery-image-desc">
					<span class="label__title">Description</span>
				</label>
				<Textarea
					id="gallery-image-desc"
					v-model="editDescription"
					:maxlength="255"
					placeholder="Enter description..."
				/>
				<label for="gallery-image-ordering">
					<span class="label__title">Order Index</span>
				</label>
				<Input
					id="gallery-image-ordering"
					v-model="editOrder"
					type="number"
					placeholder="Enter order index..."
				/>
				<label for="gallery-image-featured">
					<span class="label__title">Banner image</span>
					<span class="label__description">
						You can feature one image on your project to be used as a banner image.
					</span>
				</label>
				<Button
					v-if="!editFeatured"
					id="gallery-image-featured"
					class="w-fit"
					@click="editFeatured = true"
				>
					<StarIcon aria-hidden="true" />
					Set as banner
				</Button>
				<Button v-else id="gallery-image-featured" class="w-fit" @click="editFeatured = false">
					<StarIcon fill="currentColor" aria-hidden="true" />
					Unset as banner
				</Button>
				<div class="mt-3 flex flex-wrap justify-end gap-2">
					<Button type="outlined" @click="modalEditItem?.hide()">
						<XIcon aria-hidden="true" />
						Cancel
					</Button>
					<Button
						v-if="editIndex === -1"
						type="colored"
						color="brand"
						:disabled="shouldPreventActions"
						@click="createGalleryItem"
					>
						<PlusIcon aria-hidden="true" />
						Add gallery image
					</Button>
					<Button
						v-else
						type="colored"
						color="brand"
						:disabled="shouldPreventActions"
						@click="editGalleryItem"
					>
						<SaveIcon aria-hidden="true" />
						Save changes
					</Button>
				</div>
			</div>
		</Modal>
		<ConfirmModal
			v-if="currentMember"
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
				:max-size="5242880"
				:accept="acceptFileTypes"
				prompt="Upload an image"
				aria-label="Upload an image"
				class="button-like"
				:disabled="!isPermission(currentMember?.permissions, 1 << 2)"
				@change="handleFiles"
			>
				<UploadIcon aria-hidden="true" />
			</FileButton>
			<span class="indicator">
				<InfoIcon aria-hidden="true" /> Click to choose an image or drag one onto this page
			</span>
			<DropArea
				:accept="acceptFileTypes"
				:disabled="!isPermission(currentMember?.permissions, 1 << 2)"
				@change="handleFiles"
			/>
		</div>
		<div v-if="filteredGallery.length" class="items">
			<div v-for="(item, index) in filteredGallery" :key="index" class="card gallery-item">
				<a class="gallery-thumbnail" @click="expandImage(index)">
					<img
						:src="item.url ? item.url : 'https://cdn.modrinth.com/placeholder-banner.svg'"
						:alt="item.title ? item.title : 'gallery-image'"
						@contextmenu="onFullImageContextMenu($event, item.raw_url)"
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
				</div>
				<div class="gallery-bottom">
					<div class="gallery-created">
						<CalendarIcon aria-hidden="true" aria-label="Date created" />
						{{ formatDate(item.created) }}
					</div>
					<div v-if="currentMember" class="gallery-buttons input-group">
						<Button
							@click="
								() => {
									resetEdit()
									editIndex = index
									editTitle = item.title ?? ''
									editDescription = item.description ?? ''
									editFeatured = item.featured
									editOrder = item.ordering
									modalEditItem?.show()
								}
							"
						>
							<EditIcon aria-hidden="true" />
							Edit
						</Button>
						<Button
							@click="
								() => {
									deleteIndex = index
									modalConfirm?.show()
								}
							"
						>
							<TrashIcon aria-hidden="true" />
							Remove
						</Button>
					</div>
				</div>
			</div>
		</div>
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
import {
	CalendarIcon,
	EditIcon,
	ExternalIcon,
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
	ConfirmModal,
	DropArea,
	FileButton,
	ImageViewerEditor,
	injectProjectPageContext,
	Input,
	NewModal as Modal,
	Textarea,
	useFormatDateTime,
	useFullImageContextMenu,
} from '@modrinth/ui'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { isPermission } from '~/utils/permissions.ts'

const formatDate = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})
const onFullImageContextMenu = useFullImageContextMenu()

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
const deleteIndex = ref(-1)

// Edit state
const editIndex = ref(-1)
const editTitle = ref('')
const editDescription = ref('')
const editFeatured = ref(false)
const editOrder = ref<number | null>(null)
const editFile = ref<File | null>(null)
const previewImage = ref<string | null>(null)

// UI state
const shouldPreventActions = ref(false)

// Constant for accepted file types
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const acceptFileTypes = 'image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp'

const filteredGallery = computed(
	() => project.value.gallery?.filter((img) => img.title !== MC_SERVER_BANNER_NAME) ?? [],
)

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
function resetEdit() {
	editIndex.value = -1
	editTitle.value = ''
	editDescription.value = ''
	editFeatured.value = false
	editOrder.value = null
	editFile.value = null
	previewImage.value = null
}

async function handleFiles(files: File[]) {
	const file = files[0]
	if (!file) {
		return
	}
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	resetEdit()
	editFile.value = file

	showPreviewImage()
	modalEditItem.value?.show()
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
	shouldPreventActions.value = true
	startLoading()

	const success = await contextCreateGalleryItem(
		editFile.value!,
		editTitle.value || undefined,
		editDescription.value || undefined,
		editFeatured.value,
		editOrder.value ? Number(editOrder.value) : undefined,
	)

	if (success) {
		modalEditItem.value?.hide()
	}

	stopLoading()
	shouldPreventActions.value = false
}

async function editGalleryItem() {
	shouldPreventActions.value = true
	startLoading()

	const imageUrl = filteredGallery.value[editIndex.value].url
	const success = await contextEditGalleryItem(
		imageUrl,
		editTitle.value,
		editDescription.value,
		editFeatured.value,
		editOrder.value ? Number(editOrder.value) : undefined,
	)

	if (success) {
		modalEditItem.value?.hide()
	}

	stopLoading()
	shouldPreventActions.value = false
}

async function deleteGalleryImage() {
	startLoading()

	const imageUrl = filteredGallery.value[deleteIndex.value].url!
	await contextDeleteGalleryItem(imageUrl)

	stopLoading()
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
