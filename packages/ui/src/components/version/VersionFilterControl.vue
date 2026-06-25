<template>
	<div class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center gap-2">
			<ManySelect
				v-model="selectedPlatforms"
				:options="filterOptions.platform"
				:dropdown-id="`${baseId}-platform`"
				@change="updateFilters"
			>
				<FilterIcon class="h-5 w-5 text-secondary" />
				Platform
				<template #option="{ option }">
					<FormattedTag :tag="option" enforce-type="loader" />
				</template>
			</ManySelect>
			<MultiSelect
				v-if="filterOptions.gameVersion.length > 1"
				:model-value="selectedGameVersions"
				:options="gameVersionOptions"
				searchable
				search-placeholder="Search..."
				fit-content
				:dropdown-min-width="240"
				trigger-class="!min-h-9 !px-3 !py-0"
				@update:model-value="updateSelectedGameVersions"
			>
				<template #input-content="{ isOpen, openDirection }">
					<div class="flex items-center gap-2">
						<FilterIcon class="h-5 w-5 text-secondary" />
						<span class="font-semibold text-primary">Game versions</span>
						<ChevronLeftIcon
							class="h-5 w-5 text-secondary transition-transform duration-150"
							:class="
								isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
							"
						/>
					</div>
				</template>
				<template #bottom>
					<div class="border-0 border-t border-solid border-t-surface-5 px-3 py-3">
						<Checkbox v-model="showSnapshots" class="mx-1" :label="`Show all versions`" />
					</div>
				</template>
			</MultiSelect>
			<MultiSelect
				v-if="filterOptions.channel.length > 1"
				:model-value="selectedChannels"
				:options="channelOptions"
				fit-content
				:dropdown-min-width="180"
				trigger-class="!min-h-9 !px-3 !py-0"
				@update:model-value="updateSelectedChannels"
			>
				<template #input-content="{ isOpen, openDirection }">
					<div class="flex items-center gap-2">
						<FilterIcon class="h-5 w-5 text-secondary" />
						<span class="font-semibold text-primary">Channels</span>
						<ChevronLeftIcon
							class="h-5 w-5 text-secondary transition-transform duration-150"
							:class="
								isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
							"
						/>
					</div>
				</template>
			</MultiSelect>
		</div>
		<div class="flex flex-wrap items-center gap-1 empty:hidden">
			<TagItem
				v-if="selectedChannels.length + selectedGameVersions.length + selectedPlatforms.length > 1"
				class="transition-transform active:scale-[0.95]"
				:action="clearFilters"
			>
				<XCircleIcon />
				Clear all filters
			</TagItem>
			<TagItem
				v-for="channel in selectedChannels"
				:key="`remove-filter-${channel}`"
				:style="`--_color: var(--color-${channel === 'alpha' ? 'red' : channel === 'beta' ? 'orange' : 'green'});--_bg-color: var(--color-${channel === 'alpha' ? 'red' : channel === 'beta' ? 'orange' : 'green'}-highlight)`"
				:action="() => toggleFilter('channel', channel)"
			>
				<XIcon />
				{{ channel.slice(0, 1).toUpperCase() + channel.slice(1) }}
			</TagItem>
			<TagItem
				v-for="version in selectedGameVersions"
				:key="`remove-filter-${version}`"
				:action="() => toggleFilter('gameVersion', version)"
			>
				<XIcon />
				{{ version }}
			</TagItem>
			<TagItem
				v-for="platform in selectedPlatforms"
				:key="`remove-filter-${platform}`"
				:style="`--_color: var(--color-platform-${platform})`"
				:action="() => toggleFilter('platform', platform)"
			>
				<XIcon />
				<FormattedTag :tag="platform" enforce-type="loader" />
			</TagItem>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ChevronLeftIcon, FilterIcon, XCircleIcon, XIcon } from '@modrinth/assets'
import type { MultiSelectOption } from '@modrinth/ui'
import { Checkbox, FormattedTag, ManySelect, MultiSelect, TagItem } from '@modrinth/ui'
import type { GameVersionTag, Version } from '@modrinth/utils'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

const props = defineProps<{
	versions: Version[]
	gameVersions: GameVersionTag[]
	baseId?: string
}>()

const emit = defineEmits(['update:query'])

const allChannels = ref(['release', 'beta', 'alpha'])

const route = useRoute()

const showSnapshots = ref(false)

type FilterType = 'channel' | 'gameVersion' | 'platform'
type Filter = string

const filterOptions = computed(() => {
	const filters: Record<FilterType, Filter[]> = {
		channel: [],
		gameVersion: [],
		platform: [],
	}

	const platformSet = new Set<Filter>()
	const gameVersionSet = new Set<Filter>()
	const channelSet = new Set<Filter>()

	for (const version of props.versions) {
		for (const loader of Array.isArray(version.loaders) ? version.loaders : []) {
			platformSet.add(loader)
		}
		for (const gameVersion of Array.isArray(version.game_versions) ? version.game_versions : []) {
			gameVersionSet.add(gameVersion)
		}
		channelSet.add(version.version_type)
	}

	if (channelSet.size > 0) {
		filters.channel = Array.from(channelSet) as Filter[]
		filters.channel.sort((a, b) => allChannels.value.indexOf(a) - allChannels.value.indexOf(b))
	}
	if (gameVersionSet.size > 0) {
		const gameVersions = props.gameVersions.filter((x) => gameVersionSet.has(x.version))

		filters.gameVersion = gameVersions
			.filter((x) => (showSnapshots.value ? true : x.version_type === 'release'))
			.map((x) => x.version)
	}
	if (platformSet.size > 0) {
		filters.platform = Array.from(platformSet) as Filter[]
	}

	return filters
})

const gameVersionOptions = computed<MultiSelectOption<string>[]>(() =>
	filterOptions.value.gameVersion.map((version) => ({
		value: version,
		label: version,
	})),
)

const channelOptions = computed<MultiSelectOption<string>[]>(() =>
	filterOptions.value.channel.map((channel) => ({
		value: channel,
		label: getChannelLabel(channel),
	})),
)

const selectedChannels = ref<string[]>([])
const selectedGameVersions = ref<string[]>([])
const selectedPlatforms = ref<string[]>([])

selectedChannels.value = route.query.c ? getArrayOrString(route.query.c) : []
selectedGameVersions.value = route.query.g ? getArrayOrString(route.query.g) : []
selectedPlatforms.value = route.query.l ? getArrayOrString(route.query.l) : []

async function toggleFilters(type: FilterType, filters: Filter[]) {
	for (const filter of filters) {
		await toggleFilter(type, filter, true)
	}

	updateFilters()
}

async function toggleFilter(type: FilterType, filter: Filter, bulk = false) {
	if (type === 'channel') {
		selectedChannels.value = selectedChannels.value.includes(filter)
			? selectedChannels.value.filter((x) => x !== filter)
			: [...selectedChannels.value, filter]
	} else if (type === 'gameVersion') {
		selectedGameVersions.value = selectedGameVersions.value.includes(filter)
			? selectedGameVersions.value.filter((x) => x !== filter)
			: [...selectedGameVersions.value, filter]
	} else if (type === 'platform') {
		selectedPlatforms.value = selectedPlatforms.value.includes(filter)
			? selectedPlatforms.value.filter((x) => x !== filter)
			: [...selectedPlatforms.value, filter]
	}
	if (!bulk) {
		updateFilters()
	}
}

function updateSelectedGameVersions(versions: string[]) {
	selectedGameVersions.value = versions
	updateFilters()
}

function updateSelectedChannels(channels: string[]) {
	selectedChannels.value = channels
	updateFilters()
}

async function clearFilters() {
	selectedChannels.value = []
	selectedGameVersions.value = []
	selectedPlatforms.value = []

	updateFilters()
}

function updateFilters() {
	emit('update:query', {
		c: selectedChannels.value,
		g: selectedGameVersions.value,
		l: selectedPlatforms.value,
		page: undefined,
	})
}

defineExpose({
	toggleFilter,
	toggleFilters,
	selectedChannels,
	selectedGameVersions,
	selectedPlatforms,
})

function getArrayOrString(x: string | string[]): string[] {
	if (typeof x === 'string') {
		return [x]
	} else {
		return x
	}
}

function getChannelLabel(channel: string) {
	return channel === 'release' ? 'Release' : channel === 'beta' ? 'Beta' : 'Alpha'
}
</script>
