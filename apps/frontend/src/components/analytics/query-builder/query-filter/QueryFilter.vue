<template>
	<DropdownFilterBar
		v-model="selectedFilterValue"
		:categories="filterCategories"
		:show-clear="selectedBreakdown !== 'none'"
		@clear="clearFilterBar"
	>
		<template #search-actions="{ category, setSelectedValues }">
			<div v-if="category.key === 'game_version'" class="flex w-[150px] justify-end">
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

		<template #option-right="{ category, option }">
			<span
				v-if="category.key === 'version_id' && getProjectVersionOptionProjectMetadata(option.value)"
				v-tooltip="getProjectVersionOptionProjectMetadata(option.value)?.name"
				class="flex size-6 shrink-0 items-center justify-center overflow-hidden rounded text-primary"
			>
				<img
					v-if="getProjectVersionOptionProjectMetadata(option.value)?.iconUrl"
					:src="getProjectVersionOptionProjectMetadata(option.value)?.iconUrl"
					:alt="`${getProjectVersionOptionProjectMetadata(option.value)?.name} Icon`"
					class="h-6 w-6 rounded object-cover"
				/>
				<BoxIcon v-else class="h-full w-full" />
			</span>
		</template>

		<template #category-top="{ category, selectedValues, setSelectedValues }">
			<div v-if="shouldShowTopBreakdownFilterForCategory(category.key)">
				<button
					type="button"
					class="flex w-full cursor-pointer items-center gap-2.5 border-0 bg-surface-4 p-3 py-3.5 text-left text-contrast shadow-none transition-all duration-150 hover:brightness-110 focus:brightness-110"
					:aria-selected="isTopBreakdownFilterValueSelected(selectedValues)"
					role="option"
					@click="selectTopBreakdownFilter(setSelectedValues)"
					@keydown.enter.stop
					@keydown.space.stop
				>
					<span
						class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
						:class="
							isTopBreakdownFilterValueSelected(selectedValues)
								? 'border-button-border bg-brand text-brand-inverted'
								: 'border-surface-5 bg-surface-2'
						"
					>
						<CheckIcon
							v-if="isTopBreakdownFilterValueSelected(selectedValues)"
							aria-hidden="true"
							stroke-width="3"
						/>
					</span>
					<span class="min-w-0 flex-1 font-semibold leading-tight text-primary">Top 8</span>
				</button>
			</div>
		</template>

		<template #preview-top="{ category, selectedValues, setSelectedValues }">
			<div v-if="shouldShowTopBreakdownFilterForCategory(category.key)">
				<button
					type="button"
					class="flex w-full cursor-pointer items-center gap-2.5 border-0 bg-surface-4 p-3 py-3.5 text-left text-contrast shadow-none transition-all duration-150 hover:brightness-110 focus:brightness-110"
					:aria-selected="isTopBreakdownFilterValueSelected(selectedValues)"
					role="option"
					@click="selectTopBreakdownFilter(setSelectedValues)"
					@keydown.enter.stop
					@keydown.space.stop
				>
					<span
						class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
						:class="
							isTopBreakdownFilterValueSelected(selectedValues)
								? 'border-button-border bg-brand text-brand-inverted'
								: 'border-surface-5 bg-surface-2'
						"
					>
						<CheckIcon
							v-if="isTopBreakdownFilterValueSelected(selectedValues)"
							aria-hidden="true"
							stroke-width="3"
						/>
					</span>
					<span class="min-w-0 flex-1 font-semibold leading-tight text-primary">Top 8</span>
				</button>
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
import { BoxIcon, CheckIcon } from '@modrinth/assets'
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
	doesProjectStatusMatchFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import { getDownloadSourceLabel } from '../../breakdown'
import DownloadsThresholdInput from '../DownloadsThresholdInput.vue'
import {
	areSelectedFiltersEqual,
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
type ProjectVersionProjectOptionMetadata = {
	name: string
	iconUrl?: string
}

const TOP_BREAKDOWN_FILTER_VALUE = '__analytics_top_8__'
const TOP_BREAKDOWN_FILTER_LIMIT = 8
const TOP_BREAKDOWN_FILTER_OPTION: DropdownFilterBarOption = {
	value: TOP_BREAKDOWN_FILTER_VALUE,
	label: 'Top 8',
}

const {
	hasProjectContext,
	projects,
	selectedProjectIds,
	availableProjectStatuses,
	filterOptions,
	projectVersionDownloadsById,
	gameVersionDownloadsByVersion,
	countryDownloadsByCode,
	selectedBreakdown,
	selectedFilters,
	isTopBreakdownFilterEnabled,
	queryResetToken,
	refreshAnalyticsQuery,
	getVersionDisplayName,
	getVersionPublishedDate,
	getVersionProjectName,
	getVersionProjectIconUrl,
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
const draftSelectedFilters = ref<AnalyticsSelectedFilters>(
	cloneSelectedFilters(selectedFilters.value),
)
let selectedFiltersCommitRequestId = 0

const selectedFilterValue = computed<Record<string, string[]>>({
	get: () => getSelectedFilterBarValue(),
	set: (nextValue) => {
		const nextFilters = cloneSelectedFilters(draftSelectedFilters.value)
		const topBreakdownCategory = topBreakdownFilterCategory.value
		let nextTopBreakdownFilterEnabled = isTopBreakdownFilterEnabled.value

		for (const [categoryKey, values] of Object.entries(nextValue)) {
			if (!isAnalyticsFilterValueCategory(categoryKey)) {
				continue
			}

			const concreteValues = values.filter((value) => value !== TOP_BREAKDOWN_FILTER_VALUE)
			nextFilters[categoryKey] = normalizeSelectedFilterValues(categoryKey, concreteValues, [])

			if (
				categoryKey === topBreakdownCategory &&
				shouldShowTopBreakdownFilterForCategory(categoryKey)
			) {
				const hasTopBreakdownFilterValue = values.includes(TOP_BREAKDOWN_FILTER_VALUE)
				const hasConcreteValues = concreteValues.length > 0

				if (hasTopBreakdownFilterValue && !hasConcreteValues) {
					nextTopBreakdownFilterEnabled = true
				} else if (hasConcreteValues || isTopBreakdownFilterSelected.value) {
					nextTopBreakdownFilterEnabled = false
				}
			}
		}

		draftSelectedFilters.value = nextFilters
		if (isTopBreakdownFilterEnabled.value !== nextTopBreakdownFilterEnabled) {
			isTopBreakdownFilterEnabled.value = nextTopBreakdownFilterEnabled
		}
		void scheduleSelectedFiltersCommit()
	},
})

const topBreakdownFilterCategory = computed(() =>
	getAnalyticsFilterCategoryForBreakdown(selectedBreakdown.value),
)
const isTopBreakdownFilterSelected = computed(() => {
	const categoryKey = topBreakdownFilterCategory.value
	if (!categoryKey || !shouldShowTopBreakdownFilterForCategory(categoryKey)) {
		return false
	}

	return isTopBreakdownFilterEnabled.value && draftSelectedFilters.value[categoryKey].length === 0
})

function getSelectedFilterBarValue(): AnalyticsSelectedFilters {
	const nextValue = cloneSelectedFilters(draftSelectedFilters.value)
	const categoryKey = topBreakdownFilterCategory.value
	if (!categoryKey || !isTopBreakdownFilterSelected.value) {
		return nextValue
	}

	nextValue[categoryKey] = [TOP_BREAKDOWN_FILTER_VALUE]
	return nextValue
}

function isTopBreakdownFilterValueSelected(selectedValues: string[]): boolean {
	return selectedValues.includes(TOP_BREAKDOWN_FILTER_VALUE)
}

function selectTopBreakdownFilter(setSelectedValues: SetDropdownFilterValues) {
	setSelectedValues([TOP_BREAKDOWN_FILTER_VALUE])
}

function clearSelectedBreakdown() {
	selectedBreakdown.value = 'none'
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
			syntheticOptions: getTopBreakdownSyntheticOptions('country'),
			submenuClass: 'w-[324px]',
		},
		{
			key: 'monetization',
			label: 'Monetization',
			options: withSelectedOptions('monetization', [
				{ value: 'monetized', label: 'Monetized' },
				{ value: 'unmonetized', label: 'Unmonetized' },
			]),
			syntheticOptions: getTopBreakdownSyntheticOptions('monetization'),
		},
		{
			key: 'user_agent',
			label: 'Download Source',
			searchable: downloadSourceFilterOptions.value.length > 6,
			searchPlaceholder: 'Search download sources...',
			options: withSelectedOptions('user_agent', downloadSourceFilterOptions.value),
			syntheticOptions: getTopBreakdownSyntheticOptions('user_agent'),
		},
		{
			key: 'download_reason',
			label: 'Download Type',
			options: withSelectedOptions('download_reason', downloadReasonFilterOptions.value),
			syntheticOptions: getTopBreakdownSyntheticOptions('download_reason'),
		},
		{
			key: 'version_id',
			label: 'Project version',
			searchable: versionFilterOptions.value.length > 6,
			searchPlaceholder: 'Search project versions...',
			submenuClass: 'w-[368px]',
			options: withSelectedOptions('version_id', versionFilterOptions.value),
			syntheticOptions: getTopBreakdownSyntheticOptions('version_id'),
		},
		{
			key: 'game_version',
			label: 'Game Version',
			searchable: true,
			searchPlaceholder: 'Search versions...',
			submenuClass: 'w-[360px]',
			options: withSelectedOptions('game_version', gameVersionFilterOptions.value),
			syntheticOptions: getTopBreakdownSyntheticOptions('game_version'),
		},
		{
			key: 'loader_type',
			label: 'Loader Type',
			options: withSelectedOptions('loader_type', loaderTypeFilterOptions.value),
			syntheticOptions: getTopBreakdownSyntheticOptions('loader_type'),
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

const versionFilterOptions = computed<DropdownFilterBarOption[]>(() =>
	filterOptions.value.versionIds
		.map((versionId) => {
			const projectName = showProjectVersionProjectIcons.value
				? getVersionProjectName(versionId)
				: undefined

			return {
				value: versionId,
				label: getVersionDisplayName(versionId),
				searchTerms: projectName ? [versionId, projectName] : [versionId],
			}
		})
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

function getTopBreakdownSyntheticOptions(
	categoryKey: AnalyticsFilterValueCategory,
): DropdownFilterBarOption[] | undefined {
	return shouldShowTopBreakdownFilterForCategory(categoryKey)
		? [TOP_BREAKDOWN_FILTER_OPTION]
		: undefined
}

function shouldShowTopBreakdownFilterForCategory(categoryKey: string): boolean {
	return (
		categoryKey === topBreakdownFilterCategory.value &&
		isAnalyticsFilterValueCategory(categoryKey) &&
		getFilterOptionCountForCategory(categoryKey) > TOP_BREAKDOWN_FILTER_LIMIT
	)
}

function getFilterOptionCountForCategory(categoryKey: AnalyticsFilterValueCategory): number {
	switch (categoryKey) {
		case 'project_status':
			return projectStatusFilterOptions.value.length
		case 'country':
			return countryFilterOptions.value.length
		case 'monetization':
			return 2
		case 'user_agent':
			return downloadSourceFilterOptions.value.length
		case 'download_reason':
			return downloadReasonFilterOptions.value.length
		case 'version_id':
			return versionFilterOptions.value.length
		case 'game_version':
			return gameVersionFilterOptions.value.length
		case 'loader_type':
			return loaderTypeFilterOptions.value.length
	}
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

function getProjectVersionOptionProjectMetadata(
	versionId: string,
): ProjectVersionProjectOptionMetadata | undefined {
	if (!showProjectVersionProjectIcons.value) {
		return undefined
	}

	const projectName = getVersionProjectName(versionId)
	return projectName
		? {
				name: projectName,
				iconUrl: getVersionProjectIconUrl(versionId),
			}
		: undefined
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
	await scheduleSelectedFiltersCommit()
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
