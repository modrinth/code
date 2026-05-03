<template>
	<div>
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
						<ButtonStyled v-if="editIndex === -1" type="outlined">
							<FileInput
								class="button-like"
								prompt="Replace"
								:accept="acceptFileTypes"
								:max-size="5242880"
								should-always-reset
								aria-label="Replace image"
								@change="
									(x) => {
										editFile = x[0]
										showPreviewImage()
									}
								"
							>
								<TransferIcon aria-hidden="true" />
							</FileInput>
						</ButtonStyled>
					</div>
					<img
						:src="
							previewImage
								? previewImage
								: filteredGallery[editIndex] && filteredGallery[editIndex].url
									? filteredGallery[editIndex].url
									: 'https://cdn.modrinth.com/placeholder-banner.svg'
						"
						alt="gallery-preview"
					/>
				</div>
				<label for="gallery-image-title">
					<span class="label__title">Title</span>
				</label>
				<StyledInput
					id="gallery-image-title"
					v-model="editTitle"
					:maxlength="64"
					placeholder="Enter title..."
				/>
				<label for="gallery-image-desc">
					<span class="label__title">Description</span>
				</label>
				<StyledInput
					id="gallery-image-desc"
					v-model="editDescription"
					multiline
					:maxlength="255"
					placeholder="Enter description..."
				/>
				<label for="gallery-image-ordering">
					<span class="label__title">Order Index</span>
				</label>
				<StyledInput
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
				<ButtonStyled v-if="!editFeatured">
					<button class="w-fit" id="gallery-image-featured" @click="editFeatured = true">
						<StarIcon aria-hidden="true" />
						Set as banner
					</button>
				</ButtonStyled>
				<ButtonStyled v-else>
					<button class="w-fit" id="gallery-image-featured" @click="editFeatured = false">
						<StarIcon fill="currentColor" aria-hidden="true" />
						Unset as banner
					</button>
				</ButtonStyled>
				<div class="button-group">
					<ButtonStyled type="outlined">
						<button @click="modal_edit_item.hide()">
							<XIcon aria-hidden="true" />
							Cancel
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="editIndex === -1" color="brand">
						<button :disabled="shouldPreventActions" @click="createGalleryItem">
							<PlusIcon aria-hidden="true" />
							Add gallery image
						</button>
					</ButtonStyled>
					<ButtonStyled v-else color="brand">
						<button :disabled="shouldPreventActions" @click="editGalleryItem">
							<SaveIcon aria-hidden="true" />
							Save changes
						</button>
					</ButtonStyled>
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
		<div
			v-if="expandedGalleryItem != null"
			class="expanded-image-modal"
			@click="expandedGalleryItem = null"
		>
			<div class="content">
				<img
					class="image"
					:class="{ 'zoomed-in': zoomedIn }"
					:src="
						expandedGalleryItem.raw_url
							? expandedGalleryItem.raw_url
							: 'https://cdn.modrinth.com/placeholder-banner.svg'
					"
					:alt="expandedGalleryItem.title ? expandedGalleryItem.title : 'gallery-image'"
					@click.stop
				/>

				<div class="floating" @click.stop>
					<div class="text">
						<h2 v-if="expandedGalleryItem.title">
							{{ expandedGalleryItem.title }}
						</h2>
						<p v-if="expandedGalleryItem.description">
							{{ expandedGalleryItem.description }}
						</p>
					</div>
					<div class="controls">
						<div class="flex gap-2">
							<ButtonStyled circular>
								<button class="close" @click="expandedGalleryItem = null">
									<XIcon aria-hidden="true" />
								</button>
							</ButtonStyled>
							<ButtonStyled circular>
								<a
									class="open"
									target="_blank"
									:href="
										expandedGalleryItem.raw_url
											? expandedGalleryItem.raw_url
											: 'https://cdn.modrinth.com/placeholder-banner.svg'
									"
								>
									<ExternalIcon aria-hidden="true" />
								</a>
							</ButtonStyled>
							<ButtonStyled circular>
								<button @click="zoomedIn = !zoomedIn">
									<ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
									<ContractIcon v-else aria-hidden="true" />
								</button>
							</ButtonStyled>
							<ButtonStyled v-if="filteredGallery.length > 1" circular>
								<button class="previous" @click="previousImage()">
									<LeftArrowIcon aria-hidden="true" />
								</button>
							</ButtonStyled>
							<ButtonStyled v-if="filteredGallery.length > 1" circular>
								<button class="next" @click="nextImage()">
									<RightArrowIcon aria-hidden="true" />
								</button>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div v-if="currentMember" class="card header-buttons">
			<ButtonStyled color="brand">
				<FileInput
					:max-size="5242880"
					:accept="acceptFileTypes"
					prompt="Upload an image"
					aria-label="Upload an image"
					class="button-like"
					:disabled="!isPermission(currentMember?.permissions, 1 << 2)"
					@change="handleFiles"
				>
					<UploadIcon aria-hidden="true" />
				</FileInput>
			</ButtonStyled>
			<span class="indicator">
				<InfoIcon aria-hidden="true" /> Click to choose an image or drag one onto this page
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
					<img
						:src="item.url ? item.url : 'https://cdn.modrinth.com/placeholder-banner.svg'"
						:alt="item.title ? item.title : 'gallery-image'"
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
						<ButtonStyled>
							<button
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
								Edit
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<button
								@click="
									() => {
										deleteIndex = index
										modal_confirm.show()
									}
								"
							>
								<TrashIcon aria-hidden="true" />
								Remove
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import {
	CalendarIcon,
	ContractIcon,
	EditIcon,
	ExpandIcon,
	ExternalIcon,
	ImageIcon,
	InfoIcon,
	LeftArrowIcon,
	PlusIcon,
	RightArrowIcon,
	SaveIcon,
	StarIcon,
	TransferIcon,
	TrashIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	DropArea,
	FileInput,
	injectProjectPageContext,
	NewModal as Modal,
	StyledInput,
	useFormatDateTime,
} from '@modrinth/ui'

import { isPermission } from '~/utils/permissions.ts'

const formatDate = useFormatDateTime({
	year: 'numeric',
	month: 'long',
	day: 'numeric',
})

const {
	projectV2: project,
	currentMember,
	createGalleryItem: createGalleryItemMutation,
	editGalleryItem: editGalleryItemMutation,
	deleteGalleryItem: deleteGalleryItemMutation,
} = injectProjectPageContext()

const title = `${project.value.title} - Gallery`
const description = `View ${project.value.gallery?.length ?? 0} images of ${project.value.title} on Modrinth.`

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

const modal_edit_item = ref(null)
const modal_confirm = ref(null)

const expandedGalleryItem = ref(null)
const expandedGalleryIndex = ref(0)
const zoomedIn = ref(false)

const deleteIndex = ref(-1)

const editIndex = ref(-1)
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
	editIndex.value = -1
	editTitle.value = ''
	editDescription.value = ''
	editFeatured.value = false
	editOrder.value = null
	editFile.value = null
	previewImage.value = null
}

const handleFiles = (files) => {
	resetEdit()
	editFile.value = files[0]

	showPreviewImage()
	modal_edit_item.value.show()
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
	shouldPreventActions.value = true

	const success = await createGalleryItemMutation(
		editFile.value,
		editTitle.value || undefined,
		editDescription.value || undefined,
		editFeatured.value,
		editOrder.value ?? undefined,
	)

	if (success) {
		modal_edit_item.value.hide()
	}

	shouldPreventActions.value = false
}

const editGalleryItem = async () => {
	shouldPreventActions.value = true

	const success = await editGalleryItemMutation(
		filteredGallery.value[editIndex.value].url,
		editTitle.value,
		editDescription.value,
		editFeatured.value,
		editOrder.value ?? undefined,
	)

	if (success) {
		modal_edit_item.value.hide()
	}

	shouldPreventActions.value = false
}

const deleteGalleryImage = async () => {
	await deleteGalleryItemMutation(filteredGallery.value[deleteIndex.value].url)
}

const handleKeydown = (e) => {
	if (expandedGalleryItem.value) {
		e.preventDefault()
		if (e.key === 'Escape') {
			expandedGalleryItem.value = null
		} else if (e.key === 'ArrowLeft') {
			e.stopPropagation()
			previousImage()
		} else if (e.key === 'ArrowRight') {
			e.stopPropagation()
			nextImage()
		}
	}
}

onMounted(() => {
	document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
	document.removeEventListener('keydown', handleKeydown)
})
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

.expanded-image-modal {
	position: fixed;
	z-index: 20;
	overflow: auto;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background-color: #000000;
	background-color: rgba(0, 0, 0, 0.7);
	display: flex;
	justify-content: center;
	align-items: center;

	.content {
		position: relative;
		width: calc(100vw - 2 * var(--spacing-card-lg));
		height: calc(100vh - 2 * var(--spacing-card-lg));

		.image {
			position: absolute;
			left: 50%;
			top: 50%;
			transform: translate(-50%, -50%);
			max-width: calc(100vw - 2 * var(--spacing-card-lg));
			max-height: calc(100vh - 2 * var(--spacing-card-lg));
			border-radius: var(--size-rounded-card);

			&.zoomed-in {
				object-fit: cover;
				width: auto;
				height: calc(100vh - 2 * var(--spacing-card-lg));
				max-width: calc(100vw - 2 * var(--spacing-card-lg));
			}
		}
		.floating {
			position: absolute;
			left: 50%;
			transform: translateX(-50%);
			bottom: var(--spacing-card-md);
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: var(--spacing-card-sm);
			transition: opacity 0.25s ease-in-out;
			opacity: 1;
			padding: 2rem 2rem 0 2rem;

			&:not(&:hover) {
				opacity: 0.4;
				.text {
					transform: translateY(2.5rem) scale(0.8);
					opacity: 0;
				}
				.controls {
					transform: translateY(0.25rem) scale(0.9);
				}
			}

			.text {
				display: flex;
				flex-direction: column;
				max-width: 40rem;
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
				text-shadow: 1px 1px 10px #000000d4;
				margin-bottom: 0.25rem;
				gap: 0.5rem;

				h2 {
					color: var(--dark-color-text-dark);
					font-size: 1.25rem;
					text-align: center;
					margin: 0;
				}

				p {
					color: var(--dark-color-text);
					margin: 0;
				}
			}
			.controls {
				background-color: var(--color-raised-bg);
				padding: var(--spacing-card-md);
				border-radius: var(--size-rounded-card);
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
			}
		}
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
