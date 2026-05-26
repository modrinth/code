<template>
	<div ref="containerRef" class="relative inline-block" :class="fitContent ? 'w-auto' : 'w-full'">
		<span
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative flex items-center overflow-hidden rounded-xl bg-surface-4 px-3 py-1 text-left transition-all duration-200"
			:class="[
				fitContent ? 'w-auto max-w-full' : 'w-full',
				triggerClass,
				{
					'z-[9999]': isOpen,
					'cursor-not-allowed opacity-50': disabled,
					'cursor-pointer hover:brightness-125 active:brightness-125': !disabled,
				},
			]"
			:aria-expanded="isOpen"
			aria-haspopup="listbox"
			:aria-disabled="disabled || undefined"
			@click="handleTriggerClick($event)"
			@keydown="handleTriggerKeydown"
		>
			<slot
				v-if="hasCustomInputContent"
				name="input-content"
				:is-open="isOpen"
				:model-value="modelValue"
				:selected-options="selectedOptions"
				:clear-all="clearAll"
				:toggle-open="toggleDropdown"
				:open-direction="openDirection"
			/>
			<template v-else>
				<div
					ref="tagsContainerRef"
					class="flex min-h-8 flex-1 flex-wrap items-center gap-1.5 overflow-hidden"
					:style="{ maxHeight: `calc(${maxTagRows} * 30px + ${maxTagRows - 1} * 6px)` }"
				>
					<span
						v-for="tag in visibleTags"
						:key="String(tag.value)"
						class="inline-flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm font-medium text-primary transition-all hover:brightness-[115%]"
						@click.stop="removeTag(tag.value)"
					>
						{{ tag.label }}
						<XIcon class="size-3.5 shrink-0 text-secondary" />
					</span>
					<Menu
						v-show="overflowCount > 0"
						:delay="{ hide: 50, show: 0 }"
						no-auto-focus
						:auto-hide="false"
						@apply-show="popperOverflowTags = [...overflowTags]"
					>
						<span
							class="inline-flex cursor-default select-none items-center rounded-full border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm font-medium text-secondary"
							@click.stop
						>
							+{{ overflowCount }}
						</span>
						<template #popper>
							<div class="flex max-w-[20rem] flex-wrap gap-1" @mousedown.prevent>
								<span
									v-for="tag in overflowTags"
									:key="String(tag.value)"
									class="inline-flex cursor-pointer items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium text-primary hover:brightness-[115%]"
									@click.stop="removeTag(tag.value)"
								>
									{{ tag.label }}
									<XIcon class="size-3.5 shrink-0 text-secondary" />
								</span>
							</div>
						</template>
					</Menu>
					<span
						v-if="selectedOptions.length === 0"
						class="text-primary opacity-50 text-base font-medium"
					>
						{{ placeholder }}
					</span>
				</div>
				<div class="ml-2 flex shrink-0 items-center gap-1.5">
					<button
						v-if="clearable && modelValue.length > 0"
						type="button"
						class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-all hover:text-contrast"
						aria-label="Clear all"
						@click.stop="clearAll"
					>
						<XIcon class="size-5" />
					</button>
					<div
						v-if="clearable && modelValue.length > 0"
						class="h-5 w-[1px] shrink-0 bg-surface-5"
					></div>
					<ChevronLeftIcon
						v-if="showChevron"
						class="size-5 shrink-0 text-secondary transition-transform duration-150"
						:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
					/>
				</div>
			</template>
		</span>

		<Teleport to="#teleports">
			<Transition
				enter-active-class="transition-opacity duration-150"
				leave-active-class="transition-opacity duration-150"
				enter-from-class="opacity-0"
				leave-to-class="opacity-0"
			>
				<div
					v-if="isOpen"
					ref="dropdownRef"
					class="fixed z-[9999] flex flex-col overflow-hidden rounded-[14px] bg-surface-4 border border-solid border-surface-5"
					:class="[
						openDirection === 'up' ? 'shadow-[0_-25px_50px_-12px_rgb(0,0,0,0.25)]' : 'shadow-2xl',
					]"
					:style="dropdownStyle"
					role="listbox"
					aria-multiselectable="true"
					@mousedown.stop
					@keydown="handleDropdownKeydown"
				>
					<div class="empty:hidden">
						<div
							v-if="searchable"
							class="px-0 py-1.5 border-0 border-solid border-b border-b-surface-5 flex"
						>
							<StyledInput
								ref="searchInputRef"
								v-model="searchQuery"
								:icon="SearchIcon"
								type="text"
								:placeholder="searchPlaceholder"
								wrapper-class="grow bg-surface-4 mx-0"
								input-class="ps-9 mx-1.5"
								@input="handleSearchInput"
								@keydown="handleSearchKeydown"
							/>
						</div>

						<div
							v-if="hasFilteredOptions || shouldShowSelectAll"
							class="flex flex-col bg-surface-4 border-0 border-solid border-b border-b-surface-5 empty:hidden"
						>
							<div v-if="shouldShowSelectAll" class="sticky top-0 z-10 bg-surface-4">
								<span
									class="flex w-full items-center gap-2.5 cursor-pointer px-4 py-3 text-left transition-all duration-150 text-contrast hover:brightness-[115%]"
									:class="{ 'brightness-[115%]': focusedIndex === -2 }"
									data-option-index="-2"
									:data-focused="focusedIndex === -2"
									role="option"
									:aria-selected="isAllSelected"
									tabindex="-1"
									@click="toggleSelectAll"
									@mouseenter="focusedIndex = -2"
								>
									<span
										v-if="checkboxPosition === 'left'"
										class="w-5 h-5 rounded-md flex items-center justify-center border-[1px] border-solid shrink-0 checkbox-shadow"
										:class="[
											isAllSelected
												? 'bg-brand border-button-border text-brand-inverted'
												: 'bg-surface-2 border-surface-5',
											isIndeterminate ? 'text-primary' : '',
										]"
									>
										<MinusIcon v-if="isIndeterminate" aria-hidden="true" stroke-width="3" />
										<CheckIcon v-else-if="isAllSelected" aria-hidden="true" stroke-width="3" />
									</span>
									<span class="min-w-0 flex-1 font-semibold leading-tight text-primary">
										{{ selectAllLabel }}
									</span>
									<span
										v-if="checkboxPosition === 'right'"
										class="flex items-center justify-center shrink-0 text-brand"
									>
										<MinusIcon v-if="isIndeterminate" aria-hidden="true" class="size-5" />
										<CheckIcon v-else-if="isAllSelected" aria-hidden="true" class="size-5" />
									</span>
								</span>
							</div>
						</div>

						<div v-if="$slots.top" class="border-0 border-b border-solid border-b-surface-5">
							<slot
								name="top"
								:model-value="modelValue"
								:selected-options="selectedOptions"
								:clear-all="clearAll"
								:is-open="isOpen"
							></slot>
						</div>

						<div
							v-if="shouldShowSelectionActions && hasFilteredOptions"
							ref="selectionActionsRef"
							class="flex items-center justify-between gap-3 border-0 border-b border-solid border-b-surface-5 bg-surface-4 px-4 py-2.5 text-sm"
						>
							<span class="font-semibold text-secondary">{{ selectionActionsLabel }}</span>
							<button
								type="button"
								class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-all hover:bg-transparent hover:text-contrast"
								@click="clearAll"
								@keydown.enter.stop
								@keydown.space.stop
							>
								{{ selectionActionsClearLabel }}
							</button>
						</div>
					</div>

					<div
						v-if="hasFilteredOptions"
						ref="optionsScrollbarRef"
						class="multi-select-options-scrollbar bg-surface-4"
						data-overlayscrollbars-initialize
					>
						<div
							ref="optionsContainerRef"
							class="overflow-y-auto overscroll-contain select-none"
							:style="{ maxHeight: `${maxHeight}px` }"
							data-overlayscrollbars-viewport
						>
							<div
								ref="listContainer"
								:class="shouldVirtualizeOptions ? 'relative' : 'flex flex-col'"
								:style="optionsListStyle"
							>
								<template
									v-for="{ item, index } in renderedVisibleOptions"
									:key="getItemKey(item, index)"
								>
									<div
										class="group/option-container focus-visible:outline-none"
										:class="shouldVirtualizeOptions ? 'absolute left-0 right-0' : undefined"
										:style="getOptionWrapperStyle(index)"
									>
										<div
											v-if="isSectionHeader(item)"
											class="flex items-center justify-between gap-3 text-sm font-semibold text-secondary border-t border-surface-5 border-solid border-0 group-first/option-container:border-t-0"
											:class="[
												item.class,
												shouldVirtualizeOptions ? 'h-10 px-4' : 'h-10 px-4 pb-1 pt-2',
											]"
											role="presentation"
										>
											<span class="min-w-0 truncate">{{ item.label }}</span>
											<button
												v-if="hasSelectableSectionHeaderOptions(item)"
												type="button"
												class="shrink-0 border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-all hover:bg-transparent hover:text-contrast"
												@click.stop="toggleSectionHeaderOptions(item)"
												@keydown.enter.stop
												@keydown.space.stop
											>
												{{ areSectionHeaderOptionsSelected(item) ? 'Clear' : 'Select all' }}
											</button>
										</div>
										<span
											v-else
											role="option"
											:aria-selected="item.selected"
											:aria-disabled="item.disabled || undefined"
											:data-option-index="index"
											:data-focused="focusedIndex === index"
											class="flex w-full cursor-pointer items-center gap-2.5 px-4 py-3 outline-none focus-visible:outline-none text-left text-contrast transition-all duration-150 bg-surface-4 hover:brightness-[115%] focus-visible:brightness-[115%]"
											:class="[
												item.class,
												shouldVirtualizeOptions ? 'h-12' : undefined,
												{
													'brightness-[115%]': item.selected,
													'pointer-events-none cursor-not-allowed opacity-50': item.disabled,
												},
											]"
											tabindex="-1"
											@click="toggleOption(item, $event)"
											@mouseenter="!item.disabled && (focusedIndex = index)"
										>
											<span
												v-if="checkboxPosition === 'left'"
												class="checkbox-shadow flex h-5 w-5 shrink-0 items-center justify-center rounded-md border-[1px] border-solid"
												:class="
													item.selected
														? 'border-button-border bg-brand text-brand-inverted'
														: 'border-surface-5 bg-surface-2'
												"
											>
												<CheckIcon v-if="item.selected" aria-hidden="true" stroke-width="3" />
											</span>
											<slot name="option" :item="item" :selected="item.selected" :index="index">
												<slot
													:name="`option-${item.value}`"
													:item="item"
													:selected="item.selected"
													:index="index"
												>
													<div class="flex min-w-0 flex-1 items-center justify-between gap-3">
														<div class="flex min-w-0 items-center gap-2">
															<component
																:is="item.icon"
																v-if="item.icon"
																class="h-5 w-5 shrink-0"
															/>
															<span
																class="min-w-0 truncate font-semibold leading-tight"
																:class="item.selected ? 'text-contrast' : 'text-primary'"
															>
																{{ item.label }}
															</span>
														</div>
														<slot
															name="option-right"
															:item="item"
															:selected="item.selected"
															:index="index"
														></slot>
													</div>
												</slot>
											</slot>
											<span
												v-if="checkboxPosition === 'right'"
												class="flex shrink-0 items-center justify-center text-brand"
											>
												<CheckIcon v-if="item.selected" aria-hidden="true" class="size-5" />
											</span>
										</span>
									</div>
								</template>
							</div>
						</div>
					</div>
					<template v-else>
						<div
							v-if="shouldShowSelectionActions"
							class="flex items-center justify-between gap-3 border-0 border-b border-solid border-b-surface-5 px-3 py-2.5 text-sm"
						>
							<span class="font-semibold text-secondary">{{ selectionActionsLabel }}</span>
							<button
								type="button"
								class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-all hover:bg-transparent hover:text-contrast"
								@click="clearAll"
								@keydown.enter.stop
								@keydown.space.stop
							>
								{{ selectionActionsClearLabel }}
							</button>
						</div>
						<div
							v-if="isNoOptionsState && noOptionsMessage"
							class="p-4 mb-2 text-center text-sm text-secondary"
						>
							{{ noOptionsMessage }}
						</div>
						<div v-else-if="searchQuery" class="p-4 mb-2 text-center text-sm text-secondary">
							{{ noResultsMessage }}
						</div>
					</template>

					<div v-if="$slots.bottom" @keydown.stop>
						<slot name="bottom"></slot>
					</div>

					<slot name="dropdown-footer"></slot>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>

<script setup lang="ts" generic="T">
import 'overlayscrollbars/overlayscrollbars.css'

import { CheckIcon, ChevronLeftIcon, MinusIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import { Menu } from 'floating-vue'
import { OverlayScrollbars, type PartialOptions } from 'overlayscrollbars'
import {
	type Component,
	computed,
	nextTick,
	onMounted,
	onUnmounted,
	ref,
	shallowRef,
	useSlots,
	watch,
} from 'vue'

import { useVirtualScroll } from '../../composables/virtual-scroll'
import StyledInput from './StyledInput.vue'

export interface MultiSelectOption<T> {
	value: T
	label: string
	icon?: Component
	disabled?: boolean
	class?: string
	searchTerms?: string[]
}

export interface MultiSelectSectionHeader {
	type: 'section-header'
	label: string
	key?: string
	class?: string
}

export type MultiSelectItem<T> = MultiSelectOption<T> | MultiSelectSectionHeader

type RenderedMultiSelectOption<T> = MultiSelectOption<T> & {
	selected: boolean
}

type RenderedMultiSelectItem<T> = RenderedMultiSelectOption<T> | MultiSelectSectionHeader

type VisibleMultiSelectItem<T> = {
	item: RenderedMultiSelectItem<T>
	index: number
}

type OverlayScrollbarsInstance = NonNullable<ReturnType<typeof OverlayScrollbars>>
type ViewportRect = {
	width: number
	height: number
	offsetTop: number
	offsetLeft: number
}

const DROPDOWN_VIEWPORT_MARGIN = 8
const DROPDOWN_GAP = 8
const DEFAULT_MAX_HEIGHT = 300
const MULTI_SELECT_OPTION_ROW_HEIGHT = 48
const MULTI_SELECT_VIRTUALIZATION_THRESHOLD = 80
const MOBILE_SEARCH_AUTO_FOCUS_QUERY = '(pointer: coarse), (max-width: 800px)'
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

function isSectionHeader<T>(item: MultiSelectItem<T>): item is MultiSelectSectionHeader {
	return 'type' in item && item.type === 'section-header'
}

function isOption<T>(item: MultiSelectItem<T>): item is MultiSelectOption<T> {
	return !isSectionHeader(item)
}

const props = withDefaults(
	defineProps<{
		modelValue: T[]
		options: MultiSelectItem<T>[]
		placeholder?: string
		disabled?: boolean
		searchable?: boolean
		searchPlaceholder?: string
		showChevron?: boolean
		clearable?: boolean
		maxHeight?: number
		triggerClass?: string
		fitContent?: boolean
		/** Width for the teleported dropdown; defaults to the trigger width */
		dropdownWidth?: string | number
		/** Minimum width for the teleported dropdown */
		dropdownMinWidth?: string | number
		forceDirection?: 'up' | 'down'
		noOptionsMessage?: string
		noResultsMessage?: string
		disableSearchFilter?: boolean
		includeSelectAllOption?: boolean
		selectAllLabel?: string
		showSelectionActions?: boolean
		selectionActionsClearLabel?: string
		maxTagRows?: number
		checkboxPosition?: 'left' | 'right'
	}>(),
	{
		placeholder: 'Select options',
		disabled: false,
		searchable: false,
		searchPlaceholder: 'Search...',
		showChevron: true,
		clearable: true,
		maxHeight: DEFAULT_MAX_HEIGHT,
		fitContent: false,
		noOptionsMessage: 'No options available',
		noResultsMessage: 'No results found',
		includeSelectAllOption: false,
		selectAllLabel: 'Select all',
		showSelectionActions: false,
		selectionActionsClearLabel: 'Clear',
		maxTagRows: 1,
		checkboxPosition: 'left',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T[]]
	open: []
	close: []
	searchInput: [query: string]
}>()

const slots = useSlots()
const isOpen = ref(false)
const searchQuery = ref('')
const focusedIndex = ref(-1)
const containerRef = ref<HTMLElement>()
const triggerRef = ref<HTMLElement>()
const dropdownRef = ref<HTMLElement>()
const optionsScrollbarRef = ref<HTMLElement>()
const optionsContainerRef = ref<HTMLElement>()
const selectionActionsRef = ref<HTMLElement>()
const searchInputRef = ref<InstanceType<typeof StyledInput>>()
const rafId = ref<number | null>(null)
const tagsContainerRef = ref<HTMLElement>()
const optionsOverlayScrollbars = ref<OverlayScrollbarsInstance | null>(null)
const lastSelectionActionsHeight = ref(0)

const dropdownStyle = ref({
	top: '0px',
	left: '0px',
	width: '0px',
	minWidth: '0px',
})

const openDirection = ref<'down' | 'up'>('down')
const hasCustomInputContent = computed(() => Boolean(slots['input-content']))

const selectableOptions = computed(() => props.options.filter(isOption))
const enabledSelectableOptions = computed(() =>
	selectableOptions.value.filter((opt) => !opt.disabled),
)
const selectedValueSet = computed(() => new Set(props.modelValue))

const selectedOptions = computed(() => {
	const selectedValues = selectedValueSet.value
	return selectableOptions.value.filter((opt) => selectedValues.has(opt.value))
})

const isAllSelected = computed(() => {
	const selectableOptions = enabledSelectableOptions.value
	const selectedValues = selectedValueSet.value
	return (
		selectableOptions.length > 0 && selectableOptions.every((opt) => selectedValues.has(opt.value))
	)
})

const isIndeterminate = computed(() => {
	const selectedValues = selectedValueSet.value
	return (
		!isAllSelected.value &&
		selectableOptions.value.some((opt) => !opt.disabled && selectedValues.has(opt.value))
	)
})

const visibleTagCount = ref(Infinity)

const visibleTags = computed(() => {
	return selectedOptions.value.slice(0, visibleTagCount.value)
})

const overflowCount = computed(() => {
	return Math.max(0, selectedOptions.value.length - visibleTagCount.value)
})

const overflowTags = computed(() => {
	return selectedOptions.value.slice(visibleTagCount.value)
})

const popperOverflowTags = shallowRef<MultiSelectOption<T>[]>([])

const lastClickedValue = shallowRef<{ value: T } | null>(null)

const filteredOptions = computed(() => {
	if (!searchQuery.value || !props.searchable || props.disableSearchFilter) {
		return props.options
	}

	const query = searchQuery.value.toLowerCase()
	const items: MultiSelectItem<T>[] = []
	let pendingSectionHeader: MultiSelectSectionHeader | null = null

	for (const opt of props.options) {
		if (isSectionHeader(opt)) {
			pendingSectionHeader = opt
			continue
		}

		const matches =
			opt.label.toLowerCase().includes(query) ||
			opt.searchTerms?.some((term) => term.toLowerCase().includes(query))

		if (!matches) {
			continue
		}

		if (pendingSectionHeader) {
			items.push(pendingSectionHeader)
			pendingSectionHeader = null
		}
		items.push(opt)
	}

	return items
})

const renderedFilteredOptions = computed<RenderedMultiSelectItem<T>[]>(() => {
	const selectedValues = selectedValueSet.value
	return filteredOptions.value.map((item) => {
		if (isSectionHeader(item)) {
			return item
		}

		return {
			...item,
			selected: selectedValues.has(item.value),
		}
	})
})

const shouldVirtualizeOptions = computed(
	() => renderedFilteredOptions.value.length > MULTI_SELECT_VIRTUALIZATION_THRESHOLD,
)

const { listContainer, totalHeight, visibleRange, visibleItems } = useVirtualScroll(
	renderedFilteredOptions,
	{
		itemHeight: MULTI_SELECT_OPTION_ROW_HEIGHT,
		bufferSize: 8,
		initialItemCount: 12,
		enabled: shouldVirtualizeOptions,
	},
)

const renderedVisibleOptions = computed<VisibleMultiSelectItem<T>[]>(() =>
	visibleItems.value.map((item, offset) => ({
		item,
		index: visibleRange.value.start + offset,
	})),
)
const optionsListStyle = computed(() =>
	shouldVirtualizeOptions.value ? { height: `${totalHeight.value}px` } : undefined,
)

const hasFilteredOptions = computed(() => filteredOptions.value.some(isOption))
const isNoOptionsState = computed(() => selectableOptions.value.length === 0 && !searchQuery.value)
const shouldShowSelectAll = computed(
	() => props.includeSelectAllOption && enabledSelectableOptions.value.length > 1,
)
const selectedOptionCount = computed(() => selectedOptions.value.length)
const shouldShowSelectionActions = computed(
	() => props.showSelectionActions && selectedOptionCount.value > 0,
)
const selectionActionsLabel = computed(() => {
	return selectedOptionCount.value === 1 ? '1 selected' : `${selectedOptionCount.value} selected`
})

function isSelected(value: T) {
	return selectedValueSet.value.has(value)
}

function getItemKey(item: MultiSelectItem<T>, index: number) {
	if (isSectionHeader(item)) {
		return item.key ?? `section-header-${item.label}-${index}`
	}

	return `option-${String(item.value)}`
}

function getSectionHeaderOptions(sectionHeader: MultiSelectSectionHeader) {
	const sectionHeaderIndex = props.options.findIndex((item) => item === sectionHeader)
	if (sectionHeaderIndex === -1) {
		return []
	}

	const sectionHeaderOptions: MultiSelectOption<T>[] = []
	for (let i = sectionHeaderIndex + 1; i < props.options.length; i++) {
		const item = props.options[i]
		if (!item || isSectionHeader(item)) {
			break
		}
		if (!item.disabled) {
			sectionHeaderOptions.push(item)
		}
	}

	return sectionHeaderOptions
}

function hasSelectableSectionHeaderOptions(sectionHeader: MultiSelectSectionHeader) {
	return getSectionHeaderOptions(sectionHeader).length > 1
}

function areSectionHeaderOptionsSelected(sectionHeader: MultiSelectSectionHeader) {
	const sectionHeaderOptions = getSectionHeaderOptions(sectionHeader)
	return (
		sectionHeaderOptions.length > 0 &&
		sectionHeaderOptions.every((option) => isSelected(option.value))
	)
}

function toggleSectionHeaderOptions(sectionHeader: MultiSelectSectionHeader) {
	const sectionHeaderOptions = getSectionHeaderOptions(sectionHeader)
	if (sectionHeaderOptions.length === 0) {
		return
	}

	let newValue: T[]
	if (sectionHeaderOptions.every((option) => isSelected(option.value))) {
		const sectionHeaderValues = new Set(sectionHeaderOptions.map((option) => option.value))
		newValue = props.modelValue.filter((value) => !sectionHeaderValues.has(value))
	} else {
		newValue = [...props.modelValue]
		for (const option of sectionHeaderOptions) {
			if (!newValue.includes(option.value)) {
				newValue.push(option.value)
			}
		}
	}

	emit('update:modelValue', newValue)
	const lastSectionHeaderOption = sectionHeaderOptions[sectionHeaderOptions.length - 1]
	if (lastSectionHeaderOption) {
		lastClickedValue.value = { value: lastSectionHeaderOption.value }
	}
}

function toggleOption(option: MultiSelectOption<T>, event?: MouseEvent | KeyboardEvent) {
	if (option.disabled) return

	if (event?.shiftKey && lastClickedValue.value) {
		const anchorValue = lastClickedValue.value.value
		const anchorIndex = filteredOptions.value.findIndex(
			(opt) => isOption(opt) && opt.value === anchorValue,
		)
		const currentIndex = filteredOptions.value.findIndex(
			(opt) => isOption(opt) && opt.value === option.value,
		)

		if (anchorIndex !== -1 && currentIndex !== -1 && anchorIndex !== currentIndex) {
			const start = Math.min(anchorIndex, currentIndex)
			const end = Math.max(anchorIndex, currentIndex)
			const shouldSelect = !isSelected(option.value)
			const newValue = [...props.modelValue]

			for (let i = start; i <= end; i++) {
				const opt = filteredOptions.value[i]
				if (!opt || isSectionHeader(opt) || opt.disabled) continue
				const idx = newValue.indexOf(opt.value)
				if (shouldSelect && idx === -1) {
					newValue.push(opt.value)
				} else if (!shouldSelect && idx !== -1) {
					newValue.splice(idx, 1)
				}
			}

			emit('update:modelValue', newValue)
			lastClickedValue.value = { value: option.value }
			return
		}
	}

	const newValue = isSelected(option.value)
		? props.modelValue.filter((v) => v !== option.value)
		: [...props.modelValue, option.value]

	emit('update:modelValue', newValue)
	lastClickedValue.value = { value: option.value }
}

function removeTag(value: T) {
	emit(
		'update:modelValue',
		props.modelValue.filter((v) => v !== value),
	)
}

function clearAll() {
	emit('update:modelValue', [])
}

function toggleDropdown() {
	if (isOpen.value) {
		closeDropdown()
	} else {
		openDropdown()
	}
}

function toggleSelectAll() {
	if (isAllSelected.value) {
		emit('update:modelValue', [])
	} else {
		const allValues = enabledSelectableOptions.value.map((opt) => opt.value)
		emit('update:modelValue', allValues)
	}
}

async function calculateVisibleTags() {
	visibleTagCount.value = Infinity
	await nextTick()

	if (!tagsContainerRef.value || selectedOptions.value.length === 0) return

	const container = tagsContainerRef.value
	const maxH = container.offsetHeight
	if (container.scrollHeight <= maxH) return

	let count = selectedOptions.value.length
	while (count > 0) {
		count--
		visibleTagCount.value = count
		await nextTick()
		if (container.scrollHeight <= container.offsetHeight) return
	}
}

function determineOpenDirection(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	viewport: ViewportRect,
): 'up' | 'down' {
	if (props.forceDirection) return props.forceDirection

	const triggerTop = triggerRect.top + viewport.offsetTop
	const triggerBottom = triggerRect.bottom + viewport.offsetTop
	const viewportTop = viewport.offsetTop
	const viewportBottom = viewport.offsetTop + viewport.height
	const hasSpaceBelow =
		triggerBottom + dropdownRect.height + DROPDOWN_GAP + DROPDOWN_VIEWPORT_MARGIN <= viewportBottom
	const hasSpaceAbove =
		triggerTop - dropdownRect.height - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN > viewportTop

	return !hasSpaceBelow && hasSpaceAbove ? 'up' : 'down'
}

function calculateVerticalPosition(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	direction: 'up' | 'down',
	viewport: ViewportRect,
): number {
	const top =
		direction === 'up'
			? triggerRect.top - dropdownRect.height - DROPDOWN_GAP
			: triggerRect.bottom + DROPDOWN_GAP

	return top + viewport.offsetTop
}

function calculateHorizontalPosition(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	viewport: ViewportRect,
): number {
	const minLeft = viewport.offsetLeft + DROPDOWN_VIEWPORT_MARGIN
	const maxRight = viewport.offsetLeft + viewport.width - DROPDOWN_VIEWPORT_MARGIN
	let left = triggerRect.left + viewport.offsetLeft

	if (left + dropdownRect.width > maxRight) {
		left = Math.max(minLeft, maxRight - dropdownRect.width)
	}
	return left
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

function resolveDropdownWidth(triggerWidth: number): string {
	if (props.dropdownWidth === undefined) return `${triggerWidth}px`
	if (typeof props.dropdownWidth === 'number') return `${props.dropdownWidth}px`
	return props.dropdownWidth
}

function resolveCssSize(size: string | number | undefined): string | undefined {
	if (size === undefined) return undefined
	if (typeof size === 'number') return `${size}px`
	return size
}

async function updateDropdownPosition() {
	if (!triggerRef.value || !dropdownRef.value) return

	await nextTick()

	const triggerRect = triggerRef.value.getBoundingClientRect()
	const width = resolveDropdownWidth(triggerRect.width)
	const minWidth = resolveCssSize(props.dropdownMinWidth) ?? '0px'

	dropdownStyle.value = {
		...dropdownStyle.value,
		width,
		minWidth,
	}

	await nextTick()

	const dropdownRect = dropdownRef.value.getBoundingClientRect()
	const viewport = getViewportRect()

	const direction = determineOpenDirection(triggerRect, dropdownRect, viewport)
	const top = calculateVerticalPosition(triggerRect, dropdownRect, direction, viewport)
	const left = calculateHorizontalPosition(triggerRect, dropdownRect, viewport)

	dropdownStyle.value = {
		top: `${top}px`,
		left: `${left}px`,
		width,
		minWidth,
	}

	openDirection.value = direction
}

async function initializeOptionsOverlayScrollbars() {
	await nextTick()

	if (!isOpen.value || !hasFilteredOptions.value) {
		destroyOptionsOverlayScrollbars()
		return
	}

	if (!optionsScrollbarRef.value || !optionsContainerRef.value || !listContainer.value) {
		return
	}

	if (optionsOverlayScrollbars.value) {
		optionsOverlayScrollbars.value.update(true)
		return
	}

	optionsOverlayScrollbars.value = OverlayScrollbars(
		{
			target: optionsScrollbarRef.value,
			elements: {
				viewport: optionsContainerRef.value,
				content: listContainer.value,
			},
		},
		OPTIONS_OVERLAY_SCROLLBARS_OPTIONS,
	)
}

function getSelectionActionsHeight() {
	return shouldShowSelectionActions.value ? (selectionActionsRef.value?.offsetHeight ?? 0) : 0
}

async function syncSelectionActionsHeight() {
	await nextTick()
	lastSelectionActionsHeight.value = getSelectionActionsHeight()
}

function updateOptionsOverlayScrollbars() {
	nextTick(() => {
		optionsOverlayScrollbars.value?.update(true)
	})
}

function destroyOptionsOverlayScrollbars() {
	optionsOverlayScrollbars.value?.destroy()
	optionsOverlayScrollbars.value = null
}

function shouldAutoFocusSearch() {
	return (
		props.searchable &&
		typeof window !== 'undefined' &&
		!window.matchMedia(MOBILE_SEARCH_AUTO_FOCUS_QUERY).matches
	)
}

async function openDropdown() {
	if (props.disabled || isOpen.value) return

	isOpen.value = true
	emit('open')

	await nextTick()
	await updateDropdownPosition()
	await initializeOptionsOverlayScrollbars()
	await syncSelectionActionsHeight()

	if (shouldAutoFocusSearch() && searchInputRef.value) {
		;(searchInputRef.value as unknown as { focus: () => void }).focus()
	}

	focusedIndex.value = shouldShowSelectAll.value ? -2 : getFirstFocusableOptionIndex()
	startPositionTracking()
}

function closeDropdown() {
	if (!isOpen.value) return

	stopPositionTracking()
	destroyOptionsOverlayScrollbars()
	isOpen.value = false
	searchQuery.value = ''
	focusedIndex.value = -1
	emit('close')

	nextTick(() => {
		triggerRef.value?.focus()
	})
}

function handleTriggerClick(event: MouseEvent) {
	if (event.detail === 0) return

	if (isOpen.value) {
		closeDropdown()
	} else {
		openDropdown()
	}
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (isOpen.value) {
		handleDropdownKeydown(event)
		return
	}
	switch (event.key) {
		case 'Enter':
		case ' ':
		case 'ArrowDown':
		case 'ArrowUp':
			event.preventDefault()
			openDropdown()
			break
	}
}

function isFocusableOptionIndex(index: number) {
	const option = filteredOptions.value[index]
	return option !== undefined && isOption(option) && !option.disabled
}

function getFirstFocusableOptionIndex() {
	return filteredOptions.value.findIndex((_, index) => isFocusableOptionIndex(index))
}

function findNextFocusableOption(currentIndex: number, direction: 'next' | 'previous') {
	const length = filteredOptions.value.length
	if (length === 0) return -1

	let index = currentIndex
	for (let i = 0; i < length; i++) {
		index = direction === 'next' ? (index + 1) % length : (index - 1 + length) % length
		if (isFocusableOptionIndex(index)) {
			return index
		}
	}

	return -1
}

function focusNextOption() {
	const length = filteredOptions.value.length
	if (length === 0) return

	const nextIndex = findNextFocusableOption(
		focusedIndex.value === -2 ? -1 : focusedIndex.value,
		'next',
	)
	if (nextIndex === -1) return

	focusedIndex.value = nextIndex
	focusOptionIndex(focusedIndex.value)
}

function focusPreviousOption() {
	const length = filteredOptions.value.length
	if (length === 0) return

	if (focusedIndex.value === getFirstFocusableOptionIndex() && shouldShowSelectAll.value) {
		focusedIndex.value = -2
		focusOptionIndex(focusedIndex.value)
		return
	}

	const previousIndex = findNextFocusableOption(
		focusedIndex.value === -1 ? 0 : focusedIndex.value,
		'previous',
	)
	if (previousIndex === -1) return

	focusedIndex.value = previousIndex
	focusOptionIndex(focusedIndex.value)
}

function scrollOptionIndexIntoView(index: number) {
	if (index < 0) {
		return
	}

	const container = optionsContainerRef.value
	if (!container) {
		return
	}

	const optionElement = container.querySelector<HTMLElement>(`[data-option-index="${index}"]`)
	if (optionElement) {
		const containerRect = container.getBoundingClientRect()
		const optionRect = optionElement.getBoundingClientRect()

		if (optionRect.top < containerRect.top) {
			container.scrollTop -= containerRect.top - optionRect.top
		} else if (optionRect.bottom > containerRect.bottom) {
			container.scrollTop += optionRect.bottom - containerRect.bottom
		}
		return
	}

	const optionTop = index * MULTI_SELECT_OPTION_ROW_HEIGHT
	const optionBottom = optionTop + MULTI_SELECT_OPTION_ROW_HEIGHT
	if (optionTop < container.scrollTop) {
		container.scrollTop = optionTop
	} else if (optionBottom > container.scrollTop + container.clientHeight) {
		container.scrollTop = optionBottom - container.clientHeight
	}
}

function focusOptionIndex(index: number) {
	scrollOptionIndexIntoView(index)
	nextTick(() => {
		dropdownRef.value?.querySelector<HTMLElement>(`[data-option-index="${index}"]`)?.focus()
	})
}

function getOptionWrapperStyle(index: number) {
	if (!shouldVirtualizeOptions.value) {
		return undefined
	}

	return {
		transform: `translateY(${index * MULTI_SELECT_OPTION_ROW_HEIGHT}px)`,
	}
}

function handleDropdownKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'Escape':
			event.preventDefault()
			closeDropdown()
			break
		case 'ArrowDown':
			event.preventDefault()
			focusNextOption()
			break
		case 'ArrowUp':
			event.preventDefault()
			focusPreviousOption()
			break
		case 'Enter':
		case ' ':
			event.preventDefault()
			if (focusedIndex.value === -2) {
				toggleSelectAll()
			} else if (focusedIndex.value >= 0) {
				const option = filteredOptions.value[focusedIndex.value]
				if (option && isOption(option)) toggleOption(option, event)
			}
			break
		case 'Tab':
			event.preventDefault()
			if (event.shiftKey) {
				focusPreviousOption()
			} else {
				focusNextOption()
			}
			break
	}
}

function handleSearchKeydown(event: KeyboardEvent) {
	event.stopPropagation()

	if (event.key === 'Escape') {
		event.preventDefault()
		closeDropdown()
	} else if (event.key === 'ArrowDown') {
		event.preventDefault()
		focusOptionFromSearch('next')
	} else if (event.key === 'ArrowUp') {
		event.preventDefault()
		focusOptionFromSearch('previous')
	} else if (event.key === 'Enter' || event.key === ' ') {
		if (event.key === 'Enter') {
			event.preventDefault()
			if (focusedIndex.value === -2) {
				toggleSelectAll()
			} else if (focusedIndex.value >= 0) {
				const option = filteredOptions.value[focusedIndex.value]
				if (option && isOption(option)) toggleOption(option, event)
			}
		}
	} else if (event.key === 'Tab' && isOpen.value) {
		event.preventDefault()
		if (event.shiftKey) {
			focusPreviousOption()
		} else {
			focusNextOption()
		}
	}
}

function focusOptionFromSearch(direction: 'next' | 'previous') {
	const activeElement = document.activeElement
	if (activeElement instanceof HTMLElement) {
		activeElement.blur()
	}

	if (direction === 'previous') {
		focusPreviousOption()
		return
	}

	const nextIndex =
		focusedIndex.value === -1
			? shouldShowSelectAll.value
				? -2
				: getFirstFocusableOptionIndex()
			: focusedIndex.value

	if (nextIndex === -1) {
		return
	}

	focusedIndex.value = nextIndex
	focusOptionIndex(nextIndex)
}

function handleSearchInput() {
	emit('searchInput', searchQuery.value)
	if (!isOpen.value) {
		openDropdown()
	}
	if (optionsContainerRef.value) {
		optionsContainerRef.value.scrollTop = 0
	}
	updateOptionsOverlayScrollbars()
	focusedIndex.value = shouldShowSelectAll.value ? -2 : getFirstFocusableOptionIndex()
}

function handleWindowResize() {
	if (isOpen.value) {
		scheduleDropdownPositionUpdate()
	}
}

function scheduleDropdownPositionUpdate() {
	if (rafId.value !== null) return

	rafId.value = requestAnimationFrame(() => {
		rafId.value = null
		updateDropdownPosition()
	})
}

function handleViewportChange() {
	if (isOpen.value) {
		scheduleDropdownPositionUpdate()
	}
}

function startPositionTracking() {
	window.addEventListener('scroll', handleViewportChange, true)
	window.visualViewport?.addEventListener('scroll', handleViewportChange)
	window.visualViewport?.addEventListener('resize', handleViewportChange)
}

function stopPositionTracking() {
	window.removeEventListener('scroll', handleViewportChange, true)
	window.visualViewport?.removeEventListener('scroll', handleViewportChange)
	window.visualViewport?.removeEventListener('resize', handleViewportChange)

	if (rafId.value !== null) {
		cancelAnimationFrame(rafId.value)
		rafId.value = null
	}
}

onClickOutside(
	dropdownRef,
	() => {
		closeDropdown()
	},
	{ ignore: [triggerRef, containerRef, '.v-popper__popper'] },
)

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
	calculateVisibleTags()
})

onUnmounted(() => {
	window.removeEventListener('resize', handleWindowResize)
	stopPositionTracking()
	destroyOptionsOverlayScrollbars()
})

watch(isOpen, (value) => {
	if (value) {
		updateDropdownPosition()
	}
})

watch(filteredOptions, () => {
	if (isOpen.value) {
		updateDropdownPosition()
		if (hasFilteredOptions.value) {
			initializeOptionsOverlayScrollbars()
			syncSelectionActionsHeight()
		} else {
			destroyOptionsOverlayScrollbars()
			lastSelectionActionsHeight.value = 0
		}
	}
})

watch(shouldShowSelectionActions, async () => {
	const container = optionsContainerRef.value
	const previousHeight = lastSelectionActionsHeight.value
	const previousScrollTop = container?.scrollTop ?? 0

	await nextTick()

	const nextHeight = hasFilteredOptions.value ? getSelectionActionsHeight() : 0
	lastSelectionActionsHeight.value = nextHeight

	if (!isOpen.value || !hasFilteredOptions.value || !container || previousHeight === nextHeight) {
		if (isOpen.value) {
			updateDropdownPosition()
		}
		updateOptionsOverlayScrollbars()
		return
	}

	container.scrollTop = Math.max(0, previousScrollTop + nextHeight - previousHeight)
	updateDropdownPosition()
	updateOptionsOverlayScrollbars()
})

watch(
	() => props.modelValue,
	() => {
		calculateVisibleTags()
		if (isOpen.value) {
			updateDropdownPosition()
			updateOptionsOverlayScrollbars()
		}
	},
	{ deep: true },
)

watch(
	() => props.maxHeight,
	() => {
		if (isOpen.value) {
			updateOptionsOverlayScrollbars()
		}
	},
)
</script>

<style scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}

.multi-select-options-scrollbar :deep(.os-theme-modrinth) {
	--os-size: 10px;
	--os-padding-perpendicular: 2px;
	--os-padding-axis: 2px;
	--os-track-bg: transparent;
	--os-track-bg-hover: transparent;
	--os-track-bg-active: transparent;
	--os-handle-border-radius: 9999px;
	--os-handle-bg: var(--color-scrollbar, var(--surface-5));
	--os-handle-bg-hover: var(--color-scrollbar, var(--surface-5));
	--os-handle-bg-active: var(--color-scrollbar, var(--surface-5));
}
</style>
