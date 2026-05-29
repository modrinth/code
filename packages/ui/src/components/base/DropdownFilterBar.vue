<template>
	<span
		v-if="showLabel"
		class="flex h-10 items-center text-nowrap text-base font-medium text-primary"
		:aria-label="useFilterIcon ? label : undefined"
	>
		<FilterIcon v-if="useFilterIcon" class="size-5 text-primary" aria-hidden="true" />
		<template v-else>{{ label }}</template>
	</span>

	<MultiSelect
		v-for="preview in appliedFilterPreviews"
		:key="preview.key"
		class="min-w-0 max-w-full"
		:model-value="getPreviewSelectedValues(preview.key)"
		:options="preview.options"
		:max-height="500"
		:clearable="false"
		:show-chevron="false"
		:fit-content="true"
		:searchable="preview.category.searchable"
		:search-placeholder="preview.category.searchPlaceholder"
		:trigger-class="effectivePreviewTriggerClass"
		:dropdown-width="getPreviewDropdownWidth(preview.category)"
		:dropdown-min-width="getPreviewDropdownMinWidth(preview.category)"
		:checkbox-position="checkboxPosition"
		show-selection-actions
		@update:model-value="(nextValue) => setPreviewSelectedValues(preview.key, nextValue)"
		@open="openPreviewFilterDraft(preview.key)"
		@close="commitPreviewFilterDraft(preview.key)"
	>
		<template v-if="$slots['preview-top'] && preview.category.syntheticOptions?.length" #top>
			<slot
				name="preview-top"
				:category="preview.category"
				:selected-values="getPreviewSelectedValues(preview.key)"
				:set-selected-values="(values) => setPreviewSelectedValues(preview.key, values)"
				:close-menu="(event) => closePreviewFilterMenu(preview.key, event)"
			></slot>
		</template>
		<template v-if="$slots['search-actions']" #search-actions>
			<slot
				name="search-actions"
				:category="preview.category"
				:selected-values="getPreviewSelectedValues(preview.key)"
				:set-selected-values="(values) => setPreviewSelectedValues(preview.key, values)"
				:close-menu="(event) => closePreviewFilterMenu(preview.key, event)"
			></slot>
		</template>
		<template #input-content="{ isOpen, openDirection }">
			<div class="flex min-h-8 min-w-0 max-w-full items-center gap-2 sm:max-w-80">
				<FilterIcon
					v-if="showPreviewFilterIcon"
					class="size-5 shrink-0 text-primary"
					aria-hidden="true"
				/>
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
		<template v-if="$slots['preview-footer']" #bottom>
			<slot
				name="preview-footer"
				:category="preview.category"
				:selected-values="getPreviewSelectedValues(preview.key)"
				:set-selected-values="(values) => setPreviewSelectedValues(preview.key, values)"
				:close-menu="(event) => closePreviewFilterMenu(preview.key, event)"
			></slot>
		</template>
		<template v-if="$slots.option" #option="{ item, selected, index }">
			<slot
				name="option"
				:category="preview.category"
				:option="item"
				:selected="selected"
				:index="index"
			></slot>
		</template>
		<template v-else-if="$slots['option-right']" #option-right="{ item, selected, index }">
			<slot
				name="option-right"
				:category="preview.category"
				:option="item"
				:selected="selected"
				:index="index"
			></slot>
		</template>
	</MultiSelect>

	<div class="flex h-10 min-w-0 max-w-full items-center gap-2">
		<ButtonStyled type="outlined">
			<button
				ref="addMenuTrigger"
				type="button"
				:class="addButtonClass"
				:aria-expanded="isAddMenuOpen"
				aria-haspopup="menu"
				@click="handleAddMenuTriggerClick"
				@keydown="handleAddMenuTriggerKeydown"
			>
				<PlusIcon />
				{{ addLabel }}
			</button>
		</ButtonStyled>

		<ButtonStyled v-if="shouldShowClear" type="transparent">
			<button type="button" @click="clearAllFilters">{{ clearLabel }}</button>
		</ButtonStyled>
	</div>

	<Teleport to="#teleports">
		<Transition
			:enter-active-class="addMenuTransitionEnterActiveClass"
			leave-active-class="transition-none duration-0"
			:enter-from-class="addMenuTransitionEnterFromClass"
			leave-to-class="opacity-0"
		>
			<div
				v-if="isAddMenuOpen && !isMobileActiveSubmenu"
				ref="menuContainer"
				class="fixed z-[9999] flex flex-col overflow-hidden rounded-[14px] border border-solid border-surface-5 bg-surface-4 shadow-2xl"
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
					class="group/filter-menu-button flex h-12 w-full appearance-none items-center justify-between gap-1 border-0 px-4 text-left text-base font-semibold text-primary shadow-none transition-all duration-150 hover:brightness-110 focus:brightness-110 bg-surface-4"
					:class="category.key === activeCategoryKey ? '!brightness-110' : ''"
					role="menuitem"
					@click="activateCategory(category.key)"
					@mouseenter="handleCategoryMouseEnter(category.key)"
					@focus="handleCategoryFocus(category.key)"
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
			v-if="isAddMenuOpen && activeCategory && (isMobileAddMenuLayout || hasSubmenuPosition)"
			ref="submenu"
			class="fixed z-[10000] flex max-h-[min(70vh,32rem)] max-w-[calc(100vw-1rem)] flex-col overflow-hidden rounded-[14px] border border-solid border-surface-5 bg-surface-4 shadow-2xl"
			:class="activeCategory.submenuClass ?? DEFAULT_SUBMENU_CLASS"
			:style="submenuStyle"
			@mouseenter="handleSubmenuMouseEnter"
			@mouseleave="handleSubmenuMouseLeave"
			@keydown="handleAddMenuKeydown"
			@mousemove="(event) => handleMenuMouseMove(event, 'submenu')"
		>
			<div
				v-if="isMobileAddMenuLayout"
				class="flex items-center border-0 border-b border-solid border-b-surface-5 bg-surface-4"
			>
				<button
					type="button"
					class="flex h-12 w-full items-center gap-2 border-0 bg-transparent px-4 text-left text-base font-semibold text-primary shadow-none transition-all hover:brightness-110 focus-visible:brightness-110"
					@click="returnToCategoryMenu"
				>
					<ChevronLeftIcon class="size-5 shrink-0 text-secondary" />
					<span class="min-w-0 truncate">{{ activeCategory.label }}</span>
				</button>
			</div>
			<div
				v-if="activeCategory.searchable"
				class="flex justify-between border-0 border-b border-solid border-b-surface-5 py-1.5 w-full"
			>
				<StyledInput
					v-model="categorySearchQuery"
					:icon="SearchIcon"
					type="text"
					:placeholder="activeCategory.searchPlaceholder ?? 'Search...'"
					wrapper-class="grow bg-surface-4 mx-1"
					input-class="ps-9 mx-1.5"
				/>
				<slot
					name="search-actions"
					:category="activeCategory"
					:selected-values="activeCategorySelectedValues"
					:set-selected-values="setActiveCategorySelectedValues"
					:close-menu="closeAddMenu"
				></slot>
			</div>

			<div v-if="$slots['category-top']" class="border-0 border-b border-solid border-b-surface-5">
				<slot
					name="category-top"
					:category="activeCategory"
					:selected-values="activeCategorySelectedValues"
					:set-selected-values="setActiveCategorySelectedValues"
					:close-menu="closeAddMenu"
				></slot>
			</div>

			<div
				class="flex items-center justify-between gap-3 border-0 border-b border-solid border-b-surface-5 px-4 py-2.5 text-sm"
			>
				<span class="font-semibold text-secondary">{{ activeCategorySelectionLabel }}</span>
				<button
					type="button"
					class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-colors"
					:class="
						hasActiveCategorySelection
							? 'hover:bg-transparent hover:text-contrast'
							: 'cursor-not-allowed opacity-50'
					"
					:disabled="!hasActiveCategorySelection"
					@click="clearActiveCategorySelection"
					@keydown.enter.stop
					@keydown.space.stop
				>
					Clear
				</button>
			</div>
			<div
				ref="activeCategoryOptionsScrollbar"
				class="dropdown-filter-bar-options-scrollbar min-h-0 flex-1 bg-surface-4"
				data-overlayscrollbars-initialize
			>
				<div
					ref="activeCategoryOptionsContainer"
					class="h-full min-h-0 overflow-y-auto"
					data-overlayscrollbars-viewport
				>
					<div
						v-if="filteredActiveCategoryOptions.length === 0"
						class="px-4 py-3.5 text-base font-medium text-secondary"
					>
						{{ activeCategoryEmptyStateLabel }}
					</div>
					<div
						v-else
						ref="activeCategoryOptionsListContainer"
						:class="shouldVirtualizeActiveCategoryOptions ? 'relative' : 'flex flex-col'"
						:style="activeCategoryOptionsListStyle"
					>
						<div
							v-for="{ option, index } in renderedVisibleActiveCategoryOptions"
							:key="`${activeCategory.key}-${option.value}`"
							:class="shouldVirtualizeActiveCategoryOptions ? 'absolute left-0 right-0' : undefined"
							:style="getActiveCategoryOptionWrapperStyle(index)"
						>
							<button
								type="button"
								class="flex w-full cursor-pointer items-center gap-2.5 border-0 px-4 py-3.5 text-left text-contrast shadow-none transition-all duration-150 bg-surface-4 hover:brightness-[115%] focus-visible:brightness-[115%] focus-visible:outline-none"
								:class="[
									shouldVirtualizeActiveCategoryOptions ? 'h-12' : undefined,
									{
										'brightness-[115%]': option.selected,
										'pointer-events-none cursor-not-allowed opacity-50': option.disabled,
									},
								]"
								:aria-disabled="option.disabled || undefined"
								:aria-checked="option.selected"
								role="checkbox"
								@click="toggleFilterOption(activeCategory.key, option)"
							>
								<span
									v-if="checkboxPosition === 'left'"
									class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
									:class="
										option.selected
											? 'border-button-border bg-brand text-brand-inverted'
											: 'border-surface-5 bg-surface-2'
									"
								>
									<CheckIcon v-if="option.selected" aria-hidden="true" stroke-width="3" />
								</span>
								<div class="flex min-w-0 flex-1 items-center justify-between gap-3">
									<slot
										v-if="$slots.option"
										name="option"
										:category="activeCategory"
										:option="option"
										:selected="option.selected"
										:index="index"
									></slot>
									<template v-else>
										<span
											class="min-w-0 truncate font-semibold leading-tight"
											:class="option.selected ? 'text-contrast' : 'text-primary'"
										>
											{{ option.label }}
										</span>
										<slot
											name="option-right"
											:category="activeCategory"
											:option="option"
											:selected="option.selected"
										></slot>
									</template>
								</div>
								<span
									v-if="checkboxPosition === 'right'"
									class="flex shrink-0 items-center justify-center text-brand"
								>
									<CheckIcon v-if="option.selected" aria-hidden="true" class="size-5" />
								</span>
							</button>
						</div>
					</div>
				</div>
			</div>
			<slot
				name="category-footer"
				:category="activeCategory"
				:selected-values="activeCategorySelectedValues"
				:set-selected-values="setActiveCategorySelectedValues"
				:close-menu="closeAddMenu"
			></slot>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import 'overlayscrollbars/overlayscrollbars.css'

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
import { OverlayScrollbars, type PartialOptions } from 'overlayscrollbars'
import type { ComponentPublicInstance, CSSProperties } from 'vue'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import { useVirtualScroll } from '../../composables/virtual-scroll'
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
	syntheticOptions?: DropdownFilterBarOption[]
	searchable?: boolean
	searchPlaceholder?: string
	emptyOptionsLabel?: string
	emptySearchLabel?: string
	submenuClass?: string
	previewDropdownWidth?: string | number
	previewDropdownMinWidth?: string | number
}

type DropdownFilterBarValue = Record<string, string[]>

type RenderedDropdownFilterBarOption = DropdownFilterBarOption & {
	selected: boolean
}

type VisibleDropdownFilterBarOption = {
	option: RenderedDropdownFilterBarOption
	index: number
}

type OverlayScrollbarsInstance = NonNullable<ReturnType<typeof OverlayScrollbars>>

type Point = {
	x: number
	y: number
}

type ViewportRect = {
	width: number
	height: number
	offsetTop: number
	offsetLeft: number
}

type MenuPositionOptions = {
	triggerRect: DOMRect
	dropdownRect: DOMRect
	viewport: ViewportRect
}

type SubmenuPositionOptions = {
	buttonRect: DOMRect
	openDirection: SubmenuOpenDirection
	submenuWidth: number
	submenuHeight: number
	viewport: ViewportRect
}

type SubmenuOpenDirection = 'left' | 'right'

type SubmenuOpenDirectionOptions = {
	menuRect: DOMRect
	widestSubmenuWidth: number
	viewport: ViewportRect
}

const ADD_MENU_WIDTH = 250
const DROPDOWN_GAP = 8
const DROPDOWN_VIEWPORT_MARGIN = 8
const DEFAULT_ROOT_FONT_SIZE = 16
const DEFAULT_SUBMENU_CLASS = 'w-72'
const DEFAULT_SUBMENU_WIDTH = '18rem'
const DEFAULT_PREVIEW_DROPDOWN_MIN_WIDTH = '18rem'
const DROPDOWN_FILTER_OPTION_ROW_HEIGHT = 48
const DROPDOWN_FILTER_VIRTUALIZATION_THRESHOLD = 80
const MOBILE_ADD_MENU_LAYOUT_QUERY = '(pointer: coarse), (max-width: 800px)'
const OPTIONS_OVERLAY_SCROLLBARS_OPTIONS = Object.freeze<PartialOptions>({
	overflow: {
		x: 'hidden',
		y: 'scroll',
	},
	scrollbars: {
		theme: 'os-theme-modrinth',
		autoHide: 'leave',
		autoHideSuspend: true,
	},
})
const TAILWIND_WIDTH_CLASS_SIZE: Record<string, string> = {
	[DEFAULT_SUBMENU_CLASS]: DEFAULT_SUBMENU_WIDTH,
}

const props = withDefaults(
	defineProps<{
		modelValue: DropdownFilterBarValue
		categories: DropdownFilterBarCategory[]
		label?: string
		addLabel?: string
		clearLabel?: string
		showClear?: boolean
		showLabel?: boolean
		useFilterIcon?: boolean
		showPreviewFilterIcon?: boolean
		previewTriggerClass?: string
		addButtonClass?: string
		emptyOptionsLabel?: string
		emptySearchLabel?: string
		checkboxPosition?: 'left' | 'right'
	}>(),
	{
		label: 'Filtered by',
		addLabel: 'Add',
		clearLabel: 'Clear',
		showClear: false,
		showLabel: true,
		useFilterIcon: false,
		showPreviewFilterIcon: false,
		emptyOptionsLabel: 'No options available.',
		emptySearchLabel: 'No options found.',
		checkboxPosition: 'left',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: DropdownFilterBarValue]
	clear: []
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
const isMobileAddMenuLayout = ref(false)
const submenuOpenDirection = ref<SubmenuOpenDirection>('right')
const addMenuTrigger = ref<HTMLElement | null>(null)
const menuContainer = ref<HTMLElement | null>(null)
const submenu = ref<HTMLElement | null>(null)
const activeCategoryOptionsScrollbar = ref<HTMLElement | null>(null)
const activeCategoryOptionsContainer = ref<HTMLElement | null>(null)
const activeCategoryOptionsOverlayScrollbars = ref<OverlayScrollbarsInstance | null>(null)
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
let addMenuPositionRafId: number | null = null

const filterCategories = computed<DropdownFilterBarCategory[]>(() => {
	const source = isAddMenuOpen.value ? 'draft' : 'committed'
	return props.categories.map((category) => {
		const syntheticValues = getCategorySyntheticValueSet(category)
		const selectedValues = getSelectedValues(category.key, source).filter(
			(value) => !syntheticValues.has(value),
		)

		return {
			...category,
			options: getOptionsWithSelectedValues(category.options, selectedValues),
		}
	})
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
const hasActiveCategorySelection = computed(() => activeCategorySelectionCount.value > 0)
const activeCategorySelectionLabel = computed(() => {
	return activeCategorySelectionCount.value === 1
		? '1 selected'
		: `${activeCategorySelectionCount.value} selected`
})
const activeCategorySelectedValues = computed(() =>
	activeCategory.value ? getSelectedValues(activeCategory.value.key, 'draft') : [],
)
const activeCategorySelectedValueSet = computed(() => new Set(activeCategorySelectedValues.value))

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

const shouldVirtualizeActiveCategoryOptions = computed(
	() => filteredActiveCategoryOptions.value.length > DROPDOWN_FILTER_VIRTUALIZATION_THRESHOLD,
)

const {
	listContainer: activeCategoryOptionsListContainer,
	totalHeight: activeCategoryOptionsTotalHeight,
	visibleRange: activeCategoryOptionsVisibleRange,
	visibleItems: visibleActiveCategoryOptions,
	resetScrollState: resetActiveCategoryOptionsVirtualScrollState,
	syncScrollState: syncActiveCategoryOptionsVirtualScrollState,
} = useVirtualScroll(filteredActiveCategoryOptions, {
	itemHeight: DROPDOWN_FILTER_OPTION_ROW_HEIGHT,
	bufferSize: 8,
	initialItemCount: 12,
	enabled: shouldVirtualizeActiveCategoryOptions,
})

const renderedVisibleActiveCategoryOptions = computed<VisibleDropdownFilterBarOption[]>(() =>
	visibleActiveCategoryOptions.value.map((option, offset) => ({
		option: {
			...option,
			selected: activeCategorySelectedValueSet.value.has(option.value),
		},
		index: activeCategoryOptionsVisibleRange.value.start + offset,
	})),
)

const activeCategoryOptionsListStyle = computed<CSSProperties | undefined>(() =>
	shouldVirtualizeActiveCategoryOptions.value
		? { height: `${activeCategoryOptionsTotalHeight.value}px` }
		: undefined,
)

const activeCategoryEmptyStateLabel = computed(() => {
	const category = activeCategory.value
	if (!category) {
		return props.emptyOptionsLabel
	}

	return category.searchable && categorySearchQuery.value.trim().length > 0
		? (category.emptySearchLabel ?? props.emptySearchLabel)
		: (category.emptyOptionsLabel ?? props.emptyOptionsLabel)
})

const submenuStyle = computed<CSSProperties>(() => {
	if (isMobileAddMenuLayout.value) {
		return {
			left: addMenuStyle.value.left,
			top: addMenuStyle.value.top,
		}
	}

	return {
		left: `${submenuPosition.value.x}px`,
		top: `${submenuPosition.value.y}px`,
	}
})
const isMobileActiveSubmenu = computed(
	() =>
		isMobileAddMenuLayout.value && activeCategory.value !== undefined && hasSubmenuPosition.value,
)
const addMenuTransitionEnterActiveClass = computed(() =>
	isMobileAddMenuLayout.value ? 'transition-none duration-0' : 'transition-opacity duration-150',
)
const addMenuTransitionEnterFromClass = computed(() =>
	isMobileAddMenuLayout.value ? 'opacity-100' : 'opacity-0',
)
const addMenuOutsideClickTarget = computed(() => menuContainer.value ?? submenu.value)
const addMenuOutsideClickIgnore = computed(() => [addMenuTrigger, menuContainer, submenu])

const appliedFilterPreviews = computed(() =>
	Object.entries(props.modelValue)
		.map(([categoryKey, selectedValues]) => {
			const category = filterCategoriesByKey.value.get(categoryKey)
			if (!category || selectedValues.length === 0) {
				return undefined
			}

			return {
				key: category.key,
				label: category.label,
				summary: getCategorySelectionSummary(category),
				count: selectedValues.length,
				category,
				options: getVisiblePreviewOptions(category),
			}
		})
		.filter((preview): preview is NonNullable<typeof preview> => preview !== undefined),
)

const hasAppliedFilters = computed(() => appliedFilterPreviews.value.length > 0)
const shouldShowClear = computed(() => hasAppliedFilters.value || props.showClear)
const DEFAULT_PREVIEW_TRIGGER_CLASS =
	'h-10 max-w-[16rem] bg-surface-4 px-4 py-1.5 transition-all bg-surface-4 hover:brightness-110 active:brightness-110'
const effectivePreviewTriggerClass = computed(
	() => props.previewTriggerClass ?? DEFAULT_PREVIEW_TRIGGER_CLASS,
)

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
	if (selectedValues.length === 0) {
		return options
	}

	const knownValues = new Set(options.map((option) => option.value))
	const missingSelectedOptions = selectedValues
		.filter((value) => !knownValues.has(value))
		.map((value) => ({
			value,
			label: value,
		}))

	return missingSelectedOptions.length === 0 ? options : [...options, ...missingSelectedOptions]
}

function getCategorySyntheticValueSet(category: DropdownFilterBarCategory): Set<string> {
	return new Set((category.syntheticOptions ?? []).map((option) => option.value))
}

function getCategorySyntheticValues(categoryKey: string): Set<string> {
	const category = filterCategoriesByKey.value.get(categoryKey)
	return category ? getCategorySyntheticValueSet(category) : new Set()
}

function getVisiblePreviewOptions(
	category: DropdownFilterBarCategory,
): MultiSelectOption<string>[] {
	return category.options.map((option) => ({
		value: option.value,
		label: option.label,
		searchTerms: option.searchTerms,
		disabled: option.disabled,
	})) as MultiSelectOption<string>[]
}

function getPreviewOptionLabel(
	category: DropdownFilterBarCategory,
	selectedValue: string,
): string | undefined {
	return [...(category.syntheticOptions ?? []), ...category.options].find(
		(option) => option.value === selectedValue,
	)?.label
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

	const nextFilters = getNextSelectedFilters(currentFilters, categoryKey, normalizedValues)

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

function syncMobileAddMenuLayout(): boolean {
	if (typeof window === 'undefined') {
		return false
	}

	const nextValue = window.matchMedia(MOBILE_ADD_MENU_LAYOUT_QUERY).matches
	if (nextValue === isMobileAddMenuLayout.value) {
		return false
	}

	isMobileAddMenuLayout.value = nextValue
	return true
}

function openAddMenu() {
	if (isAddMenuOpen.value) {
		return
	}

	commitPreviewFilterDrafts()
	resetAddMenuDraft()
	syncMobileAddMenuLayout()
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

function returnToCategoryMenu() {
	activeCategoryKey.value = null
	categorySearchQuery.value = ''
	hasSubmenuPosition.value = false
	nextTick(() => scheduleAddMenuPositionUpdate())
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
	if (categoryKey === activeCategory.value?.key) {
		return activeCategorySelectedValueSet.value.has(value)
	}

	return getSelectedValues(categoryKey, 'draft').includes(value)
}

function toggleFilterValue(categoryKey: string, value: string, nextValue: boolean) {
	const currentValues = getSelectedValues(categoryKey, 'draft')

	if (nextValue) {
		const syntheticValues = getCategorySyntheticValues(categoryKey)
		const nextValues = syntheticValues.has(value)
			? [value]
			: currentValues.filter((item) => !syntheticValues.has(item))

		if (!nextValues.includes(value)) {
			setSelectedValues(categoryKey, [...nextValues, value], 'draft')
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

function getActiveCategoryOptionWrapperStyle(index: number): CSSProperties | undefined {
	if (!shouldVirtualizeActiveCategoryOptions.value) {
		return undefined
	}

	return {
		transform: `translateY(${index * DROPDOWN_FILTER_OPTION_ROW_HEIGHT}px)`,
	}
}

async function initializeActiveCategoryOptionsOverlayScrollbars() {
	await nextTick()
	syncActiveCategoryOptionsVirtualScrollState()

	if (!isAddMenuOpen.value || filteredActiveCategoryOptions.value.length === 0) {
		destroyActiveCategoryOptionsOverlayScrollbars()
		return
	}

	if (
		!activeCategoryOptionsScrollbar.value ||
		!activeCategoryOptionsContainer.value ||
		!activeCategoryOptionsListContainer.value
	) {
		return
	}

	if (activeCategoryOptionsOverlayScrollbars.value) {
		activeCategoryOptionsOverlayScrollbars.value.update(true)
		return
	}

	activeCategoryOptionsOverlayScrollbars.value = OverlayScrollbars(
		{
			target: activeCategoryOptionsScrollbar.value,
			elements: {
				viewport: activeCategoryOptionsContainer.value,
				content: activeCategoryOptionsListContainer.value,
			},
		},
		OPTIONS_OVERLAY_SCROLLBARS_OPTIONS,
	)
}

function destroyActiveCategoryOptionsOverlayScrollbars() {
	activeCategoryOptionsOverlayScrollbars.value?.destroy()
	activeCategoryOptionsOverlayScrollbars.value = null
}

function resetActiveCategoryOptionsScrollState() {
	if (activeCategoryOptionsContainer.value) {
		activeCategoryOptionsContainer.value.scrollTop = 0
	}
	resetActiveCategoryOptionsVirtualScrollState()
}

function getCategorySelectionCount(
	categoryKey: string,
	source: 'committed' | 'draft' = 'committed',
): number {
	return getSelectedValues(categoryKey, source).length
}

function getNextSelectedFilters(
	currentFilters: DropdownFilterBarValue,
	categoryKey: string,
	selectedValues: string[],
): DropdownFilterBarValue {
	const nextFilters = cloneSelectedFilters(currentFilters)

	nextFilters[categoryKey] = selectedValues
	return nextFilters
}

function getCategorySelectionSummary(category: DropdownFilterBarCategory): string {
	const count = getCategorySelectionCount(category.key)
	if (count === 0) {
		return ''
	}

	if (count === 1) {
		const selectedValue = getSelectedValues(category.key)[0]
		return getPreviewOptionLabel(category, selectedValue) ?? '1 selected'
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

	emit('clear')
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

function closePreviewFilterMenu(categoryKey: string, event?: Event) {
	commitPreviewFilterDraft(categoryKey)

	const eventTarget = event?.target
	if (!(eventTarget instanceof HTMLElement)) {
		return
	}

	const dropdown = eventTarget.closest('[role="listbox"][aria-multiselectable="true"]')
	if (!dropdown) {
		return
	}

	dropdown.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
}

function getPreviewDropdownWidth(category: DropdownFilterBarCategory): string | number | undefined {
	return category.previewDropdownWidth ?? getWidthFromClass(category.submenuClass)
}

function getPreviewDropdownMinWidth(category: DropdownFilterBarCategory): string | number {
	return (
		category.previewDropdownMinWidth ??
		getPreviewDropdownWidth(category) ??
		DEFAULT_PREVIEW_DROPDOWN_MIN_WIDTH
	)
}

function getWidthFromClass(className: string | undefined): string | undefined {
	if (!className) {
		return undefined
	}

	const arbitraryWidth = className.match(/(?:^|\s)w-\[([^\]]+)\](?:\s|$)/)
	if (arbitraryWidth) {
		return arbitraryWidth[1]
	}

	return className
		.split(/\s+/)
		.map((name) => TAILWIND_WIDTH_CLASS_SIZE[name])
		.find((width) => width !== undefined)
}

function getRootFontSizeInPixels(): number {
	if (typeof window === 'undefined') {
		return DEFAULT_ROOT_FONT_SIZE
	}

	const rootFontSize = Number.parseFloat(window.getComputedStyle(document.documentElement).fontSize)
	return Number.isFinite(rootFontSize) ? rootFontSize : DEFAULT_ROOT_FONT_SIZE
}

function getCssLengthInPixels(length: string): number | undefined {
	const trimmedLength = length.trim()
	const parsedLength = Number.parseFloat(trimmedLength)
	if (!Number.isFinite(parsedLength)) {
		return undefined
	}

	if (trimmedLength.endsWith('rem')) {
		return parsedLength * getRootFontSizeInPixels()
	}

	if (trimmedLength.endsWith('px') || /^[\d.]+$/.test(trimmedLength)) {
		return parsedLength
	}

	return undefined
}

function getSubmenuWidthInPixels(category: DropdownFilterBarCategory): number {
	return (
		getCssLengthInPixels(getWidthFromClass(category.submenuClass) ?? DEFAULT_SUBMENU_WIDTH) ??
		getCssLengthInPixels(DEFAULT_SUBMENU_WIDTH) ??
		288
	)
}

function getWidestSubmenuWidthInPixels(categories: DropdownFilterBarCategory[]): number {
	return Math.max(...categories.map((category) => getSubmenuWidthInPixels(category)), 0)
}

function activateCategory(categoryKey: string) {
	clearPendingCategoryTimeout()
	pendingCategoryKey.value = null
	if (activeCategoryKey.value !== categoryKey) {
		categorySearchQuery.value = ''
	}
	activeCategoryKey.value = categoryKey
	scheduleSubmenuPositionUpdate()
	if (isMobileAddMenuLayout.value) {
		scheduleAddMenuPositionUpdate()
	}
}

function handleCategoryFocus(categoryKey: string) {
	if (isMobileAddMenuLayout.value) {
		return
	}

	activateCategory(categoryKey)
}

function handleCategoryMouseEnter(categoryKey: string) {
	if (isMobileAddMenuLayout.value) {
		return
	}

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

function getViewportRect(): ViewportRect {
	const visualViewport = window.visualViewport

	return {
		width: visualViewport?.width ?? window.innerWidth,
		height: visualViewport?.height ?? window.innerHeight,
		offsetTop: visualViewport?.offsetTop ?? 0,
		offsetLeft: visualViewport?.offsetLeft ?? 0,
	}
}

function getAddMenuPosition({ triggerRect, dropdownRect, viewport }: MenuPositionOptions) {
	const dropdownWidth = Math.max(ADD_MENU_WIDTH, triggerRect.width)
	const positionedDropdownWidth = Math.max(dropdownRect.width, dropdownWidth)
	const triggerTop = triggerRect.top + viewport.offsetTop
	const triggerBottom = triggerRect.bottom + viewport.offsetTop
	const triggerLeft = triggerRect.left + viewport.offsetLeft
	const viewportTop = viewport.offsetTop
	const viewportBottom = viewport.offsetTop + viewport.height
	const viewportLeft = viewport.offsetLeft
	const viewportRight = viewport.offsetLeft + viewport.width
	const minLeft = viewportLeft + DROPDOWN_VIEWPORT_MARGIN
	const maxLeft = Math.max(
		minLeft,
		viewportRight - positionedDropdownWidth - DROPDOWN_VIEWPORT_MARGIN,
	)
	const hasSpaceBelow =
		triggerBottom + dropdownRect.height + DROPDOWN_GAP + DROPDOWN_VIEWPORT_MARGIN <= viewportBottom
	const hasSpaceAbove =
		triggerTop - dropdownRect.height - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN > viewportTop
	const opensUp = !hasSpaceBelow && hasSpaceAbove
	const top = opensUp
		? triggerTop - dropdownRect.height - DROPDOWN_GAP
		: triggerBottom + DROPDOWN_GAP
	const left = Math.min(Math.max(minLeft, triggerLeft), maxLeft)

	return {
		left: `${left}px`,
		minWidth: `${triggerRect.width}px`,
		top: `${Math.max(viewportTop + DROPDOWN_VIEWPORT_MARGIN, top)}px`,
		width: `${dropdownWidth}px`,
	}
}

function getSubmenuPosition({
	buttonRect,
	openDirection,
	submenuWidth,
	submenuHeight,
	viewport,
}: SubmenuPositionOptions): Point {
	const buttonTop = buttonRect.top + viewport.offsetTop
	const buttonLeft = buttonRect.left + viewport.offsetLeft
	const buttonRight = buttonRect.right + viewport.offsetLeft
	const viewportTop = viewport.offsetTop
	const viewportBottom = viewport.offsetTop + viewport.height
	const viewportLeft = viewport.offsetLeft
	const viewportRight = viewport.offsetLeft + viewport.width
	const minLeft = viewportLeft + DROPDOWN_VIEWPORT_MARGIN
	const preferredLeft =
		openDirection === 'right'
			? buttonRight + DROPDOWN_GAP
			: buttonLeft - submenuWidth - DROPDOWN_GAP
	const maxLeft = Math.max(minLeft, viewportRight - submenuWidth - DROPDOWN_VIEWPORT_MARGIN)
	const left = Math.min(Math.max(minLeft, preferredLeft), maxLeft)
	const minTop = viewportTop + DROPDOWN_VIEWPORT_MARGIN
	const top = Math.min(
		Math.max(minTop, buttonTop),
		Math.max(minTop, viewportBottom - submenuHeight - DROPDOWN_VIEWPORT_MARGIN),
	)

	return {
		x: left,
		y: top,
	}
}

function getSubmenuOpenDirection({
	menuRect,
	widestSubmenuWidth,
	viewport,
}: SubmenuOpenDirectionOptions): SubmenuOpenDirection {
	const menuLeft = menuRect.left + viewport.offsetLeft
	const menuRight = menuRect.right + viewport.offsetLeft
	const viewportLeft = viewport.offsetLeft
	const viewportRight = viewport.offsetLeft + viewport.width
	const rightSpace = viewportRight - menuRight - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN
	const leftSpace = menuLeft - viewportLeft - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN

	if (rightSpace >= widestSubmenuWidth) {
		return 'right'
	}

	if (leftSpace >= widestSubmenuWidth) {
		return 'left'
	}

	return rightSpace >= leftSpace ? 'right' : 'left'
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
	const positioningElement =
		menuContainer.value ?? (isMobileActiveSubmenu.value ? submenu.value : null)
	if (typeof window === 'undefined' || !addMenuTrigger.value || !positioningElement) {
		return false
	}

	const triggerRect = addMenuTrigger.value.getBoundingClientRect()
	const dropdownWidth = Math.max(ADD_MENU_WIDTH, triggerRect.width)

	addMenuStyle.value = {
		...addMenuStyle.value,
		minWidth: `${triggerRect.width}px`,
		width: `${dropdownWidth}px`,
	}

	const dropdownRect = positioningElement.getBoundingClientRect()
	const viewport = getViewportRect()
	addMenuStyle.value = getAddMenuPosition({
		triggerRect,
		dropdownRect,
		viewport,
	})
	return true
}

function updateSubmenuOpenDirection(): boolean {
	if (typeof window === 'undefined' || !menuContainer.value) {
		return false
	}

	submenuOpenDirection.value = getSubmenuOpenDirection({
		menuRect: menuContainer.value.getBoundingClientRect(),
		widestSubmenuWidth: getWidestSubmenuWidthInPixels(filterCategories.value),
		viewport: getViewportRect(),
	})
	return true
}

function scheduleAddMenuPositionUpdate(retries = 8) {
	if (typeof window === 'undefined') {
		return
	}

	nextTick(() => {
		if (!isAddMenuOpen.value) {
			return
		}

		if (updateAddMenuPosition()) {
			if (!isMobileAddMenuLayout.value) {
				updateSubmenuOpenDirection()
			}
			return
		}

		if (retries <= 0) {
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

	if (isMobileAddMenuLayout.value) {
		hasSubmenuPosition.value = true
		return true
	}

	const activeButton = categoryButtonRefs.get(activeCategoryKey.value)
	if (!activeButton) {
		return false
	}

	const buttonRect = activeButton.getBoundingClientRect()
	const submenuRect = submenu.value?.getBoundingClientRect()
	const submenuWidth =
		submenuRect?.width ??
		(activeCategory.value ? getSubmenuWidthInPixels(activeCategory.value) : 256)
	const submenuHeight = submenuRect?.height ?? 320

	updateSubmenuOpenDirection()
	const viewport = getViewportRect()
	submenuPosition.value = getSubmenuPosition({
		buttonRect,
		openDirection: submenuOpenDirection.value,
		submenuWidth,
		submenuHeight,
		viewport,
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

	if (syncMobileAddMenuLayout()) {
		scheduleAddMenuPositionUpdate()
		scheduleSubmenuPositionUpdate()
		return
	}

	updateAddMenuPosition()
	if (!isMobileAddMenuLayout.value) {
		updateSubmenuOpenDirection()
	}
	updateSubmenuPosition()
}

function scheduleMenuPositionsUpdate() {
	if (typeof window === 'undefined' || addMenuPositionRafId !== null) {
		return
	}

	addMenuPositionRafId = window.requestAnimationFrame(() => {
		addMenuPositionRafId = null
		updateMenuPositions()
	})
}

function handleViewportChange() {
	scheduleMenuPositionsUpdate()
}

function startAddMenuPositionTracking() {
	if (typeof window === 'undefined') {
		return
	}

	window.addEventListener('resize', handleViewportChange)
	window.addEventListener('scroll', handleViewportChange, true)
	window.visualViewport?.addEventListener('scroll', handleViewportChange)
	window.visualViewport?.addEventListener('resize', handleViewportChange)
}

function stopAddMenuPositionTracking() {
	if (typeof window === 'undefined') {
		return
	}

	window.removeEventListener('resize', handleViewportChange)
	window.removeEventListener('scroll', handleViewportChange, true)
	window.visualViewport?.removeEventListener('scroll', handleViewportChange)
	window.visualViewport?.removeEventListener('resize', handleViewportChange)

	if (addMenuPositionRafId !== null) {
		window.cancelAnimationFrame(addMenuPositionRafId)
		addMenuPositionRafId = null
	}
}

onClickOutside(
	addMenuOutsideClickTarget,
	() => {
		closeAddMenu()
	},
	{ ignore: addMenuOutsideClickIgnore },
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
		resetActiveCategoryOptionsScrollState()
		stopAddMenuPositionTracking()
		destroyActiveCategoryOptionsOverlayScrollbars()
	}
})

watch(categorySearchQuery, () => {
	resetActiveCategoryOptionsScrollState()
	initializeActiveCategoryOptionsOverlayScrollbars()
	scheduleSubmenuPositionUpdate()
})

watch(activeCategoryKey, (categoryKey) => {
	resetActiveCategoryOptionsScrollState()
	if (categoryKey) {
		initializeActiveCategoryOptionsOverlayScrollbars()
	} else {
		destroyActiveCategoryOptionsOverlayScrollbars()
	}
})

watch(filteredActiveCategoryOptions, () => {
	if (!isAddMenuOpen.value) {
		return
	}

	if (filteredActiveCategoryOptions.value.length > 0) {
		initializeActiveCategoryOptionsOverlayScrollbars()
	} else {
		destroyActiveCategoryOptionsOverlayScrollbars()
	}
	scheduleSubmenuPositionUpdate()
})

watch(filterCategoriesByKey, (nextCategories) => {
	if (activeCategoryKey.value && !nextCategories.has(activeCategoryKey.value)) {
		activeCategoryKey.value = null
		hasSubmenuPosition.value = false
	}

	if (isAddMenuOpen.value) {
		updateSubmenuOpenDirection()
		scheduleSubmenuPositionUpdate()
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
	destroyActiveCategoryOptionsOverlayScrollbars()
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

.dropdown-filter-bar-options-scrollbar :deep(.os-theme-modrinth) {
	--os-size: 10px;
	--os-padding-perpendicular: 2px;
	--os-padding-axis: 2px;
	--os-track-bg: transparent;
	--os-track-bg-hover: transparent;
	--os-track-bg-active: transparent;
	--os-handle-border-radius: 9999px;
	--os-handle-border: 2px solid var(--color-surface-4);
	--os-handle-border-hover: 2px solid var(--color-surface-4);
	--os-handle-border-active: 2px solid var(--color-surface-4);
	--os-handle-bg: var(--color-scrollbar, var(--color-surface-5));
	--os-handle-bg-hover: var(--color-scrollbar, var(--color-surface-5));
	--os-handle-bg-active: var(--color-scrollbar, var(--color-surface-5));
}
</style>
