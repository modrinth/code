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

	<div v-if="selectedVersion" class="flex flex-col gap-1">
		<div class="flex flex-wrap items-center justify-between gap-2">
			<h3 class="relative top-0.5 m-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.compatibleVersionTitle) }}
			</h3>
			<ButtonStyled v-if="downloadAllFiles.length > 1" type="transparent">
				<button :disabled="downloadingSelectedVersion" @click="downloadSelectedVersionFiles">
					<SpinnerIcon v-if="downloadingSelectedVersion" aria-hidden="true" class="animate-spin" />
					<DownloadIcon v-else aria-hidden="true" />
					{{
						formatMessage(
							downloadingSelectedVersion
								? messages.downloadingSelectedVersion
								: messages.downloadAllSelectedVersion,
							{
								current: selectedVersionDownloadProgress.current,
								total: selectedVersionDownloadProgress.total,
							},
						)
					}}
				</button>
			</ButtonStyled>
		</div>
		<div
			class="grid grid-cols-[1fr_min-content] items-center gap-3 rounded-2xl bg-surface-2 px-3 py-3"
		>
			<div class="flex min-w-0 flex-col gap-1">
				<div class="flex min-w-0 items-center gap-2">
					<nuxt-link
						v-tooltip="truncatedTooltip(versionNumberRef, selectedVersion.version_number)"
						:to="`/${project.project_type}/${project.slug || project.id}/version/${selectedVersion.id}`"
						target="_blank"
						rel="noopener noreferrer"
						class="block min-w-0 text-contrast no-underline hover:underline"
					>
						<span ref="versionNumberRef" class="block truncate font-semibold">
							{{ selectedVersion.version_number }}
						</span>
					</nuxt-link>
					<VersionChannelTag
						:channel="selectedVersion.version_type"
						class="relative -top-px !py-0.5"
					/>
				</div>
				<p
					ref="versionNameRef"
					v-tooltip="truncatedTooltip(versionNameRef, selectedVersion.name)"
					class="m-0 w-fit max-w-full truncate text-sm text-secondary"
				>
					{{ selectedVersion.name }}
				</p>
			</div>
			<ButtonStyled v-if="selectedPrimaryFile" color="brand" circular>
				<a
					v-tooltip="'Download'"
					:href="selectedPrimaryFileDownloadUrl"
					:download="selectedPrimaryFile.filename"
					:aria-label="
						formatMessage(messages.downloadVersion, {
							version: selectedVersion.version_number,
						})
					"
					@click="emit('download')"
				>
					<DownloadIcon aria-hidden="true" />
				</a>
			</ButtonStyled>
		</div>
	</div>
	<p v-else-if="currentPlatform && currentGameVersion && versions.length > 0">
		{{
			formatMessage(messages.noVersionsAvailable, {
				gameVersion: currentGameVersion,
				platform: currentPlatformText,
			})
		}}
	</p>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, SpinnerIcon, TriangleAlertIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	type CdnDownloadReason,
	Checkbox,
	Combobox,
	type ComboboxOption,
	defineMessages,
	getTagMessage,
	injectNotificationManager,
	truncatedTooltip,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import VersionChannelTag from '@modrinth/ui/src/components/version/VersionChannelTag.vue'
import type { DisplayProjectType } from '@modrinth/utils'
import dayjs from 'dayjs'
import JSZip from 'jszip'
import { computed, ref, watch } from 'vue'

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
const { createProjectDownloadUrl } = useCdnDownloadContext()
const { addNotification } = injectNotificationManager()
const debug = useDebugLogger('DownloadProject')
const tags = useGeneratedState()

const userSelectedGameVersion = ref<string | null>(props.initialGameVersion)
const userSelectedPlatform = ref<string | null>(props.initialPlatform)
const showAllVersions = ref(defaultShowAllVersions())
const versionFilter = ref('')
const versionNumberRef = ref<HTMLElement | null>(null)
const versionNameRef = ref<HTMLElement | null>(null)
const downloadingSelectedVersion = ref(false)
const selectedVersionDownloadProgress = ref({
	current: 0,
	total: 0,
})

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
		.slice()
		.reverse()
		.map((platform) => ({
			value: platform,
			label: loaderLabel(platform),
			class: '!px-0 !py-1',
		}))
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
	return filteredVersions.value.find((x) => x.version_type === 'release')
})

const filteredBeta = computed<Labrinth.Versions.v3.Version | undefined>(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'beta' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))),
	)
})

const filteredAlpha = computed<Labrinth.Versions.v3.Version | undefined>(() => {
	return filteredVersions.value.find(
		(x) =>
			x.version_type === 'alpha' &&
			(!filteredRelease.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredRelease.value.date_published))) &&
			(!filteredBeta.value ||
				dayjs(x.date_published).isAfter(dayjs(filteredBeta.value.date_published))),
	)
})

const selectedVersion = computed<Labrinth.Versions.v3.Version | null>(() => {
	return filteredRelease.value || filteredBeta.value || filteredAlpha.value || null
})

const selectedPrimaryFile = computed<Labrinth.Versions.v3.VersionFile | null>(() => {
	return (
		selectedVersion.value?.files?.find((file) => file.primary) ||
		selectedVersion.value?.files?.[0] ||
		null
	)
})

const selectedPrimaryFileDownloadUrl = computed(() => {
	if (!selectedPrimaryFile.value) return '#'
	return getDownloadUrl(selectedPrimaryFile.value.url)
})

const selectedVersionDownloadFiles = computed(() => {
	if (!selectedVersion.value) return []

	return selectedVersion.value.files.map((file) => ({
		href: getDownloadUrl(file.url),
		filename: file.filename,
	}))
})

const downloadAllFiles = computed(() => {
	const files: DownloadableFile[] = []
	const hrefs = new Set<string>()

	for (const file of [...selectedVersionDownloadFiles.value, ...props.dependencyDownloadFiles]) {
		if (hrefs.has(file.href)) continue
		hrefs.add(file.href)
		files.push(file)
	}

	return files
})

const selectedVersionZipFilename = computed(() => {
	if (!selectedVersion.value) return `${sanitizeFilename(props.project.title)}.zip`

	return `${sanitizeFilename(props.project.title)} ${sanitizeFilename(
		selectedVersion.value.version_number,
	)}.zip`
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
		userSelectedGameVersion.value = props.initialGameVersion
		userSelectedPlatform.value = props.initialPlatform
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

function getDownloadUrl(url: string) {
	return createProjectDownloadUrl(url, {
		reason: props.downloadReason,
		gameVersion: currentGameVersion.value ?? undefined,
		loader: currentPlatform.value ?? undefined,
	})
}

async function downloadSelectedVersionFiles() {
	if (downloadingSelectedVersion.value || downloadAllFiles.value.length <= 1) return

	downloadingSelectedVersion.value = true
	const files = [...downloadAllFiles.value]
	selectedVersionDownloadProgress.value = {
		current: 0,
		total: files.length,
	}

	try {
		const zip = new JSZip()
		const usedFilenames = new Set<string>()

		for (const [index, file] of files.entries()) {
			selectedVersionDownloadProgress.value = {
				current: index + 1,
				total: files.length,
			}
			const response = await fetch(file.href)

			if (!response.ok) {
				throw new Error(`Failed to download ${file.filename}`)
			}

			zip.file(uniqueFilename(file.filename, usedFilenames), await response.blob())
		}

		downloadBlob(
			await zip.generateAsync({
				type: 'blob',
				mimeType: 'application/zip',
			}),
			selectedVersionZipFilename.value,
		)
		emit('download')
	} catch (error) {
		console.error('Failed to download selected version files:', error)
		addNotification({
			title: formatMessage(messages.downloadSelectedVersionFailedTitle),
			text: formatMessage(messages.downloadSelectedVersionFailedText),
			type: 'error',
		})
	} finally {
		downloadingSelectedVersion.value = false
		selectedVersionDownloadProgress.value = {
			current: 0,
			total: 0,
		}
	}
}

function downloadBlob(blob: Blob, filename: string) {
	const url = URL.createObjectURL(blob)
	const link = document.createElement('a')

	link.href = url
	link.download = filename
	document.body.appendChild(link)
	link.click()
	link.remove()
	window.setTimeout(() => URL.revokeObjectURL(url), 0)
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

function loaderLabel(loader: string) {
	return formatMessage(getTagMessage(loader, 'loader') ?? messages.unknownLoader)
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
	downloadVersion: {
		id: 'project.download.download-version',
		defaultMessage: 'Download {version}',
	},
	compatibleVersionTitle: {
		id: 'project.download.compatible-version-title',
		defaultMessage: 'Compatible version',
	},
	downloadAllSelectedVersion: {
		id: 'project.download.selected-version-download-all',
		defaultMessage: 'Download all (.zip)',
	},
	downloadingSelectedVersion: {
		id: 'project.download.selected-version-downloading',
		defaultMessage: 'Downloading... ({current}/{total})',
	},
	downloadSelectedVersionFailedTitle: {
		id: 'project.download.selected-version-failed-title',
		defaultMessage: 'Could not download version',
	},
	downloadSelectedVersionFailedText: {
		id: 'project.download.selected-version-failed-text',
		defaultMessage: 'One or more version files could not be downloaded. Please try again.',
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
