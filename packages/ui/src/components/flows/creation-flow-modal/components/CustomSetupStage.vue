<template>
	<div class="space-y-6">
		<div v-if="!hideLoaderFields" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"
				>Content loader<span class="text-red"> *</span></span
			>
			<Chips
				v-model="selectedLoader"
				:items="loaderItems"
				:format-label="capitalize"
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
			<span class="text-sm text-secondary">It is recommended to use the latest version.</span>
		</div>

		<Collapsible
			v-if="!hideLoaderFields"
			:collapsed="!selectedLoader || !selectedGameVersion"
			overflow-visible
		>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast"
					>Loader version<span class="text-red"> *</span></span
				>
				<Chips
					v-model="loaderVersionType"
					:items="loaderVersionTypeItems"
					:format-label="capitalize"
				/>
				<div v-if="loaderVersionType === 'other'">
					<Combobox
						v-model="selectedLoaderVersion"
						:options="loaderVersionOptions"
						:no-options-message="loaderVersionsLoading ? 'Loading...' : 'No versions available'"
						searchable
						placeholder="Select loader version"
					/>
				</div>
			</div>
		</Collapsible>
	</div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import { injectTags } from '../../../../providers'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import type { LoaderVersionType } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'

const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderFields,
} = injectCreationFlowContext()

const tags = injectTags()

const loaderItems = ['fabric', 'neoforge', 'forge', 'quilt']
const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

const capitalize = (item: string) => item.charAt(0).toUpperCase() + item.slice(1)

// Game versions from tags provider (releases only), filtered by loader support
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const releases = tags.gameVersions.value.filter((v) => v.version_type === 'release')

	// For loaders with per-version entries (NeoForge, Forge), only show supported versions
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
				return releases
					.filter((v) => supportedVersions.has(v.version))
					.map((v) => ({ value: v.version, label: v.version }))
			}
		}
	}

	return releases.map((v) => ({ value: v.version, label: v.version }))
})

// Loader versions fetched from launcher-meta
interface LoaderVersionEntry {
	id: string
	stable: boolean
}

const loaderVersionsLoading = ref(false)
const loaderVersionsData = ref<LoaderVersionEntry[]>([])
const loaderVersionsCache = ref<Record<string, { id: string; loaders: LoaderVersionEntry[] }[]>>({})

async function fetchLoaderManifest(loader: string) {
	let apiLoader = loader
	if (loader === 'neoforge') apiLoader = 'neo'

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

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	let apiLoader = loader
	if (loader === 'neoforge') apiLoader = 'neo'

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
		if (!loader) return
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

		if (!loader || !gameVersion) return

		loaderVersionsLoading.value = true
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

const loaderVersionOptions = computed<ComboboxOption<string>[]>(() =>
	loaderVersionsData.value.map((v) => ({
		value: v.id,
		label: v.stable ? `${v.id} (stable)` : v.id,
	})),
)
</script>
