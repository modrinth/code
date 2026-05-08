<template>
	<span
		class="flex h-10 items-center text-nowrap text-base font-medium text-primary"
		:aria-label="useFilterIcon ? label : undefined"
	>
		<FilterIcon v-if="useFilterIcon" class="size-5 text-primary" aria-hidden="true" />
		<template v-else>{{ label }}</template>
	</span>

	<MultiSelect
		v-for="preview in appliedFilterPreviews"
		:key="preview.key"
		:model-value="getPreviewSelectedValues(preview.key)"
		:options="preview.options"
		:max-height="500"
		:clearable="false"
		:show-chevron="false"
		:fit-content="true"
		:searchable="preview.category.searchable"
		:search-placeholder="preview.category.searchPlaceholder"
		:trigger-class="previewTriggerClass"
		:dropdown-min-width="preview.category.previewDropdownMinWidth ?? '18rem'"
		show-selection-actions
		@update:model-value="(nextValue) => setPreviewSelectedValues(preview.key, nextValue)"
		@open="openPreviewFilterDraft(preview.key)"
		@close="commitPreviewFilterDraft(preview.key)"
	>
		<template #input-content="{ isOpen, openDirection }">
			<div class="flex min-h-8 max-w-80 items-center gap-2">
				<span class="min-w-0 flex-1 truncate">
					<span class="font-medium">{{ preview.label }}:</span>
					<span class="ml-1 font-semibold text-contrast">{{ preview.summary }}</span>
				</span>
				<div class="flex shrink-0 items-center gap-1.5">
					<button
						type="button"
						class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
						:aria-label="`Clear ${preview.label} filter`"
						@click.stop="clearFilterCategory(preview.key)"
					>
						<XIcon class="size-4 text-primary" />
					</button>
					<div class="h-5 w-[1px] shrink-0 bg-surface-5"></div>
					<ChevronLeftIcon
						class="size-5 shrink-0 text-secondary transition-transform duration-150"
						:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
					/>
				</div>
			</div>
		</template>
	</MultiSelect>

	<div class="flex h-10 items-center gap-2">
		<ButtonStyled type="outlined">
			<button
				ref="addMenuTrigger"
				type="button"
				:aria-expanded="isAddMenuOpen"
				aria-haspopup="menu"
				@click="handleAddMenuTriggerClick"
				@keydown="handleAddMenuTriggerKeydown"
			>
				<PlusIcon />
				{{ addLabel }}
			</button>
		</ButtonStyled>

		<ButtonStyled v-if="hasAppliedFilters" type="transparent">
			<button type="button" @click="clearAllFilters">{{ clearLabel }}</button>
		</ButtonStyled>
	</div>

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
					class="group/filter-menu-button flex h-11 w-full appearance-none items-center justify-between gap-1 rounded-xl border-0 px-3 text-left text-base font-medium text-primary shadow-none transition-colors duration-150 hover:bg-surface-5 focus:bg-surface-5"
					:class="category.key === activeCategoryKey ? 'bg-surface-5' : ''"
					role="menuitem"
					@mouseenter="handleCategoryMouseEnter(category.key)"
					@focus="activateCategory(category.key)"
				>
					<span>{{ category.label }}</span>
					<div class="flex items-center gap-1">
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
			class="fixed z-[10000] flex max-h-[min(70vh,32rem)] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-4 shadow-xl"
			:class="activeCategory.submenuClass ?? 'w-72'"
			:style="submenuStyle"
			@mouseenter="handleSubmenuMouseEnter"
			@mouseleave="handleSubmenuMouseLeave"
			@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
		>
			<div
				v-if="activeCategory.searchable"
				class="flex justify-between border-0 border-b border-solid border-b-surface-5 px-3 py-2.5"
			>
				<StyledInput
					v-model="categorySearchQuery"
					:icon="SearchIcon"
					type="text"
					:placeholder="activeCategory.searchPlaceholder ?? 'Search...'"
					wrapper-class="grow bg-surface-4"
				/>
				<slot
					name="search-actions"
					:category="activeCategory"
					:selected-values="activeCategorySelectedValues"
					:set-selected-values="setActiveCategorySelectedValues"
				></slot>
			</div>

			<div
				v-if="activeCategorySelectionCount > 0"
				class="flex items-center justify-between gap-3 border-0 border-b border-solid border-b-surface-5 px-6 py-2.5 text-sm"
			>
				<span class="font-semibold text-secondary">{{ activeCategorySelectionLabel }}</span>
				<button
					type="button"
					class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
					@click="clearActiveCategorySelection"
					@keydown.enter.stop
					@keydown.space.stop
				>
					Deselect all
				</button>
			</div>
			<div class="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-3">
				<div
					v-if="filteredActiveCategoryOptions.length === 0"
					class="px-3 py-2 text-sm font-medium text-secondary"
				>
					{{ activeCategoryEmptyStateLabel }}
				</div>
				<template v-else>
					<button
						v-for="option in filteredActiveCategoryOptions"
						:key="`${activeCategory.key}-${option.value}`"
						type="button"
						class="flex w-full cursor-pointer items-center gap-2.5 rounded-xl border-0 bg-transparent p-3 text-left text-contrast shadow-none transition-colors duration-150 hover:bg-surface-5"
						:class="{ 'pointer-events-none opacity-50': option.disabled }"
						:aria-disabled="option.disabled || undefined"
						:aria-checked="isFilterValueSelected(activeCategory.key, option.value)"
						role="checkbox"
						@click="toggleFilterOption(activeCategory.key, option)"
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
				</template>
			</div>
			<slot
				name="category-footer"
				:category="activeCategory"
				:selected-values="activeCategorySelectedValues"
				:set-selected-values="setActiveCategorySelectedValues"
			></slot>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import {
	CheckIcon,
	ChevronLeftIcon,
	ChevronRightIcon,
	FilterIcon,
	PlusIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import type { ComponentPublicInstance, CSSProperties } from 'vue'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import ButtonStyled from './ButtonStyled.vue'
import MultiSelect, { type MultiSelectOption } from './MultiSelect.vue'
import StyledInput from './StyledInput.vue'

export type DropdownFilterBarOption = {
	value: string
	label: string
	searchTerms?: string[]
	disabled?: boolean
}

export type DropdownFilterBarCategory = {
	key: string
	label: string
	options: DropdownFilterBarOption[]
	searchable?: boolean
	searchPlaceholder?: string
	submenuClass?: string
	previewDropdownMinWidth?: string | number
}

type DropdownFilterBarValue = Record<string, string[]>

type Point = {
	x: number
	y: number
}

type MenuPositionOptions = {
	triggerRect: DOMRect
	dropdownRect: DOMRect
	viewportWidth: number
	viewportHeight: number
}

type SubmenuPositionOptions = {
	buttonRect: DOMRect
	submenuWidth: number
	submenuHeight: number
	viewportWidth: number
	viewportHeight: number
}

const ADD_MENU_WIDTH = 250
const DROPDOWN_GAP = 12
const DROPDOWN_VIEWPORT_MARGIN = 8

const props = withDefaults(
	defineProps<{
		modelValue: DropdownFilterBarValue
		categories: DropdownFilterBarCategory[]
		label?: string
		addLabel?: string
		clearLabel?: string
		useFilterIcon?: boolean
		emptyOptionsLabel?: string
		emptySearchLabel?: string
	}>(),
	{
		label: 'Filtered by',
		addLabel: 'Add',
		clearLabel: 'Clear',
		useFilterIcon: false,
		emptyOptionsLabel: 'No options available.',
		emptySearchLabel: 'No options found.',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: DropdownFilterBarValue]
}>()

const isAddMenuOpen = ref(false)
const activeCategoryKey = ref<string | null>(null)
const pendingCategoryKey = ref<string | null>(null)
const draftSelectedFilters = ref<DropdownFilterBarValue>(cloneSelectedFilters(props.modelValue))
const previewSelectedValueDrafts = ref<Partial<DropdownFilterBarValue>>({})
const categorySearchQuery = ref('')
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
const categoryButtonRefs = new Map<string, HTMLElement>()
let pendingCategoryTimeout: ReturnType<typeof setTimeout> | null = null
let previousMousePosition: Point | null = null

const filterCategories = computed<DropdownFilterBarCategory[]>(() => {
	const source = isAddMenuOpen.value ? 'draft' : 'committed'
	return props.categories.map((category) => ({
		...category,
		options: getOptionsWithSelectedValues(
			category.options,
			getSelectedValues(category.key, source),
		),
	}))
})

const filterCategoriesByKey = computed(
	() => new Map(filterCategories.value.map((category) => [category.key, category] as const)),
)

const activeCategory = computed(() =>
	activeCategoryKey.value ? filterCategoriesByKey.value.get(activeCategoryKey.value) : undefined,
)
const activeCategorySelectionCount = computed(() => {
	return activeCategory.value ? getCategorySelectionCount(activeCategory.value.key, 'draft') : 0
})
const activeCategorySelectionLabel = computed(() =>
	activeCategorySelectionCount.value === 1
		? '1 selected'
		: `${activeCategorySelectionCount.value} selected`,
)
const activeCategorySelectedValues = computed(() =>
	activeCategory.value ? getSelectedValues(activeCategory.value.key, 'draft') : [],
)

const filteredActiveCategoryOptions = computed(() => {
	if (!activeCategory.value) {
		return []
	}

	if (!activeCategory.value.searchable) {
		return activeCategory.value.options
	}

	const query = categorySearchQuery.value.trim().toLowerCase()
	if (!query) {
		return activeCategory.value.options
	}

	return activeCategory.value.options.filter((option) => {
		if (option.label.toLowerCase().includes(query)) {
			return true
		}
		if (option.value.toLowerCase().includes(query)) {
			return true
		}
		return option.searchTerms?.some((term) => term.toLowerCase().includes(query)) ?? false
	})
})

const activeCategoryEmptyStateLabel = computed(() =>
	activeCategory.value?.searchable && categorySearchQuery.value.trim().length > 0
		? props.emptySearchLabel
		: props.emptyOptionsLabel,
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
				searchTerms: option.searchTerms,
				disabled: option.disabled,
			})) as MultiSelectOption<string>[],
		}))
		.filter((preview) => preview.count > 0),
)

const hasAppliedFilters = computed(() => appliedFilterPreviews.value.length > 0)
const previewTriggerClass =
	'h-10 max-w-[16rem] border border-solid border-surface-5 bg-surface-4 px-3 py-1.5 hover:bg-surface-5 hover:brightness-100 active:brightness-100'

function cloneSelectedFilters(filters: DropdownFilterBarValue): DropdownFilterBarValue {
	return Object.fromEntries(
		Object.entries(filters).map(([categoryKey, values]) => [
			categoryKey,
			normalizeSelectedValues(values),
		]),
	)
}

function normalizeSelectedValues(values: string[]): string[] {
	return Array.from(new Set(values))
}

function areStringArraysEqual(left: string[], right: string[]): boolean {
	if (left.length !== right.length) {
		return false
	}

	for (let index = 0; index < left.length; index += 1) {
		if (left[index] !== right[index]) {
			return false
		}
	}

	return true
}

function areSelectedFiltersEqual(
	left: DropdownFilterBarValue,
	right: DropdownFilterBarValue,
): boolean {
	const categoryKeys = new Set([...Object.keys(left), ...Object.keys(right)])
	for (const categoryKey of categoryKeys) {
		if (!areStringArraysEqual(left[categoryKey] ?? [], right[categoryKey] ?? [])) {
			return false
		}
	}

	return true
}

function getOptionsWithSelectedValues(
	options: DropdownFilterBarOption[],
	selectedValues: string[],
): DropdownFilterBarOption[] {
	const knownValues = new Set(options.map((option) => option.value))
	const missingSelectedOptions = selectedValues
		.filter((value) => !knownValues.has(value))
		.map((value) => ({
			value,
			label: value,
		}))

	return [...options, ...missingSelectedOptions]
}

function getSelectedValues(
	categoryKey: string,
	source: 'committed' | 'draft' = 'committed',
): string[] {
	if (source === 'draft') {
		return draftSelectedFilters.value[categoryKey] ?? []
	}

	return props.modelValue[categoryKey] ?? []
}

function setSelectedValues(
	categoryKey: string,
	values: string[],
	source: 'committed' | 'draft' = 'committed',
) {
	const normalizedValues = normalizeSelectedValues(values)
	const currentFilters = source === 'draft' ? draftSelectedFilters.value : props.modelValue
	if (areStringArraysEqual(currentFilters[categoryKey] ?? [], normalizedValues)) {
		return
	}

	const nextFilters = {
		...cloneSelectedFilters(currentFilters),
		[categoryKey]: normalizedValues,
	}

	if (source === 'draft') {
		draftSelectedFilters.value = nextFilters
		if (isAddMenuOpen.value && activeCategoryKey.value === categoryKey) {
			scheduleSubmenuPositionUpdate()
		}
	} else {
		emit('update:modelValue', nextFilters)
	}
}

function setActiveCategorySelectedValues(values: string[]) {
	if (!activeCategory.value) {
		return
	}

	setSelectedValues(activeCategory.value.key, values, 'draft')
}

function resetAddMenuDraft() {
	draftSelectedFilters.value = cloneSelectedFilters(props.modelValue)
}

function commitAddMenuDraft() {
	if (!areSelectedFiltersEqual(props.modelValue, draftSelectedFilters.value)) {
		emit('update:modelValue', cloneSelectedFilters(draftSelectedFilters.value))
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
	categorySearchQuery.value = ''
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
	categoryKey: string,
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

function isFilterValueSelected(categoryKey: string, value: string): boolean {
	return getSelectedValues(categoryKey, 'draft').includes(value)
}

function toggleFilterValue(categoryKey: string, value: string, nextValue: boolean) {
	const currentValues = getSelectedValues(categoryKey, 'draft')

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

function toggleFilterOption(categoryKey: string, option: DropdownFilterBarOption) {
	if (option.disabled) {
		return
	}

	toggleFilterValue(categoryKey, option.value, !isFilterValueSelected(categoryKey, option.value))
}

function getCategorySelectionCount(
	categoryKey: string,
	source: 'committed' | 'draft' = 'committed',
): number {
	return getSelectedValues(categoryKey, source).length
}

function getCategorySelectionSummary(category: DropdownFilterBarCategory): string {
	const count = getCategorySelectionCount(category.key)
	if (count === 0) {
		return ''
	}

	if (count === 1) {
		const selectedValue = getSelectedValues(category.key)[0]
		return category.options.find((option) => option.value === selectedValue)?.label ?? '1 selected'
	}

	return `${count} selected`
}

function clearFilterCategory(categoryKey: string) {
	commitPreviewFilterDrafts()
	setSelectedValues(categoryKey, [])
}

function clearAllFilters() {
	commitPreviewFilterDrafts()

	const nextFilters = cloneSelectedFilters(props.modelValue)
	for (const category of filterCategories.value) {
		nextFilters[category.key] = []
	}

	if (!areSelectedFiltersEqual(props.modelValue, nextFilters)) {
		emit('update:modelValue', nextFilters)
	}
}

function clearActiveCategorySelection() {
	if (!activeCategory.value) {
		return
	}

	setSelectedValues(activeCategory.value.key, [], 'draft')
}

function getPreviewSelectedValues(categoryKey: string): string[] {
	const draftValues = previewSelectedValueDrafts.value[categoryKey]
	if (draftValues !== undefined) {
		return draftValues
	}

	return getSelectedValues(categoryKey)
}

function setPreviewSelectedValues(categoryKey: string, values: string[]) {
	previewSelectedValueDrafts.value = {
		...previewSelectedValueDrafts.value,
		[categoryKey]: normalizeSelectedValues(values),
	}
}

function openPreviewFilterDraft(categoryKey: string) {
	commitPreviewFilterDrafts()
	previewSelectedValueDrafts.value = {
		...previewSelectedValueDrafts.value,
		[categoryKey]: [...getSelectedValues(categoryKey)],
	}
}

function commitPreviewFilterDraft(categoryKey: string) {
	const draftValues = previewSelectedValueDrafts.value[categoryKey]
	if (draftValues === undefined) {
		return
	}

	previewSelectedValueDrafts.value = Object.fromEntries(
		Object.entries(previewSelectedValueDrafts.value).filter(([key]) => key !== categoryKey),
	)
	setSelectedValues(categoryKey, draftValues)
}

function commitPreviewFilterDrafts() {
	for (const categoryKey of Object.keys(previewSelectedValueDrafts.value)) {
		commitPreviewFilterDraft(categoryKey)
	}
}

function activateCategory(categoryKey: string) {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	if (activeCategoryKey.value !== categoryKey) {
		categorySearchQuery.value = ''
	}
	activeCategoryKey.value = categoryKey
	scheduleSubmenuPositionUpdate()
}

function handleCategoryMouseEnter(categoryKey: string) {
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

function getAddMenuPosition({
	triggerRect,
	dropdownRect,
	viewportWidth,
	viewportHeight,
}: MenuPositionOptions) {
	const dropdownWidth = Math.max(ADD_MENU_WIDTH, triggerRect.width)
	const hasSpaceBelow =
		triggerRect.bottom + dropdownRect.height + DROPDOWN_GAP + DROPDOWN_VIEWPORT_MARGIN <=
		viewportHeight
	const hasSpaceAbove =
		triggerRect.top - dropdownRect.height - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN > 0
	const opensUp = !hasSpaceBelow && hasSpaceAbove
	const top = opensUp
		? triggerRect.top - dropdownRect.height - DROPDOWN_GAP
		: triggerRect.bottom + DROPDOWN_GAP
	const left = Math.min(triggerRect.left, viewportWidth - dropdownWidth - DROPDOWN_VIEWPORT_MARGIN)

	return {
		left: `${Math.max(DROPDOWN_VIEWPORT_MARGIN, left)}px`,
		minWidth: `${triggerRect.width}px`,
		top: `${Math.max(DROPDOWN_VIEWPORT_MARGIN, top)}px`,
		width: `${dropdownWidth}px`,
	}
}

function getSubmenuPosition({
	buttonRect,
	submenuWidth,
	submenuHeight,
	viewportWidth,
	viewportHeight,
}: SubmenuPositionOptions): Point {
	const gap = 20
	const viewportPadding = 8
	const preferredLeft = buttonRect.right + gap
	const left =
		preferredLeft + submenuWidth + viewportPadding <= viewportWidth
			? preferredLeft
			: Math.max(viewportPadding, buttonRect.left - submenuWidth - gap)
	const top = Math.min(
		Math.max(viewportPadding, buttonRect.top),
		Math.max(viewportPadding, viewportHeight - submenuHeight - viewportPadding),
	)

	return {
		x: left,
		y: top,
	}
}

function getIsCursorAimingAtSubmenu(
	cursor: Point | null,
	origin: Point | null,
	submenuRect: DOMRect | null,
): boolean {
	if (!submenuRect || !cursor || !origin) {
		return false
	}

	const submenuTargetX =
		origin.x <= submenuRect.left
			? submenuRect.left
			: origin.x >= submenuRect.right
				? submenuRect.right
				: cursor.x <= submenuRect.left
					? submenuRect.left
					: submenuRect.right
	const upperTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.top + 20,
	}
	const lowerTarget: Point = {
		x: submenuTargetX,
		y: submenuRect.bottom + 20,
	}

	return isPointInTriangle(cursor, origin, upperTarget, lowerTarget)
}

function isPointInTriangle(point: Point, a: Point, b: Point, c: Point): boolean {
	const area = triangleArea(a, b, c)
	const area1 = triangleArea(point, b, c)
	const area2 = triangleArea(a, point, c)
	const area3 = triangleArea(a, b, point)

	return Math.abs(area - (area1 + area2 + area3)) < 0.5
}

function triangleArea(a: Point, b: Point, c: Point): number {
	return Math.abs((a.x * (b.y - c.y) + b.x * (c.y - a.y) + c.x * (a.y - b.y)) / 2)
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
		if (!isAddMenuOpen.value) {
			return
		}

		const hasRenderedSubmenu = submenu.value !== null
		if (updateSubmenuPosition()) {
			if (!hasRenderedSubmenu) {
				nextTick(() => updateSubmenuPosition())
			}
			return
		}

		if (retries <= 0) {
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

watch(categorySearchQuery, () => {
	scheduleSubmenuPositionUpdate()
})

watch(filterCategoriesByKey, (nextCategories) => {
	if (activeCategoryKey.value && !nextCategories.has(activeCategoryKey.value)) {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
	}
})

watch(
	() => props.modelValue,
	() => {
		if (!isAddMenuOpen.value) {
			draftSelectedFilters.value = cloneSelectedFilters(props.modelValue)
		}
	},
	{ deep: true },
)

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
