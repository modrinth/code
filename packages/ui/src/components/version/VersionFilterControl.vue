<template>
	<ListToolbar
		v-model:search="search"
		:filters="selectedFilters"
		:filter-categories="filterCategories"
		:search-placeholder="searchPlaceholder"
		:page="page"
		:page-count="pageCount"
		:page-link="pageLink"
		@update:filters="setSelectedFilters"
		@switch-page="(nextPage) => emit('switchPage', nextPage)"
	>
		<template v-if="$slots.controls" #controls>
			<slot name="controls" />
		</template>
		<template v-if="$slots.actions" #actions>
			<slot name="actions" />
		</template>
		<template #filter-footer="{ category }">
			<div
				v-if="category?.key === 'gameVersion' && hasAnyNonReleaseGameVersions"
				class="border-0 border-t border-solid border-t-surface-5 px-3 py-3"
			>
				<Checkbox
					:model-value="showSnapshots"
					:label="formatMessage(commonMessages.showAllVersionsButton)"
					@update:model-value="updateShowSnapshots"
				/>
			</div>
		</template>
	</ListToolbar>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import type { GameVersionTag } from '@modrinth/utils'
import { computed, ref } from 'vue'
import type { LocationQueryValue } from 'vue-router'
import { useRoute } from 'vue-router'

import Checkbox from '#ui/components/base/Checkbox.vue'
import type { DropdownFilterBarCategory } from '#ui/components/base/DropdownFilterBar.vue'
import ListToolbar, { type ListToolbarValue } from '#ui/components/base/ListToolbar.vue'
import {
	ENVIRONMENT_FILTER_VALUES,
	type EnvironmentFilterValue,
	getEnvironmentFilterValue,
} from '#ui/components/project/settings/environment/environments'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, commonProjectSettingsMessages } from '#ui/utils/common-messages'
import { formatLoader } from '#ui/utils/tag-messages'

const props = withDefaults(
	defineProps<{
		versions: Labrinth.Versions.v3.Version[]
		gameVersions: GameVersionTag[]
		searchPlaceholder?: string
		page?: number
		pageCount?: number
		pageLink?: (page: number) => string | undefined
	}>(),
	{
		searchPlaceholder: undefined,
		page: 1,
		pageCount: 0,
		pageLink: undefined,
	},
)

const search = defineModel<string>('search')

const emit = defineEmits<{
	'update:query': [query: Record<string, string[] | undefined>]
	switchPage: [page: number]
}>()

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
})

type FilterType = 'channel' | 'gameVersion' | 'platform' | 'environment'
type Filter = string

type FilterDropdown = DropdownFilterBarCategory & { key: FilterType }

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

const filterCategories = computed<FilterDropdown[]>(() => {
	const dropdowns: FilterDropdown[] = []

	if (shouldShowCategory('channel', availableChannels.value)) {
		dropdowns.push({
			key: 'channel',
			label: formatMessage(messages.channel),
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

function setSelectedFilters(next: ListToolbarValue) {
	selectedFilters.value = {
		channel: next.channel ?? [],
		gameVersion: next.gameVersion ?? [],
		platform: next.platform ?? [],
		environment: next.environment ?? [],
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
