<template>
	<div class="space-y-6">
		<div v-if="!hideLoaderFields" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"
				>Content loader<span class="text-red"> *</span></span
			>
			<Chips
				v-model="selectedLoader"
				:items="ctx.availableLoaders"
				:format-label="formatLoaderLabel"
				:never-empty="false"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Game version<span class="text-red"> *</span></span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				searchable
				placeholder="Select game version"
			/>
			<Checkbox
				v-if="ctx.showSnapshotToggle"
				:model-value="ctx.showSnapshots.value"
				label="Show snapshots"
				class="text-sm"
				@update:model-value="ctx.showSnapshots.value = $event"
			/>
		</div>

		<Collapsible
			v-if="!hideLoaderFields"
			:collapsed="!selectedLoader || !selectedGameVersion"
			overflow-visible
		>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast"
					>{{ isPaperLike ? 'Build number' : 'Loader version'
					}}<span class="text-red"> *</span></span
				>
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
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { injectTags } from '../../../../providers'
import Checkbox from '../../../base/Checkbox.vue'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import type { LoaderVersionType } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderFields,
} = ctx

const tags = injectTags()

const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

const capitalize = (item: string) => item.charAt(0).toUpperCase() + item.slice(1)

const loaderDisplayNames: Record<string, string> = {
	fabric: 'Fabric',
	neoforge: 'NeoForge',
	forge: 'Forge',
	quilt: 'Quilt',
	paper: 'Paper',
	purpur: 'Purpur',
	vanilla: 'Vanilla',
}

const formatLoaderLabel = (item: string) => loaderDisplayNames[item] ?? capitalize(item)

const isPaperLike = computed(
	() => selectedLoader.value === 'paper' || selectedLoader.value === 'purpur',
)

// Game versions from tags provider, filtered by loader support
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	// For loaders with per-version entries (NeoForge, Forge, Paper, Purpur), only show supported versions
	if (selectedLoader.value) {
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
