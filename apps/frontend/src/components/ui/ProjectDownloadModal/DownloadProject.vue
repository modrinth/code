<template>
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
			@search-input="versionFilter = $event"
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
				:href="selectedPrimaryFileDownloadUrl"
				:download="selectedPrimaryFile.filename"
				:aria-label="
					formatMessage(messages.downloadVersion, {
						version: selectedVersion.version_number,
					})
				"
				v-tooltip="'Download'"
				@click="emit('download')"
			>
				<DownloadIcon aria-hidden="true" />
			</a>
		</ButtonStyled>
	</div>
	<p v-else-if="currentPlatform && currentGameVersion && !versionsLoading && versions.length > 0">
		{{
			formatMessage(messages.noVersionsAvailable, {
				gameVersion: currentGameVersion,
				platform: currentPlatformText,
			})
		}}
	</p>
</template>

<script setup>
import { DownloadIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Checkbox,
	Combobox,
	defineMessages,
	getTagMessage,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

defineOptions({
	name: 'DownloadProject',
})

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
	initialGameVersion: {
		type: String,
		default: null,
	},
	initialPlatform: {
		type: String,
		default: null,
	},
	resetKey: {
		type: Number,
		default: 0,
	},
})

const emit = defineEmits(['download', 'selectGameVersion', 'selectPlatform', 'update:selection'])
const { formatMessage } = useVIntl()
const { createProjectDownloadUrl } = useCdnDownloadContext()
const debug = useDebugLogger('DownloadProject')

const userSelectedGameVersion = ref(props.initialGameVersion)
const userSelectedPlatform = ref(props.initialPlatform)
const showAllVersions = ref(defaultShowAllVersions())
const versionFilter = ref('')

const showAllVersionsModel = computed({
	get() {
		return showAllVersions.value
	},
	set(value) {
		showAllVersions.value = value
	},
})

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

watch(
	() => props.resetKey,
	() => {
		userSelectedGameVersion.value = null
		userSelectedPlatform.value = null
		showAllVersions.value = defaultShowAllVersions()
		versionFilter.value = ''
	},
)

function selectGameVersion(gameVersion) {
	userSelectedGameVersion.value = gameVersion
	emit('selectGameVersion', gameVersion)
}

function selectPlatform(platform) {
	userSelectedPlatform.value = platform
	emit('selectPlatform', platform)
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

function defaultShowAllVersions() {
	return (
		props.project.game_versions.length > 0 &&
		props.project.game_versions.every((projectVersion) => {
			const gameVersion = props.tags.gameVersions?.find((x) => x.version === projectVersion)
			return gameVersion?.version_type && gameVersion.version_type !== 'release'
		})
	)
}

const messages = defineMessages({
	gameVersionUnsupportedTooltip: {
		id: 'project.download.game-version-unsupported-tooltip',
		defaultMessage: '{title} does not support {gameVersion} for {platform}',
	},
	downloadVersion: {
		id: 'project.download.download-version',
		defaultMessage: 'Download {version}',
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
})
</script>
