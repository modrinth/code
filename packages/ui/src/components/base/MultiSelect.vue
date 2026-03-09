<template>
	<div ref="containerRef" class="relative inline-block w-full">
		<span
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative flex w-full items-center overflow-hidden rounded-xl bg-surface-4 px-3 py-2 text-left transition-all duration-200"
			:class="[
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
			<div
				ref="tagsContainerRef"
				class="flex flex-1 items-center gap-1.5 overflow-hidden flex-wrap"
				:style="{ maxHeight: `calc(${maxTagRows} * 30px + ${maxTagRows - 1} * 6px)` }"
			>
				<span
					v-for="tag in visibleTags"
					:key="String(tag.value)"
					class="inline-flex items-center gap-1 rounded-full bg-surface-4 px-2.5 py-1 text-sm font-medium text-primary transition-colors border-solid border border-surface-5 hover:brightness-110"
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
						class="inline-flex items-center rounded-full bg-surface-4 px-2 py-1 text-sm font-medium text-secondary border-solid border border-surface-5 select-none cursor-default"
						@click.stop
					>
						+{{ overflowCount }}
					</span>
					<template #popper>
						<div class="flex gap-1 flex-wrap max-w-[20rem]" @mousedown.prevent>
							<span
								v-for="tag in overflowTags"
								:key="String(tag.value)"
								class="inline-flex items-center gap-1 rounded-full bg-surface-4 px-2.5 py-1 text-sm font-medium text-primary border-solid border border-surface-5 cursor-pointer hover:brightness-110"
								@click.stop="removeTag(tag.value)"
							>
								{{ tag.label }}
								<XIcon class="size-3.5 shrink-0 text-secondary" />
							</span>
						</div>
					</template>
				</Menu>
				<span v-if="selectedOptions.length === 0" class="py-1 px-1.5 text-secondary">
					{{ placeholder }}
				</span>
			</div>
			<div class="ml-2 flex shrink-0 items-center gap-1.5">
				<button
					v-if="clearable && modelValue.length > 0"
					type="button"
					class="flex items-center justify-center rounded p-0.5 bg-transparent border-none text-secondary hover:text-contrast transition-colors cursor-pointer"
					aria-label="Clear all"
					@click.stop="clearAll"
				>
					<XIcon class="size-5" />
				</button>
				<div class="w-[1px] h-5 bg-surface-5 shrink-0"></div>
				<ChevronLeftIcon
					v-if="showChevron"
					class="size-5 shrink-0 text-secondary transition-transform duration-150"
					:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
				/>
			</div>
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
							v-if="filteredOptions.length > 0 || shouldShowSelectAll"
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
					</div>

					<div
						v-if="filteredOptions.length > 0"
						ref="optionsContainerRef"
						class="flex flex-col gap-2 overflow-y-auto px-3 pt-1.5"
						:style="{ maxHeight: `${maxHeight}px` }"
					>
						<template v-for="(item, index) in filteredOptions" :key="String(item.value)">
							<span
								:ref="(el: any) => setOptionRef(el as HTMLElement, index)"
								role="option"
								:aria-selected="isSelected(item.value)"
								:aria-disabled="item.disabled || undefined"
								:data-focused="focusedIndex === index"
								class="flex items-center gap-2.5 cursor-pointer p-3 text-left transition-colors duration-150 text-contrast hover:bg-surface-5 focus:bg-surface-5 rounded-xl"
								:class="[
									item.class,
									{
										'bg-surface-5': focusedIndex === index,
										'cursor-not-allowed opacity-50 pointer-events-none': item.disabled,
									},
								]"
								tabindex="-1"
								@click="toggleOption(item)"
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

const DROPDOWN_VIEWPORT_MARGIN = 8
const DROPDOWN_GAP = 12
const DEFAULT_MAX_HEIGHT = 300

const props = withDefaults(
	defineProps<{
		modelValue: T[]
		options: MultiSelectOption<T>[]
		placeholder?: string
		disabled?: boolean
		searchable?: boolean
		searchPlaceholder?: string
		showChevron?: boolean
		clearable?: boolean
		maxHeight?: number
		triggerClass?: string
		forceDirection?: 'up' | 'down'
		noOptionsMessage?: string
		noResultsMessage?: string
		disableSearchFilter?: boolean
		includeSelectAllOption?: boolean
		selectAllLabel?: string
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
		noOptionsMessage: 'No options available',
		noResultsMessage: 'No results found',
		includeSelectAllOption: false,
		selectAllLabel: 'Select all',
		maxTagRows: 1,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T[]]
	open: []
	close: []
	searchInput: [query: string]
}>()

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
})

const openDirection = ref<'down' | 'up'>('down')

const selectedOptions = computed(() => {
	return props.options.filter((opt) => props.modelValue.includes(opt.value))
})

const isAllSelected = computed(() => {
	const selectableOptions = props.options.filter((opt) => !opt.disabled)
	return (
		selectableOptions.length > 0 &&
		selectableOptions.every((opt) => props.modelValue.includes(opt.value))
	)
})

const isIndeterminate = computed(() => {
	return !isAllSelected.value && props.modelValue.length > 0
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

const filteredOptions = computed(() => {
	if (!searchQuery.value || !props.searchable || props.disableSearchFilter) {
		return props.options
	}

	const query = searchQuery.value.toLowerCase()
	return props.options.filter((opt) => {
		if (opt.label.toLowerCase().includes(query)) return true
		if (opt.searchTerms?.some((term) => term.toLowerCase().includes(query))) return true
		return false
	})
})

const isNoOptionsState = computed(() => props.options.length === 0 && !searchQuery.value)
const shouldShowSelectAll = computed(() => props.includeSelectAllOption && props.options.length > 0)

function isSelected(value: T) {
	return props.modelValue.includes(value)
}

function toggleOption(option: MultiSelectOption<T>) {
	if (option.disabled) return

	const newValue = isSelected(option.value)
		? props.modelValue.filter((v) => v !== option.value)
		: [...props.modelValue, option.value]

	emit('update:modelValue', newValue)
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

function toggleSelectAll() {
	if (isAllSelected.value) {
		emit('update:modelValue', [])
	} else {
		const allValues = props.options.filter((opt) => !opt.disabled).map((opt) => opt.value)
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

async function updateDropdownPosition() {
	if (!triggerRef.value || !dropdownRef.value) return

	await nextTick()

	const triggerRect = triggerRef.value.getBoundingClientRect()
	const dropdownRect = dropdownRef.value.getBoundingClientRect()
	const viewportHeight = window.innerHeight
	const viewportWidth = window.innerWidth

	const direction = determineOpenDirection(triggerRect, dropdownRect, viewportHeight)
	const top = calculateVerticalPosition(triggerRect, dropdownRect, direction)
	const left = calculateHorizontalPosition(triggerRect, dropdownRect, viewportWidth)

	dropdownStyle.value = {
		top: `${top}px`,
		left: `${left}px`,
		width: `${triggerRect.width}px`,
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

	focusedIndex.value = shouldShowSelectAll.value ? -2 : filteredOptions.value.length > 0 ? 0 : -1
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

function focusNextOption() {
	const length = filteredOptions.value.length
	if (length === 0) return

	if (focusedIndex.value === -2) {
		focusedIndex.value = 0
	} else {
		focusedIndex.value = (focusedIndex.value + 1) % length
	}

	optionRefs.value[focusedIndex.value]?.scrollIntoView({ block: 'nearest' })
}

function focusPreviousOption() {
	const length = filteredOptions.value.length
	if (length === 0) return

	if (focusedIndex.value <= 0 && shouldShowSelectAll.value) {
		focusedIndex.value = -2
		return
	}

	if (focusedIndex.value <= 0) {
		focusedIndex.value = length - 1
	} else {
		focusedIndex.value = focusedIndex.value - 1
	}

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
				if (option) toggleOption(option)
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
				if (option) toggleOption(option)
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
	focusedIndex.value = shouldShowSelectAll.value ? -2 : filteredOptions.value.length > 0 ? 0 : -1
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
	},
	{ deep: true },
)
</script>

<style scoped>
.checkbox-shadow {
	box-shadow: 1px 1px 2px 0 rgba(0, 0, 0, 0.08);
}
</style>
