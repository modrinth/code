<template>
	<NewModal ref="modal" :on-show="onShow" :on-hide="onHide" width="544px">
		<template #title>
			<Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" />
			<div class="truncate text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.downloadTitle, { title: project.title }) }}
			</div>
		</template>
		<template #default>
			<div class="mx-auto flex w-full flex-col gap-4">
				<InstallWithModrinthApp :project="project" :tags="tags" />
				<DownloadProject
					:project="project"
					:game-version-options="gameVersionOptions"
					:current-game-version="currentGameVersion"
					:possible-game-versions="possibleGameVersions"
					:current-platform-text="currentPlatformText"
					:show-versions-checkbox="showVersionsCheckbox"
					:show-all-versions="showAllVersions"
					:version-filter="versionFilter"
					:current-platform="currentPlatform"
					:platform-options="platformOptions"
					:possible-platforms="possiblePlatforms"
					:selected-version="selectedVersion"
					:selected-primary-file="selectedPrimaryFile"
					:selected-primary-file-download-url="selectedPrimaryFileDownloadUrl"
					@select-game-version="selectGameVersion"
					@select-platform="selectPlatform"
					@update:show-all-versions="showAllVersions = $event"
					@update:version-filter="versionFilter = $event"
					@download="onDownload"
				/>
				<div class="flex flex-col gap-4">
					<p
						v-if="
							currentPlatform &&
							currentGameVersion &&
							!selectedVersion &&
							!versionsLoading &&
							versions.length > 0
						"
					>
						{{
							formatMessage(messages.noVersionsAvailable, {
								gameVersion: currentGameVersion,
								platform: currentPlatformText,
							})
						}}
					</p>
					<DownloadDependencies :dependencies="dependencyRows" @download="onDownload" />
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

<script setup>
import { DownloadIcon, FileIcon } from '@modrinth/assets'
import {
	Avatar,
	defineMessages,
	getTagMessage,
	injectModrinthClient,
	NewModal,
	ServersPromo,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, nextTick, ref, watch } from 'vue'

import { navigateTo } from '#app'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'

import DownloadDependencies from './DownloadDependencies.vue'
import DownloadProject from './DownloadProject.vue'
import InstallWithModrinthApp from './InstallWithModrinthApp.vue'

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	versions: {
		type: Array,
		default: () => [],
	},
	versionsLoading: {
		type: Boolean,
		default: false,
	},
	tags: {
		type: Object,
		required: true,
	},
	downloadReason: {
		type: String,
		default: 'standalone',
	},
	loadVersions: {
		type: Function,
		required: true,
	},
})

const emit = defineEmits(['download'])

const route = useRoute()
const flags = useFeatureFlags()
const client = injectModrinthClient()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { formatMessage } = useVIntl()
const debug = useDebugLogger('DownloadModal')

const modal = ref()
const modalOpen = ref(false)
const userSelectedGameVersion = ref(null)
const userSelectedPlatform = ref(null)
const showAllVersions = ref(false)
const versionFilter = ref('')

const currentGameVersion = computed(() => {
	return (
		userSelectedGameVersion.value ||
		(props.project.game_versions.length === 1 && props.project.game_versions[0])
	)
})

const possibleGameVersions = computed(() => {
	return props.versions
		.filter((x) => !currentPlatform.value || x.loaders.includes(currentPlatform.value))
		.flatMap((x) => x.game_versions)
})

const possiblePlatforms = computed(() => {
	return props.versions
		.filter((x) => !currentGameVersion.value || x.game_versions.includes(currentGameVersion.value))
		.flatMap((x) => x.loaders)
})

const currentPlatform = computed(() => {
	return (
		userSelectedPlatform.value || (props.project.loaders.length === 1 && props.project.loaders[0])
	)
})

const currentPlatformText = computed(() => {
	if (!currentPlatform.value) return null
	return formatMessage(getTagMessage(currentPlatform.value, 'loader'))
})

const releaseVersions = computed(() => {
	const set = new Set()
	for (const gameVersion of props.tags.gameVersions || []) {
		if (gameVersion?.version && gameVersion.version_type === 'release') {
			set.add(gameVersion.version)
		}
	}
	return set
})

const nonReleaseVersions = computed(() => {
	const set = new Set()
	for (const gameVersion of props.tags.gameVersions || []) {
		if (gameVersion?.version && gameVersion.version_type !== 'release') {
			set.add(gameVersion.version)
		}
	}
	return set
})

const showVersionsCheckbox = computed(() => {
	let hasRelease = false
	let hasNonRelease = false

	for (const version of props.project.game_versions) {
		if (isReleaseGameVersion(version)) {
			hasRelease = true
		} else {
			hasNonRelease = true
		}

		if (hasRelease && hasNonRelease) return true
	}

	return false
})

const filteredGameVersions = computed(() => {
	return props.project.game_versions
		.filter(
			(x) =>
				(versionFilter.value && x.includes(versionFilter.value)) ||
				(!versionFilter.value && (showAllVersions.value || isReleaseGameVersion(x))),
		)
		.slice()
		.reverse()
})

const gameVersionOptions = computed(() => {
	return filteredGameVersions.value.map((gameVersion) => ({
		value: gameVersion,
		label: gameVersion,
	}))
})

const platformOptions = computed(() => {
	return props.project.loaders
		.slice()
		.reverse()
		.map((platform) => ({
			value: platform,
			label: formatMessage(getTagMessage(platform, 'loader')),
		}))
})

const filteredVersions = computed(() => {
	const result = props.versions.filter(
		(x) =>
			x.game_versions?.includes(currentGameVersion.value) &&
			(x.loaders?.includes(currentPlatform.value) || props.project.project_type === 'resourcepack'),
	)
	debug('filteredVersions', {
		total: props.versions.length,
		filtered: result.length,
		currentGameVersion: currentGameVersion.value,
		currentPlatform: currentPlatform.value,
		sampleLoaders: props.versions.slice(0, 3).map((v) => v.loaders),
	})
	return result
})

const filteredRelease = computed(() => {
	return filteredVersions.value.find((x) => x.version_type === 'release')
})

const filteredBeta = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'beta' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))),
	)
})

const filteredAlpha = computed(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'alpha' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))) &&
			(!filteredBeta.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredBeta.value.date_published))),
	)
})

const selectedVersion = computed(() => {
	return filteredRelease.value || filteredBeta.value || filteredAlpha.value
})

const selectedPrimaryFile = computed(() => {
	return (
		selectedVersion.value?.files?.find((file) => file.primary) || selectedVersion.value?.files?.[0]
	)
})

const selectedPrimaryFileDownloadUrl = computed(() => {
	if (!selectedPrimaryFile.value) return null
	return getDownloadUrl(selectedPrimaryFile.value.url)
})

const additionalFiles = computed(() => {
	if (!selectedVersion.value || !selectedPrimaryFile.value) return []
	return selectedVersion.value.files.filter((file) => file !== selectedPrimaryFile.value)
})

const selectedDependencyVersionIds = computed(() => {
	if (!selectedVersion.value) return []

	return [
		...new Set(
			selectedVersion.value.dependencies.map((dependency) => dependency.version_id).filter(Boolean),
		),
	]
})

const { data: dependencyVersions } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'versions',
		selectedDependencyVersionIds.value,
	]),
	queryFn: () => client.labrinth.versions_v3.getVersions(selectedDependencyVersionIds.value),
	enabled: computed(() => selectedDependencyVersionIds.value.length > 0),
})

const fallbackDependencyProjectIds = computed(() => {
	if (!selectedVersion.value) return []

	return [
		...new Set(
			selectedVersion.value.dependencies
				.filter(
					(dependency) =>
						['required', 'optional'].includes(dependency.dependency_type) &&
						!dependency.version_id &&
						dependency.project_id,
				)
				.map((dependency) => dependency.project_id),
		),
	]
})

const { data: fallbackDependencyVersions } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'fallback-versions',
		fallbackDependencyProjectIds.value,
	]),
	queryFn: async () =>
		Promise.all(
			fallbackDependencyProjectIds.value.map(async (projectId) => {
				const versions = await client.labrinth.versions_v3.getProjectVersions(projectId, {
					include_changelog: false,
					limit: 1,
				})

				return versions[0]
			}),
		),
	enabled: computed(() => fallbackDependencyProjectIds.value.length > 0),
})

const subDependencyVersionIds = computed(() => {
	const ids = new Set()

	for (const version of [
		...(dependencyVersions.value || []),
		...(fallbackDependencyVersions.value || []),
	]) {
		if (!version) continue
		for (const dependency of version.dependencies || []) {
			if (dependency.version_id) ids.add(dependency.version_id)
		}
	}

	return [...ids]
})

const { data: subDependencyVersions } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'sub-versions',
		subDependencyVersionIds.value,
	]),
	queryFn: () => client.labrinth.versions_v3.getVersions(subDependencyVersionIds.value),
	enabled: computed(() => subDependencyVersionIds.value.length > 0),
})

const fallbackSubDependencyProjectIds = computed(() => {
	const ids = new Set()

	for (const version of [
		...(dependencyVersions.value || []),
		...(fallbackDependencyVersions.value || []),
	]) {
		if (!version) continue
		for (const dependency of version.dependencies || []) {
			if (
				['required', 'optional'].includes(dependency.dependency_type) &&
				!dependency.version_id &&
				dependency.project_id
			) {
				ids.add(dependency.project_id)
			}
		}
	}

	return [...ids]
})

const { data: fallbackSubDependencyVersions } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'fallback-sub-versions',
		fallbackSubDependencyProjectIds.value,
	]),
	queryFn: async () =>
		Promise.all(
			fallbackSubDependencyProjectIds.value.map(async (projectId) => {
				const versions = await client.labrinth.versions_v3.getProjectVersions(projectId, {
					include_changelog: false,
					limit: 1,
				})

				return versions[0]
			}),
		),
	enabled: computed(() => fallbackSubDependencyProjectIds.value.length > 0),
})

const dependencyVersionById = computed(() => {
	const map = new Map()
	for (const version of [
		...(dependencyVersions.value || []),
		...(subDependencyVersions.value || []),
		...(fallbackDependencyVersions.value || []),
		...(fallbackSubDependencyVersions.value || []),
	]) {
		if (!version) continue
		map.set(version.id, version)
	}
	return map
})

const fallbackDependencyVersionByProjectId = computed(() => {
	const map = new Map()
	for (const version of [
		...(fallbackDependencyVersions.value || []),
		...(fallbackSubDependencyVersions.value || []),
	]) {
		if (!version) continue
		map.set(version.project_id, version)
	}
	return map
})

const selectedDependencyProjectIds = computed(() => {
	const ids = new Set()

	for (const dependency of selectedVersion.value?.dependencies || []) {
		if (dependency.project_id) ids.add(dependency.project_id)
		const version = dependency.version_id
			? dependencyVersionById.value.get(dependency.version_id)
			: null
		if (version?.project_id) ids.add(version.project_id)
	}

	for (const version of [
		...(dependencyVersions.value || []),
		...(fallbackDependencyVersions.value || []),
	]) {
		if (!version) continue
		for (const dependency of version.dependencies || []) {
			if (dependency.project_id) ids.add(dependency.project_id)
			const dependencyVersion = dependency.version_id
				? dependencyVersionById.value.get(dependency.version_id)
				: null
			if (dependencyVersion?.project_id) ids.add(dependencyVersion.project_id)
		}
	}

	for (const version of [
		...(fallbackDependencyVersions.value || []),
		...(fallbackSubDependencyVersions.value || []),
	]) {
		if (version?.project_id) ids.add(version.project_id)
	}

	return [...ids]
})

const { data: dependencyProjects } = useQuery({
	queryKey: computed(() => [
		'project-download-modal',
		'projects',
		selectedDependencyProjectIds.value,
	]),
	queryFn: () => client.labrinth.projects_v2.getMultiple(selectedDependencyProjectIds.value),
	enabled: computed(() => selectedDependencyProjectIds.value.length > 0),
})

const dependencyProjectById = computed(() => {
	const map = new Map()
	for (const project of dependencyProjects.value || []) {
		map.set(project.id, project)
	}
	return map
})

const dependencyRows = computed(() => {
	if (!selectedVersion.value) return []

	return selectedVersion.value.dependencies
		.filter((dependency) => ['required', 'optional'].includes(dependency.dependency_type))
		.sort(
			(a, b) =>
				dependencyTypeSortOrder(a.dependency_type) - dependencyTypeSortOrder(b.dependency_type),
		)
		.map((dependency) => createDependencyRow(dependency))
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
	noVersionsAvailable: {
		id: 'project.download.no-versions-available',
		defaultMessage: 'No versions available for {gameVersion} and {platform}.',
	},
})

function dependencyTypeLabel(type) {
	return (
		{
			required: 'Required',
			optional: 'Optional',
			embedded: 'Embedded',
			incompatible: 'Incompatible',
		}[type] || type
	)
}

function dependencyTypeSortOrder(type) {
	return (
		{
			required: 0,
			optional: 1,
		}[type] ?? 2
	)
}

function fileTypeLabel(type) {
	return (
		{
			'required-resource-pack': 'Resourcepack',
			'optional-resource-pack': 'Resourcepack',
			unknown: 'File',
		}[type] || 'File'
	)
}

function primaryFileForVersion(version) {
	return version?.files?.find((file) => file.primary) || version?.files?.[0]
}

function createDependencyRow(dependency, includeSubDependencies = true) {
	const explicitVersion = dependency.version_id
		? dependencyVersionById.value.get(dependency.version_id)
		: null
	// todo: need algorithm for picking version.
	// try to match same game version and loader. and if cannot, then disable button and have tooltip saying cannot find any compatible versions to download.
	const version =
		explicitVersion ||
		(dependency.project_id
			? fallbackDependencyVersionByProjectId.value.get(dependency.project_id)
			: null)
	const projectId = dependency.project_id || version?.project_id
	const project = projectId ? dependencyProjectById.value.get(projectId) : null
	const primaryFile = primaryFileForVersion(version)
	const name =
		project?.title ||
		dependency.file_name ||
		version?.name ||
		version?.version_number ||
		dependency.version_id ||
		dependency.project_id ||
		'Dependency'

	return {
		key: `${dependency.dependency_type}-${dependency.version_id ?? dependency.project_id ?? dependency.file_name ?? name}`,
		name,
		icon: project?.icon_url,
		projectHref: project ? `/${project.project_type}/${project.slug || project.id}` : undefined,
		downloadHref: primaryFile ? getDownloadUrl(primaryFile.url) : undefined,
		filename: primaryFile?.filename,
		typeLabel: dependencyTypeLabel(dependency.dependency_type),
		subDependencies: includeSubDependencies
			? (version?.dependencies || [])
					.filter((subDependency) =>
						['required', 'optional'].includes(subDependency.dependency_type),
					)
					.sort(
						(a, b) =>
							dependencyTypeSortOrder(a.dependency_type) -
							dependencyTypeSortOrder(b.dependency_type),
					)
					.map((subDependency) => createDependencyRow(subDependency, false))
			: [],
	}
}

function getDownloadUrl(url) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: currentGameVersion.value ?? undefined,
		loader: currentPlatform.value ?? undefined,
	})
}

function isReleaseGameVersion(version) {
	if (releaseVersions.value.has(version)) return true
	if (nonReleaseVersions.value.has(version)) return false
	return true
}

function updateDownloadQuery() {
	navigateTo(
		{
			query: {
				...route.query,
				...(userSelectedGameVersion.value && {
					version: userSelectedGameVersion.value,
				}),
				...(userSelectedPlatform.value && {
					loader: userSelectedPlatform.value,
				}),
			},
			hash: route.hash,
		},
		{ replace: true },
	)
}

function selectGameVersion(gameVersion) {
	userSelectedGameVersion.value = gameVersion
	updateDownloadQuery()
}

function selectPlatform(platform) {
	userSelectedPlatform.value = platform
	updateDownloadQuery()
}

function onShow() {
	modalOpen.value = true
	debug('on-show fired')
	props.loadVersions()
	navigateTo({ query: route.query, hash: '#download' }, { replace: true })
}

function onHide() {
	modalOpen.value = false
	navigateTo({ query: route.query, hash: '' }, { replace: true })
}

function show(event) {
	if (!modal.value || modalOpen.value) return
	modalOpen.value = true
	modal.value.show(event)
}

function hide(event) {
	if (!modal.value || !modalOpen.value) return
	modal.value?.hide(event)
	userSelectedPlatform.value = null
	userSelectedGameVersion.value = null
	showAllVersions.value = false
}

function onDownload() {
	emit('download')
}

function onVersionNavigate(url) {
	hide()
	nextTick(() => {
		navigateTo(url)
	})
}

function openFromHash() {
	if (!modal.value || modalOpen.value || route.hash !== '#download') return

	debug('hash #download watch fired, opening modal')
	show()
}

const { version, loader } = route.query

if (
	props.project.game_versions.length > 0 &&
	props.project.game_versions.every((projectVersion) => !isReleaseGameVersion(projectVersion))
) {
	showAllVersions.value = true
}

if (version !== undefined && props.project.game_versions.includes(version)) {
	userSelectedGameVersion.value = version
}

if (loader !== undefined && props.project.loaders.includes(loader)) {
	userSelectedPlatform.value = loader
}

if (route.hash === '#download' || version !== undefined || loader !== undefined) {
	debug('eager loadVersions from setup', { hash: route.hash, version, loader })
	props.loadVersions()
}

watch(modal, openFromHash)
watch(() => route.hash, openFromHash)

defineExpose({ show, hide })
</script>
