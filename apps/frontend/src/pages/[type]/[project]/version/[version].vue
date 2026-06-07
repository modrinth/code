<template>
	<div>
		<CreateProjectVersionModal
			v-if="currentMember"
			ref="createProjectVersionModal"
			@save="resetProjectVersions"
		/>
		<ConfirmModal
			v-if="currentMember"
			ref="modal_confirm"
			title="Are you sure you want to delete this version?"
			description="This will remove this version forever (like really forever)."
			:has-to-type="false"
			proceed-label="Delete"
			@proceed="deleteVersion()"
		/>
		<NewModal
			v-if="auth.user && currentMember"
			ref="modal_package_mod"
			:header="formatMessage(messages.packageDataPackHeader)"
		>
			<div class="flex max-w-[35rem] flex-col">
				<p class="m-0 mb-4">
					{{ formatMessage(messages.packageDataPackDescription) }}
				</p>
				<label for="package-mod-loaders" class="mb-2 flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">{{
						formatMessage(messages.modLoadersLabel)
					}}</span>
					<span>{{ formatMessage(messages.modLoadersDescription) }}</span>
				</label>
				<MultiSelect
					id="package-mod-loaders"
					v-model="packageLoaders"
					class="package-loader-select"
					:options="packageLoaderOptions"
					:searchable="false"
					:placeholder="formatMessage(messages.modLoadersPlaceholder)"
				/>
				<div class="ml-auto mt-4 flex items-center gap-2">
					<ButtonStyled type="outlined">
						<button @click="modal_package_mod?.hide()">
							<XIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.cancelButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled color="brand">
						<button @click="createDataPackVersionHandler" :disabled="packageLoaders.length === 0">
							{{ formatMessage(messages.packageDataPack) }}
							<RightArrowIcon aria-hidden="true" />
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
		<div>
			<nuxt-link
				class="mb-4 flex w-fit items-center gap-2 rounded-lg p-2 pl-0 text-link"
				:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
			>
				<ChevronLeftIcon class="shrink-0" /> {{ formatMessage(messages.allVersions) }}
			</nuxt-link>
			<Admonition
				type="circle-warning"
				:header="formatMessage(messages.unknownEmbeddedContent)"
				:body="formatMessage(messages.unknownEmbeddedContentDescription)"
				class="mb-4"
			>
				<template #actions>
					<div class="flex">
						<ButtonStyled color="orange">
							<nuxt-link
								:to="`/${project.project_type}/${
									project.slug ? project.slug : project.id
								}/settings/permissions`"
							>
								{{ formatMessage(commonProjectSettingsMessages.withheldVersionsWarningResolve) }}
								<RightArrowIcon />
							</nuxt-link>
						</ButtonStyled>
					</div>
				</template>
			</Admonition>
			<VersionPage
				v-if="version"
				:version="version"
				:enrichment="enrichment"
				:dependency-link-creator="createDependencyLink"
				class="mb-4"
			>
				<template #headerActions="{ primaryFile, promotedFiles }">
					<ButtonStyled color="brand">
						<a
							v-tooltip="
								primaryFile?.url
									? primaryFile.filename + ' (' + formatBytes(primaryFile.size) + ')'
									: formatMessage(messages.noPrimaryFile)
							"
							:href="decoratedPrimaryFileUrl"
							:download="primaryFile?.filename"
							:disabled="primaryFile?.url === undefined"
							@click="emit('onDownload')"
						>
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</a>
					</ButtonStyled>
					<ButtonStyled
						v-for="file in promotedFiles.filter(
							(x) =>
								!!x &&
								(x.file_type === 'required-resource-pack' ||
									x.file_type === 'optional-resource-pack'),
						)"
						:key="`promoted-file-${file.hashes.sha1}`"
						color="brand"
						color-fill="text"
					>
						<a
							v-tooltip="file.filename + ' (' + formatBytes(file.size) + ')'"
							:href="
								createProjectDownloadUrl(file.url, {
									reason: 'dependency',
								})
							"
							:download="primaryFile?.filename"
							:disabled="primaryFile?.url === undefined"
							@click="emit('onDownload')"
						>
							<DownloadIcon aria-hidden="true" />
							<template v-if="file.file_type === 'required-resource-pack'">
								{{ formatMessage(messages.requiredResourcePack) }}
							</template>
							<template v-else-if="file.file_type === 'optional-resource-pack'">
								{{ formatMessage(messages.optionalResourcePack) }}
							</template>
						</a>
					</ButtonStyled>
					<template v-if="currentMember">
						<ButtonStyled
							v-if="
								version.loaders.some((x: string) => tags.loaderData.dataPackLoaders.includes(x))
							"
						>
							<button @click="modal_package_mod?.show()">
								<PackageClosedIcon aria-hidden="true" />
								{{ formatMessage(messages.packageAsMod) }}
							</button>
						</ButtonStyled>
						<ButtonStyled>
							<OverflowMenu
								:dropdown-id="`${baseId}-edit-overflow`"
								class="btn-dropdown-animation"
								:options="[
									{
										id: 'edit-metadata',
										action: () => handleOpenEditVersionModal(version!.id, project.id, 'metadata'),
									},
									{
										id: 'edit-details',
										action: () =>
											handleOpenEditVersionModal(version!.id, project.id, 'add-details'),
									},
									{
										id: 'edit-files',
										action: () => handleOpenEditVersionModal(version!.id, project.id, 'add-files'),
									},
									{
										id: 'delete',
										color: 'red',
										action: () => modal_confirm.show(),
									},
								]"
							>
								<SettingsIcon aria-hidden="true" /> {{ formatMessage(messages.edit) }}
								<DropdownIcon class="h-5 w-5 text-secondary" />
								<template #edit-metadata>
									<BoxIcon aria-hidden="true" />
									{{ formatMessage(messages.editMetadata) }}
								</template>
								<template #edit-details>
									<InfoIcon aria-hidden="true" />
									{{ formatMessage(messages.editDetails) }}
								</template>
								<template #edit-files>
									<FileIcon aria-hidden="true" />
									{{ formatMessage(messages.editFiles) }}
								</template>
								<template #delete>
									<TrashIcon aria-hidden="true" />
									{{ formatMessage(commonMessages.deleteLabel) }}
								</template>
							</OverflowMenu>
						</ButtonStyled>
					</template>
					<ButtonStyled v-else color="red" color-fill="text">
						<nuxt-link v-if="!auth.user" :to="signInRouteObj">
							<ReportIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.reportButton) }}
						</nuxt-link>
						<button v-else @click="() => reportVersion(version!.id)">
							<ReportIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.reportButton) }}
						</button>
					</ButtonStyled>
				</template>
				<template #supplementaryResourceActions="{ file }">
					<ButtonStyled>
						<a
							:href="decorateDownloadUrl(file.url)"
							:title="`Download ${file.filename}`"
							:download="file.filename"
							tabindex="0"
						>
							<DownloadIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.downloadButton) }}
						</a>
					</ButtonStyled>
					<ButtonStyled type="outlined" circular>
						<OverflowMenu
							:tooltip="formatMessage(commonMessages.moreOptionsButton)"
							:options="[
								{
									id: 'copy-sha1',
									action: () => copyFileHash(file, 'sha1'),
								},
								{
									id: 'copy-sha512',
									action: () => copyFileHash(file, 'sha512'),
								},
							]"
							:dropdown-id="`${baseId}-supplementary-resource-actions`"
						>
							<MoreVerticalIcon aria-hidden="true" />
							<template #copy-sha1>
								<CopyIcon aria-hidden="true" />
								{{ formatMessage(messages.copySha1) }}
							</template>
							<template #copy-sha512>
								<CopyIcon aria-hidden="true" />
								{{ formatMessage(messages.copySha512) }}
							</template>
						</OverflowMenu>
					</ButtonStyled>
				</template>
				<template #dependencyActions="{ dependency }">
					<ButtonStyled circular>
						<nuxt-link
							v-if="createDependencyLink(dependency)"
							v-tooltip="
								formatMessage(dependency.version ? messages.viewVersion : messages.viewProject)
							"
							:to="createDependencyLink(dependency)"
							target="_blank"
						>
							<ExternalIcon />
						</nuxt-link>
					</ButtonStyled>
					<ButtonStyled circular color="brand" color-fill="text">
						<a
							v-if="dependency.version && dependency.dependency.dependency_type !== 'incompatible'"
							v-tooltip="
								dependencyVersionPrimaryFiles[dependency.version.id]
									? dependencyVersionPrimaryFiles[dependency.version.id].filename +
										' (' +
										formatBytes(dependencyVersionPrimaryFiles[dependency.version.id].size) +
										')'
									: formatMessage(messages.noPrimaryFile)
							"
							:href="
								createProjectDownloadUrl(dependencyVersionPrimaryFiles[dependency.version.id].url, {
									reason: 'dependency',
								})
							"
							:download="dependencyVersionPrimaryFiles[dependency.version.id].filename"
							:disabled="dependencyVersionPrimaryFiles[dependency.version.id].url === undefined"
						>
							<DownloadIcon />
						</a>
						<a
							v-else-if="dependency.project"
							v-tooltip="formatMessage(messages.downloadProject)"
							:href="`/project/${dependency.project.id}#download`"
							target="_blank"
						>
							<DownloadIcon />
						</a>
					</ButtonStyled>
				</template>
			</VersionPage>
		</div>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxIcon,
	ChevronLeftIcon,
	CopyIcon,
	DownloadIcon,
	DropdownIcon,
	ExternalIcon,
	FileIcon,
	InfoIcon,
	MoreVerticalIcon,
	PackageClosedIcon,
	ReportIcon,
	RightArrowIcon,
	SettingsIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	commonProjectSettingsMessages,
	ConfirmModal,
	defineMessages,
	ENVIRONMENTS_COPY,
	formatLoader,
	injectNotificationManager,
	injectProjectPageContext,
	MultiSelect,
	NewModal,
	OverflowMenu,
	PROJECT_DEP_MARKER_QUERY,
	useFormatBytes,
	useFormatDateTime,
	useVIntl,
	VersionPage,
} from '@modrinth/ui'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { getSignInRouteObj } from '~/composables/auth.js'
import { useImageUpload } from '~/composables/image-upload.ts'
import { createDataPackVersion } from '~/helpers/package.js'
import { reportVersion } from '~/utils/report-helpers.ts'
const emit = defineEmits<{
	onDownload: []
}>()

// Composables
const data = useNuxtApp()
const route = useNativeRoute()
const signInRouteObj = computed(() => getSignInRouteObj(route))
const router = useRouter()
const auth = await useAuth()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const { addNotification } = injectNotificationManager()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const formatDate = useFormatDateTime({ dateStyle: 'medium' })
const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()

// Helper for accessing nuxt app $formatVersion
const formatVersionDisplay = (versions: string[]) => (data as any).$formatVersion(versions)

// Get data from DI context
const {
	projectV2: project,
	currentMember,
	allMembers: members,
	versions: contextVersions,
	loadVersions,
	dependencies: contextDependencies,
	dependenciesLoading: contextDependenciesLoading,
	loadDependencies,
	invalidate,
	cdnDownloadReason,
} = injectProjectPageContext()

const baseId = useId()

// Load versions and dependencies in parallel
await Promise.all([loadVersions(), loadDependencies()])

// Template refs
const createProjectVersionModal = useTemplateRef('createProjectVersionModal')
const modal_confirm = useTemplateRef('modal_confirm')
const modal_package_mod = useTemplateRef('modal_package_mod')

// Reactive state from data()
const newFiles = ref<File[]>([])
const newFileTypes = ref<Array<{ display: string; value: string } | null>>([])
const packageLoaders = ref(['forge', 'fabric', 'quilt', 'neoforge'])
const packageLoaderOptions = [
	{ value: 'fabric', label: formatLoader(formatMessage, 'fabric') },
	{ value: 'forge', label: formatLoader(formatMessage, 'forge') },
	{ value: 'quilt', label: formatLoader(formatMessage, 'quilt') },
	{ value: 'neoforge', label: formatLoader(formatMessage, 'neoforge') },
]
const showKnownErrors = ref(false)
const shouldPreventActions = ref(false)
const uploadedImageIds = ref<string[]>([])

const dependenciesMetaLoading = ref(true)
const dependenciesLoading = computed(
	() => contextDependenciesLoading.value || dependenciesMetaLoading.value,
)

// File types constant
const fileTypes = ref([
	{
		display: 'Required resource pack',
		value: 'required-resource-pack',
	},
	{
		display: 'Optional resource pack',
		value: 'optional-resource-pack',
	},
])

// Mutable state initialized during setup
const version = ref<Labrinth.Versions.v3.Version>()
const enrichment = ref<Labrinth.Projects.v2.DependencyInfo>()
const primaryFile = ref<Record<string, any>>({})
const alternateFile = ref<Record<string, any> | undefined>(undefined)
const replaceFile = ref<File | null>(null)
const oldFileTypes = ref<Array<{ display: string; value: string } | null>>([])

if (route.params.version === 'latest') {
	let versionList = contextVersions.value ?? []
	if (route.query.loader) {
		versionList = versionList.filter((x: any) => x.loaders.includes(route.query.loader))
	}
	if (route.query.version) {
		versionList = versionList.filter((x: any) => x.game_versions.includes(route.query.version))
	}
	if (versionList.length === 0) {
		throw createError({
			fatal: true,
			statusCode: 404,
			message: 'No version matches the filters',
		})
	}
	version.value = versionList.reduce((a: any, b: any) =>
		a.date_published > b.date_published ? a : b,
	)
} else {
	let foundVersion = ((contextVersions.value ?? []) as any[]).find(
		(x: any) => x.id === route.params.version,
	)

	if (!foundVersion) {
		foundVersion = ((contextVersions.value ?? []) as any[]).find(
			(x: any) => x.displayUrlEnding === route.params.version,
		)
	}

	if (!foundVersion) {
		foundVersion = ((contextVersions.value ?? []) as any[]).find(
			(x: any) => x.version_number === route.params.version,
		)
	}

	if (foundVersion) {
		const versionV3 = (await useBaseFetch(
			`project/${project.value.id}/version/${route.params.version}`,
			{ apiVersion: 3 },
		)) as any
		if (versionV3) {
			foundVersion.environment = versionV3.environment
			foundVersion.changelog = versionV3.changelog
		}
		version.value = foundVersion
	} else {
		// cache is stale (e.g., version was just created/reuploaded)
		try {
			const versionV3 = (await useBaseFetch(
				`project/${project.value.id}/version/${route.params.version}`,
				{ apiVersion: 3 },
			)) as any
			if (versionV3) {
				version.value = versionV3
				// Refresh cache to include this version
				await invalidate()
			}
		} catch {
			// API fetch failed - version truly doesn't exist, will 404 below
		}
	}
}

if (!version.value || Object.keys(version.value).length === 0) {
	throw createError({
		fatal: true,
		statusCode: 404,
		message: 'Version not found',
	})
}

// Deep clone version to make it reactive and avoid mutating the original
version.value = JSON.parse(JSON.stringify(version.value))
primaryFile.value =
	version.value.files?.find((file: any) => file.primary) ?? version.value.files?.[0] ?? {}
alternateFile.value = version.value.files?.find(
	(file: any) => file.file_type && file.file_type.includes('resource-pack'),
)

// Process dependencies
watch(
	[contextDependencies],
	() => {
		const deps = contextDependencies.value ?? { projects: [], versions: [] }

		if (contextDependencies.value) {
			enrichment.value = contextDependencies.value
		}

		dependenciesMetaLoading.value = false
	},
	{ deep: true, immediate: true },
)

oldFileTypes.value = (version.value.files ?? []).map(
	(x: any) => fileTypes.value.find((y) => y.value === x.file_type) ?? null,
)

// Computed properties
const title = computed(() => `${version.value.name} - ${project.value.title}`)

const modpackLoaders = computed<string[]>(() => {
	if (project.value.project_type !== 'modpack') {
		return []
	}

	if (Array.isArray(version.value.mrpack_loaders) && version.value.mrpack_loaders.length > 0) {
		return version.value.mrpack_loaders
	}

	return (version.value.loaders ?? []).filter((loader: string) => loader !== 'mrpack')
})

const noModpackLoader = computed(
	() =>
		project.value.project_type === 'modpack' &&
		((modpackLoaders.value.length === 1 && modpackLoaders.value[0] === 'minecraft') ||
			modpackLoaders.value.length === 0),
)

const description = computed(
	() =>
		`Download ${project.value.title} ${
			version.value.version_number
		} on Modrinth. Supports ${(data as any).$formatVersion(version.value.game_versions)} ${(
			version.value.loaders ?? []
		)
			.map((x: string) => x.charAt(0).toUpperCase() + x.slice(1))
			.join(
				' & ',
			)}. Published on ${formatDate(version.value.date_published)}. ${version.value.downloads} downloads.`,
)

const usesFeaturedVersions = computed(() =>
	(contextVersions.value ?? []).some((v: any) => v.featured),
)

const fieldErrors = computed(
	() =>
		version.value.version_number === '' ||
		(version.value.game_versions?.length ?? 0) === 0 ||
		((version.value.loaders?.length ?? 0) === 0 && project.value.project_type !== 'resourcepack') ||
		(newFiles.value.length === 0 && (version.value.files?.length ?? 0) === 0 && !replaceFile.value),
)

const sortedDeps = computed(() => {
	const order = ['required', 'optional', 'incompatible', 'embedded']
	return [...(version.value.dependencies ?? [])].sort(
		(a, b) => order.indexOf(a.dependency_type) - order.indexOf(b.dependency_type),
	)
})

const decoratedPrimaryFileUrl = computed(() =>
	createProjectDownloadUrl(primaryFile.value?.url, { reason: cdnDownloadReason.value }),
)

function decorateDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, { reason: cdnDownloadReason.value })
}

function navigateToDependency(dependency: { link: string }) {
	return router.push({
		path: dependency.link,
		query: { ...PROJECT_DEP_MARKER_QUERY },
	})
}

const environment = computed(
	() => ENVIRONMENTS_COPY[version.value.environment as keyof typeof ENVIRONMENTS_COPY],
)

// SEO
useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})

// Methods
function handleOpenEditVersionModal(versionId: string, projectId: string, stageId: string) {
	if (!currentMember.value) return
	createProjectVersionModal.value?.openEditVersionModal(versionId, projectId, stageId)
}

async function _onImageUpload(file: File) {
	const response = await useImageUpload(file, { context: 'version' })

	uploadedImageIds.value.push(response.id)
	uploadedImageIds.value = uploadedImageIds.value.slice(-10)

	return response.url
}

function getPreviousLink() {
	if (router.options.history.state.back) {
		if ((router.options.history.state.back as string).includes('/versions')) {
			return router.options.history.state.back as string
		}
	}
	return `/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/versions`
}

function getPreviousLabel() {
	return router.options.history.state.back &&
		(router.options.history.state.back as string).endsWith('/versions')
		? 'Back to versions'
		: 'All versions'
}

async function createVersionRaw(versionData: Record<string, any>) {
	const formData = new FormData()

	const fileParts = newFiles.value.map((f, idx) => `${f.name}-${idx}`)
	if (replaceFile.value) {
		fileParts.unshift(replaceFile.value.name.concat('-primary'))
	}

	if (project.value.project_type === 'resourcepack') {
		versionData.loaders = ['minecraft']
	}

	const newVersion = {
		project_id: versionData.project_id,
		file_parts: fileParts,
		version_number: versionData.version_number,
		version_title: versionData.name || versionData.version_number,
		version_body: versionData.changelog,
		dependencies: versionData.dependencies,
		game_versions: versionData.game_versions,
		loaders: versionData.loaders,
		release_channel: versionData.version_type,
		featured: versionData.featured,
		file_types: newFileTypes.value.reduce(
			(acc, x, i) => ({
				...acc,
				[fileParts[replaceFile.value ? i + 1 : i]]: x ? x.value : null,
			}),
			{},
		),
	}

	formData.append('data', JSON.stringify(newVersion))

	if (replaceFile.value) {
		formData.append(
			replaceFile.value.name.concat('-primary'),
			new Blob([replaceFile.value]),
			replaceFile.value.name,
		)
	}

	for (let i = 0; i < newFiles.value.length; i++) {
		formData.append(
			fileParts[replaceFile.value ? i + 1 : i],
			new Blob([newFiles.value[i]]),
			newFiles.value[i].name,
		)
	}

	const responseData = (await useBaseFetch('version', {
		method: 'POST',
		body: formData,
		headers: {
			'Content-Disposition': formData as any,
		},
	})) as any

	await resetProjectVersions()

	await router.push(
		`/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}/version/${responseData.id}`,
	)
}

async function deleteVersion() {
	startLoading()

	await useBaseFetch(`version/${version.value.id}`, {
		method: 'DELETE',
	})

	await resetProjectVersions()
	await router.replace(`/${project.value.project_type}/${project.value.id}/versions`)
	stopLoading()
}

async function createDataPackVersionHandler() {
	shouldPreventActions.value = true
	startLoading()
	try {
		const blob = await createDataPackVersion(
			project.value,
			version.value,
			primaryFile.value,
			members.value ?? [],
			tags.value.gameVersions,
			packageLoaders.value,
		)

		newFiles.value = []
		newFileTypes.value = []
		replaceFile.value = new File(
			[blob],
			`${project.value.slug}-${version.value.version_number}.jar`,
		)

		await createVersionRaw({
			project_id: project.value.id,
			author_id: currentMember.value?.user.id,
			name: version.value.name,
			version_number: `${version.value.version_number}+mod`,
			changelog: version.value.changelog,
			version_type: version.value.version_type,
			dependencies: version.value.dependencies,
			game_versions: version.value.game_versions,
			loaders: packageLoaders.value,
			featured: version.value.featured,
		})

		modal_package_mod.value?.hide()

		addNotification({
			title: 'Packaging Success',
			text: 'Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.',
			type: 'success',
		})
	} catch (err: any) {
		addNotification({
			title: 'An error occurred',
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
	stopLoading()
	shouldPreventActions.value = false
}

async function resetProjectVersions() {
	await invalidate()
}

const messages = defineMessages({
	noPrimaryFile: {
		id: 'version.download.no-primary-file',
		defaultMessage: 'Error: No primary file found',
	},
	edit: {
		id: 'version.edit.button',
		defaultMessage: 'Edit',
	},
	editMetadata: {
		id: 'version.edit.metadata',
		defaultMessage: 'Edit metadata',
	},
	editDetails: {
		id: 'version.edit.details',
		defaultMessage: 'Edit details',
	},
	editFiles: {
		id: 'version.edit.files',
		defaultMessage: 'Edit files',
	},
	packageAsMod: {
		id: 'version.package-as-mod.button',
		defaultMessage: 'Package as mod',
	},
	copySha1: {
		id: 'version.supplementary-resources.copy-hash-sha1',
		defaultMessage: 'Copy SHA-1',
	},
	copySha512: {
		id: 'version.supplementary-resources.copy-hash-sha512',
		defaultMessage: 'Copy SHA-512',
	},
	allVersions: {
		id: 'version.all-versions',
		defaultMessage: 'All versions',
	},
	unknownEmbeddedContent: {
		id: 'version.unknown-embedded-content.title',
		defaultMessage: 'Withheld due to unknown embedded content',
	},
	unknownEmbeddedContentDescription: {
		id: 'version.unknown-embedded-content.description',
		defaultMessage: `This version is currently withheld and not publicly listed. Please provide proof that you have permission to redistribute certain files included.`,
	},
	viewProject: {
		id: 'version.dependency.view-project',
		defaultMessage: `View project`,
	},
	viewVersion: {
		id: 'version.dependency.view-version',
		defaultMessage: `View version`,
	},
	downloadProject: {
		id: 'version.download.download-dependency',
		defaultMessage: 'Download dependency',
	},
	requiredResourcePack: {
		id: 'version.download.required-resource-pack',
		defaultMessage: 'Required resource pack',
	},
	optionalResourcePack: {
		id: 'version.download.optional-resource-pack',
		defaultMessage: 'Optional resource pack',
	},
	packageDataPack: {
		id: 'version.package-as-mod.submit-button',
		defaultMessage: 'Package data pack',
	},
	packageDataPackHeader: {
		id: 'version.package-as-mod.header',
		defaultMessage: 'Packaging data pack as a mod',
	},
	packageDataPackDescription: {
		id: 'version.package-as-mod.description',
		defaultMessage:
			'This will create a new version with support for the selected mod loaders. You will be redirected to the new version and can edit it to your liking.',
	},
	modLoadersLabel: {
		id: 'version.package-as-mod.mod-loaders',
		defaultMessage: 'Mod loaders',
	},
	modLoadersDescription: {
		id: 'version.package-as-mod.mod-loaders.description',
		defaultMessage: 'The mod loaders you would like to package your data pack for.',
	},
	modLoadersPlaceholder: {
		id: 'version.package-as-mod.mod-loaders.placeholder',
		defaultMessage: 'Choose mod loaders...',
	},
})

const copyFileHash = async (
	file: Labrinth.Versions.v3.VersionFile,
	method: Labrinth.Versions.v3.FileHashType,
) => {
	await navigator.clipboard.writeText(file.hashes[method])
}

const dependencyVersionPrimaryFiles = computed(() => {
	const versions = enrichment.value?.versions
	const primaryFileMap: Record<string, Labrinth.Versions.v2.VersionFile> = {}
	versions?.forEach((version) => {
		const primaryFile = version.files.find((version) => version.primary) ?? version.files[0]

		primaryFileMap[version.id] = primaryFile
	})
	return primaryFileMap
})

function createDependencyLink(context: {
	project?: Labrinth.Projects.v2.Project
	version?: Labrinth.Versions.v2.Version
}) {
	const baseUrl = context.version
		? `/project/${context.version.project_id}/version/${context.version.id}`
		: context.project
			? `/project/${context.project.id}`
			: undefined
	return baseUrl
		? createProjectDownloadUrl(baseUrl, {
				reason: 'dependency',
			})
		: undefined
}
</script>

<style lang="scss" scoped>
.changelog-editor-spacing {
	padding-block: var(--gap-md);
}

.version-page {
	display: grid;

	grid-template:
		'title' auto
		'changelog' auto
		'dependencies' auto
		'metadata' auto
		'files' auto
		/ 1fr;

	@media (min-width: 1200px) {
		grid-template:
			'title title' auto
			'changelog metadata' auto
			'dependencies metadata' auto
			'files metadata' auto
			'dummy metadata' 1fr
			/ 1fr 20rem;
	}

	column-gap: var(--spacing-card-md);

	.version-page__title {
		grid-area: title;

		.version-header {
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: var(--spacing-card-md);

			h2,
			input[type='text'] {
				margin: 0;
				font-size: var(--font-size-2xl);
				font-weight: bold;
			}

			input[type='text'] {
				max-width: 100%;
				min-width: 0;
				flex-grow: 1;
				width: 2rem;
			}

			.featured {
				display: flex;
				align-items: center;
				gap: var(--spacing-card-xs);

				svg {
					height: 1.45rem;
				}
			}
		}

		.known-errors {
			margin-bottom: 1rem;
		}
	}

	h3 {
		font-size: var(--font-size-lg);
		margin: 0 0 0.5rem 0;
	}

	.version-page__changelog {
		grid-area: changelog;
		overflow-x: hidden;
	}

	.version-page__dependencies {
		grid-area: dependencies;

		.dependency {
			align-items: center;
			display: flex;
			gap: var(--spacing-card-sm);
			padding: var(--spacing-card-sm);

			.info {
				display: flex;
				flex-direction: column;
				gap: var(--spacing-card-xs);

				.project-title {
					font-weight: bold;
				}

				.dep-type {
					color: var(--color-text-secondary);

					&.incompatible {
						color: var(--color-red);
					}

					&::first-letter {
						text-transform: capitalize;
					}
				}
			}

			button {
				margin-left: auto;
			}
		}

		.add-dependency {
			h4 {
				margin-bottom: var(--spacing-card-sm);
			}

			.input-group {
				&:not(:last-child) {
					margin-bottom: var(--spacing-card-sm);
				}

				input {
					flex-grow: 2;
				}
			}
		}
	}

	.version-page__files {
		grid-area: files;

		.file {
			--text-color: var(--color-button-text);
			--background-color: var(--color-button-bg);

			&.primary {
				--background-color: var(--color-brand-highlight);
				--text-color: var(--color-button-text-active);
			}

			display: flex;
			align-items: center;

			font-weight: 500;
			color: var(--text-color);
			background-color: var(--background-color);
			padding: var(--spacing-card-sm) var(--spacing-card-bg);
			border-radius: var(--size-rounded-sm);

			svg {
				min-width: 1.1rem;
				min-height: 1.1rem;
				margin-right: 0.5rem;
			}

			.filename {
				word-wrap: anywhere;
			}

			.file-size {
				margin-left: 1ch;
				font-weight: 400;
				white-space: nowrap;
			}

			.file-type {
				margin-left: 1ch;
				font-style: italic;
				font-weight: 300;
			}

			.raised-button {
				margin-left: auto;
				background-color: var(--color-raised-bg);
			}

			&:not(:nth-child(2)) {
				margin-top: 0.5rem;
			}
		}

		.additional-files {
			h4 {
				margin-bottom: 0.5rem;
			}

			label {
				margin-top: 0.5rem;
			}
		}
	}
}

.version-page__metadata {
	grid-area: metadata;

	h4 {
		margin: 1rem 0 0.25rem 0;
	}

	.maven-section {
		display: flex;
		align-items: center;
		gap: 0.5rem;

		button {
			max-width: 100%;
		}
	}

	.team-member {
		align-items: center;
		padding: 0.25rem 0.5rem;

		.member-info {
			overflow: hidden;
			margin: auto 0 auto 0.75rem;

			.name {
				font-weight: bold;
			}

			p {
				font-size: var(--font-size-sm);
				margin: 0.2rem 0;
			}
		}
	}
}

.separator {
	margin: var(--spacing-card-sm) 0;
}

.modal-package-mod {
	padding: var(--spacing-card-bg);
	display: flex;
	flex-direction: column;

	.markdown-body {
		margin-bottom: 1rem;
	}

	.package-loader-select {
		max-width: 20rem;
	}
}
</style>
