<template>
	<NewModal ref="modal" :on-show="onShow" :on-hide="onHide" width="544px">
		<template v-if="project" #title>
			<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
			<div class="truncate text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.downloadTitle, { title: project.title }) }}
			</div>
		</template>
		<template #default>
			<div v-if="project" class="mx-auto flex w-full flex-col gap-4">
				<InstallWithModrinthApp :project="project" />
				<DownloadProject
					:project="project"
					:versions="versions"
					:download-reason="downloadReason"
					:initial-game-version="initialGameVersion"
					:initial-platform="initialPlatform"
					:incompatible-game-versions="showOptions.incompatibleGameVersions"
					:incompatible-loaders="showOptions.incompatibleLoaders"
					:reset-key="downloadProjectResetKey"
					@select-game-version="selectGameVersion"
					@select-platform="selectPlatform"
					@update:selection="projectDownloadSelection = $event"
					@download="onDownload"
				/>
				<div class="flex flex-col gap-4">
					<DownloadDependencies
						:project="project"
						:selected-version="selectedVersion"
						:current-game-version="currentGameVersion"
						:current-platform="currentPlatform"
						:download-reason="downloadReason"
						@download="onDownload"
					/>
					<div v-if="additionalFiles.length > 0" class="flex flex-col gap-2">
						<h3 class="m-0 text-sm font-bold text-contrast">
							{{ formatMessage(messages.additionalFilesTitle) }}
						</h3>
						<div class="flex flex-col gap-2">
							<a
								v-for="file in additionalFiles"
								:key="file.hashes?.sha1 ?? file.filename"
								:href="getDownloadUrl(file.url)"
								:download="file.filename"
								class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary no-underline"
								@click="onDownload"
							>
								<span class="flex min-w-0 items-center gap-2">
									<FileIcon aria-hidden="true" class="size-5 flex-shrink-0 text-secondary" />
									<span class="min-w-0 truncate font-semibold text-contrast">
										{{ file.filename }}
									</span>
									<span
										class="rounded-full bg-button-bgSelected px-2 py-0.5 text-xs text-secondary"
									>
										{{ fileTypeLabel(file.file_type) }}
									</span>
								</span>
								<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
							</a>
						</div>
					</div>
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
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, FileIcon } from '@modrinth/assets'
import {
	Avatar,
	type CdnDownloadReason,
	defineMessages,
	injectModrinthClient,
	NewModal,
	ServersPromo,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'

import { navigateTo } from '#app'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'
import { STALE_TIME, STALE_TIME_LONG } from '~/composables/queries/project'

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
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { formatMessage } = useVIntl()
const debug = useDebugLogger('DownloadModal')

const modal = ref<NewModalRef | null>(null)
const modalOpen = ref(false)
const showProjectId = ref<string | null>(null)
const showOptions = ref<ResolvedProjectDownloadModalShowOptions>(getDefaultShowOptions())
const downloadProjectResetKey = ref(0)
const projectDownloadSelection = ref<ProjectDownloadSelection>(getDefaultProjectDownloadSelection())
const MODAL_CLOSE_STATE_RESET_MS = 350
let closeStateResetTimeout: ReturnType<typeof setTimeout> | null = null

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

const versionsEnabled = ref(false)
const {
	data: versionsV3,
	error: _versionsV3Error,
	isFetching: versionsV3Loading,
} = useQuery({
	queryKey: computed(() => ['project', resolvedProjectId.value, 'versions', 'v3']),
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(resolvedProjectId.value!, {
			include_changelog: false,
			apiVersion: 3,
		}),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!resolvedProjectId.value && versionsEnabled.value),
})

const versions = computed<Labrinth.Versions.v3.Version[]>(() => {
	const isModpack =
		project.value?.actualProjectType === 'modpack' || project.value?.project_type === 'modpack'

	return (versionsV3.value ?? []).map((version) => {
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
})

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
	additionalFilesTitle: {
		id: 'project.download.additional-files-title',
		defaultMessage: 'Additional files',
	},
})

const fileTypeLabels: Partial<Record<Labrinth.Versions.v3.FileType, string>> = {
	'required-resource-pack': 'Resourcepack',
	'optional-resource-pack': 'Resourcepack',
	unknown: 'File',
}

function fileTypeLabel(type?: Labrinth.Versions.v3.FileType) {
	return fileTypeLabels[type ?? 'unknown'] || 'File'
}

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

function getDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: currentGameVersion.value ?? undefined,
		loader: currentPlatform.value ?? undefined,
	})
}

function updateDownloadQuery({
	gameVersion,
	platform,
}: {
	gameVersion: string | null
	platform: string | null
}) {
	if (!props.updateRouteSelection) return
	navigateTo(
		{
			query: {
				...route.query,
				...(gameVersion && {
					version: gameVersion,
				}),
				...(platform && {
					loader: platform,
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
		platform: currentPlatform.value,
	})
}

function selectPlatform(platform: string) {
	updateDownloadQuery({
		gameVersion: currentGameVersion.value,
		platform,
	})
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
	if (!modal.value || modalOpen.value) return
	clearCloseStateResetTimeout()
	showOptions.value = {
		...getDefaultShowOptions(),
		...options,
	}
	showProjectId.value = showOptions.value.projectId ?? null
	await nextTick()
	if (!(await loadProjectForModal(!!showOptions.value.projectId))) return
	modalOpen.value = true
	modal.value.show(event)
}

async function hide() {
	if (!modal.value || !modalOpen.value) return
	modal.value?.hide()
	await nextTick()
	downloadProjectResetKey.value += 1
}

function onDownload() {
	emit('download')
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

function clearCloseStateResetTimeout() {
	if (!closeStateResetTimeout) return
	clearTimeout(closeStateResetTimeout)
	closeStateResetTimeout = null
}

async function loadProjectForModal(forceRefetch: boolean) {
	if (!routeProjectId.value) return false
	if (!forceRefetch && projectRaw.value) return true

	const { data } = await refetchProject()
	return !!data
}

function openFromHash() {
	if (
		!props.useRouteHash ||
		!modal.value ||
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

onUnmounted(clearCloseStateResetTimeout)

defineExpose({ show, hide })
</script>
