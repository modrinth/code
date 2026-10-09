<script setup lang="ts">
import { InfoIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref, toValue, useTemplateRef, watch } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import Toggle from '#ui/components/base/Toggle.vue'
import PhotosensitivityWarningModal from '#ui/components/modal/PhotosensitivityWarningModal.vue'
import SearchSidebarFilter from '#ui/components/search/SearchSidebarFilter.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useAdvancedPrefs } from '#ui/utils/advanced-filter-preferences'
import { commonMessages } from '#ui/utils/common-messages'
import {
	type FilterType,
	type FilterValue,
	LOADER_FILTER_TYPES,
	parseServerContentEnvironment,
	SERVER_CONTENT_ENVIRONMENTS,
	type ServerContentEnvironment,
} from '#ui/utils/search'

import AdvancedFiltersPersistenceNote from './components/AdvancedFiltersPersistenceNote.vue'
import { injectBrowseManager } from './providers/browse-manager'

const PHOTOSENSITIVITY_FILTER_OPTION = 'epilepsy_triggers'

const ctx = injectBrowseManager()
const { formatMessage } = useVIntl()
const advancedPrefs = useAdvancedPrefs()

const messages = defineMessages({
	serverEnvironment: {
		id: 'search.server-environment.title',
		defaultMessage: 'Environment',
	},
	playerAndServer: {
		id: 'search.server-environment.player-and-server',
		defaultMessage: 'Player and server',
	},
	playerOnly: {
		id: 'search.server-environment.player-only',
		defaultMessage: 'Player only',
	},
})

const isApp = computed(() => ctx.variant === 'app')

const SERVER_ENVIRONMENT_FILTER_ID = 'server_environment'

function getServerEnvironmentLabel(environment: ServerContentEnvironment) {
	switch (environment) {
		case 'server_and_player':
			return formatMessage(messages.playerAndServer)
		case 'server':
			return formatMessage(commonMessages.serverOnlyLabel)
		case 'player':
			return formatMessage(messages.playerOnly)
	}
}

const serverEnvironmentFilterType = computed<FilterType>(() => ({
	id: SERVER_ENVIRONMENT_FILTER_ID,
	formatted_name: formatMessage(messages.serverEnvironment),
	options: SERVER_CONTENT_ENVIRONMENTS.map((environment) => ({
		id: environment,
		formatted_name: getServerEnvironmentLabel(environment),
		method: 'or',
		value: environment,
	})),
	supported_project_types: [],
	query_param: '',
	supports: ['include'],
	searchable: false,
	display: 'all',
}))

const serverEnvironmentToggledGroups = ref<string[]>([])

/** Bridges the single-select server environment onto the multi-select sidebar filter model. */
const serverEnvironmentSelection = computed<FilterValue[]>({
	get: () =>
		ctx.serverEnvironment?.value
			? [
					{
						type: SERVER_ENVIRONMENT_FILTER_ID,
						option: ctx.serverEnvironment.value,
						negative: false,
					},
				]
			: [],
	set: (filters) => {
		if (!ctx.serverEnvironment) return
		ctx.serverEnvironment.value = parseServerContentEnvironment(filters.at(-1)?.option)
		ctx.onFilterChange()
	},
})
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))
const hiddenFilterTypes = computed(() => ctx.hiddenFilterTypes?.value ?? [])
const visibleFilters = computed(() => {
	const filters = ctx.filters.value.filter(
		(filter) => filter.display !== 'none' && !hiddenFilterTypes.value.includes(filter.id),
	)
	if (!isApp.value) return filters

	const priority = (id: string) =>
		id === 'game_version' ? 2 : LOADER_FILTER_TYPES.some((type) => type === id) ? 1 : 0
	return filters.sort((a, b) => priority(b.id) - priority(a.id))
})

const advancedFiltersCollapsed = computed(() => ctx.advancedFiltersCollapsed?.value ?? true)
const photosensitivityWarningModal = useTemplateRef('photosensitivityWarningModal')

function setAdvancedFiltersCollapsed(collapsed: boolean) {
	if (ctx.advancedFiltersCollapsed) {
		ctx.advancedFiltersCollapsed.value = collapsed
	}
}

function isPhotosensitivityExclusionSelected(filters: { type: string; option: string }[]) {
	return filters.some(
		(filter) => filter.type === 'advanced' && filter.option === PHOTOSENSITIVITY_FILTER_OPTION,
	)
}

watch(
	() => ({
		selected: isPhotosensitivityExclusionSelected(
			ctx.isServerType.value ? ctx.serverCurrentFilters.value : ctx.currentFilters.value,
		),
		saved: advancedPrefs.value.includes(PHOTOSENSITIVITY_FILTER_OPTION),
	}),
	(current, previous) => {
		if (!previous) {
			return
		}

		const selected = current.selected && !previous.selected
		const alreadySaved = previous.saved
		const dismissed = ctx.dismissedPhotosensitivityFilterWarning?.value

		if (selected && !alreadySaved && !dismissed) {
			nextTick(() => photosensitivityWarningModal.value?.show())
		}
	},
)

function onPhotosensitivityWarningDismiss(dontShowAgain: boolean) {
	if (dontShowAgain && ctx.dismissedPhotosensitivityFilterWarning) {
		ctx.dismissedPhotosensitivityFilterWarning.value = true
	}
}

function closeFiltersMenu() {
	if (ctx.filtersMenuOpen) {
		ctx.filtersMenuOpen.value = false
	}
	window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior })
}

const filterClass = computed(() => {
	if (isApp.value) {
		return 'border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
	}
	if (ctx.filtersMenuOpen?.value) {
		return 'border-0 border-b-[1px] border-solid border-divider last:border-b-0'
	}
	return 'card-shadow rounded-2xl bg-surface-3 border border-solid border-surface-4'
})

const buttonClass = computed(() => {
	if (isApp.value) {
		return 'button-animation flex flex-col gap-1 px-3 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg'
	}
	return 'button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none'
})

const contentClass = computed(() => (isApp.value ? 'mt-2 mb-3' : 'mb-4 mx-3'))
const innerPanelClass = computed(() => (isApp.value ? 'ml-2 mr-3' : 'p-1'))
const selectedProjectClass = computed(() =>
	isApp.value ? 'bg-surface-1 border border-solid border-surface-3' : 'bg-surface-2',
)

function hasProvidedFilter(filterId: string): boolean {
	return (ctx.providedFilters?.value ?? []).some((filter) => filter.type === filterId)
}

function getFilterOpenByDefault(filterId: string): boolean {
	if (filterId === 'advanced') {
		return !advancedFiltersCollapsed.value
	}
	if (hasProvidedFilter(filterId)) {
		return true
	}
	if (filterId === 'compatible_dependency_project_ids') {
		return true
	}
	if (ctx.isServerType.value) {
		return ![
			'server_category_minecraft_server_meta',
			'server_category_minecraft_server_community',
			'server_game_version',
			'server_status',
		].includes(filterId)
	}
	if (isApp.value) {
		return filterId.startsWith('category') || filterId === 'environment' || filterId === 'license'
	}
	if (
		lockedMessages.value?.gameVersionShaderMessage &&
		ctx.projectType.value === 'shader' &&
		filterId === 'game_version'
	) {
		return false
	}
	return true
}
</script>

<template>
	<slot name="prepend" />

	<PhotosensitivityWarningModal
		ref="photosensitivityWarningModal"
		@dismiss="onPhotosensitivityWarningDismiss"
	/>

	<div v-if="ctx.filtersMenuOpen?.value" class="fixed inset-0 z-40 bg-bg" />

	<div
		class="flex flex-col"
		:class="{
			'gap-3': !isApp,
			'fixed inset-0 z-50 m-4 mb-0 overflow-auto rounded-t-3xl bg-bg-raised':
				ctx.filtersMenuOpen?.value,
		}"
	>
		<div
			v-if="ctx.filtersMenuOpen?.value"
			class="sticky top-0 z-10 mx-1 flex items-center justify-between gap-3 border-0 border-b-[1px] border-solid border-divider bg-bg-raised px-6 py-4"
		>
			<h3 class="m-0 text-lg text-contrast">{{ formatMessage(commonMessages.filtersLabel) }}</h3>
			<IconButton label="Close" @click="closeFiltersMenu">
				<XIcon />
			</IconButton>
		</div>

		<div
			v-if="ctx.showHideInstalled?.value || ctx.showHideSelected?.value"
			:class="
				isApp
					? 'flex flex-col gap-3 border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
					: 'card-shadow flex flex-col gap-3 rounded-2xl bg-bg-raised border-solid border-surface-4 border p-4'
			"
		>
			<label
				v-if="ctx.showHideInstalled?.value"
				class="flex cursor-pointer items-center justify-between gap-3 text-contrast font-medium"
			>
				{{
					ctx.hideInstalledLabel?.value ?? formatMessage(commonMessages.hideInstalledContentLabel)
				}}
				<Toggle
					v-model="ctx.hideInstalled!.value"
					small
					class="shrink-0"
					@update:model-value="ctx.onFilterChange()"
				/>
			</label>
			<label
				v-if="ctx.showHideSelected?.value"
				class="flex cursor-pointer items-center justify-between gap-3 text-contrast font-medium"
			>
				{{ ctx.hideSelectedLabel?.value ?? formatMessage(commonMessages.hideSelectedContentLabel) }}
				<Toggle
					v-model="ctx.hideSelected!.value"
					small
					class="shrink-0"
					@update:model-value="ctx.onFilterChange()"
				/>
			</label>
		</div>

		<SearchSidebarFilter
			v-if="ctx.showServerEnvironment?.value"
			v-model:selected-filters="serverEnvironmentSelection"
			v-model:toggled-groups="serverEnvironmentToggledGroups"
			:provided-filters="[]"
			:filter-type="serverEnvironmentFilterType"
			:project-type="ctx.projectType.value"
			:class="filterClass"
			:button-class="buttonClass"
			:content-class="contentClass"
			:inner-panel-class="innerPanelClass"
			:selected-project-class="selectedProjectClass"
			:open-by-default="true"
		>
			<template #header>
				<h3 :class="isApp ? 'text-base m-0' : 'm-0 text-lg font-semibold'">
					{{ serverEnvironmentFilterType.formatted_name }}
				</h3>
			</template>
		</SearchSidebarFilter>

		<template v-if="ctx.isServerType.value">
			<SearchSidebarFilter
				v-for="filterType in ctx.serverFilterTypes.value.filter(
					(f) => f.options.length > 0 && !hiddenFilterTypes.includes(f.id),
				)"
				:key="`server-filter-${filterType.id}`"
				v-model:selected-filters="ctx.serverCurrentFilters.value"
				v-model:toggled-groups="ctx.serverToggledGroups.value"
				:provided-filters="[]"
				:filter-type="filterType"
				:project-type="ctx.projectType.value"
				:class="filterClass"
				:button-class="buttonClass"
				:content-class="contentClass"
				:inner-panel-class="innerPanelClass"
				:selected-project-class="selectedProjectClass"
				:open-by-default="getFilterOpenByDefault(filterType.id)"
				@on-open="() => filterType.id === 'advanced' && setAdvancedFiltersCollapsed(false)"
				@on-close="() => filterType.id === 'advanced' && setAdvancedFiltersCollapsed(true)"
			>
				<template #header>
					<h3 :class="isApp ? 'text-base m-0' : 'm-0 text-base font-semibold'">
						{{ filterType.formatted_name }}
					</h3>
				</template>
				<template v-if="filterType.id === 'advanced'" #prefix>
					<AdvancedFiltersPersistenceNote />
				</template>
			</SearchSidebarFilter>
		</template>
		<template v-else>
			<SearchSidebarFilter
				v-for="filter in visibleFilters"
				:key="`filter-${filter.id}`"
				v-model:selected-filters="ctx.currentFilters.value"
				v-model:toggled-groups="ctx.toggledGroups.value"
				v-model:overridden-provided-filter-types="ctx.overriddenProvidedFilterTypes.value"
				:provided-filters="ctx.providedFilters?.value ?? []"
				:filter-type="filter"
				:project-type="ctx.projectType.value"
				:result-count="ctx.totalHits.value"
				:loading="ctx.loading.value"
				:refreshing="ctx.refreshing.value"
				:class="filterClass"
				:button-class="buttonClass"
				:content-class="contentClass"
				:inner-panel-class="innerPanelClass"
				:selected-project-class="selectedProjectClass"
				:open-by-default="getFilterOpenByDefault(filter.id)"
				@on-open="() => filter.id === 'advanced' && setAdvancedFiltersCollapsed(false)"
				@on-close="() => filter.id === 'advanced' && setAdvancedFiltersCollapsed(true)"
			>
				<template #header>
					<h3 :class="isApp ? 'text-base m-0' : 'm-0 text-lg font-semibold'">
						{{ filter.formatted_name }}
					</h3>
				</template>
				<template v-if="filter.id === 'advanced'" #prefix>
					<AdvancedFiltersPersistenceNote />
				</template>
				<template
					v-else-if="
						lockedMessages?.gameVersionShaderMessage &&
						ctx.projectType.value === 'shader' &&
						filter.id === 'game_version'
					"
					#prefix
				>
					<div class="mb-4 grid grid-cols-[auto_1fr] gap-2 px-3 text-sm font-medium text-blue">
						<InfoIcon class="mt-1 size-4" />
						<span>{{ lockedMessages?.gameVersionShaderMessage }}</span>
					</div>
				</template>
				<template v-if="lockedMessages?.gameVersion" #locked-game_version>
					{{ lockedMessages.gameVersion }}
				</template>
				<template v-if="lockedMessages?.modLoader" #locked-mod_loader>
					{{ lockedMessages.modLoader }}
				</template>
				<template v-if="lockedMessages?.modLoader" #locked-shader_loader>
					{{ lockedMessages.modLoader }}
				</template>
				<template v-if="lockedMessages?.environment" #locked-environment>
					{{ lockedMessages.environment }}
				</template>
				<template v-if="lockedMessages?.syncButton" #sync-button>
					{{ lockedMessages.syncButton }}
				</template>
			</SearchSidebarFilter>
		</template>
	</div>
</template>
