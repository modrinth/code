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
			<div class="flex max-w-[540px] flex-col gap-6">
				<div class="label">
					<h3>
						<span class="label__title size-card-header">Project information</span>
					</h3>
				</div>

				<div>
					<label for="project-icon">
						<span class="label__title">Icon</span>
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

				<div>
					<label for="project-name">
						<span class="label__title">Name</span>
					</label>
					<input
						id="project-name"
						v-model="name"
						maxlength="2048"
						type="text"
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
						<input
							id="project-slug"
							v-model="slug"
							type="text"
							maxlength="64"
							autocomplete="off"
							:disabled="!hasPermission"
						/>
					</div>
				</div>

				<div>
					<label for="project-summary">
						<span class="label__title">Summary</span>
					</label>
					<div v-if="summaryWarning" class="my-2 flex items-center gap-1.5 text-orange">
						<TriangleAlertIcon class="my-auto" />
						{{ summaryWarning }}
					</div>
					<div class="textarea-wrapper min-h-36 !w-full">
						<textarea
							id="project-summary"
							v-model="summary"
							maxlength="256"
							:disabled="!hasPermission"
						/>
					</div>
				</div>

				<!-- Server Project Settings -->
				<template v-if="flags.serverProjectSettings">
					<!-- Banner -->
					<div>
						<label>
							<span class="label__title">Banner</span>
						</label>
						<div class="mt-2">
							<label
								class="flex aspect-[468/60] w-full cursor-pointer flex-col items-center justify-center rounded-2xl border-2 border-dashed border-surface-5 bg-surface-2 transition-colors"
							>
								<div
									v-if="!deletedBanner && (bannerPreview || project.banner_url)"
									class="relative h-full w-full overflow-hidden rounded-2xl"
								>
									<img
										:src="bannerPreview || project.banner_url"
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
								v-if="!deletedBanner && (bannerPreview || project.banner_url)"
								class="iconified-button"
								:disabled="!hasPermission"
								@click="markBannerForDeletion"
							>
								<TrashIcon aria-hidden="true" />
								Remove banner
							</button>
						</div>
						<p class="mt-2 text-sm text-secondary">Gif, 468×60px recommended.</p>
					</div>

					<!-- Java Address -->
					<div>
						<label for="java-address">
							<span class="label__title">Java address</span>
						</label>
						<div class="mt-2 flex items-center gap-2">
							<input
								id="java-address"
								v-model="javaAddress"
								type="text"
								placeholder="Enter address"
								class="flex-grow rounded-xl bg-bg-raised"
								:disabled="!hasPermission"
							/>
							<input
								v-model.number="javaPort"
								type="number"
								min="1"
								max="65535"
								class="w-24 rounded-xl bg-bg-raised text-center"
								:disabled="!hasPermission"
							/>
						</div>
					</div>

					<!-- Bedrock Address -->
					<div>
						<label for="bedrock-address">
							<span class="label__title">Bedrock/PE address</span>
						</label>
						<div class="mt-2 flex items-center gap-2">
							<input
								id="bedrock-address"
								v-model="bedrockAddress"
								type="text"
								placeholder="Enter address"
								class="flex-grow rounded-xl bg-bg-raised"
								:disabled="!hasPermission"
							/>
							<input
								v-model.number="bedrockPort"
								type="number"
								min="1"
								max="65535"
								class="w-24 rounded-xl bg-bg-raised text-center"
								:disabled="!hasPermission"
							/>
						</div>
					</div>

					<!-- Server Version -->
					<div>
						<label for="server-version">
							<span class="label__title">Server version</span>
						</label>
						<input
							id="server-version"
							v-model="serverVersion"
							type="text"
							placeholder="e.g. 1.21.4"
							class="w-full"
							:disabled="!hasPermission"
						/>
					</div>

					<!-- Country -->
					<div>
						<label for="server-country">
							<span class="label__title">Country</span>
						</label>
						<DropdownSelect
							id="server-country"
							v-model="serverCountry"
							:options="countries.map((c) => c.value)"
							:display-name="(val) => countries.find((c) => c.value === val)?.label ?? val"
							name="server-country"
							placeholder="Select country"
							:disabled="!hasPermission"
						/>
					</div>
				</template>

				<template
					v-if="
						!flags.serverProjectSettings &&
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
						<Multiselect
							id="project-env-client"
							v-model="clientSide"
							class="small-multiselect"
							placeholder="Select one"
							:options="sideTypes"
							:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
							:searchable="false"
							:close-on-select="true"
							:show-labels="false"
							:allow-empty="false"
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
						<Multiselect
							id="project-env-server"
							v-model="serverSide"
							class="small-multiselect"
							placeholder="Select one"
							:options="sideTypes"
							:custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
							:searchable="false"
							:close-on-select="true"
							:show-labels="false"
							:allow-empty="false"
							:disabled="!hasPermission"
						/>
					</div>
				</template>
				<div class="">
					<label for="project-visibility">
						<span class="label__title">Visibility</span>
						<div class="label__description">
							Public and archived projects are visible in search. Unlisted projects are published,
							but not visible in search or on user profiles. Private projects are only accessible by
							members of the project.

							<p>If approved by the moderators:</p>
						</div>
					</label>
					<div class="flex gap-4">
						<Multiselect
							id="project-visibility"
							v-model="visibility"
							class="max-w-[20rem]"
							placeholder="Select one"
							:options="tags.approvedStatuses"
							:custom-label="(value) => formatProjectStatus(value)"
							:searchable="false"
							:close-on-select="true"
							:show-labels="false"
							:allow-empty="false"
							:disabled="!hasPermission"
						/>
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
				<div class="button-group">
					<button
						type="button"
						class="iconified-button brand-button"
						:disabled="!hasChanges"
						@click="saveChanges()"
					>
						<SaveIcon aria-hidden="true" />
						Save changes
					</button>
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
	</div>
</template>

<script setup>
import {
	CheckIcon,
	ImageIcon,
	IssuesIcon,
	SaveIcon,
	TrashIcon,
	TriangleAlertIcon,
	UploadIcon,
	XIcon,
} from '@modrinth/assets'
import { MIN_SUMMARY_CHARS } from '@modrinth/moderation'
import {
	Avatar,
	ConfirmModal,
	DropdownSelect,
	injectNotificationManager,
	injectProjectPageContext,
} from '@modrinth/ui'
import { formatProjectStatus, formatProjectType } from '@modrinth/utils'
import { Multiselect } from 'vue-multiselect'

import FileInput from '~/components/ui/FileInput.vue'
import { useFormattedCountries } from '~/composables/country.ts'
import { useFeatureFlags } from '~/composables/featureFlags.ts'

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	currentMember,
	patchProject,
	patchIcon,
	refreshProject,
} = injectProjectPageContext()

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
const bannerPreview = ref(null)
const deletedBanner = ref(false)
const bannerFile = ref(null)
const javaAddress = ref(props.project.java_address ?? '')
const javaPort = ref(props.project.java_port ?? 25565)
const bedrockAddress = ref(props.project.bedrock_address ?? '')
const bedrockPort = ref(props.project.bedrock_port ?? 19132)
const serverVersion = ref(props.project.server_version ?? '')
const serverCountry = ref(props.project.country ?? null)

const countries = useFormattedCountries()

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

const sideTypes = ['required', 'optional', 'unsupported']

const patchData = computed(() => {
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

const hasChanges = computed(() => {
	return Object.keys(patchData.value).length > 0 || deletedIcon.value || icon.value
})

const hasModifiedVisibility = () => {
	const originalVisibility = tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status

	return originalVisibility !== visibility.value
}

const saveChanges = async () => {
	if (hasChanges.value) {
		await patchProject(patchData.value)
	}

	if (deletedIcon.value) {
		await deleteIcon()
		deletedIcon.value = false
	} else if (icon.value) {
		await patchIcon(icon.value)
		icon.value = null
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
	await refreshProject()
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

.summary-input {
	min-height: 8rem;
	max-width: 24rem;
}

.button-group {
	justify-content: flex-start;
}
</style>
