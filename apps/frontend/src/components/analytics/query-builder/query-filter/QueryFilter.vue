<template>
	<DropdownFilterBar
		v-model="selectedFilterValue"
		:categories="filterCategories"
		:show-clear="showClearAction && canClearSelectedBreakdown"
		:show-label="showLabel"
		:show-preview-filter-icon="showPreviewFilterIcon"
		:preview-trigger-class="previewTriggerClass"
		:add-button-class="addButtonClass"
		clear-label="Reset"
		:add-label="addLabel"
		checkbox-position="right"
		@clear="clearFilterBar"
	>
		<template #search-actions="{ category, setSelectedValues }">
			<div v-if="category.key === 'game_version'" class="mr-2 flex min-w-[124px] justify-end">
				<Tabs
					:value="gameVersionType"
					:tabs="gameVersionTypeTabs"
					aria-label="Game version type"
					@update:value="(type) => setGameVersionType(type, setSelectedValues)"
				/>
			</div>
		</template>

		<template #option="{ category, option, selected }">
			<div class="flex min-w-0 flex-1 items-center gap-2">
				<template v-if="category.key === 'version_id'">
					<span
						v-for="metadata in getProjectVersionOptionProjectMetadata(option.value)"
						:key="`${option.value}-${metadata.name}`"
						v-tooltip="metadata.name"
						class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
					>
						<img
							v-if="metadata.iconUrl"
							:src="metadata.iconUrl"
							:alt="`${metadata.name} Icon`"
							class="h-6 w-6 rounded object-cover"
						/>
						<BoxIcon v-else class="h-full w-full" />
					</span>
				</template>
				<span
					class="min-w-0 truncate font-semibold leading-tight"
					:class="selected ? 'text-contrast' : 'text-primary'"
				>
					{{ option.label }}
				</span>
			</div>
		</template>

		<template #category-footer="{ category, setSelectedValues, closeMenu }">
			<DownloadsThresholdInput
				v-if="category.key === 'country'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Countries above"
				input-aria-label="Country downloads threshold"
				:threshold="countryDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setCountryDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyCountryDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
			<DownloadsThresholdInput
				v-else-if="category.key === 'version_id'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Project versions above"
				input-aria-label="Project version downloads threshold"
				:threshold="projectVersionDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setProjectVersionDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyProjectVersionDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
			<DownloadsThresholdInput
				v-else-if="category.key === 'game_version'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Game Versions above"
				input-aria-label="Game version downloads threshold"
				:threshold="gameVersionDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setGameVersionDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyGameVersionDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
		</template>

		<template #preview-footer="{ category, setSelectedValues, closeMenu }">
			<DownloadsThresholdInput
				v-if="category.key === 'country'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Countries above"
				input-aria-label="Country downloads threshold"
				:threshold="countryDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setCountryDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyCountryDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
			<DownloadsThresholdInput
				v-else-if="category.key === 'version_id'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Project versions above"
				input-aria-label="Project version downloads threshold"
				:threshold="projectVersionDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setProjectVersionDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyProjectVersionDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
			<DownloadsThresholdInput
				v-else-if="category.key === 'game_version'"
				class="border-0 border-t border-solid border-surface-5 px-3 py-2.5"
				label="Game Versions above"
				input-aria-label="Game version downloads threshold"
				:threshold="gameVersionDownloadsThreshold"
				input-width-class="w-16"
				@update:threshold="
					(threshold) => setGameVersionDownloadsThreshold(threshold, setSelectedValues)
				"
				@submit="
					(event) =>
						runDownloadsThresholdQuery(
							applyGameVersionDownloadsThreshold,
							setSelectedValues,
							closeMenu,
							event,
						)
				"
			/>
		</template>
	</DropdownFilterBar>
</template>

<script setup lang="ts">
import { BoxIcon } from '@modrinth/assets'
import {
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	type DropdownFilterBarOption,
	Tabs,
	type TabsTab,
	type TabsValue,
} from '@modrinth/ui'

import { useFormattedCountries } from '@/composables/country.ts'
import { useGeneratedState } from '~/composables/generated'
import {
	type AnalyticsQueryFilterCategory,
	type AnalyticsSelectedFilters,
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'
import {
	areStringArraysEqual,
	getDefaultAnalyticsBreakdownPresets,
} from '~/providers/analytics/query-builder-url'

import { getDownloadSourceLabel } from '../../breakdown'
import DownloadsThresholdInput from '../DownloadsThresholdInput.vue'
import {
	areSelectedFiltersEqual,
	buildProjectVersionFilterOptionProjectMetadataById,
	buildProjectVersionFilterOptions,
	cloneSelectedFilters,
	FILTER_VALUE_CATEGORIES,
	getOptionsWithSelectedValues,
	getProjectVersionFilterOptionMetadataIds,
	getProjectVersionFilterOptionProjectMetadataCacheKey,
	getProjectVersionFilterOptionsCacheKey,
	getVisibleAnalyticsFilterCategoriesForState,
	normalizeSelectedValues as normalizeSelectedFilterValues,
	type ProjectVersionFilterOption,
	type ProjectVersionFilterOptionProjectMetadata,
} from './query-filter.ts'

type AnalyticsFilterValueCategory = Exclude<AnalyticsQueryFilterCategory, 'project'>
type GameVersionType = 'release' | 'all'
type SetDropdownFilterValues = (values: string[]) => void
type ApplyDownloadsThreshold = (setSelectedValues: SetDropdownFilterValues) => void
type CloseDownloadsThresholdMenu = (event?: Event) => void

withDefaults(
	defineProps<{
		addLabel?: string
		showLabel?: boolean
		showPreviewFilterIcon?: boolean
		previewTriggerClass?: string
		addButtonClass?: string
		showClearAction?: boolean
	}>(),
	{
		addLabel: 'Add',
		showLabel: true,
		showPreviewFilterIcon: false,
		showClearAction: true,
	},
)

const {
	hasProjectContext,
	projects,
	selectedProjectIds,
	availableProjectStatuses,
	filterOptions,
	projectVersionDownloadsById,
	gameVersionDownloadsByVersion,
	countryDownloadsByCode,
	isAnalyticsFilterOptionsLoading,
	selectedBreakdowns,
	selectedFilters,
	queryResetToken,
	refreshAnalyticsQuery,
	hasCompletedAnalyticsLoading,
	versionNumbersById,
	versionPublishedDatesById,
	versionProjectNamesById,
	versionProjectIconUrlsById,
	getVersionDisplayName,
} = injectAnalyticsDashboardContext()
const formattedCountries = useFormattedCountries()
const generatedState = useGeneratedState()

const gameVersionType = ref<GameVersionType>('release')
const countryDownloadsThreshold = ref<number | null>(null)
const projectVersionDownloadsThreshold = ref<number | null>(null)
const gameVersionDownloadsThreshold = ref<number | null>(null)
const gameVersionTypeTabs: TabsTab[] = [
	{ value: 'release', label: 'Release' },
	{ value: 'all', label: 'All' },
]
const filterValueCategoryKeys = new Set<string>(FILTER_VALUE_CATEGORIES)
const downloadsThresholdFilterCategories = ['country', 'version_id', 'game_version'] as const
type DownloadsThresholdFilterCategory = (typeof downloadsThresholdFilterCategories)[number]
const downloadsThresholdSelections = ref<
	Partial<Record<DownloadsThresholdFilterCategory, string[]>>
>({})
const projectStatusFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	availableProjectStatuses.value.map((status) => ({
		value: status,
		label: getProjectStatusFilterOptionLabel(status),
	})),
)
const selectedProjectIdSet = computed(() => new Set(selectedProjectIds.value))
const effectiveSelectedProjectCount = computed(
	() =>
		projects.value.filter(
			(project) =>
				selectedProjectIdSet.value.has(project.id) &&
				doesProjectStatusMatchFilters(project.status, selectedFilters.value),
		).length,
)
const showProjectVersionProjectIcons = computed(() => effectiveSelectedProjectCount.value > 1)
const defaultSelectedBreakdown = computed(() =>
	getDefaultAnalyticsBreakdownPresets(selectedProjectIds.value),
)
const canClearSelectedBreakdown = computed(
	() => !areStringArraysEqual(selectedBreakdowns.value, defaultSelectedBreakdown.value),
)
const analyticsFilterOptionsEmptyLabel = computed(() =>
	isAnalyticsFilterOptionsLoading.value ? 'Loading...' : undefined,
)
const projectVersionFilterOptions = shallowRef<ProjectVersionFilterOption[]>([])
const projectVersionFilterOptionProjectMetadataById = shallowRef(
	new Map<string, ProjectVersionFilterOptionProjectMetadata[]>(),
)
const draftSelectedFilters = ref<AnalyticsSelectedFilters>(
	cloneSelectedFilters(selectedFilters.value),
)
let selectedFiltersCommitRequestId = 0
let projectVersionFilterOptionsCacheKey = ''
let projectVersionFilterOptionProjectMetadataCacheKey = ''

const selectedFilterValue = computed<Record<string, string[]>>({
	get: () => getSelectedFilterBarValue(),
	set: (nextValue) => {
		const nextFilters = cloneSelectedFilters(draftSelectedFilters.value)

		for (const [categoryKey, values] of Object.entries(nextValue)) {
			if (!isAnalyticsFilterValueCategory(categoryKey)) {
				continue
			}

			nextFilters[categoryKey] = normalizeSelectedFilterValues(categoryKey, values, [])
		}

		draftSelectedFilters.value = nextFilters
		void scheduleSelectedFiltersCommit()
	},
})

function getSelectedFilterBarValue(): AnalyticsSelectedFilters {
	return cloneSelectedFilters(draftSelectedFilters.value)
}

function clearSelectedBreakdown() {
	selectedBreakdowns.value = defaultSelectedBreakdown.value
}

function clearFilterBar() {
	clearSelectedBreakdown()
	clearDownloadsThresholds()
}

watch(queryResetToken, () => {
	selectedFiltersCommitRequestId++
	draftSelectedFilters.value = cloneSelectedFilters(selectedFilters.value)
	clearDownloadsThresholds()
})

watch(
	selectedFilters,
	(nextFilters, previousFilters) => {
		selectedFiltersCommitRequestId++
		draftSelectedFilters.value = cloneSelectedFilters(nextFilters)
		clearDownloadsThresholdsForChangedFilters(previousFilters, nextFilters)
	},
	{ deep: true },
)

watch(
	[
		hasCompletedAnalyticsLoading,
		filterOptions,
		versionNumbersById,
		versionPublishedDatesById,
		versionProjectNamesById,
	],
	([
		hasCompletedLoading,
		nextFilterOptions,
		nextVersionNumbersById,
		nextVersionPublishedDatesById,
		nextVersionProjectNamesById,
	]) => {
		if (!hasCompletedLoading) {
			projectVersionFilterOptionsCacheKey = ''
			if (projectVersionFilterOptions.value.length > 0) {
				projectVersionFilterOptions.value = []
			}
			return
		}

		const nextCacheKey = getProjectVersionFilterOptionsCacheKey(
			nextFilterOptions.versionIds,
			nextVersionNumbersById,
			nextVersionPublishedDatesById,
			nextVersionProjectNamesById,
		)
		if (nextCacheKey === projectVersionFilterOptionsCacheKey) {
			return
		}

		projectVersionFilterOptionsCacheKey = nextCacheKey
		projectVersionFilterOptions.value = buildProjectVersionFilterOptions(
			nextFilterOptions.versionIds,
			nextVersionNumbersById,
			nextVersionPublishedDatesById,
			nextVersionProjectNamesById,
		)
	},
	{ immediate: true },
)

watch(
	[
		hasCompletedAnalyticsLoading,
		filterOptions,
		selectedFilters,
		versionProjectNamesById,
		versionProjectIconUrlsById,
	],
	([
		hasCompletedLoading,
		nextFilterOptions,
		nextSelectedFilters,
		nextVersionProjectNamesById,
		nextVersionProjectIconUrlsById,
	]) => {
		if (!hasCompletedLoading) {
			projectVersionFilterOptionProjectMetadataCacheKey = ''
			if (projectVersionFilterOptionProjectMetadataById.value.size > 0) {
				projectVersionFilterOptionProjectMetadataById.value = new Map()
			}
			return
		}

		const metadataIds = getProjectVersionFilterOptionMetadataIds(
			nextFilterOptions.versionIds,
			nextSelectedFilters.version_id,
		)
		const nextCacheKey = getProjectVersionFilterOptionProjectMetadataCacheKey(
			metadataIds,
			nextVersionProjectNamesById,
			nextVersionProjectIconUrlsById,
		)
		if (nextCacheKey === projectVersionFilterOptionProjectMetadataCacheKey) {
			return
		}

		projectVersionFilterOptionProjectMetadataCacheKey = nextCacheKey
		projectVersionFilterOptionProjectMetadataById.value =
			buildProjectVersionFilterOptionProjectMetadataById(
				metadataIds,
				nextVersionProjectNamesById,
				nextVersionProjectIconUrlsById,
			)
	},
	{ immediate: true },
)

async function scheduleSelectedFiltersCommit() {
	const requestId = ++selectedFiltersCommitRequestId
	const nextFilters = cloneSelectedFilters(draftSelectedFilters.value)

	await waitForDeferredQueryFilterCommit()

	if (requestId !== selectedFiltersCommitRequestId) {
		return
	}

	if (!areSelectedFiltersEqual(selectedFilters.value, nextFilters)) {
		selectedFilters.value = nextFilters
	}
}

function waitForDeferredQueryFilterCommit(): Promise<void> {
	if (!import.meta.client) {
		return nextTick()
	}

	return new Promise((resolve) => {
		nextTick(() => {
			requestAnimationFrame(() => {
				requestAnimationFrame(() => resolve())
			})
		})
	})
}

const filterCategories = computed<DropdownFilterBarCategory[]>(() => {
	const visibleCategoryKeys = new Set(
		getVisibleAnalyticsFilterCategoriesForState(selectedBreakdowns.value, selectedFilters.value),
	)
	const categories: DropdownFilterBarCategory[] = []

	if (!hasProjectContext.value) {
		categories.push({
			key: 'project_status',
			label: 'Project status',
			options: withSelectedOptions('project_status', projectStatusFilterOptions.value),
		})
	}

	categories.push(
		{
			key: 'country',
			label: 'Country',
			searchable: countryFilterOptions.value.length > 6,
			searchPlaceholder: 'Search countries...',
			emptyOptionsLabel: analyticsFilterOptionsEmptyLabel.value,
			emptySearchLabel: analyticsFilterOptionsEmptyLabel.value,
			options: withSelectedOptions('country', countryFilterOptions.value),
			submenuClass: 'w-fit',
		},
		{
			key: 'monetization',
			label: 'Monetization',
			options: withSelectedOptions('monetization', [
				{ value: 'monetized', label: 'Monetized' },
				{ value: 'unmonetized', label: 'Unmonetized' },
			]),
		},
		{
			key: 'user_agent',
			label: 'Download source',
			searchable: downloadSourceFilterOptions.value.length > 6,
			searchPlaceholder: 'Search download sources...',
			emptyOptionsLabel: analyticsFilterOptionsEmptyLabel.value,
			emptySearchLabel: analyticsFilterOptionsEmptyLabel.value,
			options: withSelectedOptions('user_agent', downloadSourceFilterOptions.value),
		},
		{
			key: 'download_reason',
			label: 'Download reason',
			emptyOptionsLabel: analyticsFilterOptionsEmptyLabel.value,
			emptySearchLabel: analyticsFilterOptionsEmptyLabel.value,
			options: withSelectedOptions('download_reason', downloadReasonFilterOptions.value),
		},
		{
			key: 'version_id',
			label: 'Project version',
			searchable: projectVersionFilterOptions.value.length > 6,
			searchPlaceholder: 'Search project versions...',
			submenuClass: 'w-fit',
			options: withSelectedOptions('version_id', projectVersionFilterOptions.value),
		},
		{
			key: 'game_version',
			label: 'Game version',
			searchable: true,
			searchPlaceholder: 'Search versions...',
			submenuClass: 'w-fit max-w-[340px]',
			options: withSelectedOptions('game_version', gameVersionFilterOptions.value),
		},
		{
			key: 'loader_type',
			label: 'Loader',
			options: withSelectedOptions('loader_type', loaderTypeFilterOptions.value),
		},
	)

	return categories.filter((category) =>
		visibleCategoryKeys.has(category.key as AnalyticsFilterValueCategory),
	)
})

const countryLabelsByCode = computed(
	() =>
		new Map(
			formattedCountries.value.map(
				(country) => [country.value.toUpperCase(), country.label] as const,
			),
		),
)

const countryFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.countries
		.map((countryCode) => ({
			value: countryCode,
			label: getCountryFilterOptionLabel(countryCode),
			searchTerms: [countryCode],
		}))
		.sort((left, right) => left.label.localeCompare(right.label)),
)

const gameVersionReleaseDatesByVersion = computed(
	() =>
		new Map(
			generatedState.value.gameVersions.map(
				(gameVersion) => [gameVersion.version, gameVersion.date] as const,
			),
		),
)
const gameVersionTypesByVersion = computed(
	() =>
		new Map(
			generatedState.value.gameVersions.map(
				(gameVersion) => [gameVersion.version, gameVersion.version_type] as const,
			),
		),
)

const downloadSourceFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.downloadSources
		.map((downloadSource) => ({
			value: downloadSource,
			label: getDownloadSourceLabel(downloadSource),
		}))
		.sort((left, right) => left.label.localeCompare(right.label)),
)

const downloadReasonFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.downloadReasons.map((downloadReason) => ({
		value: downloadReason,
		label: getDownloadReasonFilterOptionLabel(downloadReason),
	})),
)

const gameVersionFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.gameVersions
		.filter((gameVersion) => {
			const versionType = gameVersionTypesByVersion.value.get(gameVersion)
			return (
				gameVersionType.value === 'all' || versionType === undefined || versionType === 'release'
			)
		})
		.map((gameVersion) => ({
			value: gameVersion,
			label: gameVersion,
		}))
		.sort((left, right) =>
			compareOptionalDateStringsDescending(
				gameVersionReleaseDatesByVersion.value.get(left.value),
				gameVersionReleaseDatesByVersion.value.get(right.value),
				left.label,
				right.label,
			),
		),
)

const loaderTypeFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.loaderTypes
		.map((loaderType) => ({
			value: loaderType,
			label: getLoaderTypeFilterOptionLabel(loaderType),
			searchTerms: [loaderType],
		}))
		.sort((left, right) => left.label.localeCompare(right.label)),
)

function isAnalyticsFilterValueCategory(
	categoryKey: string,
): categoryKey is AnalyticsFilterValueCategory {
	return filterValueCategoryKeys.has(categoryKey)
}

function withSelectedOptions(
	categoryKey: AnalyticsFilterValueCategory,
	options: DropdownFilterBarOption[],
): DropdownFilterBarOption[] {
	return getOptionsWithSelectedValues(
		options,
		selectedFilters.value[categoryKey],
		getMissingSelectedOptionLabel(categoryKey),
	)
}

function getMissingSelectedOptionLabel(
	categoryKey: AnalyticsFilterValueCategory,
): ((value: string) => string) | undefined {
	if (categoryKey === 'country') {
		return getCountryFilterOptionLabel
	}
	if (categoryKey === 'version_id') {
		return getVersionDisplayName
	}
	if (categoryKey === 'download_reason') {
		return getDownloadReasonFilterOptionLabel
	}
	if (categoryKey === 'user_agent') {
		return getDownloadSourceLabel
	}
	if (categoryKey === 'loader_type') {
		return getLoaderTypeFilterOptionLabel
	}
	return undefined
}

function getProjectVersionOptionProjectMetadata(versionId: string) {
	if (!showProjectVersionProjectIcons.value) {
		return []
	}

	return projectVersionFilterOptionProjectMetadataById.value.get(versionId) ?? []
}

function getCountryFilterOptionLabel(countryCode: string): string {
	const normalizedCode = countryCode.trim().toUpperCase()
	if (normalizedCode === 'XX') {
		return 'Other'
	}

	return countryLabelsByCode.value.get(normalizedCode) ?? countryCode
}

function getProjectStatusFilterOptionLabel(status: string): string {
	const normalizedStatus = status.trim()
	if (normalizedStatus.length === 0) {
		return status
	}

	return `${normalizedStatus.charAt(0).toUpperCase()}${normalizedStatus.slice(1)}`
}

function getLoaderTypeFilterOptionLabel(loaderType: string): string {
	const normalizedLoaderType = loaderType.trim()
	if (normalizedLoaderType.length === 0) {
		return loaderType
	}

	return `${normalizedLoaderType.charAt(0).toUpperCase()}${normalizedLoaderType.slice(1)}`
}

function getDownloadReasonFilterOptionLabel(reason: string): string {
	switch (reason) {
		case 'standalone':
			return 'Standalone'
		case 'dependency':
			return 'Dependency'
		case 'modpack':
			return 'Modpack'
		case 'update':
			return 'Update'
		default:
			return reason
	}
}

function getDateTimestamp(date: string | undefined): number | undefined {
	if (!date) {
		return undefined
	}

	const timestamp = new Date(date).getTime()
	return Number.isFinite(timestamp) ? timestamp : undefined
}

function compareOptionalDateStringsDescending(
	leftDate: string | undefined,
	rightDate: string | undefined,
	leftFallback: string,
	rightFallback: string,
): number {
	const leftTimestamp = getDateTimestamp(leftDate)
	const rightTimestamp = getDateTimestamp(rightDate)

	if (leftTimestamp !== undefined && rightTimestamp !== undefined) {
		return rightTimestamp - leftTimestamp
	}
	if (leftTimestamp !== undefined) {
		return -1
	}
	if (rightTimestamp !== undefined) {
		return 1
	}

	return leftFallback.localeCompare(rightFallback)
}

function applyGameVersionDownloadsThreshold(setSelectedValues: SetDropdownFilterValues) {
	const threshold = gameVersionDownloadsThreshold.value
	if (threshold === null) {
		return
	}

	const selectedValues = gameVersionFilterOptions.value
		.filter((gameVersion) => {
			return (gameVersionDownloadsByVersion.value.get(gameVersion.value) ?? 0) >= threshold
		})
		.map((gameVersion) => gameVersion.value)

	setDownloadsThresholdSelectedValues('game_version', selectedValues, setSelectedValues)
}

function applyCountryDownloadsThreshold(setSelectedValues: SetDropdownFilterValues) {
	const threshold = countryDownloadsThreshold.value
	if (threshold === null) {
		return
	}

	const selectedValues = countryFilterOptions.value
		.filter((country) => {
			return (
				(countryDownloadsByCode.value.get(country.value.trim().toUpperCase()) ?? 0) >= threshold
			)
		})
		.map((country) => country.value)

	setDownloadsThresholdSelectedValues('country', selectedValues, setSelectedValues)
}

function applyProjectVersionDownloadsThreshold(setSelectedValues: SetDropdownFilterValues) {
	const threshold = projectVersionDownloadsThreshold.value
	if (threshold === null) {
		return
	}

	const selectedValues = projectVersionFilterOptions.value
		.filter((version) => {
			return (projectVersionDownloadsById.value.get(version.value) ?? 0) >= threshold
		})
		.map((version) => version.value)

	setDownloadsThresholdSelectedValues('version_id', selectedValues, setSelectedValues)
}

function setCountryDownloadsThreshold(
	threshold: number | null,
	setSelectedValues: SetDropdownFilterValues,
) {
	countryDownloadsThreshold.value = threshold
	if (threshold === null) {
		clearDownloadsThreshold('country')
		setSelectedValues([])
		return
	}

	applyCountryDownloadsThreshold(setSelectedValues)
}

function setProjectVersionDownloadsThreshold(
	threshold: number | null,
	setSelectedValues: SetDropdownFilterValues,
) {
	projectVersionDownloadsThreshold.value = threshold
	if (threshold === null) {
		clearDownloadsThreshold('version_id')
		setSelectedValues([])
		return
	}

	applyProjectVersionDownloadsThreshold(setSelectedValues)
}

function setGameVersionDownloadsThreshold(
	threshold: number | null,
	setSelectedValues: SetDropdownFilterValues,
) {
	gameVersionDownloadsThreshold.value = threshold
	if (threshold === null) {
		clearDownloadsThreshold('game_version')
		setSelectedValues([])
		return
	}

	applyGameVersionDownloadsThreshold(setSelectedValues)
}

function clearDownloadsThresholdsForChangedFilters(
	previousFilters: AnalyticsSelectedFilters,
	nextFilters: AnalyticsSelectedFilters,
) {
	for (const categoryKey of downloadsThresholdFilterCategories) {
		if (areFilterSelectionsEqual(previousFilters[categoryKey], nextFilters[categoryKey])) {
			continue
		}

		const thresholdSelection = downloadsThresholdSelections.value[categoryKey]
		if (
			thresholdSelection &&
			areFilterSelectionsEqual(thresholdSelection, nextFilters[categoryKey])
		) {
			continue
		}

		if (previousFilters[categoryKey].length > 0 || nextFilters[categoryKey].length > 0) {
			clearDownloadsThreshold(categoryKey)
		}
	}
}

function setDownloadsThresholdSelectedValues(
	categoryKey: DownloadsThresholdFilterCategory,
	selectedValues: string[],
	setSelectedValues: SetDropdownFilterValues,
) {
	downloadsThresholdSelections.value = {
		...downloadsThresholdSelections.value,
		[categoryKey]: normalizeSelectedFilterValues(categoryKey, selectedValues, []),
	}
	setSelectedValues(selectedValues)
}

function clearDownloadsThreshold(categoryKey: DownloadsThresholdFilterCategory) {
	switch (categoryKey) {
		case 'country':
			countryDownloadsThreshold.value = null
			break
		case 'version_id':
			projectVersionDownloadsThreshold.value = null
			break
		case 'game_version':
			gameVersionDownloadsThreshold.value = null
			break
	}

	const { [categoryKey]: _removedSelection, ...nextSelections } = downloadsThresholdSelections.value
	downloadsThresholdSelections.value = nextSelections
}

function clearDownloadsThresholds() {
	for (const categoryKey of downloadsThresholdFilterCategories) {
		clearDownloadsThreshold(categoryKey)
	}
}

function areFilterSelectionsEqual(left: string[], right: string[]): boolean {
	const leftValues = new Set(left)
	const rightValues = new Set(right)
	if (leftValues.size !== rightValues.size) {
		return false
	}

	return [...leftValues].every((value) => rightValues.has(value))
}

async function runDownloadsThresholdQuery(
	applyDownloadsThreshold: ApplyDownloadsThreshold,
	setSelectedValues: SetDropdownFilterValues,
	closeMenu: CloseDownloadsThresholdMenu,
	event?: KeyboardEvent,
) {
	applyDownloadsThreshold(setSelectedValues)
	closeMenu(event)
	await scheduleSelectedFiltersCommit()
	await refreshAnalyticsQuery()
}

function setGameVersionType(type: TabsValue, setSelectedValues: SetDropdownFilterValues) {
	if (!isGameVersionType(type)) {
		return
	}

	gameVersionType.value = type
	applyGameVersionDownloadsThreshold(setSelectedValues)
}

function isGameVersionType(type: TabsValue): type is GameVersionType {
	return type === 'release' || type === 'all'
}
</script>
