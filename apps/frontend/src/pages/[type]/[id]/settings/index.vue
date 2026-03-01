<template>
	<div>
		<ConfirmModal
			ref="modal_confirm"
			title="Are you sure you want to delete this project?"
			description="If you proceed, all versions and any attached data will be removed from our servers. This may break other projects, so be careful."
			:has-to-type="true"
			:confirmation-text="project.title"
			proceed-label="Delete"
			@proceed="deleteProject"
		/>
		<section class="universal-card">
			<div class="flex max-w-[600px] flex-col gap-6">
				<div class="label">
					<h3>
						<span class="label__title size-card-header">Project information</span>
					</h3>
				</div>

				<div>
					<label for="project-name">
						<span class="label__title">Name</span>
					</label>
					<StyledInput
						id="project-name"
						v-model="name"
						:maxlength="2048"
						:disabled="!hasPermission"
					/>
				</div>

				<div>
					<label for="project-slug">
						<span class="label__title">URL</span>
					</label>
					<div class="text-input-wrapper !w-full">
						<div class="text-input-wrapper__before">
							<span class="hidden sm:inline">https://modrinth.com</span>/{{
								$getProjectTypeForUrl(project.project_type, project.loaders)
							}}/
						</div>
						<StyledInput
							id="project-slug"
							v-model="slug"
							:maxlength="64"
							autocomplete="off"
							:disabled="!hasPermission"
						/>
					</div>
				</div>

				<div>
					<label for="project-summary">
						<span class="label__title">Summary</span>
					</label>
					<StyledInput
						id="project-summary"
						v-model="summary"
						multiline
						:maxlength="256"
						:disabled="!hasPermission"
						resize="vertical"
					/>
					<div v-if="summaryWarning" class="my-2 flex items-center gap-1.5 text-orange">
						<TriangleAlertIcon class="my-auto" />
						{{ summaryWarning }}
					</div>
				</div>

				<div>
					<label for="project-icon">
						<span class="label__title"
							>Icon <span class="font-normal text-secondary">(optional)</span></span
						>
					</label>

					<div class="input-group">
						<Avatar
							:src="deletedIcon ? null : previewImage ? previewImage : project.icon_url"
							:alt="project.title"
							size="md"
							class="project__icon"
						/>
						<div class="input-stack">
							<FileInput
								id="project-icon"
								:max-size="262144"
								:show-icon="true"
								accept="image/png,image/jpeg,image/gif,image/webp"
								class="choose-image iconified-button"
								prompt="Upload icon"
								aria-label="Upload icon"
								:disabled="!hasPermission"
								@change="showPreviewImage"
							>
								<UploadIcon aria-hidden="true" />
							</FileInput>
							<button
								v-if="!deletedIcon && (previewImage || project.icon_url)"
								class="iconified-button"
								:disabled="!hasPermission"
								@click="markIconForDeletion"
							>
								<TrashIcon aria-hidden="true" />
								Remove icon
							</button>
						</div>
					</div>
				</div>

				<!-- Server Project Settings -->
				<template v-if="isServerProject">
					<!-- Banner -->
					<div>
						<label>
							<span class="label__title"
								>Banner <span class="font-normal text-secondary">(optional)</span></span
							>
						</label>
						<div class="mt-2">
							<label
								class="flex cursor-pointer flex-col items-center justify-center rounded-2xl border-dashed border-surface-5 transition-colors"
								:class="
									!deletedBanner && (bannerPreview || bannerGalleryImage?.url)
										? 'border-none'
										: 'aspect-[468/60] border-2 bg-surface-2'
								"
							>
								<div
									v-if="!deletedBanner && (bannerPreview || bannerGalleryImage?.url)"
									class="relative h-full w-full overflow-hidden rounded-2xl"
								>
									<img
										:src="bannerPreview || bannerGalleryImage?.url"
										alt="Banner preview"
										class="h-full w-full object-cover"
									/>
								</div>
								<ImageIcon v-else aria-hidden="true" class="h-8 w-8 text-secondary" />
								<input
									type="file"
									accept="image/png,image/jpeg,image/gif,image/webp"
									class="hidden"
									:disabled="!hasPermission"
									@change="
										(e) => {
											const input = e.target
											if (input.files?.length) {
												if (fileIsValid(input.files[0], { maxSize: 524288, alertOnInvalid: true }))
													showBannerPreview(Array.from(input.files))
											}
										}
									"
								/>
							</label>
						</div>
						<div class="mt-2 flex items-center gap-2">
							<FileInput
								:max-size="524288"
								:show-icon="true"
								accept="image/png,image/jpeg,image/gif,image/webp"
								class="iconified-button"
								prompt="Upload banner"
								:disabled="!hasPermission"
								@change="showBannerPreview"
							>
								<UploadIcon aria-hidden="true" />
							</FileInput>
							<button
								v-if="!deletedBanner && (bannerPreview || bannerGalleryImage?.url)"
								class="iconified-button"
								:disabled="!hasPermission"
								@click="markBannerForDeletion"
							>
								<TrashIcon aria-hidden="true" />
								Remove banner
							</button>
						</div>
						<div class="mt-2 text-secondary">Gif, 468×60px recommended.</div>
					</div>
				</template>

				<template
					v-if="
						!isServerProject &&
						!flags.newProjectEnvironmentSettings &&
						project.versions?.length !== 0 &&
						project.project_type !== 'resourcepack' &&
						project.project_type !== 'plugin' &&
						project.project_type !== 'shader' &&
						project.project_type !== 'datapack'
					"
				>
					<div class="adjacent-input">
						<label for="project-env-client">
							<span class="label__title">Client-side</span>
							<span class="label__description">
								Select based on if the
								{{ formatProjectType(project.project_type).toLowerCase() }} has functionality on the
								client side. Just because a mod works in Singleplayer doesn't mean it has actual
								client-side functionality.
							</span>
						</label>
						<Combobox
							v-model="clientSide"
							:options="sideTypeOptions"
							placeholder="Select one"
							:disabled="!hasPermission"
						/>
					</div>
					<div class="adjacent-input">
						<label for="project-env-server">
							<span class="label__title">Server-side</span>
							<span class="label__description">
								Select based on if the
								{{ formatProjectType(project.project_type).toLowerCase() }} has functionality on the
								<strong>logical</strong> server. Remember that Singleplayer contains an integrated
								server.
							</span>
						</label>
						<Combobox
							v-model="serverSide"
							:options="sideTypeOptions"
							placeholder="Select one"
							:disabled="!hasPermission"
						/>
					</div>
				</template>
				<div>
					<label>
						<span class="label__title">Visibility</span>
					</label>
					<div class="flex flex-col gap-2.5">
						<Combobox
							v-model="visibility"
							:options="visibilityOptions"
							placeholder="Select one"
							:disabled="!hasPermission"
							:max-height="500"
						/>
						<div>If approved by the moderators:</div>
						<ul class="visibility-info m-0">
							<li>
								<CheckIcon
									v-if="visibility === 'approved' || visibility === 'archived'"
									class="good"
								/>
								<XIcon v-else class="bad" />
								{{ hasModifiedVisibility() ? 'Will be v' : 'V' }}isible in search
							</li>
							<li>
								<XIcon v-if="visibility === 'unlisted' || visibility === 'private'" class="bad" />
								<CheckIcon v-else class="good" />
								{{ hasModifiedVisibility() ? 'Will be v' : 'V' }}isible on profile
							</li>
							<li>
								<CheckIcon v-if="visibility !== 'private'" class="good" />
								<IssuesIcon
									v-else
									v-tooltip="{
										content:
											visibility === 'private'
												? 'Only members will be able to view the project.'
												: '',
									}"
									class="warn"
								/>
								{{ hasModifiedVisibility() ? 'Will be v' : 'V' }}isible via URL
							</li>
						</ul>
					</div>
				</div>
			</div>
		</section>

		<section class="universal-card">
			<div class="label">
				<h3>
					<span class="label__title size-card-header">Delete project</span>
				</h3>
			</div>
			<p>
				Removes your project from Modrinth's servers and search. Clicking on this will delete your
				project, so be extra careful!
			</p>
			<button
				type="button"
				class="iconified-button danger-button"
				:disabled="!hasDeletePermission"
				@click="$refs.modal_confirm.show()"
			>
				<TrashIcon aria-hidden="true" />
				Delete project
			</button>
		</section>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			@reset="resetChanges"
			@save="handleSave"
		/>
	</div>
</template>

<script setup>
import {
	CheckIcon,
	ImageIcon,
	IssuesIcon,
	TrashIcon,
	TriangleAlertIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { MIN_SUMMARY_CHARS } from '@modrinth/moderation'
import {
	Avatar,
	Combobox,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	StyledInput,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import { fileIsValid, formatProjectStatus, formatProjectType } from '@modrinth/utils'

import FileInput from '~/components/ui/FileInput.vue'
import { useFeatureFlags } from '~/composables/featureFlags.ts'

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	projectV3,
	currentMember,
	patchProject,
	patchIcon,
	invalidate,
} = injectProjectPageContext()
const { labrinth } = injectModrinthClient()

const flags = useFeatureFlags()

const tags = useGeneratedState()
const router = useNativeRouter()

const name = ref(project.value.title)
const slug = ref(project.value.slug)
const summary = ref(project.value.description)
const icon = ref(null)
const previewImage = ref(null)
const clientSide = ref(project.value.client_side)
const serverSide = ref(project.value.server_side)
const deletedIcon = ref(false)
const visibility = ref(
	tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status,
)

// Server project specific refs
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const bannerPreview = ref(null)
const deletedBanner = ref(false)
const bannerFile = ref(null)
const bannerGalleryImage = computed(() =>
	project.value.gallery?.find((img) => img.title === MC_SERVER_BANNER_NAME),
)
const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
})

const hasDeletePermission = computed(() => {
	const DELETE_PROJECT = 1 << 7
	return ((currentMember.value?.permissions ?? 0) & DELETE_PROJECT) === DELETE_PROJECT
})

const summaryWarning = computed(() => {
	const text = summary.value?.trim() || ''
	const charCount = text.length

	if (charCount < MIN_SUMMARY_CHARS) {
		return `It's recommended to have a summary with at least ${MIN_SUMMARY_CHARS} characters. (${charCount}/${MIN_SUMMARY_CHARS})`
	}

	return null
})

const sideTypeOptions = [
	{ value: 'required', label: 'Required' },
	{ value: 'optional', label: 'Optional' },
	{ value: 'unsupported', label: 'Unsupported' },
]

const visibilityOptions = computed(() =>
	tags.value.approvedStatuses.map((status) => {
		const subLabel = () => {
			switch (status) {
				case 'approved':
					return 'Visible via URL, on your profile, and in search.'
				case 'archived':
					return 'Visible via URL, on your profile, and in search, but marked as archived.'
				case 'unlisted':
					return 'Visible via URL only. Not shown on your profile or in search.'
				case 'private':
					return 'Not publicly visible. Only accessible to project members.'
				default:
					return ''
			}
		}
		return {
			value: status,
			label: formatProjectStatus(status),
			subLabel: subLabel(),
		}
	}),
)

const basePatchData = computed(() => {
	const data = {}

	if (name.value !== project.value.title) {
		data.title = name.value.trim()
	}
	if (slug.value !== project.value.slug) {
		data.slug = slug.value.trim()
	}
	if (summary.value !== project.value.description) {
		data.description = summary.value.trim()
	}
	if (clientSide.value !== project.value.client_side) {
		data.client_side = clientSide.value
	}
	if (serverSide.value !== project.value.server_side) {
		data.server_side = serverSide.value
	}
	if (tags.value.approvedStatuses.includes(project.value.status)) {
		if (visibility.value !== project.value.status) {
			data.status = visibility.value
		}
	} else if (visibility.value !== project.value.requested_status) {
		data.requested_status = visibility.value
	}

	return data
})

const saving = ref(false)

const original = computed(() => ({
	name: project.value.title,
	slug: project.value.slug,
	summary: project.value.description,
	clientSide: project.value.client_side,
	serverSide: project.value.server_side,
	visibility: tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status,
	icon: null,
	deletedIcon: false,
	bannerFile: null,
	deletedBanner: false,
}))

const modified = computed(() => ({
	name: name.value,
	slug: slug.value,
	summary: summary.value,
	clientSide: clientSide.value,
	serverSide: serverSide.value,
	visibility: visibility.value,
	icon: icon.value,
	deletedIcon: deletedIcon.value,
	bannerFile: bannerFile.value,
	deletedBanner: deletedBanner.value,
}))

function resetChanges() {
	name.value = project.value.title
	slug.value = project.value.slug
	summary.value = project.value.description
	clientSide.value = project.value.client_side
	serverSide.value = project.value.server_side
	visibility.value = tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status
	icon.value = null
	previewImage.value = null
	deletedIcon.value = false
	bannerFile.value = null
	bannerPreview.value = null
	deletedBanner.value = false
}

const hasModifiedVisibility = () => {
	const originalVisibility = tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status

	return originalVisibility !== visibility.value
}

async function handleSave() {
	saving.value = true
	try {
		const hasV2Changes = Object.keys(basePatchData.value).length > 0

		if (hasV2Changes) {
			await patchProject(basePatchData.value)
		}

		if (deletedIcon.value) {
			await deleteIcon()
			deletedIcon.value = false
		} else if (icon.value) {
			await patchIcon(icon.value)
			icon.value = null
		}

		if (deletedBanner.value) {
			await deleteBanner()
			deletedBanner.value = false
		} else if (bannerFile.value) {
			await uploadBanner()
			bannerFile.value = null
			bannerPreview.value = null
		}
	} finally {
		saving.value = false
	}
}

const showPreviewImage = (files) => {
	const reader = new FileReader()
	icon.value = files[0]
	deletedIcon.value = false
	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target?.result
	}
}

const showBannerPreview = (files) => {
	const file = files[0]
	if (file) {
		bannerFile.value = file
		const reader = new FileReader()
		reader.onload = (e) => {
			bannerPreview.value = e.target.result
		}
		reader.readAsDataURL(file)
		deletedBanner.value = false
	}
}

const markBannerForDeletion = () => {
	bannerPreview.value = null
	bannerFile.value = null
	deletedBanner.value = true
}

const uploadBanner = async () => {
	if (!bannerFile.value) return

	try {
		// First, delete existing banner image if there is one
		const existingBanner = project.value.gallery?.find(
			(img) => img.title === MC_SERVER_BANNER_NAME,
		)
		if (existingBanner) {
			await labrinth.projects_v2.deleteGalleryImage(project.value.id, existingBanner.url)
		}

		// Upload new banner as gallery image with special title
		const ext = bannerFile.value.type.split('/').pop() ?? 'png'
		await labrinth.projects_v2.createGalleryImage(project.value.id, bannerFile.value, {
			ext,
			featured: false,
			title: MC_SERVER_BANNER_NAME,
		})

		await invalidate()
		addNotification({
			title: 'Banner updated',
			text: 'Your project banner has been updated.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'Failed to update banner',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	}
}

const deleteBanner = async () => {
	try {
		const bannerImage = project.value.gallery?.find(
			(img) => img.title === MC_SERVER_BANNER_NAME,
		)
		if (bannerImage) {
			await labrinth.projects_v2.deleteGalleryImage(project.value.id, bannerImage.url)
			await invalidate()
			addNotification({
				title: 'Banner removed',
				text: 'Your project banner has been removed.',
				type: 'success',
			})
		}
	} catch (err) {
		addNotification({
			title: 'Failed to remove banner',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	}
}

const deleteProject = async () => {
	await useBaseFetch(`project/${project.value.id}`, {
		method: 'DELETE',
	})
	await initUserProjects()
	await router.push('/dashboard/projects')
	addNotification({
		title: 'Project deleted',
		text: 'Your project has been deleted.',
		type: 'success',
	})
}

const markIconForDeletion = () => {
	deletedIcon.value = true
	icon.value = null
	previewImage.value = null
}

const deleteIcon = async () => {
	await useBaseFetch(`project/${project.value.id}/icon`, {
		method: 'DELETE',
	})
	await invalidate()
	addNotification({
		title: 'Project icon removed',
		text: "Your project's icon has been removed.",
		type: 'success',
	})
}
</script>

<style lang="scss" scoped>
.visibility-info {
	padding: 0;
	list-style: none;
	display: flex;
	flex-direction: column;
	gap: var(--spacing-card-xs);

	li {
		display: flex;
		align-items: center;
		gap: var(--spacing-card-xs);
	}
}

svg {
	&.good {
		color: var(--color-green);
	}

	&.bad {
		color: var(--color-red);
	}

	&.warn {
		color: var(--color-orange);
	}
}

.button-group {
	justify-content: flex-start;
}
</style>
