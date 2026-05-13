<template>
	<DropdownFilterBar
		v-model="selectedFilterValue"
		:categories="filterCategories"
		:show-clear="selectedBreakdown !== 'none'"
		@clear="clearFilterBar"
	>
		<template #search-actions="{ category, setSelectedValues }">
			<div v-if="category.key === 'game_version'" class="flex w-40 justify-end">
				<Chips
					:model-value="gameVersionType"
					:items="gameVersionTypeOptions"
					:never-empty="true"
					aria-label="Game version type"
					size="small"
					hide-checkmark-icon
					@update:model-value="(type) => setGameVersionType(type, setSelectedValues)"
				/>
			</div>
		</template>

		<template #category-footer="{ category, setSelectedValues, closeMenu }">
			<DownloadsThresholdInput
				v-if="category.key === 'country'"
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
				class="border-0 border-t border-solid border-surface-5 px-6 py-2.5"
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
import {
	Chips,
	DropdownFilterBar,
	type DropdownFilterBarCategory,
	type DropdownFilterBarOption,
} from '@modrinth/ui'

import { useFormattedCountries } from '@/composables/country.ts'
import { useGeneratedState } from '~/composables/generated'
import {
	type AnalyticsQueryFilterCategory,
	type AnalyticsSelectedFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import DownloadsThresholdInput from '../DownloadsThresholdInput.vue'
import {
	cloneSelectedFilters,
	FILTER_VALUE_CATEGORIES,
	getAnalyticsFilterCategoryForBreakdown,
	getOptionsWithSelectedValues,
	getVisibleAnalyticsFilterCategoriesForState,
	normalizeSelectedValues as normalizeSelectedFilterValues,
} from './queryFilter'

type AnalyticsFilterValueCategory = Exclude<AnalyticsQueryFilterCategory, 'project'>
type SetDropdownFilterValues = (values: string[]) => void
type ApplyDownloadsThreshold = (setSelectedValues: SetDropdownFilterValues) => void
type CloseDownloadsThresholdMenu = (event?: Event) => void

const {
	hasProjectContext,
	availableProjectStatuses,
	filterOptions,
	projectVersionDownloadsById,
	gameVersionDownloadsByVersion,
	countryDownloadsByCode,
	selectedBreakdown,
	selectedFilters,
	queryResetToken,
	refreshAnalyticsQuery,
	getVersionDisplayName,
	getVersionPublishedDate,
} = injectAnalyticsDashboardContext()
const formattedCountries = useFormattedCountries()
const generatedState = useGeneratedState()

const gameVersionType = ref<'release' | 'all'>('release')
const countryDownloadsThreshold = ref<number | null>(null)
const projectVersionDownloadsThreshold = ref<number | null>(null)
const gameVersionDownloadsThreshold = ref<number | null>(null)
const gameVersionTypeOptions: Array<'release' | 'all'> = ['release', 'all']
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

const selectedFilterValue = computed<Record<string, string[]>>({
	get: () => selectedFilters.value,
	set: (nextValue) => {
		const nextFilters = cloneSelectedFilters(selectedFilters.value)

		for (const [categoryKey, values] of Object.entries(nextValue)) {
			if (!isAnalyticsFilterValueCategory(categoryKey)) {
				continue
			}

			nextFilters[categoryKey] = normalizeSelectedFilterValues(categoryKey, values, [])
		}

		const breakdownFilterCategory = getAnalyticsFilterCategoryForBreakdown(selectedBreakdown.value)
		if (breakdownFilterCategory && nextFilters[breakdownFilterCategory].length > 0) {
			selectedBreakdown.value = 'none'
		}

		selectedFilters.value = nextFilters
	},
})

function clearSelectedBreakdown() {
	selectedBreakdown.value = 'none'
}

function clearFilterBar() {
	clearSelectedBreakdown()
	clearDownloadsThresholds()
}

watch(queryResetToken, () => {
	clearDownloadsThresholds()
})

watch(
	selectedFilters,
	(nextFilters, previousFilters) => {
		clearDownloadsThresholdsForChangedFilters(previousFilters, nextFilters)
	},
	{ deep: true },
)

const filterCategories = computed<DropdownFilterBarCategory[]>(() => {
	const visibleCategoryKeys = new Set(
		getVisibleAnalyticsFilterCategoriesForState(selectedBreakdown.value, selectedFilters.value),
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
			options: withSelectedOptions('country', countryFilterOptions.value),
			submenuClass: 'w-[324px]',
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
			key: 'download_source',
			label: 'Download Source',
			searchable: downloadSourceFilterOptions.value.length > 6,
			searchPlaceholder: 'Search download sources...',
			options: withSelectedOptions('download_source', downloadSourceFilterOptions.value),
		},
		{
			key: 'download_reason',
			label: 'Download Type',
			options: withSelectedOptions('download_reason', downloadReasonFilterOptions.value),
		},
		{
			key: 'version_id',
			label: 'Project version',
			searchable: versionFilterOptions.value.length > 6,
			searchPlaceholder: 'Search project versions...',
			submenuClass: 'w-[368px]',
			options: withSelectedOptions('version_id', versionFilterOptions.value),
		},
		{
			key: 'game_version',
			label: 'Game Version',
			searchable: true,
			searchPlaceholder: 'Search versions...',
			submenuClass: 'w-[360px]',
			options: withSelectedOptions('game_version', gameVersionFilterOptions.value),
		},
		{
			key: 'loader_type',
			label: 'Loader Type',
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
			label: downloadSource,
		}))
		.sort((left, right) => left.label.localeCompare(right.label)),
)

const downloadReasonFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.downloadReasons.map((downloadReason) => ({
		value: downloadReason,
		label: getDownloadReasonFilterOptionLabel(downloadReason),
	})),
)

const versionFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.versionIds
		.map((versionId) => ({
			value: versionId,
			label: getVersionDisplayName(versionId),
			searchTerms: [versionId],
		}))
		.sort((left, right) =>
			compareOptionalDateStringsDescending(
				getVersionPublishedDate(left.value),
				getVersionPublishedDate(right.value),
				left.label,
				right.label,
			),
		),
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
	if (categoryKey === 'loader_type') {
		return getLoaderTypeFilterOptionLabel
	}
	return undefined
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

	const selectedValues = versionFilterOptions.value
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
	await nextTick()
	await refreshAnalyticsQuery()
}

function setGameVersionType(
	type: 'release' | 'all' | null | undefined,
	setSelectedValues: SetDropdownFilterValues,
) {
	if (type === null || type === undefined) {
		return
	}

	gameVersionType.value = type
	applyGameVersionDownloadsThreshold(setSelectedValues)
}
</script>
