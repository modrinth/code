<template>
	<NewModal ref="modal" :on-show="onShow" :on-hide="onHide" width="544px" actions-divider>
		<template #title>
			<template v-if="project">
				<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
				<div
					ref="downloadTitleRef"
					v-tooltip="truncatedTooltip(downloadTitleRef, downloadTitle)"
					class="truncate text-lg font-extrabold text-contrast"
				>
					{{ downloadTitle }}
				</div>
			</template>
		</template>
		<template #default>
			<div v-if="project" class="mx-auto flex w-full flex-col gap-4">
				<InstallWithModrinthApp :project="project" />
				<DownloadProject
					:project="project"
					:versions="versions"
					:dependency-download-files="dependencyDownloadFiles"
					:download-data-loaded="selectedVersionDownloadLoaded"
					:download-reason="downloadReason"
					:initial-game-version="initialGameVersion"
					:initial-platform="initialPlatform"
					:incompatible-game-versions="showOptions.incompatibleGameVersions"
					:incompatible-loaders="showOptions.incompatibleLoaders"
					:reset-key="downloadProjectResetKey"
					@select-game-version="selectGameVersion"
					@select-platform="selectPlatform"
					@update:selection="updateProjectDownloadSelection"
					@download="onDownload"
				/>
				<div class="flex flex-col gap-4">
					<DownloadDependencies @download="onDownload" />
				</div>
				<ServersPromo
					v-if="flags.showProjectPageDownloadModalServersPromo"
					:link="`/hosting#plan`"
					@close="
						() => {
							flags.showProjectPageDownloadModalServersPromo = false
							saveFeatureFlags()
						}
					"
				/>
			</div>
		</template>
		<template v-if="showDependencyDownloadActions" #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<ButtonStyled>
					<button
						class="!shadow-none"
						:disabled="!!downloadingActionType"
						@click="downloadSelectedVersionZip"
					>
						<SpinnerIcon
							v-if="downloadingActionType === 'zip'"
							aria-hidden="true"
							class="animate-spin"
						/>
						<DownloadIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.downloadAsZip) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button
						class="!shadow-none"
						:disabled="!!downloadingActionType || !dependencyDownloadFilesLoaded"
						@click="downloadSelectedVersionFilesWithDependencies"
					>
						<SpinnerIcon
							v-if="downloadingActionType === 'dependencies'"
							aria-hidden="true"
							class="animate-spin"
						/>
						<DownloadIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.downloadWithDependencies) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	type CdnDownloadReason,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	ServersPromo,
	truncatedTooltip,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import JSZip from 'jszip'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

import { navigateTo } from '#app'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'
import { STALE_TIME, STALE_TIME_LONG } from '~/composables/queries/project'

import { provideDownloadModalProvider } from './download-modal-provider'
import DownloadDependencies from './DownloadDependencies.vue'
import DownloadProject from './DownloadProject.vue'
import InstallWithModrinthApp from './InstallWithModrinthApp.vue'

type DownloadModalProject = Omit<Labrinth.Projects.v2.Project, 'project_type'> & {
	project_type: DisplayProjectType
	actualProjectType: Labrinth.Projects.v2.ProjectType
}

type ProjectDownloadSelection = {
	currentGameVersion: string | null
	currentPlatform: string | null
	selectedVersion: Labrinth.Versions.v3.Version | null
	selectedPrimaryFile: Labrinth.Versions.v3.VersionFile | null
}

type DownloadableFile = {
	href: string
	filename: string
}

type DownloadedFile = DownloadableFile & {
	blob: Blob
}

type DownloadActionType = 'zip' | 'dependencies'

type NewModalRef = {
	show: (event?: MouseEvent) => void
	hide: () => void
}

type ProjectDownloadModalShowOptions = {
	projectId?: string
	incompatibleGameVersions?: string[]
	incompatibleLoaders?: string[]
}

type ResolvedProjectDownloadModalShowOptions = {
	projectId?: string
	incompatibleGameVersions: string[]
	incompatibleLoaders: string[]
}

const props = withDefaults(
	defineProps<{
		projectId?: string
		downloadReason?: CdnDownloadReason
		useRouteHash?: boolean
		updateRouteSelection?: boolean
	}>(),
	{
		downloadReason: 'standalone',
		useRouteHash: true,
		updateRouteSelection: true,
	},
)

const emit = defineEmits<{
	download: []
}>()

const route = useRoute()
const flags = useFeatureFlags()
const tags = useGeneratedState()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const debug = useDebugLogger('DownloadModal')

const modal = ref<NewModalRef | null>(null)
const downloadTitleRef = ref<HTMLElement | null>(null)
const modalOpening = ref(false)
const modalOpen = ref(false)
const showProjectId = ref<string | null>(null)
const showOptions = ref<ResolvedProjectDownloadModalShowOptions>(getDefaultShowOptions())
const downloadProjectResetKey = ref(0)
const projectDownloadSelection = ref<ProjectDownloadSelection>(getDefaultProjectDownloadSelection())
const pendingRouteSelection = ref({
	gameVersion: getStringQueryValue(route.query.version),
	platform: getStringQueryValue(route.query.loader),
})
const downloadingActionType = ref<DownloadActionType | null>(null)
const MODAL_CLOSE_STATE_RESET_MS = 350
const DOWNLOAD_URL_REVOKE_MS = 60000
const DOWNLOAD_STAGGER_MS = 500
let closeStateResetTimeout: ReturnType<typeof setTimeout> | null = null
let modalShowRequestId = 0
let unmounted = false

const routeProjectId = computed(() => showProjectId.value ?? props.projectId ?? null)

const {
	data: projectRaw,
	error: projectV2Error,
	refetch: refetchProject,
} = useQuery({
	queryKey: computed(() => ['project', 'v2', routeProjectId.value]),
	queryFn: () => client.labrinth.projects_v2.get(routeProjectId.value!),
	enabled: computed(() => !!routeProjectId.value),
	staleTime: STALE_TIME,
})

const resolvedProjectId = computed(() => projectRaw.value?.id)

const project = computed<DownloadModalProject | null>(() => {
	if (!projectRaw.value) return null

	return {
		...projectRaw.value,
		actualProjectType: projectRaw.value.project_type,
		project_type: getProjectTypeForUrl(projectRaw.value.project_type, projectRaw.value.loaders),
	}
})

const downloadTitle = computed(() => {
	if (!project.value) return ''
	return formatMessage(messages.downloadTitle, { title: project.value.title })
})

const versionsEnabled = ref(false)
const { data: versionsV3, isFetching: versionsV3Loading } = useQuery({
	queryKey: computed(() => ['project', resolvedProjectId.value, 'versions', 'v3']),
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(resolvedProjectId.value!, {
			include_changelog: false,
			apiVersion: 3,
		}),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!resolvedProjectId.value && versionsEnabled.value),
})

const versions = computed<Labrinth.Versions.v3.Version[]>(() =>
	normalizeVersionsForDownload(versionsV3.value ?? []),
)

const initialGameVersion = computed(() => {
	const version = route.query.version
	if (typeof version !== 'string' || !project.value?.game_versions.includes(version)) return null
	return version
})
const initialPlatform = computed(() => {
	const loader = route.query.loader
	if (typeof loader !== 'string' || !project.value?.loaders.includes(loader)) return null
	return loader
})

const currentGameVersion = computed(() => projectDownloadSelection.value.currentGameVersion)
const currentPlatform = computed(() => projectDownloadSelection.value.currentPlatform)
const selectedVersion = computed(() => projectDownloadSelection.value.selectedVersion)
const selectedPrimaryFile = computed(() => projectDownloadSelection.value.selectedPrimaryFile)

const additionalFiles = computed(() => {
	if (!selectedVersion.value || !selectedPrimaryFile.value) return []
	return selectedVersion.value.files.filter((file) => file !== selectedPrimaryFile.value)
})

const downloadModalProvider = provideDownloadModalProvider({
	project,
	selectedVersion,
	selectedPrimaryFile,
	currentGameVersion,
	currentPlatform,
	downloadReason: computed(() => props.downloadReason),
	additionalFiles,
})
const dependencyDownloadFiles = downloadModalProvider.downloadableDependencyFiles
const dependencyDownloadFilesLoaded = downloadModalProvider.downloadableDependencyFilesLoaded
const selectedVersionDownloadLoaded = computed(() => !!selectedPrimaryFile.value)

const selectedVersionDownloadFiles = computed<DownloadableFile[]>(() => {
	if (!selectedVersion.value) return []

	return selectedVersion.value.files.map((file) => ({
		href: createProjectDownloadUrl(file.url, {
			reason: props.downloadReason,
			gameVersion: currentGameVersion.value ?? undefined,
			loader: currentPlatform.value ?? undefined,
		}),
		filename: file.filename,
	}))
})

const showDependencyDownloadActions = computed(
	() => dependencyDownloadFiles.value.length > 0 && selectedVersionDownloadFiles.value.length > 0,
)

watch(projectV2Error, (error) => {
	if (error) {
		debug('project query failed', error)
	}
})

const messages = defineMessages({
	downloadTitle: {
		id: 'project.download.title',
		defaultMessage: 'Download {title}',
	},
	downloadAsZip: {
		id: 'project.download.download-as-zip',
		defaultMessage: 'Download as .zip',
	},
	downloadWithDependencies: {
		id: 'project.download.download-with-dependencies',
		defaultMessage: 'Download with deps',
	},
	downloadZipFailedTitle: {
		id: 'project.download.zip-failed-title',
		defaultMessage: 'Could not download files',
	},
	downloadZipFailedText: {
		id: 'project.download.zip-failed-text',
		defaultMessage: 'One or more files could not be downloaded. Please try again.',
	},
})

function getProjectTypeForUrl(
	type: Labrinth.Projects.v2.ProjectType,
	loaders: string[],
): DisplayProjectType {
	if (type !== 'mod') return type as DisplayProjectType

	const isMod = loaders.some((loader) => tags.value.loaderData.modLoaders.includes(loader))
	const isPlugin = loaders.some((loader) => tags.value.loaderData.allPluginLoaders.includes(loader))
	const isDataPack = loaders.some((loader) =>
		tags.value.loaderData.dataPackLoaders.includes(loader),
	)

	if (isDataPack) return 'datapack'
	if (isPlugin) return 'plugin'
	if (isMod) return 'mod'

	return 'mod'
}

function updateDownloadQuery({
	gameVersion,
	platform,
}: {
	gameVersion: string | null
	platform: string | null
}) {
	if (!props.updateRouteSelection) return
	const nextGameVersion =
		gameVersion ??
		pendingRouteSelection.value.gameVersion ??
		getStringQueryValue(route.query.version)
	const nextPlatform =
		platform ?? pendingRouteSelection.value.platform ?? getStringQueryValue(route.query.loader)

	pendingRouteSelection.value = {
		gameVersion: nextGameVersion,
		platform: nextPlatform,
	}

	navigateTo(
		{
			query: {
				...route.query,
				...(nextGameVersion && {
					version: nextGameVersion,
				}),
				...(nextPlatform && {
					loader: nextPlatform,
				}),
			},
			hash: route.hash,
		},
		{ replace: true },
	)
}

function selectGameVersion(gameVersion: string) {
	updateDownloadQuery({
		gameVersion,
		platform: null,
	})
}

function selectPlatform(platform: string) {
	updateDownloadQuery({
		gameVersion: null,
		platform,
	})
}

function updateProjectDownloadSelection(selection: ProjectDownloadSelection) {
	projectDownloadSelection.value = selection
	pendingRouteSelection.value = {
		gameVersion: selection.currentGameVersion,
		platform: selection.currentPlatform,
	}
}

function onShow() {
	clearCloseStateResetTimeout()
	modalOpen.value = true
	debug('on-show fired')
	versionsEnabled.value = true
	if (props.useRouteHash && !showProjectId.value) {
		navigateTo({ query: route.query, hash: '#download' }, { replace: true })
	}
}

function onHide() {
	const hadShowProjectId = !!showProjectId.value
	modalOpen.value = false
	clearCloseStateResetTimeout()
	closeStateResetTimeout = setTimeout(() => {
		showProjectId.value = null
		showOptions.value = getDefaultShowOptions()
		closeStateResetTimeout = null
	}, MODAL_CLOSE_STATE_RESET_MS)
	if (props.useRouteHash && !hadShowProjectId) {
		navigateTo({ query: route.query, hash: '' }, { replace: true })
	}
}

async function show(
	event?: MouseEvent,
	options: ProjectDownloadModalShowOptions = {},
): Promise<void> {
	if (!modal.value || modalOpening.value || modalOpen.value) return
	const showRequestId = ++modalShowRequestId
	modalOpening.value = true

	try {
		await waitForCloseStateReset()
		if (!isActiveShowRequest(showRequestId)) return
		showOptions.value = {
			...getDefaultShowOptions(),
			...options,
		}
		showProjectId.value = showOptions.value.projectId ?? null
		await nextTick()
		if (!isActiveShowRequest(showRequestId)) return
		if (!(await loadProjectForModal(!!showOptions.value.projectId))) return
		if (!isActiveShowRequest(showRequestId)) return
		resetDownloadState()
		await preloadRouteSelectedDownload()
		if (!isActiveShowRequest(showRequestId)) return
		modalOpen.value = true
		modal.value.show(event)
	} finally {
		if (modalShowRequestId === showRequestId) {
			modalOpening.value = false
		}
	}
}

function hide() {
	modalShowRequestId += 1
	modalOpening.value = false
	if (!modal.value || !modalOpen.value) return
	modal.value?.hide()
}

function onDownload() {
	emit('download')
}

async function downloadSelectedVersionZip() {
	if (downloadingActionType.value) return

	downloadingActionType.value = 'zip'
	const files = dedupeDownloadFiles([
		...selectedVersionDownloadFiles.value,
		...dependencyDownloadFiles.value,
	])

	try {
		const zip = new JSZip()
		const usedFilenames = new Set<string>()
		const downloadedFiles = await downloadFileBlobs(files)

		for (const file of downloadedFiles) {
			zip.file(uniqueFilename(file.filename, usedFilenames), file.blob)
		}

		downloadBlob(
			await zip.generateAsync({
				type: 'blob',
				mimeType: 'application/zip',
			}),
			selectedVersionZipFilename(),
		)
		emit('download')
	} catch (error) {
		console.error('Failed to download selected version files:', error)
		addNotification({
			title: formatMessage(messages.downloadZipFailedTitle),
			text: formatMessage(messages.downloadZipFailedText),
			type: 'error',
		})
	} finally {
		downloadingActionType.value = null
	}
}

async function downloadSelectedVersionFilesWithDependencies() {
	if (downloadingActionType.value || !dependencyDownloadFilesLoaded.value) return

	downloadingActionType.value = 'dependencies'

	try {
		const files = dedupeDownloadFiles([
			...selectedVersionDownloadFiles.value,
			...dependencyDownloadFiles.value,
		])

		await downloadFiles(files)

		emit('download')
	} catch (error) {
		console.error('Failed to download selected version files:', error)
		addNotification({
			title: formatMessage(messages.downloadZipFailedTitle),
			text: formatMessage(messages.downloadZipFailedText),
			type: 'error',
		})
	} finally {
		downloadingActionType.value = null
	}
}

async function downloadFileBlobs(files: DownloadableFile[]): Promise<DownloadedFile[]> {
	return Promise.all(files.map((file) => downloadFileBlob(file)))
}

async function downloadFileBlob(file: DownloadableFile): Promise<DownloadedFile> {
	const response = await fetch(file.href)

	if (!response.ok) {
		throw new Error(`Failed to download ${file.filename}`)
	}

	return {
		...file,
		blob: await response.blob(),
	}
}

async function downloadFiles(files: DownloadableFile[]) {
	await Promise.all(
		files.map(async (file, index) => {
			await delay(DOWNLOAD_STAGGER_MS * index)
			downloadFileLink(file)
		}),
	)
}

function dedupeDownloadFiles(files: DownloadableFile[]) {
	const result: DownloadableFile[] = []
	const hrefs = new Set<string>()

	for (const file of files) {
		if (hrefs.has(file.href)) continue
		hrefs.add(file.href)
		result.push(file)
	}

	return result
}

function downloadBlob(blob: Blob, filename: string) {
	const url = URL.createObjectURL(blob)
	const link = document.createElement('a')

	link.href = url
	link.download = filename
	document.body.appendChild(link)
	link.click()
	link.remove()
	window.setTimeout(() => URL.revokeObjectURL(url), DOWNLOAD_URL_REVOKE_MS)
}

function downloadFileLink(file: DownloadableFile) {
	const link = document.createElement('a')
	link.href = file.href
	link.download = file.filename
	document.body.appendChild(link)
	link.click()
	link.remove()
}

function selectedVersionZipFilename() {
	if (!project.value || !selectedVersion.value) return 'download.zip'

	return `${sanitizeFilename(project.value.title)} ${sanitizeFilename(
		selectedVersion.value.version_number,
	)}.zip`
}

function sanitizeFilename(value: string) {
	const sanitized = value
		.replace(/[<>:"/\\|?*]/g, '')
		.replace(/\s+/g, ' ')
		.trim()

	return sanitized || 'download'
}

function uniqueFilename(filename: string, usedFilenames: Set<string>) {
	const sanitizedFilename = sanitizeFilename(filename)

	if (!usedFilenames.has(sanitizedFilename)) {
		usedFilenames.add(sanitizedFilename)
		return sanitizedFilename
	}

	const extensionIndex = sanitizedFilename.lastIndexOf('.')
	const basename =
		extensionIndex > 0 ? sanitizedFilename.slice(0, extensionIndex) : sanitizedFilename
	const extension = extensionIndex > 0 ? sanitizedFilename.slice(extensionIndex) : ''
	let index = 2
	let candidate = `${basename} (${index})${extension}`

	while (usedFilenames.has(candidate)) {
		index += 1
		candidate = `${basename} (${index})${extension}`
	}

	usedFilenames.add(candidate)
	return candidate
}

function getDefaultProjectDownloadSelection(): ProjectDownloadSelection {
	return {
		currentGameVersion: null,
		currentPlatform: null,
		selectedVersion: null,
		selectedPrimaryFile: null,
	}
}

function getDefaultShowOptions(): ResolvedProjectDownloadModalShowOptions {
	return {
		projectId: undefined,
		incompatibleGameVersions: [],
		incompatibleLoaders: [],
	}
}

function getStringQueryValue(value: unknown) {
	return typeof value === 'string' ? value : null
}

function shouldPreloadRouteSelectedDownload() {
	return (
		props.useRouteHash &&
		!showOptions.value.projectId &&
		!!getStringQueryValue(route.query.version) &&
		!!getStringQueryValue(route.query.loader)
	)
}

function clearCloseStateResetTimeout() {
	if (!closeStateResetTimeout) return
	clearTimeout(closeStateResetTimeout)
	closeStateResetTimeout = null
}

async function waitForCloseStateReset() {
	if (!closeStateResetTimeout) return
	await delay(MODAL_CLOSE_STATE_RESET_MS)
}

async function delay(ms: number) {
	await new Promise((resolve) => setTimeout(resolve, ms))
}

async function loadProjectForModal(forceRefetch: boolean) {
	if (!routeProjectId.value) return false
	if (!forceRefetch && projectRaw.value) return true

	const { data } = await refetchProject()
	return !!data
}

async function loadVersionsForModal() {
	if (!resolvedProjectId.value) return null
	versionsEnabled.value = true
	if (versionsV3.value) return versions.value

	const data = await queryClient.ensureQueryData({
		queryKey: ['project', resolvedProjectId.value, 'versions', 'v3'],
		queryFn: () =>
			client.labrinth.versions_v3.getProjectVersions(resolvedProjectId.value!, {
				include_changelog: false,
				apiVersion: 3,
			}),
		staleTime: STALE_TIME_LONG,
	})

	return Array.isArray(data) ? normalizeVersionsForDownload(data) : null
}

async function preloadRouteSelectedDownload() {
	if (!shouldPreloadRouteSelectedDownload()) return

	try {
		const routeVersions = await loadVersionsForModal()
		if (!routeVersions) return

		const selection = getRouteSelectedDownloadSelection(routeVersions)
		if (!selection) return

		await preloadDependenciesForSelection(selection)
	} catch (error) {
		debug('failed to preload selected route download', error)
	}
}

function getRouteSelectedDownloadSelection(
	versionList: Labrinth.Versions.v3.Version[] = versions.value,
): ProjectDownloadSelection | null {
	const gameVersion = initialGameVersion.value
	const platform = initialPlatform.value
	const version = getSelectedRouteVersion(gameVersion, platform, versionList)
	const primaryFile =
		version?.files?.find((file) => file.primary) || version?.files?.[0] || null

	if (!gameVersion || !platform || !version || !primaryFile) return null

	return {
		currentGameVersion: gameVersion,
		currentPlatform: platform,
		selectedVersion: version,
		selectedPrimaryFile: primaryFile,
	}
}

function getSelectedRouteVersion(
	gameVersion: string | null,
	platform: string | null,
	versionList: Labrinth.Versions.v3.Version[],
) {
	if (!gameVersion || !platform || !project.value) return null

	const filteredVersions = versionList.filter((version) => {
		const matchesPlatform =
			project.value?.project_type === 'resourcepack' ||
			(!!platform && version.loaders.includes(platform))

		return version.game_versions.includes(gameVersion) && matchesPlatform
	})

	return (
		latestVersionByType(filteredVersions, 'release') ||
		latestVersionByType(filteredVersions, 'beta') ||
		latestVersionByType(filteredVersions, 'alpha') ||
		null
	)
}

function normalizeVersionsForDownload(
	versionList: Labrinth.Versions.v3.Version[],
): Labrinth.Versions.v3.Version[] {
	const isModpack =
		project.value?.actualProjectType === 'modpack' || project.value?.project_type === 'modpack'

	return versionList.map((version) => {
		const files = Array.isArray(version.files) ? version.files : []
		const gameVersions = Array.isArray(version.game_versions) ? version.game_versions : []
		const loaders = Array.isArray(version.loaders) ? version.loaders : []
		const mrpackLoaders = Array.isArray(version.mrpack_loaders) ? version.mrpack_loaders : []

		return {
			...version,
			files,
			game_versions: gameVersions,
			loaders: isModpack && mrpackLoaders.length ? mrpackLoaders : loaders,
		}
	})
}

function latestVersionByType(
	versionList: Labrinth.Versions.v3.Version[],
	type: Labrinth.Versions.v3.VersionChannel,
) {
	return versionList
		.filter((version) => version.version_type === type)
		.reduce<Labrinth.Versions.v3.Version | undefined>((latest, version) => {
			if (!latest || dayjs(version.date_published).isAfter(dayjs(latest.date_published))) {
				return version
			}

			return latest
		}, undefined)
}

async function preloadDependenciesForSelection(selection: ProjectDownloadSelection) {
	await downloadModalProvider.preloadDependenciesForSelection(selection)
}

function isActiveShowRequest(showRequestId: number) {
	return (
		!unmounted &&
		modalShowRequestId === showRequestId &&
		!!modal.value &&
		!modalOpen.value
	)
}

function resetDownloadState() {
	projectDownloadSelection.value = getDefaultProjectDownloadSelection()
	downloadProjectResetKey.value += 1
}

function openFromHash() {
	if (
		!props.useRouteHash ||
		!modal.value ||
		modalOpening.value ||
		modalOpen.value ||
		showProjectId.value ||
		route.hash !== '#download'
	) {
		return
	}

	debug('hash #download watch fired, opening modal')
	show()
}

if (
	props.useRouteHash &&
	(route.hash === '#download' ||
		route.query.version !== undefined ||
		route.query.loader !== undefined)
) {
	debug('eager loadVersions from setup', {
		hash: route.hash,
		version: route.query.version,
		loader: route.query.loader,
		loading: versionsV3Loading.value,
	})
	versionsEnabled.value = true
}

watch(modal, openFromHash)
watch(() => route.hash, openFromHash)
watch(routeProjectId, () => {
	projectDownloadSelection.value = getDefaultProjectDownloadSelection()
	downloadProjectResetKey.value += 1
})

onUnmounted(() => {
	unmounted = true
	modalShowRequestId += 1
	clearCloseStateResetTimeout()
})

defineExpose({ show, hide })
</script>
