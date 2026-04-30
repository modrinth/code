<template>
	<div class="flex flex-wrap items-center gap-2">
		<span class="text-base font-medium text-primary">Filtered by</span>

		<MultiSelect
			v-for="preview in appliedFilterPreviews"
			:key="preview.key"
			:model-value="getPreviewSelectedValues(preview.key)"
			:options="preview.options"
			:max-height="500"
			:clearable="false"
			:show-chevron="false"
			:fit-content="true"
			:trigger-class="previewTriggerClass"
			@update:model-value="(nextValue) => setPreviewSelectedValues(preview.key, nextValue)"
			@open="openPreviewFilterDraft(preview.key)"
			@close="commitPreviewFilterDraft(preview.key)"
		>
			<template #input-content="{ isOpen }">
				<div class="flex min-w-0 items-center gap-2">
					<span class="truncate">
						<span class="font-medium">{{ preview.label }}:</span>
						<span class="ml-1 font-semibold text-contrast">{{ preview.summary }}</span>
					</span>
					<ChevronDownIcon
						class="size-4 shrink-0 text-secondary transition-transform duration-150"
						:class="isOpen ? 'rotate-180' : ''"
					/>
					<button
						type="button"
						class="-mr-1 inline-flex size-5 shrink-0 items-center justify-center rounded-full border-0 bg-transparent text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
						:aria-label="`Clear ${preview.label} filter`"
						@click.stop="clearFilterCategory(preview.key)"
					>
						<XIcon class="size-4" />
					</button>
				</div>
			</template>
		</MultiSelect>

		<button
			ref="addMenuTrigger"
			type="button"
			class="inline-flex h-10 items-center gap-2 rounded-xl border border-dashed border-surface-5 bg-surface-2 px-3 py-1.5 text-sm font-semibold text-primary transition-colors hover:bg-surface-4"
			:aria-expanded="isAddMenuOpen"
			aria-haspopup="menu"
			@click="handleAddMenuTriggerClick"
			@keydown="handleAddMenuTriggerKeydown"
		>
			<PlusIcon class="size-5" />
			Add
		</button>

		<Teleport to="#teleports">
			<Transition
				enter-active-class="transition-opacity duration-150"
				leave-active-class="transition-none duration-0"
				enter-from-class="opacity-0"
				leave-to-class="opacity-0"
			>
				<div
					v-if="isAddMenuOpen"
					ref="menuContainer"
					class="fixed z-[9999] flex flex-col gap-2 overflow-hidden rounded-[14px] border border-solid border-surface-5 bg-surface-4 p-3 shadow-2xl"
					:style="addMenuStyle"
					role="menu"
					@mousedown.stop
					@keydown="handleAddMenuKeydown"
					@mousemove="(event) => handleMenuMouseMove(event, 'menu')"
				>
					<button
						v-for="category in filterCategories"
						:key="category.key"
						:ref="(element) => setCategoryButtonRef(category.key, element)"
						type="button"
						class="flex w-full appearance-none items-center justify-between gap-1 rounded-xl border-0 p-3 text-left text-base font-medium text-primary shadow-none transition-colors duration-150 hover:bg-surface-5 focus:bg-surface-5"
						:class="category.key === activeCategoryKey ? 'bg-surface-5' : ''"
						role="menuitem"
						@mouseenter="handleCategoryMouseEnter(category.key)"
						@focus="activateCategory(category.key)"
					>
						<span>{{ category.label }}</span>
						<div class="flex items-center gap-1">
							<span
								v-if="getCategorySelectionCount(category.key, 'draft') > 0"
								class="rounded-full bg-surface-5 px-1.5 py-0.5 text-xs text-primary"
							>
								{{ getCategorySelectionCount(category.key, 'draft') }}
							</span>
							<ChevronRightIcon class="size-5 text-secondary" />
						</div>
					</button>
				</div>
			</Transition>
		</Teleport>

		<Teleport to="#teleports">
			<div
				v-if="isAddMenuOpen && activeCategory && hasSubmenuPosition"
				ref="submenu"
				class="fixed z-[10000] min-w-[16rem] rounded-xl border border-solid border-surface-5 bg-surface-4 shadow-xl"
				:style="submenuStyle"
				@mouseenter="handleSubmenuMouseEnter"
				@mouseleave="handleSubmenuMouseLeave"
				@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
			>
				<div class="flex max-h-[min(70vh,32rem)] flex-col gap-2 overflow-y-auto p-3">
					<button
						v-for="option in activeCategory.options"
						:key="`${activeCategory.key}-${option.value}`"
						type="button"
						class="flex w-full cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent p-3 text-left text-contrast shadow-none transition-colors duration-150 hover:bg-surface-5 focus:bg-surface-5"
						:aria-checked="isFilterValueSelected(activeCategory.key, option.value)"
						role="checkbox"
						@click="toggleFilterOption(activeCategory.key, option.value)"
					>
						<span
							class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
							:class="
								isFilterValueSelected(activeCategory.key, option.value)
									? 'border-button-border bg-brand text-brand-inverted'
									: 'border-surface-5 bg-surface-2'
							"
						>
							<CheckIcon
								v-if="isFilterValueSelected(activeCategory.key, option.value)"
								aria-hidden="true"
								stroke-width="3"
							/>
						</span>
						<span
							class="font-semibold leading-tight"
							:class="
								isFilterValueSelected(activeCategory.key, option.value)
									? 'text-contrast'
									: 'text-primary'
							"
						>
							{{ option.label }}
						</span>
					</button>
				</div>
			</div>
		</Teleport>

		<button
			v-if="hasAppliedFilters"
			type="button"
			class="border-0 bg-transparent px-1.5 py-1 text-sm font-medium text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
			@click="clearAllFilters"
		>
			Clear
		</button>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, ChevronDownIcon, ChevronRightIcon, PlusIcon, XIcon } from '@modrinth/assets'
import { MultiSelect, type MultiSelectOption } from '@modrinth/ui'
import { onClickOutside } from '@vueuse/core'
import type { ComponentPublicInstance, CSSProperties } from 'vue'

import { useFormattedCountries } from '@/composables/country.ts'
import {
	type AnalyticsQueryFilterCategory,
	type AnalyticsSelectedFilters,
	injectAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'

import {
	ADD_MENU_WIDTH,
	ALL_FILTER_VALUE,
	type FilterCategory,
	type FilterOption,
	type FilterSelectionSource,
	type Point,
	areSelectedFiltersEqual,
	areStringArraysEqual,
	cloneSelectedFilters,
	getAddMenuPosition,
	getCategorySelectionCount as getCategorySelectionCountValue,
	getCategorySelectionSummary as getCategorySelectionSummaryValue,
	getOptionsWithSelectedValues,
	getSubmenuPosition,
	isCursorAimingAtSubmenu as getIsCursorAimingAtSubmenu,
	isFilterValueSelected as getIsFilterValueSelected,
	normalizeSelectedValues as normalizeSelectedFilterValues,
} from './queryFilter'

const { filterOptions, selectedFilters } = injectAnalyticsDashboardContext()
const formattedCountries = useFormattedCountries()

const isAddMenuOpen = ref(false)
const activeCategoryKey = ref<AnalyticsQueryFilterCategory | null>(null)
const pendingCategoryKey = ref<AnalyticsQueryFilterCategory | null>(null)
const draftSelectedFilters = ref<AnalyticsSelectedFilters>(
	cloneSelectedFilters(selectedFilters.value),
)
const previewSelectedValueDrafts = ref<Partial<Record<AnalyticsQueryFilterCategory, string[]>>>({})
const lastMousePosition = ref<Point | null>(null)
const isCursorInsideSubmenu = ref(false)
const hasSubmenuPosition = ref(false)
const addMenuTrigger = ref<HTMLElement | null>(null)
const menuContainer = ref<HTMLElement | null>(null)
const submenu = ref<HTMLElement | null>(null)
const addMenuStyle = ref<CSSProperties>({
	left: '0px',
	minWidth: '0px',
	top: '0px',
	width: `${ADD_MENU_WIDTH}px`,
})
const submenuPosition = ref<Point>({ x: 0, y: 0 })
const categoryButtonRefs = new Map<AnalyticsQueryFilterCategory, HTMLElement>()
let pendingCategoryTimeout: NodeJS.Timeout | null = null
let previousMousePosition: Point | null = null

const filterCategories = computed<FilterCategory[]>(() => [
	{
		key: 'country',
		label: 'Country',
		allLabel: 'All countries',
		options: withSelectedOptions('country', [
			{ value: ALL_FILTER_VALUE, label: 'All countries' },
			...countryFilterOptions.value,
		]),
	},
	{
		key: 'monetization',
		label: 'Monetization',
		allLabel: 'All',
		options: [
			{ value: ALL_FILTER_VALUE, label: 'All' },
			{ value: 'monetized', label: 'Monetized' },
			{ value: 'unmonetized', label: 'Unmonetized' },
		],
	},
	{
		key: 'download_source',
		label: 'Download Source',
		allLabel: 'All download sources',
		options: withSelectedOptions('download_source', [
			{ value: ALL_FILTER_VALUE, label: 'All download sources' },
		]),
	},
	{
		key: 'version_id',
		label: 'Project version',
		allLabel: 'All project versions',
		options: withSelectedOptions('version_id', [
			{ value: ALL_FILTER_VALUE, label: 'All project versions' },
		]),
	},
	{
		key: 'game_version',
		label: 'Game Version',
		allLabel: 'All game versions',
		options: withSelectedOptions('game_version', [
			{ value: ALL_FILTER_VALUE, label: 'All game versions' },
		]),
	},
	{
		key: 'loader_type',
		label: 'Loader Type',
		allLabel: 'All loader types',
		options: withSelectedOptions('loader_type', [
			{ value: ALL_FILTER_VALUE, label: 'All loader types' },
		]),
	},
])

const filterCategoriesByKey = computed(
	() => new Map(filterCategories.value.map((category) => [category.key, category] as const)),
)

const activeCategory = computed(() =>
	activeCategoryKey.value ? filterCategoriesByKey.value.get(activeCategoryKey.value) : undefined,
)

const submenuStyle = computed<CSSProperties>(() => ({
	left: `${submenuPosition.value.x}px`,
	top: `${submenuPosition.value.y}px`,
}))

const appliedFilterPreviews = computed(() =>
	filterCategories.value
		.map((category) => ({
			key: category.key,
			label: category.label,
			summary: getCategorySelectionSummary(category),
			count: getCategorySelectionCount(category.key),
			category,
			options: category.options.map((option) => ({
				value: option.value,
				label: option.label,
			})) as MultiSelectOption<string>[],
		}))
		.filter((preview) => preview.count > 0),
)

const hasAppliedFilters = computed(() => appliedFilterPreviews.value.length > 0)
const previewTriggerClass =
	'h-10 max-w-[16rem] border border-solid border-surface-5 bg-surface-4 px-3 py-1.5 hover:bg-surface-5 hover:brightness-100 active:brightness-100'

const countryLabelsByCode = computed(
	() =>
		new Map(
			formattedCountries.value.map(
				(country) => [country.value.toUpperCase(), country.label] as const,
			),
		),
)

const countryFilterOptions = computed<FilterOption[]>(() =>
	filterOptions.value.countries
		.map((countryCode) => ({
			value: countryCode,
			label: getCountryFilterOptionLabel(countryCode),
		}))
		.sort((left, right) => left.label.localeCompare(right.label)),
)

function getCountryFilterOptionLabel(countryCode: string): string {
	const normalizedCode = countryCode.trim().toUpperCase()
	if (normalizedCode === 'XX') {
		return 'Other'
	}

	return countryLabelsByCode.value.get(normalizedCode) ?? countryCode
}

function resetAddMenuDraft() {
	draftSelectedFilters.value = cloneSelectedFilters(selectedFilters.value)
}

function commitAddMenuDraft() {
	if (!areSelectedFiltersEqual(selectedFilters.value, draftSelectedFilters.value)) {
		selectedFilters.value = cloneSelectedFilters(draftSelectedFilters.value)
	}
}

function getSelectedValues(
	categoryKey: AnalyticsQueryFilterCategory,
	source: FilterSelectionSource = 'committed',
): string[] {
	if (source === 'draft') {
		return draftSelectedFilters.value[categoryKey]
	}

	return selectedFilters.value[categoryKey]
}

function setSelectedValues(
	categoryKey: AnalyticsQueryFilterCategory,
	values: string[],
	source: FilterSelectionSource = 'committed',
) {
	const normalizedValues = normalizeSelectedFilterValues(categoryKey, values, [])

	const currentFilters = source === 'draft' ? draftSelectedFilters.value : selectedFilters.value
	if (areStringArraysEqual(currentFilters[categoryKey], normalizedValues)) {
		return
	}

	const nextFilters = {
		...currentFilters,
		[categoryKey]: normalizedValues,
	}

	if (source === 'draft') {
		draftSelectedFilters.value = nextFilters
	} else {
		selectedFilters.value = nextFilters
	}
}

function openAddMenu() {
	if (isAddMenuOpen.value) {
		return
	}

	commitPreviewFilterDrafts()
	resetAddMenuDraft()
	isAddMenuOpen.value = true
}

function closeAddMenu() {
	if (!isAddMenuOpen.value) {
		return
	}

	commitAddMenuDraft()
	isAddMenuOpen.value = false
}

function handleAddMenuTriggerClick(event: MouseEvent) {
	if (event.detail === 0) {
		return
	}

	if (isAddMenuOpen.value) {
		closeAddMenu()
	} else {
		openAddMenu()
	}
}

function handleAddMenuTriggerKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'Enter':
		case ' ':
		case 'ArrowDown':
			event.preventDefault()
			openAddMenu()
			break
		case 'Escape':
			event.preventDefault()
			closeAddMenu()
			break
	}
}

function handleAddMenuKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape') {
		return
	}

	event.preventDefault()
	closeAddMenu()
	nextTick(() => addMenuTrigger.value?.focus())
}

function setCategoryButtonRef(
	categoryKey: AnalyticsQueryFilterCategory,
	element: Element | ComponentPublicInstance | null,
) {
	if (element instanceof HTMLElement) {
		categoryButtonRefs.set(categoryKey, element)
		if (isAddMenuOpen.value && categoryKey === activeCategoryKey.value) {
			scheduleSubmenuPositionUpdate()
		}
	} else {
		categoryButtonRefs.delete(categoryKey)
	}
}

function withSelectedOptions(
	categoryKey: AnalyticsQueryFilterCategory,
	options: FilterOption[],
): FilterOption[] {
	const selectedValues = getSelectedValues(categoryKey, isAddMenuOpen.value ? 'draft' : 'committed')
	return getOptionsWithSelectedValues(
		options,
		selectedValues,
		categoryKey === 'country' ? getCountryFilterOptionLabel : undefined,
	)
}

function isFilterValueSelected(categoryKey: AnalyticsQueryFilterCategory, value: string): boolean {
	const selectedValues = getSelectedValues(categoryKey, 'draft')
	return getIsFilterValueSelected(categoryKey, value, selectedValues, 0)
}

function toggleFilterValue(
	categoryKey: AnalyticsQueryFilterCategory,
	value: string,
	nextValue: boolean,
) {
	const currentValues = getSelectedValues(categoryKey, 'draft')

	if (value === ALL_FILTER_VALUE) {
		if (nextValue) {
			setSelectedValues(categoryKey, [ALL_FILTER_VALUE], 'draft')
		}
		return
	}

	if (nextValue) {
		if (!currentValues.includes(value)) {
			setSelectedValues(categoryKey, [...currentValues, value], 'draft')
		}
	} else {
		setSelectedValues(
			categoryKey,
			currentValues.filter((item) => item !== value),
			'draft',
		)
	}
}

function toggleFilterOption(categoryKey: AnalyticsQueryFilterCategory, value: string) {
	toggleFilterValue(categoryKey, value, !isFilterValueSelected(categoryKey, value))
}

function getCategorySelectionCount(
	categoryKey: AnalyticsQueryFilterCategory,
	source: FilterSelectionSource = 'committed',
): number {
	const selectedValues = getSelectedValues(categoryKey, source)
	return getCategorySelectionCountValue(categoryKey, selectedValues, 0)
}

function getCategorySelectionSummary(category: FilterCategory): string {
	const count = getCategorySelectionCount(category.key)
	return getCategorySelectionSummaryValue(category, getSelectedValues(category.key), count, [])
}

function clearFilterCategory(categoryKey: AnalyticsQueryFilterCategory) {
	commitPreviewFilterDrafts()
	setSelectedValues(categoryKey, [ALL_FILTER_VALUE])
}

function clearAllFilters() {
	commitPreviewFilterDrafts()
	for (const category of filterCategories.value) {
		setSelectedValues(category.key, [])
	}
}

function getPreviewSelectedValues(categoryKey: AnalyticsQueryFilterCategory): string[] {
	const draftValues = previewSelectedValueDrafts.value[categoryKey]
	if (draftValues !== undefined) {
		return draftValues
	}

	return getSelectedValues(categoryKey)
}

function setPreviewSelectedValues(categoryKey: AnalyticsQueryFilterCategory, values: string[]) {
	previewSelectedValueDrafts.value = {
		...previewSelectedValueDrafts.value,
		[categoryKey]: normalizeSelectedFilterValues(categoryKey, values, []),
	}
}

function openPreviewFilterDraft(categoryKey: AnalyticsQueryFilterCategory) {
	commitPreviewFilterDrafts()
	previewSelectedValueDrafts.value = {
		...previewSelectedValueDrafts.value,
		[categoryKey]: [...getSelectedValues(categoryKey)],
	}
}

function commitPreviewFilterDraft(categoryKey: AnalyticsQueryFilterCategory) {
	const draftValues = previewSelectedValueDrafts.value[categoryKey]
	if (draftValues === undefined) {
		return
	}

	const nextDrafts = { ...previewSelectedValueDrafts.value }
	delete nextDrafts[categoryKey]
	previewSelectedValueDrafts.value = nextDrafts
	setSelectedValues(categoryKey, draftValues)
}

function commitPreviewFilterDrafts() {
	for (const categoryKey of Object.keys(
		previewSelectedValueDrafts.value,
	) as AnalyticsQueryFilterCategory[]) {
		commitPreviewFilterDraft(categoryKey)
	}
}

function activateCategory(categoryKey: AnalyticsQueryFilterCategory) {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	activeCategoryKey.value = categoryKey
	scheduleSubmenuPositionUpdate()
}

function handleCategoryMouseEnter(categoryKey: AnalyticsQueryFilterCategory) {
	if (!activeCategoryKey.value) {
		activateCategory(categoryKey)
		return
	}

	if (categoryKey === activeCategoryKey.value) {
		clearPendingCategoryTimeout()
		pendingCategoryKey.value = null
		return
	}

	if (!isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)) {
		activateCategory(categoryKey)
		return
	}

	pendingCategoryKey.value = categoryKey
	clearPendingCategoryTimeout()
	pendingCategoryTimeout = setTimeout(() => {
		if (pendingCategoryKey.value !== categoryKey) {
			return
		}

		if (
			isCursorInsideSubmenu.value ||
			isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)
		) {
			pendingCategoryKey.value = null
			return
		}

		activateCategory(categoryKey)
	}, 180)
}

function handleMenuMouseMove(event: MouseEvent, source: 'menu' | 'submenu') {
	previousMousePosition = lastMousePosition.value
	lastMousePosition.value = {
		x: event.clientX,
		y: event.clientY,
	}

	if (source !== 'menu') {
		return
	}

	if (
		!pendingCategoryKey.value ||
		isCursorAimingAtSubmenu(lastMousePosition.value, previousMousePosition)
	) {
		return
	}

	activateCategory(pendingCategoryKey.value)
}

function isCursorAimingAtSubmenu(cursor: Point | null, origin: Point | null): boolean {
	return getIsCursorAimingAtSubmenu(cursor, origin, submenu.value?.getBoundingClientRect() ?? null)
}

function clearPendingCategoryTimeout() {
	if (pendingCategoryTimeout) {
		clearTimeout(pendingCategoryTimeout)
		pendingCategoryTimeout = null
	}
}

function updateAddMenuPosition(): boolean {
	if (typeof window === 'undefined' || !addMenuTrigger.value || !menuContainer.value) {
		return false
	}

	const triggerRect = addMenuTrigger.value.getBoundingClientRect()
	const dropdownWidth = Math.max(ADD_MENU_WIDTH, triggerRect.width)

	addMenuStyle.value = {
		...addMenuStyle.value,
		minWidth: `${triggerRect.width}px`,
		width: `${dropdownWidth}px`,
	}

	const dropdownRect = menuContainer.value.getBoundingClientRect()
	addMenuStyle.value = getAddMenuPosition({
		triggerRect,
		dropdownRect,
		viewportWidth: window.innerWidth,
		viewportHeight: window.innerHeight,
	})
	return true
}

function scheduleAddMenuPositionUpdate(retries = 8) {
	if (typeof window === 'undefined') {
		return
	}

	nextTick(() => {
		if (!isAddMenuOpen.value || updateAddMenuPosition() || retries <= 0) {
			return
		}

		setTimeout(() => scheduleAddMenuPositionUpdate(retries - 1), 0)
	})
}

function updateSubmenuPosition(): boolean {
	if (typeof window === 'undefined') {
		return false
	}

	if (!activeCategoryKey.value) {
		return false
	}

	const activeButton = categoryButtonRefs.get(activeCategoryKey.value)
	if (!activeButton) {
		return false
	}

	const buttonRect = activeButton.getBoundingClientRect()
	const submenuRect = submenu.value?.getBoundingClientRect()
	const submenuWidth = submenuRect?.width ?? 256
	const submenuHeight = submenuRect?.height ?? 320

	submenuPosition.value = getSubmenuPosition({
		buttonRect,
		submenuWidth,
		submenuHeight,
		viewportWidth: window.innerWidth,
		viewportHeight: window.innerHeight,
	})
	hasSubmenuPosition.value = true
	return true
}

function scheduleSubmenuPositionUpdate(retries = 8) {
	if (typeof window === 'undefined') {
		return
	}

	nextTick(() => {
		if (!isAddMenuOpen.value || updateSubmenuPosition() || retries <= 0) {
			return
		}

		setTimeout(() => scheduleSubmenuPositionUpdate(retries - 1), 0)
	})
}

function resetPendingCategory() {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	isCursorInsideSubmenu.value = false
	lastMousePosition.value = null
	previousMousePosition = null
}

function handleSubmenuMouseEnter() {
	isCursorInsideSubmenu.value = true
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
}

function handleSubmenuMouseLeave() {
	isCursorInsideSubmenu.value = false
}

function updateMenuPositions() {
	if (!isAddMenuOpen.value) {
		return
	}

	updateAddMenuPosition()
	updateSubmenuPosition()
}

function startAddMenuPositionTracking() {
	if (typeof window === 'undefined') {
		return
	}

	window.addEventListener('resize', updateMenuPositions)
	window.addEventListener('scroll', updateMenuPositions, true)
}

function stopAddMenuPositionTracking() {
	if (typeof window === 'undefined') {
		return
	}

	window.removeEventListener('resize', updateMenuPositions)
	window.removeEventListener('scroll', updateMenuPositions, true)
}

onClickOutside(
	menuContainer,
	() => {
		closeAddMenu()
	},
	{ ignore: [addMenuTrigger, submenu] },
)

watch(isAddMenuOpen, (isOpen) => {
	if (isOpen) {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
		scheduleAddMenuPositionUpdate()
		startAddMenuPositionTracking()
	} else {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
		resetPendingCategory()
		stopAddMenuPositionTracking()
	}
})

onBeforeUnmount(() => {
	clearPendingCategoryTimeout()
	stopAddMenuPositionTracking()
	if (isAddMenuOpen.value) {
		commitAddMenuDraft()
	}
	commitPreviewFilterDrafts()
})
</script>

<style lang="scss" scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}
</style>
