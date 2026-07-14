<template>
	<div class="flex w-full gap-2 max-sm:flex-wrap">
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
			select-search-text-on-focus
			@update:model-value="selectGameVersion"
			@search-input="versionFilter = $event"
			@close="versionFilter = ''"
		>
			<template #option="{ item, isSelected }">
				<div
					v-tooltip="gameVersionOptionTooltip(item.value)"
					class="flex w-full items-center justify-between gap-2 px-4 py-2"
					:class="{
						'text-brand-red opacity-40': isGameVersionUnavailable(item.value),
						'text-green': isSelected,
						'!opacity-100': isSelected,
						'text-primary': !isGameVersionUnavailable(item.value) && !isSelected,
					}"
				>
					<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
					<TriangleAlertIcon
						v-if="isSelected && isGameVersionUnavailable(item.value)"
						aria-hidden="true"
						class="size-5 shrink-0 text-orange"
					/>
				</div>
			</template>
			<template #dropdown-footer>
				<div
					v-if="showVersionsCheckbox"
					class="border-0 border-t border-solid border-surface-5 p-3"
				>
					<Checkbox
						v-model="showAllVersionsModel"
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
					v-tooltip="platformOptionTooltip(item.value, item.label)"
					class="flex w-full items-center justify-between gap-2 px-4 py-2"
					:class="{
						'text-brand-red opacity-40': isPlatformUnavailable(item.value),
						'text-green': isSelected,
						'!opacity-100': isSelected,
						'text-primary': !isPlatformUnavailable(item.value) && !isSelected,
					}"
				>
					<span class="min-w-0 truncate font-semibold leading-tight">{{ item.label }}</span>
					<TriangleAlertIcon
						v-if="isSelected && isPlatformUnavailable(item.value)"
						aria-hidden="true"
						class="size-5 shrink-0 text-orange"
					/>
				</div>
			</template>
		</Combobox>
	</div>

	<div
		v-if="selectedVersion && downloadDataLoaded"
		:role="compatibleVersions.length > 1 ? 'radiogroup' : undefined"
		:aria-label="
			compatibleVersions.length > 1 ? formatMessage(messages.compatibleVersionTitle) : undefined
		"
		class="flex flex-col gap-2.5"
	>
		<h3
			v-if="compatibleVersions.length > 1"
			class="relative top-0.5 m-0 text-base font-semibold text-contrast"
		>
			{{ formatMessage(messages.compatibleVersionTitle) }}
		</h3>
		<CompatibleVersionCard
			v-for="compatibleVersion in compatibleVersions"
			:key="compatibleVersion.id"
			:project="project"
			:version="compatibleVersion"
			:download-reason="downloadReason"
			:current-game-version="currentGameVersion"
			:current-platform="currentPlatform"
			:selectable="compatibleVersions.length > 1"
			:selected="compatibleVersion.id === selectedVersion.id"
			:show-download="
				compatibleVersions.length === 1 || compatibleVersion.id === selectedVersion.id
			"
			:color="
				compatibleVersion.id === selectedVersion.id &&
				compatibleVersions.length === 1 &&
				!hasAdditionalDownloads
					? 'brand'
					: 'standard'
			"
			:type="
				compatibleVersion.id === selectedVersion.id &&
				compatibleVersions.length === 1 &&
				!hasAdditionalDownloads
					? 'standard'
					: 'transparent'
			"
			:circular="hasAdditionalDownloads || compatibleVersions.length > 1"
			@select="selectCompatibleVersion(compatibleVersion)"
			@download="emit('download')"
		/>
	</div>
	<div v-else-if="showNoCompatibleVersions" class="pl-1 text-base text-primary" role="status">
		{{ noCompatibleVersionsDescription }}
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { TriangleAlertIcon } from '@modrinth/assets'
import {
	type CdnDownloadReason,
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	getTagMessage,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import type { DisplayProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

import CompatibleVersionCard from './CompatibleVersionCard.vue'

defineOptions({
	name: 'DownloadProject',
})

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

const props = withDefaults(
	defineProps<{
		project: DownloadModalProject
		versions?: Labrinth.Versions.v3.Version[]
		dependencyDownloadFiles?: DownloadableFile[]
		downloadDataLoaded?: boolean
		versionsLoaded?: boolean
		downloadReason?: CdnDownloadReason
		initialGameVersion?: string | null
		initialPlatform?: string | null
		incompatibleGameVersions?: string[]
		incompatibleLoaders?: string[]
		resetKey?: number
	}>(),
	{
		versions: () => [],
		dependencyDownloadFiles: () => [],
		downloadDataLoaded: false,
		versionsLoaded: false,
		downloadReason: 'standalone',
		initialGameVersion: null,
		initialPlatform: null,
		incompatibleGameVersions: () => [],
		incompatibleLoaders: () => [],
		resetKey: 0,
	},
)

const emit = defineEmits<{
	download: []
	selectGameVersion: [gameVersion: string]
	selectPlatform: [platform: string]
	'update:selection': [selection: ProjectDownloadSelection]
}>()
const { formatMessage } = useVIntl()
const debug = useDebugLogger('DownloadProject')
const tags = useGeneratedState()

const userSelectedGameVersion = ref<string | null>(props.initialGameVersion)
const userSelectedPlatform = ref<string | null>(props.initialPlatform)
const userSelectedCompatibleVersionId = ref<string | null>(null)
const showAllVersions = ref(defaultShowAllVersions())
const versionFilter = ref('')
const preferredPlatformRanks = new Map([
	['fabric', 0],
	['forge', 1],
	['neoforge', 2],
])

const incompatibleGameVersionsSet = computed(() => new Set(props.incompatibleGameVersions))
const incompatibleLoadersSet = computed(() => new Set(props.incompatibleLoaders))

const showAllVersionsModel = computed({
	get() {
		return showAllVersions.value
	},
	set(value) {
		showAllVersions.value = value
	},
})

const selectedPlatform = computed<string | null>(() => {
	if (userSelectedPlatform.value) return userSelectedPlatform.value
	return props.project.loaders.length === 1 ? props.project.loaders[0] : null
})

const selectedGameVersion = computed<string | null>(() => {
	if (userSelectedGameVersion.value) return userSelectedGameVersion.value
	return props.project.game_versions.length === 1 ? props.project.game_versions[0] : null
})

const compatiblePlatforms = computed<string[]>(() => {
	return props.project.loaders.filter(
		(platform) =>
			props.versions.some(
				(version) =>
					version.loaders.includes(platform) &&
					(!selectedGameVersion.value || version.game_versions.includes(selectedGameVersion.value)),
			) && !incompatibleLoadersSet.value.has(platform),
	)
})

const currentPlatform = computed<string | null>(() => {
	if (selectedPlatform.value) return selectedPlatform.value
	return compatiblePlatforms.value.length === 1 ? compatiblePlatforms.value[0] : null
})

const possibleGameVersions = computed<string[]>(() => {
	return props.versions
		.filter((x) => !currentPlatform.value || x.loaders.includes(currentPlatform.value))
		.flatMap((x) => x.game_versions)
})

const compatibleGameVersions = computed<string[]>(() => {
	return props.project.game_versions.filter(
		(gameVersion) =>
			possibleGameVersions.value.includes(gameVersion) &&
			!incompatibleGameVersionsSet.value.has(gameVersion),
	)
})

const currentGameVersion = computed<string | null>(() => {
	if (selectedGameVersion.value) return selectedGameVersion.value
	return compatibleGameVersions.value.length === 1 ? compatibleGameVersions.value[0] : null
})

const possiblePlatforms = computed<string[]>(() => {
	return props.versions
		.filter((x) => !currentGameVersion.value || x.game_versions.includes(currentGameVersion.value))
		.flatMap((x) => x.loaders)
})

const currentPlatformText = computed(() => {
	if (!currentPlatform.value) return ''
	return loaderLabel(currentPlatform.value)
})

const releaseVersions = computed<Set<string>>(() => {
	const set = new Set<string>()
	for (const gameVersion of tags.value.gameVersions || []) {
		if (gameVersion?.version && gameVersion.version_type === 'release') {
			set.add(gameVersion.version)
		}
	}
	return set
})

const nonReleaseVersions = computed<Set<string>>(() => {
	const set = new Set<string>()
	for (const gameVersion of tags.value.gameVersions || []) {
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

const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	return filteredGameVersions.value.map((gameVersion) => ({
		value: gameVersion,
		label: gameVersion,
		class: '!px-0 !py-1',
	}))
})

const platformOptions = computed<ComboboxOption<string>[]>(() => {
	return props.project.loaders
		.map((platform) => ({
			value: platform,
			label: loaderLabel(platform),
			class: '!px-0 !py-1',
		}))
		.sort(comparePlatformOptions)
})

const filteredVersions = computed<Labrinth.Versions.v3.Version[]>(() => {
	const gameVersion = currentGameVersion.value
	if (!gameVersion) return []

	const platform = currentPlatform.value
	const result = props.versions.filter((x) => {
		const matchesPlatform =
			props.project.project_type === 'resourcepack' || (!!platform && x.loaders.includes(platform))

		return x.game_versions.includes(gameVersion) && matchesPlatform
	})
	debug('filteredVersions', {
		total: props.versions.length,
		filtered: result.length,
		currentGameVersion: currentGameVersion.value,
		currentPlatform: currentPlatform.value,
		sampleLoaders: props.versions.slice(0, 3).map((v) => v.loaders),
	})
	return result
})

const filteredRelease = computed<Labrinth.Versions.v3.Version | undefined>(() => {
	return latestVersionByType('release')
})

const filteredBeta = computed<Labrinth.Versions.v3.Version | undefined>(() => {
	return latestVersionByType('beta')
})

const filteredAlpha = computed<Labrinth.Versions.v3.Version | undefined>(() => {
	return latestVersionByType('alpha')
})

const defaultSelectedVersion = computed<Labrinth.Versions.v3.Version | null>(() => {
	return filteredRelease.value || filteredBeta.value || filteredAlpha.value || null
})

const suggestedPreReleaseVersions = computed<Labrinth.Versions.v3.Version[]>(() => {
	if (!defaultSelectedVersion.value || defaultSelectedVersion.value.version_type !== 'release')
		return []

	const versions: Labrinth.Versions.v3.Version[] = []
	const beta = filteredBeta.value
	if (beta && isNewerThan(beta, defaultSelectedVersion.value)) {
		versions.push(beta)
	}

	const alpha = filteredAlpha.value
	if (alpha && isNewerThan(alpha, defaultSelectedVersion.value)) {
		versions.push(alpha)
	}

	return versions
})

const compatibleVersions = computed<Labrinth.Versions.v3.Version[]>(() => {
	if (!defaultSelectedVersion.value) return []
	return [defaultSelectedVersion.value, ...suggestedPreReleaseVersions.value]
})

const showNoCompatibleVersions = computed(() => {
	return (
		props.versionsLoaded &&
		compatibleVersions.value.length === 0 &&
		!!currentGameVersion.value &&
		!!currentPlatform.value
	)
})

const noCompatibleVersionsDescription = computed(() => {
	const gameVersion = currentGameVersion.value
	if (!gameVersion || !currentPlatform.value) return ''

	return formatMessage(messages.noVersionsAvailable, {
		gameVersion,
		platform: currentPlatformText.value,
	})
})

const selectedVersion = computed<Labrinth.Versions.v3.Version | null>(() => {
	return (
		compatibleVersions.value.find(
			(version) => version.id === userSelectedCompatibleVersionId.value,
		) ||
		defaultSelectedVersion.value ||
		null
	)
})

const selectedPrimaryFile = computed<Labrinth.Versions.v3.VersionFile | null>(() => {
	return (
		selectedVersion.value?.files?.find((file) => file.primary) ||
		selectedVersion.value?.files?.[0] ||
		null
	)
})

const requiredResourcePackFile = computed<Labrinth.Versions.v3.VersionFile | null>(() => {
	if (props.project.project_type !== 'datapack') return null

	return (
		selectedVersion.value?.files?.find(
			(file) => file !== selectedPrimaryFile.value && file.file_type === 'required-resource-pack',
		) || null
	)
})

const recommendedResourcePackFiles = computed<Labrinth.Versions.v3.VersionFile[]>(() => {
	if (props.project.project_type !== 'datapack') return []

	return (
		selectedVersion.value?.files?.filter(
			(file) => file !== selectedPrimaryFile.value && file.file_type === 'optional-resource-pack',
		) || []
	)
})

const hasAdditionalDownloads = computed(() => {
	const hrefs = new Set<string>()

	if (selectedPrimaryFile.value) {
		hrefs.add(selectedPrimaryFile.value.url)
	}

	if (requiredResourcePackFile.value) {
		hrefs.add(requiredResourcePackFile.value.url)
	}

	for (const file of recommendedResourcePackFiles.value) {
		hrefs.add(file.url)
	}

	for (const file of props.dependencyDownloadFiles) {
		if (hrefs.has(file.href)) continue
		hrefs.add(file.href)
	}

	return hrefs.size > 1
})

watch(
	[currentGameVersion, currentPlatform, selectedVersion, selectedPrimaryFile],
	() => {
		emit('update:selection', {
			currentGameVersion: currentGameVersion.value,
			currentPlatform: currentPlatform.value,
			selectedVersion: selectedVersion.value,
			selectedPrimaryFile: selectedPrimaryFile.value,
		})
	},
	{ immediate: true },
)

watch([currentGameVersion, currentPlatform], () => {
	userSelectedCompatibleVersionId.value = null
})

watch(
	() => props.resetKey,
	() => {
		userSelectedGameVersion.value = props.initialGameVersion
		userSelectedPlatform.value = props.initialPlatform
		userSelectedCompatibleVersionId.value = null
		showAllVersions.value = defaultShowAllVersions()
		versionFilter.value = ''
	},
)

function selectGameVersion(gameVersion?: string) {
	if (!gameVersion) return
	userSelectedGameVersion.value = gameVersion
	emit('selectGameVersion', gameVersion)
	selectOnlyCompatiblePlatform()
}

function selectPlatform(platform?: string) {
	if (!platform) return
	userSelectedPlatform.value = platform
	emit('selectPlatform', platform)
}

function selectCompatibleVersion(version: Labrinth.Versions.v3.Version) {
	userSelectedCompatibleVersionId.value = version.id
}

function latestVersionByType(type: Labrinth.Versions.v3.VersionChannel) {
	return filteredVersions.value
		.filter((version) => version.version_type === type)
		.reduce<Labrinth.Versions.v3.Version | undefined>((latest, version) => {
			if (!latest || isNewerThan(version, latest)) return version
			return latest
		}, undefined)
}

function isNewerThan(
	version: Labrinth.Versions.v3.Version,
	comparison: Labrinth.Versions.v3.Version,
) {
	return dayjs(version.date_published).isAfter(dayjs(comparison.date_published))
}

function loaderLabel(loader: string) {
	return formatMessage(getTagMessage(loader, 'loader') ?? messages.unknownLoader)
}

function comparePlatformOptions(a: ComboboxOption<string>, b: ComboboxOption<string>) {
	const aRank = preferredPlatformRanks.get(a.value) ?? Number.MAX_SAFE_INTEGER
	const bRank = preferredPlatformRanks.get(b.value) ?? Number.MAX_SAFE_INTEGER

	if (aRank !== bRank) return aRank - bRank

	return a.label.localeCompare(b.label)
}

function isReleaseGameVersion(version: string) {
	if (releaseVersions.value.has(version)) return true
	if (nonReleaseVersions.value.has(version)) return false
	return true
}

function defaultShowAllVersions() {
	return (
		props.project.game_versions.length > 0 &&
		props.project.game_versions.every((projectVersion) => {
			const gameVersion = tags.value.gameVersions?.find((x) => x.version === projectVersion)
			return !!gameVersion?.version_type && gameVersion.version_type !== 'release'
		})
	)
}

function isGameVersionUnavailable(gameVersion: string) {
	return (
		incompatibleGameVersionsSet.value.has(gameVersion) ||
		!possibleGameVersions.value.includes(gameVersion)
	)
}

function isPlatformUnavailable(platform: string) {
	return incompatibleLoadersSet.value.has(platform) || !possiblePlatforms.value.includes(platform)
}

function selectOnlyCompatiblePlatform() {
	const compatiblePlatforms = props.project.loaders.filter(
		(platform) =>
			possiblePlatforms.value.includes(platform) && !incompatibleLoadersSet.value.has(platform),
	)

	if (compatiblePlatforms.length !== 1) return

	userSelectedPlatform.value = compatiblePlatforms[0]
	emit('selectPlatform', compatiblePlatforms[0])
}

function gameVersionOptionTooltip(gameVersion: string) {
	if (incompatibleGameVersionsSet.value.has(gameVersion)) {
		return formatMessage(messages.baseGameVersionIncompatibleTooltip)
	}

	if (!possibleGameVersions.value.includes(gameVersion)) {
		return formatMessage(messages.gameVersionUnsupportedTooltip, {
			title: props.project.title,
			gameVersion,
			platform: currentPlatformText.value,
		})
	}

	return null
}

function platformOptionTooltip(platform: string, platformLabel: string) {
	if (incompatibleLoadersSet.value.has(platform)) {
		return formatMessage(messages.baseLoaderIncompatibleTooltip)
	}

	if (!possiblePlatforms.value.includes(platform)) {
		return formatMessage(messages.platformUnsupportedTooltip, {
			title: props.project.title,
			platform: platformLabel,
			gameVersion: currentGameVersion.value,
		})
	}

	return null
}

const messages = defineMessages({
	baseGameVersionIncompatibleTooltip: {
		id: 'project.download.base-game-version-incompatible-tooltip',
		defaultMessage: 'This game version is incompatible with the base project.',
	},
	baseLoaderIncompatibleTooltip: {
		id: 'project.download.base-loader-incompatible-tooltip',
		defaultMessage: 'This loader is incompatible with the base project.',
	},
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	compatibleVersionTitle: {
		id: 'project.download.compatible-version-title',
		defaultMessage: 'Compatible versions',
	},
	noGameVersionsFound: {
		id: 'project.download.no-game-versions-found',
		defaultMessage: 'No game versions found',
	},
	noVersionsAvailable: {
		id: 'project.download.no-versions-available',
		defaultMessage: 'No versions available for {gameVersion} and {platform}.',
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
	unknownLoader: {
		id: 'project.download.unknown-loader',
		defaultMessage: 'Unknown loader',
	},
})
</script>
