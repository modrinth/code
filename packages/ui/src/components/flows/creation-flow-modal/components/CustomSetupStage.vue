<template>
	<div class="space-y-6">
		<!-- Instance-specific: Icon upload -->
		<div v-if="ctx.flowType === 'instance'" class="flex items-center gap-4">
			<Avatar :src="ctx.instanceIconUrl.value ?? undefined" size="5rem" />
			<div class="flex flex-col gap-2">
				<ButtonStyled type="outlined">
					<button @click="triggerIconInput">
						<UploadIcon />
						{{ formatMessage(messages.selectIcon) }}
					</button>
				</ButtonStyled>
				<ButtonStyled type="outlined">
					<button :disabled="!ctx.instanceIcon.value" @click="removeIcon">
						<XIcon />
						{{ formatMessage(messages.removeIcon) }}
					</button>
				</ButtonStyled>
			</div>
		</div>

		<!-- Instance-specific: Name field -->
		<div v-if="ctx.flowType === 'instance'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.nameLabel) }}</span>
			<StyledInput
				v-model="ctx.instanceName.value"
				:placeholder="ctx.autoInstanceName.value || formatMessage(messages.instanceNamePlaceholder)"
			/>
		</div>

		<!-- Loader chips -->
		<div v-if="!hideLoaderChips" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{
				ctx.flowType === 'instance'
					? formatMessage(messages.loaderLabel)
					: formatMessage(messages.contentLoaderLabel)
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
			<span class="font-semibold text-contrast">{{
				formatMessage(commonMessages.gameVersionLabel)
			}}</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				:no-options-message="
					gameVersionsLoading
						? formatMessage(commonMessages.loadingLabel)
						: formatMessage(messages.noVersionsAvailable)
				"
				searchable
				sync-with-selection
				:placeholder="formatMessage(messages.selectGameVersion)"
				:search-placeholder="formatMessage(messages.searchGameVersion)"
				@option-hover="handleGameVersionHover"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
						@mousedown.prevent
						@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
					>
						<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
						<EyeIcon v-else class="size-4" />
						{{
							ctx.showSnapshots.value
								? formatMessage(commonMessages.hideSnapshotsButton)
								: formatMessage(commonMessages.showAllVersionsButton)
						}}
					</button>
				</template>
			</Combobox>
		</div>

		<!-- Loader version -->
		<template v-if="!hideLoaderVersion">
			<Collapsible :collapsed="!selectedLoader || !selectedGameVersion" overflow-visible>
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						isPaperLike
							? formatMessage(messages.buildNumberLabel)
							: formatMessage(messages.loaderVersionLabel)
					}}</span>
					<Chips
						v-if="!isPaperLike"
						v-model="loaderVersionType"
						:items="loaderVersionTypeItems"
						:disabled-items="loaderVersionTypeDisabledItems"
						:disabled-tooltip="'No such versions available'"
						:format-label="formatLoaderVersionTypeLabel"
					/>
					<div v-if="isPaperLike || loaderVersionType === 'other'">
						<Combobox
							v-model="selectedLoaderVersion"
							:options="loaderVersionOptions"
							:no-options-message="
								loaderVersionsLoading
									? formatMessage(commonMessages.loadingLabel)
									: formatMessage(messages.noVersionsAvailable)
							"
							searchable
							sync-with-selection
							:placeholder="
								isPaperLike
									? formatMessage(messages.selectBuildNumber)
									: formatMessage(messages.selectLoaderVersion)
							"
							:search-placeholder="
								isPaperLike
									? formatMessage(messages.searchBuildNumber)
									: formatMessage(messages.searchLoaderVersion)
							"
						>
							<!-- When not Paper, this scoped slot is omitted and Combobox uses default option markup. -->
							<template v-if="selectedLoader === 'paper'" #option="{ item, isSelected }">
								<div class="flex w-full items-center justify-between gap-2">
									<div class="flex flex-wrap items-center gap-2">
										<span
											class="font-semibold leading-tight"
											:class="isSelected ? 'text-contrast' : 'text-primary'"
										>
											{{ item.label }}
										</span>
										<PaperChannelBadge :channel="paperBuildChannelTag(String(item.value))" />
									</div>
								</div>
							</template>
							<template v-if="selectedLoader === 'paper'" #search-selection-affix="{ option }">
								<PaperChannelBadge
									affix
									:channel="option ? paperBuildChannelTag(String(option.value)) : null"
								/>
							</template>
						</Combobox>
					</div>
				</div>
			</Collapsible>
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Paper } from '@modrinth/api-client'
import { EyeIcon, EyeOffIcon, UploadIcon, XIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectFilePicker, injectModrinthClient, injectTags } from '../../../../providers'
import Avatar from '../../../base/Avatar.vue'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import PaperChannelBadge from '../../../base/PaperChannelBadge.vue'
import StyledInput from '../../../base/StyledInput.vue'
import type { LoaderVersionEntry, LoaderVersionType } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'
import { formatLoaderLabel } from '../shared'

const debug = useDebugLogger('CustomSetupStage')
const client = injectModrinthClient()
const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()
const {
	selectedLoader,
	selectedGameVersion,
	loaderVersionType,
	selectedLoaderVersion,
	hideLoaderChips,
	hideLoaderVersion,
} = ctx

const messages = defineMessages({
	selectIcon: {
		id: 'creation-flow.modal.custom-setup.icon.select',
		defaultMessage: 'Select icon',
	},
	removeIcon: {
		id: 'creation-flow.modal.custom-setup.icon.remove',
		defaultMessage: 'Remove icon',
	},
	nameLabel: {
		id: 'creation-flow.modal.custom-setup.name.label',
		defaultMessage: 'Name',
	},
	instanceNamePlaceholder: {
		id: 'creation-flow.modal.custom-setup.name.placeholder',
		defaultMessage: 'Enter instance name',
	},
	loaderLabel: {
		id: 'creation-flow.modal.custom-setup.loader.label',
		defaultMessage: 'Loader',
	},
	contentLoaderLabel: {
		id: 'creation-flow.modal.custom-setup.content-loader.label',
		defaultMessage: 'Content loader',
	},
	noVersionsAvailable: {
		id: 'creation-flow.modal.custom-setup.options.no-versions-available',
		defaultMessage: 'No versions available',
	},
	selectGameVersion: {
		id: 'creation-flow.modal.custom-setup.game-version.placeholder',
		defaultMessage: 'Select game version',
	},
	searchGameVersion: {
		id: 'creation-flow.modal.custom-setup.game-version.search-placeholder',
		defaultMessage: 'Search game version...',
	},
	buildNumberLabel: {
		id: 'creation-flow.modal.custom-setup.build-number.label',
		defaultMessage: 'Build number',
	},
	loaderVersionLabel: {
		id: 'creation-flow.modal.custom-setup.loader-version.label',
		defaultMessage: 'Loader version',
	},
	selectBuildNumber: {
		id: 'creation-flow.modal.custom-setup.build-number.placeholder',
		defaultMessage: 'Select build number',
	},
	selectLoaderVersion: {
		id: 'creation-flow.modal.custom-setup.loader-version.placeholder',
		defaultMessage: 'Select loader version',
	},
	searchBuildNumber: {
		id: 'creation-flow.modal.custom-setup.build-number.search-placeholder',
		defaultMessage: 'Search build number...',
	},
	searchLoaderVersion: {
		id: 'creation-flow.modal.custom-setup.loader-version.search-placeholder',
		defaultMessage: 'Search loader version...',
	},
	stableLoaderVersionType: {
		id: 'creation-flow.modal.custom-setup.loader-version-type.stable',
		defaultMessage: 'Stable',
	},
	latestLoaderVersionType: {
		id: 'creation-flow.modal.custom-setup.loader-version-type.latest',
		defaultMessage: 'Latest',
	},
	otherLoaderVersionType: {
		id: 'creation-flow.modal.custom-setup.loader-version-type.other',
		defaultMessage: 'Other',
	},
})

function formatLoaderVersionTypeLabel(type: LoaderVersionType): string {
	switch (type) {
		case 'stable':
			return formatMessage(messages.stableLoaderVersionType)
		case 'latest':
			return formatMessage(messages.latestLoaderVersionType)
		case 'other':
			return formatMessage(messages.otherLoaderVersionType)
	}
}

// For instance flow, prepend 'vanilla' to available loaders.
// For server flows, vanilla is a separate option in the setup type stage, so exclude it here.
const effectiveLoaders = computed(() => {
	if (ctx.flowType === 'instance') {
		return ['vanilla', ...ctx.availableLoaders.filter((l) => l !== 'vanilla')]
	}
	if (ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server') {
		return ctx.availableLoaders.filter((l) => l !== 'vanilla')
	}
	return ctx.availableLoaders
})

// Pre-select loader and game version from initial values
onMounted(() => {
	debug('mounted, initialLoader:', ctx.initialLoader, 'initialGameVersion:', ctx.initialGameVersion)
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
	debug('after init:', { loader: selectedLoader.value, gameVersion: selectedGameVersion.value })
})

const tags = injectTags()

const loaderVersionTypeItems: LoaderVersionType[] = ['stable', 'latest', 'other']

const loaderVersionTypeDisabledItems = computed<LoaderVersionType[]>(() => {
	const noStableVersions = !loaderVersionsData.value.some((v: LoaderVersionEntry) => v.stable)
	return noStableVersions ? ['stable'] : []
})

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

const loaderVersionsLoading = ref(false)
const loaderVersionsData = ref<LoaderVersionEntry[]>([])

// Paper/Purpur build caches
const paperVersions = ref<Record<string, Paper.Versions.v3.Build[]>>({})
const purpurVersions = ref<Record<string, string[]>>({})

function toApiLoaderName(loader: string): string {
	return loader === 'neoforge' ? 'neo' : loader
}

const gameVersionsLoading = computed(() => {
	const loader = selectedLoader.value
	if (!loader || loader === 'vanilla') return false
	if (loader === 'paper') return ctx.paperSupportedVersions.value === null
	if (loader === 'purpur') return ctx.purpurSupportedVersions.value === null
	return ctx.loaderVersionsCache.value[toApiLoaderName(loader)] === undefined
})

// Game versions from tags provider, filtered by loader support
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')

	// For loaders with per-version data, only show game versions that have builds
	if (selectedLoader.value && selectedLoader.value !== 'vanilla') {
		if (selectedLoader.value === 'paper') {
			if (!ctx.paperSupportedVersions.value) return []
			return versions
				.filter((v) => ctx.paperSupportedVersions.value!.has(v.version))
				.map((v) => ({ value: v.version, label: v.version }))
		}

		if (selectedLoader.value === 'purpur') {
			if (!ctx.purpurSupportedVersions.value) return []
			return versions
				.filter((v) => ctx.purpurSupportedVersions.value!.has(v.version))
				.map((v) => ({ value: v.version, label: v.version }))
		}

		const apiLoader = toApiLoaderName(selectedLoader.value)
		const manifest = ctx.loaderVersionsCache.value[apiLoader]
		if (!manifest) return []

		const hasPlaceholder = manifest.some((x) => x.id === '${modrinth.gameVersion}')
		const supportedVersions = new Set(
			manifest
				.filter(
					(x) => x.id !== '${modrinth.gameVersion}' && (hasPlaceholder || x.loaders.length > 0),
				)
				.map((x) => x.id),
		)
		return versions
			.filter((v) => supportedVersions.has(v.version))
			.map((v) => ({ value: v.version, label: v.version }))
	}

	return versions.map((v) => ({ value: v.version, label: v.version }))
})

// Auto-select latest game version when options change and current selection is missing or invalid
watch(
	gameVersionOptions,
	(options) => {
		if (options.length === 0) {
			selectedGameVersion.value = null
			return
		}
		if (!selectedGameVersion.value || !options.some((o) => o.value === selectedGameVersion.value)) {
			selectedGameVersion.value = options[0].value
		}
	},
	{ immediate: true },
)

async function fetchLoaderManifest(loader: string) {
	const apiLoader = toApiLoaderName(loader)
	debug(
		'fetchLoaderManifest:',
		loader,
		'apiLoader:',
		apiLoader,
		'cached:',
		!!ctx.loaderVersionsCache.value[apiLoader],
	)
	await ctx.fetchLoaderMetadata(loader)
}

async function fetchLoaderMetadata(loader?: string | null) {
	await ctx.fetchLoaderMetadata(loader)
}

function paperBuildChannelTag(buildId: string): 'ALPHA' | 'BETA' | null {
	const gv = selectedGameVersion.value
	if (!gv || selectedLoader.value !== 'paper') return null
	const b = paperVersions.value[gv]?.find((x) => String(x.id) === buildId)
	if (!b) return null
	const u = String(b.channel).toUpperCase()
	if (u === 'ALPHA' || u === 'BETA') return u
	return null
}

async function fetchPaperVersions(mcVersion: string) {
	if (paperVersions.value[mcVersion]) return
	try {
		const data = await client.paper.versions_v3.getBuilds(mcVersion)
		paperVersions.value[mcVersion] = data.builds.toSorted((a, b) => b.id - a.id)
	} catch {
		paperVersions.value[mcVersion] = []
	}
}

function handleGameVersionHover(option: ComboboxOption<string | null>) {
	const v = option.value
	if (v == null || v === '') return
	if (selectedLoader.value === 'paper') void fetchPaperVersions(v)
	else if (selectedLoader.value === 'purpur') void fetchPurpurVersions(v)
}

async function fetchPurpurVersions(mcVersion: string) {
	if (purpurVersions.value[mcVersion]) return
	try {
		const data = await client.purpur.versions_v2.getBuilds(mcVersion)
		purpurVersions.value[mcVersion] = data.builds.all.sort((a, b) => parseInt(b) - parseInt(a))
	} catch {
		purpurVersions.value[mcVersion] = []
	}
}

function getLoaderVersionsForGameVersion(
	loader: string,
	gameVersion: string,
): LoaderVersionEntry[] {
	const apiLoader = toApiLoaderName(loader)
	const manifest = ctx.loaderVersionsCache.value[apiLoader]
	debug('getLoaderVersionsForGameVersion:', {
		loader,
		apiLoader,
		gameVersion,
		hasManifest: !!manifest,
		manifestLength: manifest?.length,
	})
	if (!manifest) return []

	// Some loaders (e.g. Fabric) list all versions under a placeholder entry
	const placeholder = manifest.find((x) => x.id === '${modrinth.gameVersion}')
	if (placeholder) {
		if (!manifest.some((x) => x.id === gameVersion)) return []
		debug(
			'getLoaderVersionsForGameVersion: using placeholder, loaders:',
			placeholder.loaders.length,
		)
		return placeholder.loaders
	}

	const entry = manifest.find((x) => x.id === gameVersion)
	debug(
		'getLoaderVersionsForGameVersion: entry for',
		gameVersion,
		':',
		entry ? entry.loaders.length + ' loaders' : 'NOT FOUND',
	)
	return entry?.loaders ?? []
}

// Fetch version data when loader changes so game versions can be filtered
watch(
	() => selectedLoader.value,
	async (loader) => {
		await fetchLoaderMetadata(loader)
	},
	{ immediate: true },
)

// Watch loader + game version to resolve loader versions
let loaderVersionWatchId = 0
watch(
	[() => selectedLoader.value, () => selectedGameVersion.value],
	async ([loader, gameVersion]) => {
		const watchId = ++loaderVersionWatchId
		debug('watch [loader, gameVersion] fired:', { loader, gameVersion, watchId })
		loaderVersionsData.value = []
		selectedLoaderVersion.value = null

		if (!loader || !gameVersion || loader === 'vanilla') return

		loaderVersionsLoading.value = true

		if (loader === 'paper') {
			await fetchPaperVersions(gameVersion)
			if (watchId !== loaderVersionWatchId) return
			loaderVersionsLoading.value = false
			const builds = paperVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = `${builds[0].id}`
			}
			return
		}

		if (loader === 'purpur') {
			await fetchPurpurVersions(gameVersion)
			if (watchId !== loaderVersionWatchId) return
			loaderVersionsLoading.value = false
			const builds = purpurVersions.value[gameVersion]
			if (builds?.length) {
				selectedLoaderVersion.value = builds[0]
			}
			return
		}

		await fetchLoaderManifest(loader)
		if (watchId !== loaderVersionWatchId) {
			debug('watch [loader, gameVersion]: stale execution, skipping', {
				watchId,
				current: loaderVersionWatchId,
			})
			return
		}
		loaderVersionsData.value = getLoaderVersionsForGameVersion(loader, gameVersion)
		debug(
			'watch [loader, gameVersion]: loaderVersionsData set, count:',
			loaderVersionsData.value.length,
		)
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
	debug(
		'autoSelectLoaderVersion: type:',
		loaderVersionType.value,
		'dataCount:',
		loaderVersionsData.value.length,
		'stableCount:',
		loaderVersionsData.value.filter((v) => v.stable).length,
		'first:',
		loaderVersionsData.value[0]?.id,
	)
	if (
		loaderVersionType.value === 'stable' &&
		loaderVersionTypeDisabledItems.value.includes('stable')
	) {
		debug("'stable' loader version type is disabled, switching to 'latest'...")
		loaderVersionType.value = 'latest'
	}
	if (loaderVersionType.value === 'stable') {
		const stable = loaderVersionsData.value.find((v) => v.stable)
		selectedLoaderVersion.value = stable?.id ?? loaderVersionsData.value[0]?.id ?? null
	} else if (loaderVersionType.value === 'latest') {
		selectedLoaderVersion.value = loaderVersionsData.value[0]?.id ?? null
	} else if (loaderVersionType.value === 'other' && !selectedLoaderVersion.value) {
		selectedLoaderVersion.value = loaderVersionsData.value[0]?.id ?? null
	}
	debug('autoSelectLoaderVersion: result:', selectedLoaderVersion.value)
}

const loaderVersionOptions = computed<ComboboxOption<string>[]>(() => {
	if (selectedLoader.value === 'paper' && selectedGameVersion.value) {
		const builds = paperVersions.value[selectedGameVersion.value] ?? []
		return builds.map((b) => ({
			value: `${b.id}`,
			label: `Build ${b.id}`,
		}))
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
