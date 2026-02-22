<template>
	<div class="space-y-6">
		<!-- Instance-specific: Icon upload -->
		<div v-if="ctx.flowType === 'instance'" class="flex items-center gap-4">
			<Avatar :src="ctx.instanceIconUrl.value ?? undefined" size="5rem" />
			<div class="flex flex-col gap-2">
				<ButtonStyled type="outlined">
					<button class="!border-surface-5" @click="triggerIconInput">
						<UploadIcon />
						Select icon
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button class="!border-surface-5" :disabled="!ctx.instanceIcon.value" @click="removeIcon">
						<XIcon />
						Remove icon
					</button>
				</ButtonStyled>
			</div>
		</div>

		<!-- Instance-specific: Name field -->
		<div v-if="ctx.flowType === 'instance'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Name <span class="text-secondary font-normal">(Optional)</span></span>
			<StyledInput v-model="ctx.instanceName.value" placeholder="Enter instance name" />
		</div>

		<!-- Loader chips -->
		<div v-if="!hideLoaderChips" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{
				ctx.flowType === 'instance' ? 'Loader' : 'Content loader'
			}}</span>
			<Chips
				v-model="selectedLoader"
				:items="effectiveLoaders"
				:format-label="formatLoaderLabel"
				:never-empty="false"
			/>
		</div>

		<!-- Game version -->
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Game version</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				search-mode
				sync-with-selection
				placeholder="Select game version"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
						@mousedown.prevent
						@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
					>
						<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
						<EyeIcon v-else class="size-4" />
						{{ ctx.showSnapshots.value ? 'Hide snapshots' : 'Show all versions' }}
					</button>
				</template>
			</Combobox>
		</div>

		<!-- Loader version -->
		<template v-if="!hideLoaderVersion">
			<Collapsible :collapsed="!selectedLoader || !selectedGameVersion" overflow-visible>
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						isPaperLike ? 'Build number' : 'Loader version'
					}}</span>
					<Chips
						v-if="!isPaperLike"
						v-model="loaderVersionType"
						:items="loaderVersionTypeItems"
						:format-label="capitalize"
					/>
					<div v-if="isPaperLike || loaderVersionType === 'other'">
						<Combobox
							v-model="selectedLoaderVersion"
							:options="loaderVersionOptions"
							:no-options-message="loaderVersionsLoading ? 'Loading...' : 'No versions available'"
							searchable
							:placeholder="isPaperLike ? 'Select build number' : 'Select loader version'"
						/>
					</div>
				</div>
			</Collapsible>
		</template>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon, EyeOffIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { computed, onMounted, ref, watch } from 'vue'

import { injectFilePicker, injectTags } from '../../../../providers'
import Avatar from '../../../base/Avatar.vue'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import type { LoaderVersionType } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'
import { capitalize, formatLoaderLabel } from '../shared'

const ctx = injectCreationFlowContext()
const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderChips,
	hideLoaderVersion,
} = ctx

// For instance flow, prepend 'vanilla' to available loaders
const effectiveLoaders = computed(() => {
	if (ctx.flowType === 'instance') {
		return ['vanilla', ...ctx.availableLoaders.filter((l) => l !== 'vanilla')]
	}
	return ctx.availableLoaders
})

// Pre-select loader and game version from initial values
onMounted(() => {
	if (!selectedLoader.value) {
		if (ctx.initialLoader) {
			selectedLoader.value = ctx.initialLoader
		} else {
			selectedLoader.value = 'fabric'
		}
	}
	if (ctx.initialGameVersion && !selectedGameVersion.value) {
		selectedGameVersion.value = ctx.initialGameVersion
	}
})

const tags = injectTags()

const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

const isPaperLike = computed(
	() => selectedLoader.value === 'paper' || selectedLoader.value === 'purpur',
)

// Icon upload handling
const filePicker = injectFilePicker()

async function triggerIconInput() {
	const picked = await filePicker.pickImage()
	if (picked) {
		ctx.instanceIcon.value = picked.file
		ctx.instanceIconUrl.value = picked.previewUrl
		ctx.instanceIconPath.value = picked.path ?? null
	}
}

function removeIcon() {
	ctx.instanceIcon.value = null
	ctx.instanceIconUrl.value = null
	ctx.instanceIconPath.value = null
}

// Game versions from tags provider, filtered by loader support
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	// For loaders with per-version entries (NeoForge, Forge, Paper, Purpur), only show supported versions
	if (selectedLoader.value && selectedLoader.value !== 'vanilla') {
		let apiLoader = selectedLoader.value
		if (apiLoader === 'neoforge') apiLoader = 'neo'

		const manifest = loaderVersionsCache.value[apiLoader]
		if (manifest) {
			const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
			if (!hasPlaceholder) {
				const supportedVersions = new Set(
					manifest.filter((x) => x.loaders.length > 0).map((x) => x.id),
				)
				return versions
					.filter((v) => supportedVersions.has(v.version))
					.map((v) => ({ value: v.version, label: v.version }))
			}
		}
	}

	return versions.map((v) => ({ value: v.version, label: v.version }))
})

// Auto-select latest game version when options become available and none is selected
watch(
	gameVersionOptions,
	(options) => {
		if (!selectedGameVersion.value && options.length > 0) {
			selectedGameVersion.value = options[0].value
		}
	},
	{ immediate: true },
)

// Loader versions fetched from launcher-meta
interface LoaderVersionEntry {
	id: string
	stable: boolean
}

const loaderVersionsLoading = ref(false)
const loaderVersionsData = ref<LoaderVersionEntry[]>([])
const loaderVersionsCache = ref<Record<string, { id: string; loaders: LoaderVersionEntry[] }[]>>({})

// Paper/Purpur build caches
const paperVersions = ref<Record<string, number[]>>({})
const purpurVersions = ref<Record<string, string[]>>({})

async function fetchLoaderManifest(loader: string) {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	if (loaderVersionsCache.value[apiLoader]) return

	try {
		const res = await fetch(`https://launcher-meta.modrinth.com/${apiLoader}/v0/manifest.json`)
		const data = (await res.json()) as {
			gameVersions: { id: string; loaders: LoaderVersionEntry[] }[]
		}
		loaderVersionsCache.value[apiLoader] = data.gameVersions
	} catch {
		loaderVersionsCache.value[apiLoader] = []
	}
}

async function fetchPaperVersions(mcVersion: string) {
	if (paperVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://fill.papermc.io/v3/projects/paper/versions/${mcVersion}`)
		const data = (await res.json()) as { builds: number[] }
		paperVersions.value[mcVersion] = data.builds.sort((a, b) => b - a)
	} catch {
		paperVersions.value[mcVersion] = []
	}
}

async function fetchPurpurVersions(mcVersion: string) {
	if (purpurVersions.value[mcVersion]) return
	try {
		const res = await fetch(`https://api.purpurmc.org/v2/purpur/${mcVersion}`)
		const data = (await res.json()) as { builds: { all: string[] } }
		purpurVersions.value[mcVersion] = data.builds.all.sort((a, b) => parseInt(b) - parseInt(a))
	} catch {
		purpurVersions.value[mcVersion] = []
	}
}

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	let apiLoader = loader
	if (apiLoader === 'neoforge') apiLoader = 'neo'

	const manifest = loaderVersionsCache.value[apiLoader]
	if (!manifest) return []

	// Some loaders (e.g. Fabric) list all versions under a placeholder entry
	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) return placeholder.loaders

	const entry = manifest.find((x) => x.id === gameVersion)
	return entry?.loaders ?? []
}

// Fetch manifest when loader changes so game versions can be filtered
watch(
	() => selectedLoader.value,
	async (loader) => {
		if (!loader || loader === 'vanilla' || loader === 'paper' || loader === 'purpur') return
		await fetchLoaderManifest(loader)
	},
	{ immediate: true },
)

// Watch loader + game version to resolve loader versions
watch(
	[() => selectedLoader.value, () => selectedGameVersion.value],
	async ([loader, gameVersion]) => {
		loaderVersionsData.value = []
		selectedLoaderVersion.value = null

		if (!loader || !gameVersion || loader === 'vanilla') return

		loaderVersionsLoading.value = true

		if (loader === 'paper') {
			await fetchPaperVersions(gameVersion)
			loaderVersionsLoading.value = false
			// Auto-select latest build
			const builds = paperVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = `${builds[0]}`
			}
			return
		}

		if (loader === 'purpur') {
			await fetchPurpurVersions(gameVersion)
			loaderVersionsLoading.value = false
			// Auto-select latest build
			const builds = purpurVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = builds[0]
			}
			return
		}

		await fetchLoaderManifest(loader)
		loaderVersionsData.value = getLoaderVersionsForGameVersion(loader, gameVersion)
		loaderVersionsLoading.value = false

		// Auto-select based on loaderVersionType
		autoSelectLoaderVersion()
	},
)

watch(
	() => loaderVersionType.value,
	() => autoSelectLoaderVersion(),
)

function autoSelectLoaderVersion() {
	if (loaderVersionType.value === 'stable') {
		const stable = loaderVersionsData.value.find((v) => v.stable)
		selectedLoaderVersion.value = stable?.id ?? loaderVersionsData.value[0]?.id ?? null
	} else if (loaderVersionType.value === 'latest') {
		selectedLoaderVersion.value = loaderVersionsData.value[0]?.id ?? null
	}
	// 'other' — user picks manually via Combobox
}

const loaderVersionOptions = computed<ComboboxOption<string>[]>(() => {
	if (selectedLoader.value === 'paper' && selectedGameVersion.value) {
		const builds = paperVersions.value[selectedGameVersion.value] ?? []
		return builds.map((b) => ({ value: `${b}`, label: `Build ${b}` }))
	}

	if (selectedLoader.value === 'purpur' && selectedGameVersion.value) {
		const builds = purpurVersions.value[selectedGameVersion.value] ?? []
		return builds.map((b) => ({ value: b, label: `Build ${b}` }))
	}

	return loaderVersionsData.value.map((v) => ({
		value: v.id,
		label: v.stable ? `${v.id} (stable)` : v.id,
	}))
})
</script>
