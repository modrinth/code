<template>
	<div
		v-if="visibleDropdowns.length > 0"
		class="flex min-w-0 flex-1 flex-wrap items-center gap-1.5"
	>
		<FilterIcon class="size-5 shrink-0 text-secondary" aria-hidden="true" />
		<MultiSelect
			v-for="dropdown in visibleDropdowns"
			:key="dropdown.key"
			class="min-w-0 max-w-full"
			:model-value="selectedFilters[dropdown.key]"
			:options="dropdown.options"
			:searchable="dropdown.searchable"
			:search-placeholder="dropdown.searchPlaceholder"
			:max-height="500"
			:clearable="false"
			:show-chevron="false"
			fit-content
			trigger-type="base"
			trigger-size="lg"
			:trigger-class="getDropdownTriggerClass(dropdown.key)"
			:dropdown-min-width="dropdown.dropdownMinWidth"
			checkbox-position="right"
			show-selection-actions
			@update:model-value="(values) => updateSelected(dropdown.key, values)"
		>
			<template #input-content="{ isOpen, openDirection }">
				<div class="flex min-h-8 min-w-0 max-w-full items-center gap-2 sm:max-w-80">
					<span class="min-w-0 flex-1 truncate">
						<template v-if="selectedFilters[dropdown.key].length > 0">
							<span class="font-medium">{{ dropdown.label }}:</span>
							<span class="ml-1 font-semibold text-contrast">{{
								getDropdownSummary(dropdown)
							}}</span>
						</template>
						<span v-else class="font-semibold text-inherit">{{ dropdown.label }}</span>
					</span>
					<div class="flex shrink-0 items-center gap-1.5">
						<button
							v-if="selectedFilters[dropdown.key].length > 0"
							type="button"
							class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
							:aria-label="formatMessage(messages.clearNamedFilter, { name: dropdown.label })"
							@click.stop="clearCategory(dropdown.key)"
						>
							<XIcon class="size-4 text-primary" />
						</button>
						<div
							v-if="selectedFilters[dropdown.key].length > 0"
							class="h-5 w-[1px] shrink-0 bg-surface-5"
						></div>
						<ChevronLeftIcon
							class="size-5 shrink-0 text-secondary transition-transform duration-150"
							:class="
								isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'
							"
						/>
					</div>
				</div>
			</template>
			<template v-if="dropdown.key === 'gameVersion' && hasAnyNonReleaseGameVersions" #bottom>
				<div class="border-0 border-t border-solid border-t-surface-5 px-3 py-3">
					<Checkbox
						:model-value="showSnapshots"
						:label="formatMessage(commonMessages.showAllVersionsButton)"
						@update:model-value="updateShowSnapshots"
					/>
				</div>
			</template>
		</MultiSelect>
		<Button v-if="hasSelectedFilters" type="quiet" native-type="button" @click="clearAllFilters">
			{{ formatMessage(commonMessages.clearButton) }}
		</Button>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ChevronLeftIcon, FilterIcon, XIcon } from '@modrinth/assets'
import type { GameVersionTag } from '@modrinth/utils'
import { computed, ref } from 'vue'
import type { LocationQueryValue } from 'vue-router'
import { useRoute } from 'vue-router'

import { Button } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import MultiSelect, { type MultiSelectOption } from '#ui/components/base/MultiSelect.vue'
import {
	ENVIRONMENT_FILTER_VALUES,
	type EnvironmentFilterValue,
	getEnvironmentFilterValue,
} from '#ui/components/project/settings/environment/environments'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, commonProjectSettingsMessages } from '#ui/utils/common-messages'
import { formatLoader } from '#ui/utils/tag-messages'

const props = defineProps<{
	versions: Labrinth.Versions.v3.Version[]
	gameVersions: GameVersionTag[]
	baseId?: string
}>()

const emit = defineEmits(['update:query'])

const { formatMessage } = useVIntl()
const route = useRoute()

const CHANNEL_ORDER = ['release', 'beta', 'alpha'] as const

const messages = defineMessages({
	channel: {
		id: 'project.versions.filter.channel',
		defaultMessage: 'Channel',
	},
	clientSideOnly: {
		id: 'project.settings.environment.client_only.title',
		defaultMessage: 'Client-side only',
	},
	serverSideOnly: {
		id: 'project.settings.environment.server_only.title',
		defaultMessage: 'Server-side only',
	},
	clientAndServer: {
		id: 'project.settings.environment.client_and_server.title',
		defaultMessage: 'Client and server',
	},
	singleplayerOnly: {
		id: 'project.settings.environment.singleplayer.title',
		defaultMessage: 'Singleplayer only',
	},
	clearNamedFilter: {
		id: 'filter-bar.clear-named-filter',
		defaultMessage: 'Clear {name} filter',
	},
	selectedCount: {
		id: 'project.versions.filter.selected-count',
		defaultMessage: '{count, number} selected',
	},
})

const filterTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-surface-5 !bg-transparent !px-3 !text-sm !font-medium !text-primary !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-surface-3 focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5'
const filterPreviewTriggerClass =
	'!h-[34px] !rounded-xl !border !border-solid !border-brand !bg-brand-highlight !px-3 !text-sm !font-medium !text-brand !shadow-[0_1px_1.5px_rgba(0,0,0,0.15)] transition-all duration-100 active:scale-[0.97] hover:!bg-brand-highlight focus-visible:!outline-none focus-visible:!ring-4 focus-visible:!ring-brand-shadow [&>svg]:!size-5 [&>svg]:!text-brand'

type FilterType = 'channel' | 'gameVersion' | 'platform' | 'environment'
type Filter = string

type FilterDropdown = {
	key: FilterType
	label: string
	options: MultiSelectOption<string>[]
	searchable?: boolean
	searchPlaceholder?: string
	dropdownMinWidth: number
}

const showSnapshots = ref(false)

const selectedFilters = ref<Record<FilterType, string[]>>({
	channel: route.query.c ? getArrayOrString(route.query.c) : [],
	gameVersion: route.query.g ? getArrayOrString(route.query.g) : [],
	platform: route.query.l ? getArrayOrString(route.query.l) : [],
	environment: route.query.e ? getArrayOrString(route.query.e) : [],
})

const selectedChannels = computed(() => selectedFilters.value.channel)
const selectedGameVersions = computed(() => selectedFilters.value.gameVersion)
const selectedPlatforms = computed(() => selectedFilters.value.platform)
const selectedEnvironments = computed(() => selectedFilters.value.environment)

const hasSelectedFilters = computed(() =>
	Object.values(selectedFilters.value).some((values) => values.length > 0),
)

const gameVersionTags = computed(() => new Map(props.gameVersions.map((x) => [x.version, x])))

const availableGameVersions = computed(() => {
	const gameVersionSet = new Set<Filter>()

	for (const version of props.versions) {
		for (const gameVersion of Array.isArray(version.game_versions) ? version.game_versions : []) {
			gameVersionSet.add(gameVersion)
		}
	}

	const knownGameVersions = props.gameVersions.filter((x) => gameVersionSet.has(x.version))
	const knownGameVersionSet = new Set(knownGameVersions.map((x) => x.version))
	const unknownGameVersions = Array.from(gameVersionSet).filter(
		(version) => !knownGameVersionSet.has(version),
	)

	return [...knownGameVersions.map((x) => x.version), ...unknownGameVersions]
})

const hasAnyReleaseGameVersions = computed(() =>
	availableGameVersions.value.some((version) => isReleaseGameVersion(version)),
)

const hasAnyNonReleaseGameVersions = computed(() =>
	availableGameVersions.value.some((version) => !isReleaseGameVersion(version)),
)

const availableChannels = computed(() => {
	const channelSet = new Set<Filter>()
	for (const version of props.versions) {
		channelSet.add(version.version_type)
	}

	const knownChannels = CHANNEL_ORDER.filter((channel) => channelSet.has(channel))
	const unknownChannels = [...channelSet].filter(
		(channel) => !(CHANNEL_ORDER as readonly string[]).includes(channel),
	)
	return [...knownChannels, ...unknownChannels]
})

const availablePlatforms = computed(() => {
	const platformSet = new Set<Filter>()
	for (const version of props.versions) {
		for (const loader of Array.isArray(version.loaders) ? version.loaders : []) {
			platformSet.add(loader)
		}
	}
	return Array.from(platformSet)
})

const availableEnvironments = computed(() => {
	const environmentSet = new Set<EnvironmentFilterValue>()
	for (const version of props.versions) {
		const environment = getEnvironmentFilterValue(version.environment)
		if (environment) {
			environmentSet.add(environment)
		}
	}
	return ENVIRONMENT_FILTER_VALUES.filter((environment) => environmentSet.has(environment))
})

const visibleGameVersions = computed(() =>
	availableGameVersions.value.filter(
		(version) =>
			showSnapshots.value || !hasAnyReleaseGameVersions.value || isReleaseGameVersion(version),
	),
)

if (selectedGameVersions.value.some((version) => !isReleaseGameVersion(version))) {
	showSnapshots.value = true
}

const visibleDropdowns = computed<FilterDropdown[]>(() => {
	const dropdowns: FilterDropdown[] = []

	if (shouldShowCategory('channel', availableChannels.value)) {
		dropdowns.push({
			key: 'channel',
			label: formatMessage(messages.channel),
			dropdownMinWidth: 180,
			options: availableChannels.value.map((channel) => ({
				value: channel,
				label: getChannelLabel(channel),
			})),
		})
	}

	if (shouldShowCategory('gameVersion', availableGameVersions.value)) {
		dropdowns.push({
			key: 'gameVersion',
			label: formatMessage(commonMessages.gameVersionLabel),
			searchable: true,
			searchPlaceholder: formatMessage(commonMessages.searchVersionPlaceholder),
			dropdownMinWidth: 240,
			options: visibleGameVersions.value.map((version) => ({
				value: version,
				label: version,
			})),
		})
	}

	if (shouldShowCategory('platform', availablePlatforms.value)) {
		dropdowns.push({
			key: 'platform',
			label: formatMessage(commonMessages.platformLabel),
			dropdownMinWidth: 180,
			options: availablePlatforms.value.map((platform) => ({
				value: platform,
				label: formatLoader(formatMessage, platform),
			})),
		})
	}

	if (shouldShowCategory('environment', availableEnvironments.value)) {
		dropdowns.push({
			key: 'environment',
			label: formatMessage(commonProjectSettingsMessages.environment),
			dropdownMinWidth: 220,
			options: availableEnvironments.value.map((environment) => ({
				value: environment,
				label: getEnvironmentFilterLabel(environment),
			})),
		})
	}

	return dropdowns
})

function shouldShowCategory(type: FilterType, options: string[]) {
	return options.length > 1 || (selectedFilters.value[type]?.length ?? 0) > 0
}

function getDropdownTriggerClass(type: FilterType) {
	return selectedFilters.value[type].length > 0 ? filterPreviewTriggerClass : filterTriggerClass
}

function getDropdownSummary(dropdown: FilterDropdown) {
	const selected = selectedFilters.value[dropdown.key]
	if (selected.length === 0) {
		return ''
	}
	if (selected.length === 1) {
		return dropdown.options.find((option) => option.value === selected[0])?.label ?? selected[0]
	}
	return formatMessage(messages.selectedCount, { count: selected.length })
}

function updateSelected(type: FilterType, values: string[]) {
	selectedFilters.value = {
		...selectedFilters.value,
		[type]: values,
	}
	updateFilters()
}

function clearCategory(type: FilterType) {
	updateSelected(type, [])
}

function clearAllFilters() {
	selectedFilters.value = {
		channel: [],
		gameVersion: [],
		platform: [],
		environment: [],
	}
	updateFilters()
}

function selectedFiltersOfType(type: FilterType) {
	return selectedFilters.value[type] ?? []
}

function toggleFilters(type: FilterType, filters: Filter[]) {
	const selected = selectedFiltersOfType(type)
	const allSelected = filters.every((filter) => selected.includes(filter))

	selectedFilters.value = {
		...selectedFilters.value,
		[type]: allSelected
			? selected.filter((x) => !filters.includes(x))
			: [...selected, ...filters.filter((filter) => !selected.includes(filter))],
	}

	updateFilters()
}

function toggleFilter(type: FilterType, filter: Filter) {
	const selected = selectedFiltersOfType(type)

	selectedFilters.value = {
		...selectedFilters.value,
		[type]: selected.includes(filter)
			? selected.filter((x) => x !== filter)
			: [...selected, filter],
	}

	updateFilters()
}

function updateShowSnapshots(value: boolean) {
	showSnapshots.value = value

	if (value || !hasAnyReleaseGameVersions.value) {
		return
	}

	const selectedReleaseGameVersions = selectedGameVersions.value.filter((version) =>
		isReleaseGameVersion(version),
	)

	if (selectedReleaseGameVersions.length !== selectedGameVersions.value.length) {
		selectedFilters.value = {
			...selectedFilters.value,
			gameVersion: selectedReleaseGameVersions,
		}
		updateFilters()
	}
}

function updateFilters() {
	emit('update:query', {
		c: selectedChannels.value,
		g: selectedGameVersions.value,
		l: selectedPlatforms.value,
		e: selectedEnvironments.value,
		page: undefined,
	})
}

defineExpose({
	toggleFilter,
	toggleFilters,
	selectedChannels,
	selectedGameVersions,
	selectedPlatforms,
	selectedEnvironments,
})

function getArrayOrString(x: LocationQueryValue | LocationQueryValue[]): string[] {
	if (x === null) {
		return []
	}
	if (typeof x === 'string') {
		return [x]
	}

	return x.filter((value): value is string => value !== null)
}

function getChannelLabel(channel: string) {
	if (channel === 'release') {
		return formatMessage(commonMessages.release)
	}
	if (channel === 'beta') {
		return formatMessage(commonMessages.beta)
	}
	if (channel === 'alpha') {
		return formatMessage(commonMessages.alpha)
	}
	return channel.slice(0, 1).toUpperCase() + channel.slice(1)
}

function getEnvironmentFilterLabel(environment: EnvironmentFilterValue) {
	switch (environment) {
		case 'client':
			return formatMessage(messages.clientSideOnly)
		case 'server':
			return formatMessage(messages.serverSideOnly)
		case 'client_and_server':
			return formatMessage(messages.clientAndServer)
		case 'singleplayer':
			return formatMessage(messages.singleplayerOnly)
	}
}

function isReleaseGameVersion(version: string) {
	return gameVersionTags.value.get(version)?.version_type === 'release'
}
</script>
