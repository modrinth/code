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
						class="inline-flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium text-primary transition-colors hover:brightness-110"
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
									class="inline-flex cursor-pointer items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-4 px-2.5 py-1 text-sm font-medium text-primary hover:brightness-110"
									@click.stop="removeTag(tag.value)"
								>
									{{ tag.label }}
									<XIcon class="size-3.5 shrink-0 text-secondary" />
								</span>
							</div>
						</template>
					</Menu>
					<span v-if="selectedOptions.length === 0" class="px-1.5 py-1 text-secondary">
						{{ placeholder }}
					</span>
				</div>
				<div class="ml-2 flex shrink-0 items-center gap-1.5">
					<button
						v-if="clearable && modelValue.length > 0"
						type="button"
						class="flex cursor-pointer items-center justify-center rounded border-none bg-transparent p-0.5 text-secondary transition-colors hover:text-contrast"
						aria-label="Clear all"
						@click.stop="clearAll"
					>
						<XIcon class="size-5" />
					</button>
					<div class="h-5 w-[1px] shrink-0 bg-surface-5"></div>
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
					<div class="empty:hidden pt-1.5">
						<div
							v-if="searchable"
							class="px-3 py-1.5 pb-2.5 border-0 border-solid border-b border-b-surface-5"
						>
							<StyledInput
								ref="searchInputRef"
								v-model="searchQuery"
								:icon="SearchIcon"
								type="text"
								:placeholder="searchPlaceholder"
								wrapper-class="w-full bg-surface-4"
								@input="handleSearchInput"
								@keydown="handleSearchKeydown"
							/>
						</div>

						<div
							v-if="hasFilteredOptions || shouldShowSelectAll"
							class="flex flex-col gap-2 bg-surface-4 border-0 border-solid border-b border-b-surface-5 py-1.5 empty:hidden"
						>
							<div v-if="shouldShowSelectAll" class="sticky top-0 z-10 bg-surface-4 px-3">
								<span
									class="flex items-center gap-2.5 cursor-pointer p-3 text-left transition-colors duration-150 text-contrast hover:bg-surface-5 focus:bg-surface-5 rounded-xl"
									:class="{ 'bg-surface-5': focusedIndex === -2 }"
									:data-focused="focusedIndex === -2"
									role="option"
									:aria-selected="isAllSelected"
									tabindex="-1"
									@click="toggleSelectAll"
									@mouseenter="focusedIndex = -2"
								>
									<span
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
									<span class="font-semibold leading-tight text-primary">
										{{ selectAllLabel }}
									</span>
								</span>
							</div>
						</div>

						<div v-if="$slots.top" class="border-0 border-b border-solid border-b-surface-5 py-1.5">
							<slot
								name="top"
								:model-value="modelValue"
								:selected-options="selectedOptions"
								:clear-all="clearAll"
								:is-open="isOpen"
							></slot>
						</div>

						<div
							v-if="shouldShowSelectionActions"
							class="flex items-center justify-between gap-3 border-0 border-b border-solid border-b-surface-5 px-6 py-2.5 text-sm"
						>
							<span class="font-semibold text-secondary">{{ selectionActionsLabel }}</span>
							<button
								type="button"
								class="border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
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
						ref="optionsContainerRef"
						class="flex flex-col gap-2 overflow-y-auto px-3 py-1.5 select-none"
						:style="{ maxHeight: `${maxHeight}px` }"
					>
						<template v-for="(item, index) in filteredOptions" :key="getItemKey(item, index)">
							<div
								v-if="isSectionHeader(item)"
								class="flex items-center justify-between gap-3 px-3 pr-0 pb-1 pt-2 text-sm font-bold text-secondary"
								:class="item.class"
								role="presentation"
							>
								<span class="min-w-0 truncate">{{ item.label }}</span>
								<button
									v-if="hasSelectableSectionHeaderOptions(item)"
									type="button"
									class="shrink-0 border-0 bg-transparent p-0 text-sm font-semibold text-secondary shadow-none transition-colors hover:bg-transparent hover:text-contrast"
									@click.stop="toggleSectionHeaderOptions(item)"
									@keydown.enter.stop
									@keydown.space.stop
								>
									{{ areSectionHeaderOptionsSelected(item) ? 'Clear' : 'Select all' }}
								</button>
							</div>
							<span
								v-else
								:ref="(el: any) => setOptionRef(el as HTMLElement, index)"
								role="option"
								:aria-selected="isSelected(item.value)"
								:aria-disabled="item.disabled || undefined"
								:data-focused="focusedIndex === index"
								class="flex items-center gap-2.5 cursor-pointer p-3 text-left transition-colors duration-150 text-contrast hover:bg-surface-5 rounded-xl"
								:class="[
									item.class,
									{
										'bg-surface-5': focusedIndex === index,
										'cursor-not-allowed opacity-50 pointer-events-none': item.disabled,
									},
								]"
								tabindex="-1"
								@click="toggleOption(item, $event)"
								@mouseenter="!item.disabled && (focusedIndex = index)"
							>
								<span
									class="w-5 h-5 rounded-md flex items-center justify-center border-[1px] border-solid shrink-0 checkbox-shadow"
									:class="
										isSelected(item.value)
											? 'bg-brand border-button-border text-brand-inverted'
											: 'bg-surface-2 border-surface-5'
									"
								>
									<CheckIcon v-if="isSelected(item.value)" aria-hidden="true" stroke-width="3" />
								</span>
								<slot :name="`option-${item.value}`" :item="item">
									<div class="flex items-center gap-2">
										<component :is="item.icon" v-if="item.icon" class="h-5 w-5" />
										<span
											class="font-semibold leading-tight"
											:class="isSelected(item.value) ? 'text-contrast' : 'text-primary'"
										>
											{{ item.label }}
										</span>
									</div>
								</slot>
							</span>
						</template>
					</div>
					<div
						v-else-if="isNoOptionsState && noOptionsMessage"
						class="p-4 mb-2 text-center text-sm text-secondary"
					>
						{{ noOptionsMessage }}
					</div>
					<div v-else-if="searchQuery" class="p-4 mb-2 text-center text-sm text-secondary">
						{{ noResultsMessage }}
					</div>

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
import { CheckIcon, ChevronLeftIcon, MinusIcon, SearchIcon, XIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import { Menu } from 'floating-vue'
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

const DROPDOWN_VIEWPORT_MARGIN = 8
const DROPDOWN_GAP = 12
const DEFAULT_MAX_HEIGHT = 300

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
const optionsContainerRef = ref<HTMLElement>()
const searchInputRef = ref<InstanceType<typeof StyledInput>>()
const optionRefs = ref<(HTMLElement | null)[]>([])
const rafId = ref<number | null>(null)
const tagsContainerRef = ref<HTMLElement>()

const dropdownStyle = ref({
	top: '0px',
	left: '0px',
	width: '0px',
	minWidth: '0px',
})

const openDirection = ref<'down' | 'up'>('down')
const hasCustomInputContent = computed(() => Boolean(slots['input-content']))

const selectableOptions = computed(() => props.options.filter(isOption))

const selectedOptions = computed(() => {
	return selectableOptions.value.filter((opt) => props.modelValue.includes(opt.value))
})

const isAllSelected = computed(() => {
	const selectableOptions = props.options.filter(isOption).filter((opt) => !opt.disabled)
	return (
		selectableOptions.length > 0 &&
		selectableOptions.every((opt) => props.modelValue.includes(opt.value))
	)
})

const isIndeterminate = computed(() => {
	return (
		!isAllSelected.value &&
		selectableOptions.value.some((opt) => !opt.disabled && props.modelValue.includes(opt.value))
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

const hasFilteredOptions = computed(() => filteredOptions.value.some(isOption))
const isNoOptionsState = computed(() => selectableOptions.value.length === 0 && !searchQuery.value)
const shouldShowSelectAll = computed(
	() => props.includeSelectAllOption && selectableOptions.value.length > 0,
)
const selectedOptionCount = computed(() => selectedOptions.value.length)
const shouldShowSelectionActions = computed(
	() => props.showSelectionActions && selectedOptionCount.value > 0,
)
const selectionActionsLabel = computed(() => {
	return selectedOptionCount.value === 1 ? '1 selected' : `${selectedOptionCount.value} selected`
})

function isSelected(value: T) {
	return props.modelValue.includes(value)
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
	return getSectionHeaderOptions(sectionHeader).length > 0
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
		const allValues = selectableOptions.value.filter((opt) => !opt.disabled).map((opt) => opt.value)
		emit('update:modelValue', allValues)
	}
}

function setOptionRef(el: HTMLElement | null, index: number) {
	optionRefs.value[index] = el
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
	viewportHeight: number,
): 'up' | 'down' {
	if (props.forceDirection) return props.forceDirection

	const hasSpaceBelow =
		triggerRect.bottom + dropdownRect.height + DROPDOWN_GAP + DROPDOWN_VIEWPORT_MARGIN <=
		viewportHeight
	const hasSpaceAbove =
		triggerRect.top - dropdownRect.height - DROPDOWN_GAP - DROPDOWN_VIEWPORT_MARGIN > 0

	return !hasSpaceBelow && hasSpaceAbove ? 'up' : 'down'
}

function calculateVerticalPosition(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	direction: 'up' | 'down',
): number {
	return direction === 'up'
		? triggerRect.top - dropdownRect.height - DROPDOWN_GAP
		: triggerRect.bottom + DROPDOWN_GAP
}

function calculateHorizontalPosition(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	viewportWidth: number,
): number {
	let left = triggerRect.left
	if (left + dropdownRect.width > viewportWidth - DROPDOWN_VIEWPORT_MARGIN) {
		left = Math.max(
			DROPDOWN_VIEWPORT_MARGIN,
			viewportWidth - dropdownRect.width - DROPDOWN_VIEWPORT_MARGIN,
		)
	}
	return left
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
	const viewportHeight = window.innerHeight
	const viewportWidth = window.innerWidth

	const direction = determineOpenDirection(triggerRect, dropdownRect, viewportHeight)
	const top = calculateVerticalPosition(triggerRect, dropdownRect, direction)
	const left = calculateHorizontalPosition(triggerRect, dropdownRect, viewportWidth)

	dropdownStyle.value = {
		top: `${top}px`,
		left: `${left}px`,
		width,
		minWidth,
	}

	openDirection.value = direction
}

async function openDropdown() {
	if (props.disabled || isOpen.value) return

	isOpen.value = true
	emit('open')

	await nextTick()
	await updateDropdownPosition()

	if (props.searchable && searchInputRef.value) {
		;(searchInputRef.value as unknown as { focus: () => void }).focus()
	}

	focusedIndex.value = shouldShowSelectAll.value ? -2 : getFirstFocusableOptionIndex()
	startPositionTracking()
}

function closeDropdown() {
	if (!isOpen.value) return

	stopPositionTracking()
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
	optionRefs.value[focusedIndex.value]?.scrollIntoView({ block: 'nearest' })
}

function focusPreviousOption() {
	const length = filteredOptions.value.length
	if (length === 0) return

	if (focusedIndex.value === getFirstFocusableOptionIndex() && shouldShowSelectAll.value) {
		focusedIndex.value = -2
		return
	}

	const previousIndex = findNextFocusableOption(
		focusedIndex.value === -1 ? 0 : focusedIndex.value,
		'previous',
	)
	if (previousIndex === -1) return

	focusedIndex.value = previousIndex
	optionRefs.value[focusedIndex.value]?.scrollIntoView({ block: 'nearest' })
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
	if (event.key === 'Escape') {
		event.preventDefault()
		closeDropdown()
	} else if (event.key === 'ArrowDown') {
		event.preventDefault()
		focusNextOption()
	} else if (event.key === 'ArrowUp') {
		event.preventDefault()
		focusPreviousOption()
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

function handleSearchInput() {
	emit('searchInput', searchQuery.value)
	if (!isOpen.value) {
		openDropdown()
	}
	focusedIndex.value = shouldShowSelectAll.value ? -2 : getFirstFocusableOptionIndex()
}

function handleWindowResize() {
	if (isOpen.value) {
		updateDropdownPosition()
	}
}

function startPositionTracking() {
	function track() {
		updateDropdownPosition()
		rafId.value = requestAnimationFrame(track)
	}
	rafId.value = requestAnimationFrame(track)
}

function stopPositionTracking() {
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
})

watch(isOpen, (value) => {
	if (value) {
		updateDropdownPosition()
	}
})

watch(filteredOptions, () => {
	if (isOpen.value) {
		updateDropdownPosition()
	}
})

watch(
	() => props.modelValue,
	() => {
		calculateVisibleTags()
		if (isOpen.value) {
			updateDropdownPosition()
		}
	},
	{ deep: true },
)
</script>

<style scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}
</style>
