<script setup lang="ts">
import { InfoIcon, XIcon } from '@modrinth/assets'
import { computed, nextTick, ref, toValue, useTemplateRef, watch } from 'vue'

import { IconButton } from '#ui/components/base/buttons'
import Toggle from '#ui/components/base/Toggle.vue'
import PhotosensitivityWarningModal from '#ui/components/modal/PhotosensitivityWarningModal.vue'
import SearchSidebarFilter from '#ui/components/search/SearchSidebarFilter.vue'
import { useVIntl } from '#ui/composables/i18n'
import { useAdvancedPrefs } from '#ui/utils/advanced-filter-preferences'
import { commonMessages } from '#ui/utils/common-messages'
import { type FilterType, LOADER_FILTER_TYPES } from '#ui/utils/search'

import AdvancedFiltersPersistenceNote from './components/AdvancedFiltersPersistenceNote.vue'
import { useServerEnvironmentFilter } from './composables/use-server-environment-filter'
import { injectBrowseManager } from './providers/browse-manager'

const PHOTOSENSITIVITY_FILTER_OPTION = 'epilepsy_triggers'
const COLLAPSED_SERVER_FILTERS = [
	'server_category_minecraft_server_meta',
	'server_category_minecraft_server_community',
	'server_game_version',
	'server_status',
]

const props = defineProps<{
	dense?: boolean
}>()

const ctx = injectBrowseManager()
const { view } = ctx
const { formatMessage } = useVIntl()
const advancedPrefs = useAdvancedPrefs()

const { filterType: environmentFilterType, selection: environmentSelection } =
	useServerEnvironmentFilter(ctx)
const environmentToggledGroups = ref<string[]>([])
const lockedMessages = computed(() => toValue(ctx.lockedFilterMessages))
const providedFilters = computed(() =>
	view.isServerType.value ? [] : (ctx.providedFilters?.value ?? []),
)

function filterPriority(filter: FilterType) {
	if (filter.id === 'game_version') return 2
	return LOADER_FILTER_TYPES.some((type) => type === filter.id) ? 1 : 0
}

const visibleFilters = computed(() => {
	const hidden = ctx.hiddenFilterTypes?.value ?? []
	if (view.isServerType.value) {
		return view.filterTypes.value.filter(
			(filter) => filter.options.length > 0 && !hidden.includes(filter.id),
		)
	}
	const filters = view.filterTypes.value.filter(
		(filter) => filter.display !== 'none' && !hidden.includes(filter.id),
	)
	return props.dense ? filters.sort((a, b) => filterPriority(b) - filterPriority(a)) : filters
})

const photosensitivityWarningModal = useTemplateRef('photosensitivityWarningModal')

watch(
	() => ({
		selected: view.filters.value.some(
			(filter) => filter.type === 'advanced' && filter.option === PHOTOSENSITIVITY_FILTER_OPTION,
		),
		saved: advancedPrefs.value.includes(PHOTOSENSITIVITY_FILTER_OPTION),
	}),
	(current, previous) => {
		if (!previous) return
		const newlySelected = current.selected && !previous.selected
		const dismissed = ctx.preferences?.dismissedPhotosensitivityWarning.value
		if (newlySelected && !previous.saved && !dismissed) {
			void nextTick(() => photosensitivityWarningModal.value?.show())
		}
	},
)

function onPhotosensitivityWarningDismiss(dontShowAgain: boolean) {
	if (dontShowAgain && ctx.preferences) {
		ctx.preferences.dismissedPhotosensitivityWarning.value = true
	}
}

function setAdvancedFiltersCollapsed(collapsed: boolean) {
	if (ctx.preferences) ctx.preferences.advancedFiltersCollapsed.value = collapsed
}

function closeFiltersMenu() {
	if (ctx.filtersMenuOpen) ctx.filtersMenuOpen.value = false
	window.scrollTo({ top: 0, behavior: 'instant' as ScrollBehavior })
}

function isShaderGameVersionNote(filterId: string) {
	return (
		!!lockedMessages.value?.gameVersionShaderMessage &&
		ctx.projectType.value === 'shader' &&
		filterId === 'game_version'
	)
}

function openByDefault(filterId: string): boolean {
	if (filterId === 'advanced') return !(ctx.preferences?.advancedFiltersCollapsed.value ?? true)
	if (providedFilters.value.some((filter) => filter.type === filterId)) return true
	if (filterId === 'compatible_dependency_project_ids') return true
	if (view.isServerType.value) return !COLLAPSED_SERVER_FILTERS.includes(filterId)
	if (props.dense) {
		return filterId.startsWith('category') || filterId === 'environment' || filterId === 'license'
	}
	return !isShaderGameVersionNote(filterId)
}

const filterClass = computed(() => {
	if (props.dense) {
		return 'border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
	}
	if (ctx.filtersMenuOpen?.value) {
		return 'border-0 border-b-[1px] border-solid border-divider last:border-b-0'
	}
	return 'card-shadow rounded-2xl bg-surface-3 border border-solid border-surface-4'
})
const buttonClass = computed(() =>
	props.dense
		? 'button-animation flex flex-col gap-1 px-3 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg'
		: 'button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none',
)
const contentClass = computed(() => (props.dense ? 'mt-2 mb-3' : 'mb-4 mx-3'))
const innerPanelClass = computed(() => (props.dense ? 'ml-2 mr-3' : 'p-1'))
const selectedProjectClass = computed(() =>
	props.dense ? 'bg-surface-4 border border-solid border-surface-5' : 'bg-surface-2',
)
const headerClass = computed(() => (props.dense ? 'text-base m-0' : 'm-0 text-lg font-semibold'))
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
			'gap-3': !dense,
			'fixed inset-0 z-50 m-4 mb-0 overflow-auto rounded-t-3xl bg-bg-raised':
				ctx.filtersMenuOpen?.value,
		}"
	>
		<div
			v-if="ctx.filtersMenuOpen?.value"
			class="sticky top-0 z-10 mx-1 flex items-center justify-between gap-3 border-0 border-b-[1px] border-solid border-divider bg-bg-raised px-6 py-4"
		>
			<h3 class="m-0 text-lg text-contrast">{{ formatMessage(commonMessages.filtersLabel) }}</h3>
			<IconButton :label="formatMessage(commonMessages.closeButton)" @click="closeFiltersMenu">
				<XIcon />
			</IconButton>
		</div>

		<div
			v-if="ctx.toggles?.value.length"
			:class="
				dense
					? 'flex flex-col gap-3 border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid'
					: 'card-shadow flex flex-col gap-3 rounded-2xl bg-bg-raised border-solid border-surface-4 border p-4'
			"
		>
			<label
				v-for="toggle in ctx.toggles.value"
				:key="toggle.id"
				class="flex cursor-pointer items-center justify-between gap-3 font-medium text-contrast"
			>
				{{ toggle.label }}
				<Toggle
					v-model="toggle.model.value"
					small
					class="shrink-0"
					@update:model-value="view.onFilterChange()"
				/>
			</label>
		</div>

		<SearchSidebarFilter
			v-if="ctx.serverEnvironment?.shown.value && !view.isServerType.value"
			v-model:selected-filters="environmentSelection"
			v-model:toggled-groups="environmentToggledGroups"
			:provided-filters="[]"
			:filter-type="environmentFilterType"
			:project-type="ctx.projectType.value"
			:class="filterClass"
			:button-class="buttonClass"
			:content-class="contentClass"
			:inner-panel-class="innerPanelClass"
			:selected-project-class="selectedProjectClass"
			:open-by-default="true"
		>
			<template #header>
				<h3 :class="headerClass">{{ environmentFilterType.formatted_name }}</h3>
			</template>
		</SearchSidebarFilter>

		<SearchSidebarFilter
			v-for="filter in visibleFilters"
			:key="`filter-${filter.id}`"
			v-model:selected-filters="view.filters.value"
			v-model:toggled-groups="view.toggledGroups.value"
			v-model:overridden-provided-filter-types="view.overriddenProvidedFilterTypes.value"
			:provided-filters="providedFilters"
			:filter-type="filter"
			:project-type="ctx.projectType.value"
			:dropdown-direction="dense ? 'up' : undefined"
			:result-count="view.isServerType.value ? undefined : view.totalHits.value"
			:loading="view.isServerType.value ? undefined : view.loading.value"
			:refreshing="view.isServerType.value ? undefined : view.refreshing.value"
			:class="filterClass"
			:button-class="buttonClass"
			:content-class="contentClass"
			:inner-panel-class="innerPanelClass"
			:selected-project-class="selectedProjectClass"
			:open-by-default="openByDefault(filter.id)"
			@on-open="() => filter.id === 'advanced' && setAdvancedFiltersCollapsed(false)"
			@on-close="() => filter.id === 'advanced' && setAdvancedFiltersCollapsed(true)"
		>
			<template #header>
				<h3
					:class="view.isServerType.value && !dense ? 'm-0 text-base font-semibold' : headerClass"
				>
					{{ filter.formatted_name }}
				</h3>
			</template>
			<template v-if="filter.id === 'advanced'" #prefix>
				<AdvancedFiltersPersistenceNote />
			</template>
			<template v-else-if="isShaderGameVersionNote(filter.id)" #prefix>
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
	</div>
</template>
