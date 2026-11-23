<template>
	<AiImageWarningModal ref="aiImageWarningModal" />
	<h2 class="settings-section-title">About</h2>
	<section class="universal-card settings-content-card">
		<div class="flex flex-col gap-6">
			<Field
				id="project-settings-field-title"
				label="Name"
				input-id="project-name"
				data-project-field="title"
				class="max-w-[600px]"
			>
				<template #right>
					<span class="text-xs text-secondary">{{ name.length }} / 64</span>
				</template>
				<Input
					id="project-name"
					v-model="name"
					:maxlength="64"
					wrapper-class="w-full"
					:disabled="!hasPermission"
				/>
				<template #bottom>
					<ValidationMessage
						:check="nameValidation"
						:project-field="project.name"
						:current-field="name"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('name')" class="mt-2" />
					<ProjectIssueCard target="modify_title" :field-action="nameIssueAction" class="mt-2" />
				</template>
			</Field>

			<Field
				id="project-settings-field-summary"
				label="Summary"
				input-id="project-summary"
				data-project-field="summary"
				class="max-w-[600px]"
			>
				<template #right>
					<span class="text-xs text-secondary">{{ summary.length }} / 256</span>
				</template>
				<Textarea
					id="project-summary"
					v-model="summary"
					:maxlength="256"
					:disabled="!hasPermission"
					resize="vertical"
				/>
				<template #bottom>
					<p class="mb-0 mt-2 text-sm text-secondary">
						{{ formatMessage(projectSummaryHelp) }}
					</p>
					<ValidationMessage
						:check="summaryValidation"
						:project-field="project.summary"
						:current-field="summary"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('summary')" class="mt-2" />
					<ProjectIssueCard
						target="modify_summary"
						:field-action="summaryIssueAction"
						class="mt-2"
					/>
				</template>
			</Field>

			<div class="flex max-w-[600px] flex-wrap items-start gap-x-8 gap-y-6">
				<Field
					id="project-settings-field-icon"
					label="Icon"
					label-optional
					data-project-field="icon"
					:class="isServerProject ? 'min-w-32' : 'w-full'"
				>
					<template #default="{ labelId }">
						<div
							class="flex flex-wrap items-start gap-2"
							:class="{ 'flex-col': isServerProject }"
							role="group"
							:aria-labelledby="labelId"
						>
							<Avatar
								:src="iconSrc"
								:alt="project.name"
								size="108px"
								class="project__icon"
								no-shadow
							/>
							<div class="max-w-full">
								<div class="flex items-center gap-2">
									<FileButton
										id="project-icon"
										:max-size="globals?.max_project_icon_size"
										accept="image/png,image/jpeg,image/gif,image/webp"
										class="button-like choose-image"
										:prompt="formatMessage(commonProjectSettingsMessages.upload)"
										:disabled="!globals?.max_project_icon_size || !hasPermission"
										@change="showPreviewImage"
									>
										<UploadIcon aria-hidden="true" />
									</FileButton>
									<IconButton
										v-if="iconSrc"
										class="shrink-0"
										:label="removeIconLabel"
										:circular="false"
										:disabled="!hasPermission"
										@click="markIconForDeletion"
									>
										<TrashIcon aria-hidden="true" />
									</IconButton>
								</div>
								<div class="mt-2 text-sm text-secondary">Square, 512 px</div>
							</div>
						</div>
					</template>
					<template #bottom>
						<ValidationMessage
							:check="iconValidation"
							:project-field="false"
							:current-field="Boolean(icon || deletedIcon)"
							class="mt-2"
						/>
						<ValidationMessage :check="saveValidation.forField('icon')" class="mt-2" />
						<ProjectIssueCard
							:target="['modify_icon', 'remove_icon']"
							:field-action="iconIssueAction"
							class="mt-2"
						/>
					</template>
				</Field>

				<Field v-if="isServerProject" label="Banner" label-optional class="w-[324px] max-w-full">
					<label
						class="relative flex h-[108px] cursor-pointer items-center justify-center overflow-hidden rounded-2xl"
						:class="{ 'border-2 border-dashed border-surface-5 bg-surface-2': !bannerSrc }"
					>
						<img
							v-if="bannerSrc"
							:src="bannerSrc"
							alt="Banner preview"
							class="absolute inset-0 h-full w-full object-contain"
						/>
						<ImageIcon v-else aria-hidden="true" class="h-8 w-8 text-secondary" />
						<input
							type="file"
							accept="image/png,image/jpeg,image/gif,image/webp"
							class="hidden"
							:disabled="!globals?.max_gallery_image_size || !hasPermission"
							@change="
								(e) => {
									const input = e.target
									if (input.files?.length) {
										if (
											globals?.max_gallery_image_size &&
											fileIsValid(
												input.files[0],
												{
													maxSize: globals.max_gallery_image_size,
													alertOnInvalid: true,
												},
												formatBytes,
											)
										)
											showBannerPreview(Array.from(input.files))
									}
								}
							"
						/>
					</label>
					<div class="mt-2 flex items-center gap-2">
						<FileButton
							:max-size="globals?.max_gallery_image_size"
							accept="image/png,image/jpeg,image/gif,image/webp"
							class="button-like"
							prompt="Upload banner"
							:disabled="!globals?.max_gallery_image_size || !hasPermission"
							@change="showBannerPreview"
						>
							<UploadIcon aria-hidden="true" />
						</FileButton>
						<IconButton
							v-if="bannerSrc"
							class="shrink-0"
							label="Remove banner"
							:circular="false"
							:disabled="!hasPermission"
							@click="markBannerForDeletion"
						>
							<TrashIcon aria-hidden="true" />
						</IconButton>
					</div>
					<div class="mt-2 text-sm text-secondary">3:1, 1920 × 640 px</div>
				</Field>
			</div>

			<Field
				id="project-settings-field-slug"
				label="URL"
				input-id="project-slug"
				data-project-field="slug"
				class="max-w-[600px]"
			>
				<Input
					id="project-slug"
					v-model="slug"
					:maxlength="64"
					autocomplete="off"
					:disabled="!hasPermission"
					wrapper-class="w-full"
					@focusin="onSlugSuggestionFocusIn"
					@focusout="onSlugSuggestionFocusOut"
				>
					<template #prefix>
						<span class="whitespace-nowrap">
							<span class="hidden sm:inline">https://modrinth.com</span>/{{ projectTypeForUrl }}/
						</span>
					</template>
				</Input>
				<template #bottom>
					<ValidationMessage :check="saveValidation.forField('slug')" class="mt-2" />
					<ProjectIssueCard target="modify_slug" :field-action="slugIssueAction" class="mt-2" />
					<SlugSuggestions
						:selected="slug"
						:suggestions="slugSuggestions"
						:visible="hasPermission && showSlugSuggestions"
						@select="slug = $event"
					/>
				</template>
			</Field>

			<Field id="visibility" label="Visibility" class="w-full">
				<template #default="{ labelId }">
					<div
						role="radiogroup"
						:aria-labelledby="labelId"
						class="grid grid-cols-1 gap-2 sm:grid-cols-[repeat(3,minmax(0,250px))]"
					>
						<RadioCard
							v-for="option in visibilityOptions"
							:key="option.value"
							v-model="visibility"
							name="project-visibility"
							max-width="250px"
							:value="option.value"
							:label="option.label"
							:description="option.subLabel"
							:icon="option.icon"
							:disabled="!hasPermission || saving || pageSaving"
						/>
					</div>
				</template>
				<template #bottom>
					<p
						v-if="!tags.approvedStatuses.includes(project.status)"
						class="mb-0 mt-2 text-sm text-secondary"
					>
						{{ formatMessage(projectVisibilityApprovalHelp) }}
					</p>
				</template>
			</Field>
		</div>
	</section>
</template>

<script setup>
import { GlobeIcon, ImageIcon, LinkIcon, LockIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Avatar,
	commonMessages,
	commonProjectSettingsMessages,
	Field,
	FileButton,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	RadioCard,
	Textarea,
	useFormatBytes,
	useGlobals,
	useVIntl,
} from '@modrinth/ui'
import { fileIsValid, formatProjectStatus, isAdmin } from '@modrinth/utils'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import SlugSuggestions from '~/components/ui/SlugSuggestions.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import {
	useProjectSlugSuggestions,
	useSlugSuggestionVisibility,
} from '~/composables/project-slug-suggestions'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { getProjectTypeForUrl } from '~/helpers/projects.js'
import { projectSummaryHelp, projectVisibilityApprovalHelp } from '~/utils/project-messages'

const props = defineProps({ pageSaving: Boolean })

const { data: globals } = useGlobals()

const { formatMessage } = useVIntl()

const { addNotification } = injectNotificationManager()
const {
	projectV3: project,
	currentMember,
	allMembers,
	patchProjectV3,
	patchIcon,
	invalidate,
} = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')

const tags = useGeneratedState()

const formatBytes = useFormatBytes()

const name = ref(project.value.name)
const slug = ref(project.value.slug ?? '')
const summary = ref(project.value.summary)
const {
	onFocusIn: onSlugSuggestionFocusIn,
	onFocusOut: onSlugSuggestionFocusOut,
	visible: showSlugSuggestions,
} = useSlugSuggestionVisibility()
const ownerUsername = computed(
	() => (allMembers.value.find((member) => member.is_owner) ?? allMembers.value[0])?.user.username,
)
const { suggestions: slugSuggestions } = useProjectSlugSuggestions({
	title: name,
	username: ownerUsername,
	currentProjectId: () => project.value.id,
	enabled: showSlugSuggestions,
})
const icon = ref(null)
const previewImage = ref(null)
const deletedIcon = ref(false)
const iconSrc = computed(() =>
	deletedIcon.value ? null : (previewImage.value ?? project.value.icon_url),
)
const removeIconLabel = computed(() => formatMessage(commonMessages.removeButton))
const visibility = ref(
	tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status,
)

// Server project specific refs
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const isServerProject = computed(() => project.value?.minecraft_server != null)
const projectTypeForUrl = computed(() => {
	if (isServerProject.value) return 'server'
	const type = project.value.project_types?.[0] ?? 'mod'
	return getProjectTypeForUrl(type, project.value.loaders)
})
const bannerPreview = ref(null)
const deletedBanner = ref(false)
const bannerFile = ref(null)
const bannerGalleryImage = computed(() =>
	project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME),
)
const bannerSrc = computed(() =>
	deletedBanner.value ? null : (bannerPreview.value ?? bannerGalleryImage.value?.url),
)
const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (
		isAdminUser.value || ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
	)
})

const nameValidation = useProjectNagMessages('name')
const summaryValidation = useProjectNagMessages('summary')
const iconValidation = useProjectNagMessages('icon')
const canSave = computed(() => hasPermission.value && !saveValidation.hasErrors.value)

const visibilityOptions = computed(() =>
	tags.value.approvedStatuses
		.filter((status) => status !== 'archived')
		.map((status) => {
			const subLabel = () => {
				switch (status) {
					case 'approved':
						return 'Visible via URL, on your profile, and in search.'
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
				icon: status === 'approved' ? GlobeIcon : status === 'unlisted' ? LinkIcon : LockIcon,
			}
		}),
)

const basePatchData = computed(() => {
	const data = {}

	if (name.value !== project.value.name) {
		data.name = name.value
	}
	if (slug.value !== (project.value.slug ?? '')) {
		data.slug = slug.value
	}
	if (summary.value !== project.value.summary) {
		data.summary = summary.value
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
	name: project.value.name,
	slug: project.value.slug ?? '',
	summary: project.value.summary,
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
	visibility: visibility.value,
	icon: icon.value,
	deletedIcon: deletedIcon.value,
	bannerFile: bannerFile.value,
	deletedBanner: deletedBanner.value,
}))

const hasChanges = computed(() =>
	Object.keys(modified.value).some((key) => original.value[key] !== modified.value[key]),
)

function reset() {
	saveValidation.clear()
	name.value = project.value.name
	slug.value = project.value.slug ?? ''
	summary.value = project.value.summary
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

const saveValidation = useProjectSaveValidation(() => modified.value)
const validationMessages = computed(() =>
	saveValidation.withoutFields(['name', 'summary', 'icon', 'slug']),
)

function useTextIssueAction(field, input) {
	return useProjectIssueFieldAction({
		draft: () => ({ [field]: input.value.trim() }),
		canSave: () =>
			hasPermission.value &&
			!props.pageSaving &&
			!saveValidation.forField(field).some((message) => message.severity === 'error'),
		saving,
		validation: saveValidation,
		save: async () => {
			const submitted = input.value
			const value = submitted.trim()
			if (value !== (project.value[field] ?? '')) {
				if (field === 'slug') {
					await labrinth.projects_v3.edit(project.value.id, {
						slug: value,
					})
				} else {
					await patchProjectV3({ [field]: value }, true, true)
				}
			}
			if (input.value === submitted) input.value = value
		},
	})
}

const nameIssueAction = useTextIssueAction('name', name)
const slugIssueAction = useTextIssueAction('slug', slug)
const summaryIssueAction = useTextIssueAction('summary', summary)

const iconIssueAction = useProjectIssueFieldAction({
	draft: () => ({
		icon_url: deletedIcon.value ? null : (previewImage.value ?? project.value.icon_url),
	}),
	canSave: () =>
		hasPermission.value &&
		!props.pageSaving &&
		!saveValidation.forField('icon').some((message) => message.severity === 'error'),
	saving,
	validation: saveValidation,
	save: saveIconChanges,
})

async function saveIconChanges() {
	if (deletedIcon.value) {
		await deleteIcon()
		deletedIcon.value = false
	} else if (icon.value) {
		const submittedIcon = icon.value
		if ((await patchIcon(submittedIcon)) === false) return false
		if (icon.value === submittedIcon) {
			icon.value = null
			previewImage.value = null
		}
	}
	return true
}

async function save() {
	if (!canSave.value || saving.value) return false
	name.value = name.value.trim()
	slug.value = slug.value.trim()
	summary.value = summary.value.trim()
	const submittedState = saveValidation.snapshot()
	saving.value = true
	try {
		const hasPatchChanges = Object.keys(basePatchData.value).length > 0

		if (hasPatchChanges) {
			await patchProjectV3(basePatchData.value, false, true)
		}

		if (!(await saveIconChanges())) return false

		if (deletedBanner.value) {
			await deleteBanner()
			deletedBanner.value = false
		} else if (bannerFile.value) {
			await uploadBanner()
			bannerFile.value = null
			bannerPreview.value = null
		}
		saveValidation.clear()
		return true
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
		return false
	} finally {
		saving.value = false
	}
}

const showPreviewImage = async (files) => {
	const file = files[0]
	if (!file) {
		return
	}
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	const reader = new FileReader()
	icon.value = file
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
		const existingBanner = project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME)
		if (existingBanner) {
			await labrinth.projects_v3.deleteGalleryImage(existingBanner.url)
		}

		const ext = bannerFile.value.type.split('/').pop() ?? 'png'
		await labrinth.projects_v3.createGalleryImage(project.value.id, bannerFile.value, {
			ext,
			featured: false,
			name: MC_SERVER_BANNER_NAME,
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
		const bannerImage = project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME)
		if (bannerImage) {
			await labrinth.projects_v3.deleteGalleryImage(bannerImage.url)
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

const markIconForDeletion = () => {
	deletedIcon.value = true
	icon.value = null
	previewImage.value = null
}

const deleteIcon = async () => {
	await labrinth.projects_v3.deleteIcon(project.value.id)
	await invalidate()
	addNotification({
		title: 'Project icon removed',
		text: "Your project's icon has been removed.",
		type: 'success',
	})
}

defineExpose({ original, modified, hasChanges, canSave, saving, validationMessages, reset, save })
</script>
