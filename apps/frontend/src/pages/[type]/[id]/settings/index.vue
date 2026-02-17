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
									!deletedBanner && (bannerPreview || featuredGalleryImage?.url)
										? 'border-none'
										: 'aspect-[468/60] border-2 bg-surface-2'
								"
							>
								<div
									v-if="!deletedBanner && (bannerPreview || featuredGalleryImage?.url)"
									class="relative h-full w-full overflow-hidden rounded-2xl"
								>
									<img
										:src="bannerPreview || featuredGalleryImage?.url"
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
								v-if="!deletedBanner && (bannerPreview || featuredGalleryImage?.url)"
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

					<!-- Java Address -->
					<div>
						<label for="java-address">
							<span class="label__title">Java address</span>
						</label>
						<div class="mt-2 flex items-center gap-2" @focusout="pingJavaServer">
							<StyledInput
								id="java-address"
								v-model="javaAddress"
								placeholder="Enter address"
								:disabled="!hasPermission"
								wrapper-class="flex-grow"
							/>
							<StyledInput
								v-model="javaPort"
								type="number"
								:min="1"
								:max="65535"
								:disabled="!hasPermission"
								wrapper-class="w-24"
								input-class="text-center"
							/>
						</div>
						<div
							v-if="javaPingResult !== null"
							class="mt-2 flex items-center gap-1.5"
							:class="javaPingResult.online ? 'text-green' : 'text-orange'"
						>
							<CheckIcon v-if="javaPingResult.online" class="h-4 w-4" />
							<TriangleAlertIcon v-else class="h-4 w-4" />
							{{
								javaPingResult.online
									? `Server is online! ${javaPingResult.latency ? `Latency: ${javaPingResult.latency}ms` : ``}`
									: 'Cannot ping server'
							}}
						</div>
					</div>

					<!-- Bedrock Address -->
					<div>
						<label for="bedrock-address">
							<span class="label__title">Bedrock/PE address</span>
						</label>
						<div class="mt-2 flex items-center gap-2">
							<StyledInput
								id="bedrock-address"
								v-model="bedrockAddress"
								placeholder="Enter address"
								:disabled="!hasPermission"
								wrapper-class="flex-grow"
							/>
							<StyledInput
								v-model="bedrockPort"
								type="number"
								:min="1"
								:max="65535"
								:disabled="!hasPermission"
								wrapper-class="w-24"
								input-class="text-center"
							/>
						</div>
					</div>

					<CompatibilityCard />
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
				<div v-if="isServerProject">
					<label for="server-country">
						<span class="label__title">Country</span>
					</label>
					<Combobox
						id="server-country"
						v-model="country"
						:options="countryOptions"
						searchable
						placeholder="Select country"
						:disabled="!hasPermission"
					/>
				</div>

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
	Combobox,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	StyledInput,
} from '@modrinth/ui'
import { fileIsValid, formatProjectStatus, formatProjectType } from '@modrinth/utils'

import FileInput from '~/components/ui/FileInput.vue'
import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'
import { useFeatureFlags } from '~/composables/featureFlags.ts'

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	projectV3,
	currentMember,
	patchProject,
	patchProjectV3,
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
const isServerProject = computed(() => projectV3.value?.minecraft_server !== undefined)
const bannerPreview = ref(null)
const deletedBanner = ref(false)
const bannerFile = ref(null)
const featuredGalleryImage = computed(() => project.value.gallery?.find((img) => img.featured))
const javaAddress = ref('')
const javaPort = ref(25565)
const bedrockAddress = ref('')
const bedrockPort = ref(19132)
const supportedGameVersions = ref([])
const recommendedGameVersion = ref('')
const usingMrpack = ref(false)
const country = ref('')

watch(
	() => projectV3.value,
	(v3) => {
		if (!v3) return
		javaAddress.value = v3.minecraft_java_server?.address ?? ''
		javaPort.value = v3.minecraft_java_server?.port ?? 25565
		bedrockAddress.value = v3.minecraft_bedrock_server?.address ?? ''
		bedrockPort.value = v3.minecraft_bedrock_server?.port ?? 19132
		const javaContent = v3.minecraft_java_server?.content
		if (javaContent && 'supported_game_versions' in javaContent) {
			supportedGameVersions.value = javaContent.supported_game_versions ?? []
			recommendedGameVersion.value = javaContent.recommended_game_version ?? ''
		} else {
			supportedGameVersions.value = []
			recommendedGameVersion.value = ''
		}
		country.value = v3.minecraft_server?.country ?? ''
	},
	{ immediate: true },
)

const javaPingLoading = ref(false)
const javaPingResult = ref(null)

const pingJavaServer = async () => {
	const address = javaAddress.value?.trim()
	if (!address) {
		javaPingResult.value = null
		return
	}

	javaPingLoading.value = true
	javaPingResult.value = null

	const port = javaPort.value || 25565
	const query = port !== 25565 ? `${address}:${port}` : address

	try {
		// TODO replace with api-client labrinth server ping route
		// const response = await $fetch(`https://api.mcstatus.io/v2/status/java/${query}`, {
		// 	timeout: 10000,
		// })
		// console.log(response)
		// javaPingResult.value = {
		// 	online: response.online,
		// 	latency: response.latency ?? null,
		// }
	} catch {
		javaPingResult.value = { online: false, latency: null }
	} finally {
		javaPingLoading.value = false
	}
}

const countryOptions = [
	{ value: 'US', label: 'United States' },
	{ value: 'CA', label: 'Canada' },
	{ value: 'GB', label: 'United Kingdom' },
	{ value: 'DE', label: 'Germany' },
	{ value: 'FR', label: 'France' },
	{ value: 'NL', label: 'Netherlands' },
	{ value: 'FI', label: 'Finland' },
	{ value: 'SE', label: 'Sweden' },
	{ value: 'NO', label: 'Norway' },
	{ value: 'DK', label: 'Denmark' },
	{ value: 'PL', label: 'Poland' },
	{ value: 'CZ', label: 'Czech Republic' },
	{ value: 'RO', label: 'Romania' },
	{ value: 'CH', label: 'Switzerland' },
	{ value: 'AT', label: 'Austria' },
	{ value: 'BE', label: 'Belgium' },
	{ value: 'IE', label: 'Ireland' },
	{ value: 'ES', label: 'Spain' },
	{ value: 'IT', label: 'Italy' },
	{ value: 'PT', label: 'Portugal' },
	{ value: 'RU', label: 'Russia' },
	{ value: 'UA', label: 'Ukraine' },
	{ value: 'LT', label: 'Lithuania' },
	{ value: 'LV', label: 'Latvia' },
	{ value: 'EE', label: 'Estonia' },
	{ value: 'BG', label: 'Bulgaria' },
	{ value: 'HR', label: 'Croatia' },
	{ value: 'HU', label: 'Hungary' },
	{ value: 'SK', label: 'Slovakia' },
	{ value: 'RS', label: 'Serbia' },
	{ value: 'GR', label: 'Greece' },
	{ value: 'TR', label: 'Turkey' },
	{ value: 'IL', label: 'Israel' },
	{ value: 'AE', label: 'United Arab Emirates' },
	{ value: 'SA', label: 'Saudi Arabia' },
	{ value: 'IN', label: 'India' },
	{ value: 'SG', label: 'Singapore' },
	{ value: 'JP', label: 'Japan' },
	{ value: 'KR', label: 'South Korea' },
	{ value: 'CN', label: 'China' },
	{ value: 'HK', label: 'Hong Kong' },
	{ value: 'TW', label: 'Taiwan' },
	{ value: 'AU', label: 'Australia' },
	{ value: 'NZ', label: 'New Zealand' },
	{ value: 'BR', label: 'Brazil' },
	{ value: 'AR', label: 'Argentina' },
	{ value: 'CL', label: 'Chile' },
	{ value: 'CO', label: 'Colombia' },
	{ value: 'MX', label: 'Mexico' },
	{ value: 'ZA', label: 'South Africa' },
	{ value: 'NG', label: 'Nigeria' },
	{ value: 'KE', label: 'Kenya' },
	{ value: 'EG', label: 'Egypt' },
	{ value: 'MY', label: 'Malaysia' },
	{ value: 'TH', label: 'Thailand' },
	{ value: 'VN', label: 'Vietnam' },
	{ value: 'PH', label: 'Philippines' },
	{ value: 'ID', label: 'Indonesia' },
	{ value: 'PK', label: 'Pakistan' },
	{ value: 'BD', label: 'Bangladesh' },
]

const generatedState = useGeneratedState()
const gameVersions = generatedState.value.gameVersions

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

const javaServerPatchData = computed(() => {
	if (!isServerProject.value) return {}

	const origJava = projectV3.value?.minecraft_java_server
	const origContent = origJava?.content
	const origSupported =
		origContent && 'supported_game_versions' in origContent
			? origContent.supported_game_versions
			: []
	const origRecommended =
		origContent && 'recommended_game_version' in origContent
			? origContent.recommended_game_version
			: ''

	const addressChanged =
		(javaAddress.value && javaAddress.value !== origJava?.address) ||
		javaPort.value !== (origJava?.port ?? 25565)
	const contentChanged =
		JSON.stringify(supportedGameVersions.value) !== JSON.stringify(origSupported) ||
		recommendedGameVersion.value !== (origRecommended ?? '')

	if (addressChanged || contentChanged) {
		return {
			address: javaAddress.value.trim(),
			port: javaPort.value,
			content: {
				kind: 'vanilla',
				supported_game_versions: supportedGameVersions.value,
				...(recommendedGameVersion.value
					? { recommended_game_version: recommendedGameVersion.value }
					: {}),
			},
		}
	}

	return {}
})

const bedrockServerPatchData = computed(() => {
	if (!isServerProject.value) return {}

	const origBedrock = projectV3.value?.minecraft_bedrock_server
	if (
		(bedrockAddress.value && bedrockAddress.value !== origBedrock?.address) ||
		bedrockPort.value !== (origBedrock?.port ?? 19132)
	) {
		return {
			address: bedrockAddress.value.trim(),
			port: bedrockPort.value,
		}
	}

	return {}
})

const serverPatchData = computed(() => {
	if (!isServerProject.value) return {}

	const origServer = projectV3.value?.minecraft_server
	if (country.value && country.value !== origServer?.country) {
		return {
			...origServer,
			country: country.value,
		}
	}

	return {}
})

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

const patchData = computed(() => {
	return {
		...basePatchData.value,
		minecraft_server: serverPatchData.value,
		minecraft_java_server: javaServerPatchData.value,
		minecraft_bedrock_server: bedrockServerPatchData.value,
	}
})

const hasChanges = computed(() => {
	return (
		Object.keys(basePatchData.value).length > 0 ||
		Object.keys(serverPatchData.value).length > 0 ||
		Object.keys(javaServerPatchData.value).length > 0 ||
		Object.keys(bedrockServerPatchData.value).length > 0 ||
		deletedIcon.value ||
		icon.value ||
		deletedBanner.value ||
		bannerFile.value
	)
})

const hasModifiedVisibility = () => {
	const originalVisibility = tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status

	return originalVisibility !== visibility.value
}

const saveChanges = async () => {
	if (Object.keys(patchData.value).length > 0) {
		await patchProjectV3(patchData.value)
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
		// First, delete existing featured image if there is one
		const existingFeatured = project.value.gallery?.find((img) => img.featured)
		if (existingFeatured) {
			await labrinth.projects_v2.deleteGalleryImage(project.value.id, existingFeatured.url)
		}

		// Upload new banner as featured gallery image
		const ext = bannerFile.value.type.split('/').pop() ?? 'png'
		await labrinth.projects_v2.createGalleryImage(project.value.id, bannerFile.value, {
			ext,
			featured: true,
			title: 'Banner',
		})

		await refreshProject()
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
		const featuredImage = project.value.gallery?.find((img) => img.featured)
		if (featuredImage) {
			await labrinth.projects_v2.deleteGalleryImage(project.value.id, featuredImage.url)
			await refreshProject()
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
