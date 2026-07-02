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
				<div
					v-if="
						project.project_type !== 'plugin' ||
						project.loaders.some((x) => !tags.loaderData.allPluginLoaders.includes(x))
					"
					class="modrinth-app-section contents"
				>
					<div class="flex flex-col">
						<a
							class="modrinth-app-install-card flex items-center justify-between gap-3 rounded-2xl border border-solid border-brand-highlight bg-surface-1 px-4 py-3 text-primary no-underline transition-[filter] hover:brightness-110"
							:href="`modrinth://mod/${project.slug}`"
							@click="installWithApp"
						>
							<span class="flex w-full min-w-0 flex-col gap-1">
								<div class="flex items-center justify-between">
									<span class="flex min-w-0 items-center gap-1.5 font-medium text-contrast">
										Install with
										<span class="text-brand">Modrinth App</span>
										<ModrinthIcon aria-hidden="true" class="size-4 flex-shrink-0 text-brand" />
									</span>
									<ExternalIcon
										aria-hidden="true"
										class="size-4 flex-shrink-0 text-contrast transition-colors"
									/>
								</div>
								<span class="truncate text-base text-secondary">
									{{ formatMessage(messages.installWithModrinthAppDescription) }}
								</span>
							</span>
						</a>
						<Accordion ref="getModrinthAppAccordion">
							<nuxt-link class="mt-2 flex justify-center text-brand-blue hover:underline" to="/app">
								{{ formatMessage(messages.dontHaveModrinthApp) }}
							</nuxt-link>
						</Accordion>
					</div>

					<div class="flex items-center gap-4">
						<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
						<span class="flex-shrink-0 text-sm font-medium text-secondary">
							{{ formatMessage(messages.downloadManually) }}
						</span>
						<div class="flex h-[2px] w-full rounded-2xl bg-button-bg"></div>
					</div>
				</div>

				<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">
					<Combobox
						:model-value="currentGameVersion || undefined"
						class="w-full"
						:options="gameVersionOptions"
						:placeholder="formatMessage(messages.selectGameVersion)"
						:searchable="project.game_versions.length > 4"
						search-autocomplete="off"
						:search-placeholder="formatMessage(messages.searchGameVersions)"
						:no-options-message="formatMessage(messages.noGameVersionsFound)"
						trigger-class="!rounded-xl !bg-button-bg !px-3 !py-2"
						dropdown-class="!rounded-xl"
						@update:model-value="selectGameVersion"
						@search-input="(query) => (versionFilter = query)"
						@close="versionFilter = ''"
					>
						<template #option="{ item, isSelected }">
							<div
								v-tooltip="
									!possibleGameVersions.includes(item.value)
										? formatMessage(messages.gameVersionUnsupportedTooltip, {
												title: project.title,
												gameVersion: item.value,
												platform: currentPlatformText,
											})
										: null
								"
								class="flex w-full items-center justify-between gap-2"
								:class="{
									'text-brand-red opacity-40': !possibleGameVersions.includes(item.value),
									'text-green': isSelected,
									'text-primary': possibleGameVersions.includes(item.value) && !isSelected,
								}"
							>
								<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
							</div>
						</template>
						<template #dropdown-footer>
							<div
								v-if="showVersionsCheckbox"
								class="border-0 border-t border-solid border-surface-5 p-3"
							>
								<Checkbox
									v-model="showAllVersions"
									:label="formatMessage(messages.showAllVersions)"
									:disabled="!!versionFilter"
								/>
							</div>
						</template>
					</Combobox>
					<Combobox
						v-if="project.project_type !== 'resourcepack'"
						:model-value="currentPlatform || undefined"
						class="w-full"
						:options="platformOptions"
						:placeholder="formatMessage(messages.selectPlatform)"
						trigger-class="!rounded-xl !bg-button-bg !px-3 !py-2"
						dropdown-class="!rounded-xl"
						@update:model-value="selectPlatform"
					>
						<template #option="{ item, isSelected }">
							<div
								v-tooltip="
									!possiblePlatforms.includes(item.value)
										? formatMessage(messages.platformUnsupportedTooltip, {
												title: project.title,
												platform: item.label,
												gameVersion: currentGameVersion,
											})
										: null
								"
								class="flex w-full items-center justify-between gap-2"
								:class="{
									'text-brand-red opacity-40': !possiblePlatforms.includes(item.value),
									'text-green': isSelected,
									'text-primary': possiblePlatforms.includes(item.value) && !isSelected,
								}"
							>
								<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
							</div>
						</template>
					</Combobox>
				</div>
				<div class="flex flex-col gap-4">
					<div
						v-if="selectedVersion"
						class="grid grid-cols-[minmax(0,1fr)_min-content] items-center gap-3 rounded-2xl bg-bg px-3 py-3"
					>
						<div class="flex min-w-0 flex-col gap-1">
							<div class="flex min-w-0 items-center gap-2">
								<span class="truncate font-bold text-contrast">
									{{ selectedVersion.version_number }}
								</span>
								<VersionChannelTag :channel="selectedVersion.version_type" class="!py-0.5" />
							</div>
							<p class="m-0 truncate text-sm text-secondary">
								{{ selectedVersion.name }}
							</p>
						</div>
						<ButtonStyled v-if="selectedPrimaryFile" color="brand" circular>
							<a
								:href="getDownloadUrl(selectedPrimaryFile.url)"
								:download="selectedPrimaryFile.filename"
								:aria-label="
									formatMessage(messages.downloadVersion, {
										version: selectedVersion.version_number,
									})
								"
								v-tooltip="'Download'"
								@click="onDownload"
							>
								<DownloadIcon aria-hidden="true" />
							</a>
						</ButtonStyled>
					</div>
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
					<div v-if="dependencyRows.length > 0" class="flex flex-col gap-2">
						<h3 class="m-0 text-sm font-bold text-contrast">
							{{ formatMessage(messages.dependenciesTitle) }}
						</h3>
						<div class="flex flex-col gap-2">
							<div
								v-for="dependency in dependencyRows"
								:key="dependency.key"
								class="flex flex-col gap-1.5"
							>
								<div
									class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary"
								>
									<span class="flex min-w-0 items-center gap-2">
										<Avatar
											v-if="dependency.icon"
											:src="dependency.icon"
											:alt="dependency.name"
											size="20px"
										/>
										<PackageIcon
											v-else
											aria-hidden="true"
											class="size-5 flex-shrink-0 text-secondary"
										/>
										<a
											v-if="dependency.projectHref"
											:href="dependency.projectHref"
											target="_blank"
											rel="noopener noreferrer"
											class="min-w-0 truncate font-semibold text-contrast no-underline hover:underline"
										>
											{{ dependency.name }}
										</a>
										<span v-else class="min-w-0 truncate font-semibold text-contrast">
											{{ dependency.name }}
										</span>
										<TagItem class="shrink-0 border !border-solid border-surface-5">
											{{ dependency.typeLabel }}
										</TagItem>
									</span>
									<ButtonStyled v-if="dependency.downloadHref" circular type="transparent">
										<a
											v-tooltip="'Download'"
											:href="dependency.downloadHref"
											:download="dependency.filename"
											:aria-label="`Download ${dependency.name}`"
											@click="onDownload"
										>
											<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
										</a>
									</ButtonStyled>
								</div>
								<div
									v-for="subDependency in dependency.subDependencies"
									:key="subDependency.key"
									class="grid grid-cols-[1.5rem_minmax(0,1fr)] items-center gap-1 pl-5"
								>
									<RightArrowIcon aria-hidden="true" class="size-4 text-secondary" />
									<div
										class="grid min-h-9 grid-cols-[minmax(0,1fr)_min-content] items-center gap-2 rounded-xl bg-button-bg px-3 py-2 text-primary"
									>
										<span class="flex min-w-0 items-center gap-2">
											<Avatar
												v-if="subDependency.icon"
												:src="subDependency.icon"
												:alt="subDependency.name"
												size="20px"
											/>
											<PackageIcon
												v-else
												aria-hidden="true"
												class="size-5 flex-shrink-0 text-secondary"
											/>
											<a
												v-if="subDependency.projectHref"
												:href="subDependency.projectHref"
												target="_blank"
												rel="noopener noreferrer"
												class="min-w-0 truncate font-semibold text-contrast no-underline hover:underline"
											>
												{{ subDependency.name }}
											</a>
											<span v-else class="min-w-0 truncate font-semibold text-contrast">
												{{ subDependency.name }}
											</span>
											<TagItem class="shrink-0 border !border-solid border-surface-5">
												{{ subDependency.typeLabel }}
											</TagItem>
										</span>
										<ButtonStyled v-if="subDependency.downloadHref" circular type="transparent">
											<a
												v-tooltip="'Download'"
												:href="subDependency.downloadHref"
												:download="subDependency.filename"
												:aria-label="`Download ${subDependency.name}`"
												@click="onDownload"
											>
												<DownloadIcon aria-hidden="true" class="size-5 text-secondary" />
											</a>
										</ButtonStyled>
									</div>
								</div>
							</div>
						</div>
					</div>
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
import {
	CheckIcon,
	DownloadIcon,
	ExternalIcon,
	FileIcon,
	ModrinthIcon,
	PackageIcon,
	RightArrowIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	Combobox,
	defineMessages,
	getTagMessage,
	injectModrinthClient,
	NewModal,
	ServersPromo,
	TagItem,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, nextTick, ref, watch } from 'vue'

import { navigateTo } from '#app'
import Accordion from '~/components/ui/Accordion.vue'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'

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
const getModrinthAppAccordion = ref()

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
	dontHaveModrinthApp: {
		id: 'project.download.no-app',
		defaultMessage: "Don't have Modrinth App?",
	},
	downloadTitle: {
		id: 'project.download.title',
		defaultMessage: 'Download {title}',
	},
	gameVersionLabel: {
		id: 'project.download.game-version',
		defaultMessage: 'Game version: {version}',
	},
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	installWithModrinthApp: {
		id: 'project.download.install-with-app',
		defaultMessage: 'Install with Modrinth App',
	},
	downloadManually: {
		id: 'project.download.manually',
		defaultMessage: 'Download manually',
	},
	installWithModrinthAppDescription: {
		id: 'project.download.install-with-app-description',
		defaultMessage: 'Automatically install the correct version and dependencies.',
	},
	downloadVersion: {
		id: 'project.download.download-version',
		defaultMessage: 'Download {version}',
	},
	dependenciesTitle: {
		id: 'project.download.dependencies-title',
		defaultMessage: 'Dependencies',
	},
	additionalFilesTitle: {
		id: 'project.download.additional-files-title',
		defaultMessage: 'Additional files',
	},
	noVersionsAvailable: {
		id: 'project.download.no-versions-available',
		defaultMessage: 'No versions available for {gameVersion} and {platform}.',
	},
	noGameVersionsFound: {
		id: 'project.download.no-game-versions-found',
		defaultMessage: 'No game versions found',
	},
	platformLabel: {
		id: 'project.download.platform',
		defaultMessage: 'Platform: {platform}',
	},
	platformUnsupportedTooltip: {
		id: 'project.download.platform-unsupported-tooltip',
		defaultMessage: '{title} does not support {platform} for {gameVersion}',
	},
	searchGameVersions: {
		id: 'project.download.search-game-versions',
		defaultMessage: 'Select game version',
	},
	selectGameVersion: {
		id: 'project.download.select-game-version',
		defaultMessage: 'Select game version',
	},
	selectPlatform: {
		id: 'project.download.select-platform',
		defaultMessage: 'Select platform',
	},
	showAllVersions: {
		id: 'project.download.show-all-versions',
		defaultMessage: 'Show all versions',
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

function installWithApp() {
	setTimeout(() => {
		getModrinthAppAccordion.value?.open()
	}, 1500)
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

<style lang="scss" scoped>
.modrinth-app-install-card {
	background: radial-gradient(circle at 50% 300%, #0d2f17 0%, var(--surface-1) 72%);
}

@media (hover: none) and (max-width: 767px) {
	.modrinth-app-section {
		display: none;
	}
}
</style>
