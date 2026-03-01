<template>
	<div ref="containerRef" class="relative inline-block w-full">
		<!-- Searchable mode: input trigger -->
		<StyledInput
			v-if="searchable"
			ref="searchTriggerRef"
			v-model="searchQuery"
			:icon="showSearchIcon ? SearchIcon : undefined"
			type="text"
			:placeholder="searchPlaceholder || placeholder"
			:disabled="disabled"
			wrapper-class="w-full"
			:input-class="showChevron ? '!pr-9' : undefined"
			class="relative"
			@input="handleSearchInput"
			@keydown="handleSearchKeydown"
			@focus="handleSearchFocus"
			@click="handleSearchClick"
		>
			<template v-if="showChevron" #right>
				<ChevronLeftIcon
					class="pointer-events-none absolute right-3 top-1/2 size-5 -translate-y-1/2 text-secondary transition-transform duration-150"
					:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
				/>
			</template>
		</StyledInput>

		<!-- Standard mode: button trigger -->
		<span
			v-else
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative cursor-pointer flex min-h-5 w-full items-center justify-between overflow-hidden rounded-xl bg-surface-4 px-4 py-2.5 text-left transition-all duration-200 text-button-text hover:brightness-125 active:brightness-125"
			:class="[
				props.triggerClass,
				{
					'z-[9999]': isOpen,
					'cursor-not-allowed opacity-50': disabled,
				},
			]"
			:aria-expanded="isOpen"
			:aria-haspopup="listbox ? 'listbox' : 'menu'"
			:aria-disabled="disabled || undefined"
			@click="handleTriggerClick($event)"
			@keydown="handleTriggerKeydown"
		>
			<div class="flex items-center gap-2">
				<slot name="prefix"></slot>
				<component
					:is="selectedOption?.icon"
					v-if="showIconInSelected && selectedOption?.icon"
					class="h-5 w-5"
				/>
				<span class="text-primary font-semibold leading-tight">
					<slot name="selected">{{ triggerText }}</slot>
				</span>
			</div>
			<div class="flex items-center gap-1">
				<slot name="suffix"></slot>
				<ChevronLeftIcon
					v-if="showChevron"
					class="size-5 shrink-0 transition-transform duration-150"
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
					:role="listbox ? 'listbox' : 'menu'"
					@mousedown.stop
					@keydown="handleDropdownKeydown"
				>
					<div
						v-if="filteredOptions.length > 0"
						ref="optionsContainerRef"
						class="flex flex-col gap-2 overflow-y-auto p-3"
						:style="{ maxHeight: `${maxHeight}px` }"
					>
						<template v-for="(item, index) in filteredOptions" :key="item.key">
							<div v-if="item.type === 'divider'" class="h-px bg-surface-5"></div>
							<component
								:is="item.type === 'link' ? 'a' : 'span'"
								v-else
								:ref="(el: HTMLElement) => setOptionRef(el as HTMLElement, index)"
								:href="item.type === 'link' && !item.disabled ? item.href : undefined"
								:target="item.type === 'link' && !item.disabled ? item.target : undefined"
								:role="listbox ? 'option' : 'menuitem'"
								:aria-selected="listbox && item.value === modelValue"
								:aria-disabled="item.disabled || undefined"
								:data-focused="focusedIndex === index"
								class="flex items-center gap-2.5 cursor-pointer rounded-xl p-3 text-left transition-colors duration-150 text-contrast hover:bg-surface-5 focus:bg-surface-5"
								:class="getOptionClasses(item, index)"
								tabindex="-1"
								@click="handleOptionClick(item, index)"
								@mouseenter="!item.disabled && (focusedIndex = index)"
							>
								<slot :name="`option-${item.value}`" :item="item">
									<div class="flex items-center gap-2">
										<component :is="item.icon" v-if="item.icon" class="h-5 w-5" />
										<span
											class="font-semibold leading-tight"
											:class="item.value === modelValue ? 'text-contrast' : 'text-primary'"
										>
											{{ item.label }}
										</span>
									</div>
								</slot>
							</component>
						</template>
					</div>

					<div v-else-if="searchQuery" class="p-4 mb-2 text-center text-sm text-secondary">
						{{ noOptionsMessage }}
					</div>

					<slot name="dropdown-footer"></slot>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>

<script setup lang="ts" generic="T">
import { ChevronLeftIcon, SearchIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import { type Component, computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import StyledInput from './StyledInput.vue'

export interface ComboboxOption<T> {
	value: T
	label: string
	icon?: Component
	disabled?: boolean
	class?: string
	type?: 'button' | 'link' | 'divider'
	href?: string
	target?: string
	action?: () => void
}

const DROPDOWN_VIEWPORT_MARGIN = 8
const DROPDOWN_GAP = 12
const DEFAULT_MAX_HEIGHT = 300

function isDropdownOption<T>(
	opt: ComboboxOption<T> | { type: 'divider' },
): opt is ComboboxOption<T> {
	return 'value' in opt
}

function isDivider<T>(opt: ComboboxOption<T> | { type: 'divider' }): opt is { type: 'divider' } {
	return opt.type === 'divider'
}

const props = withDefaults(
	defineProps<{
		modelValue?: T
		options: (ComboboxOption<T> | { type: 'divider' })[]
		placeholder?: string
		disabled?: boolean
		searchable?: boolean
		searchPlaceholder?: string
		listbox?: boolean
		showChevron?: boolean
		showIconInSelected?: boolean
		maxHeight?: number
		displayValue?: string
		triggerClass?: string
		forceDirection?: 'up' | 'down'
		noOptionsMessage?: string
		disableSearchFilter?: boolean
		/** Keep the selected option's label in the input after selection, and show all options on focus */
		syncWithSelection?: boolean
		/** Show a search icon in the searchable input */
		showSearchIcon?: boolean
	}>(),
	{
		placeholder: 'Select an option',
		disabled: false,
		searchable: false,
		searchPlaceholder: 'Search...',
		listbox: true,
		showChevron: true,
		showIconInSelected: false,
		maxHeight: DEFAULT_MAX_HEIGHT,
		noOptionsMessage: 'No results found',
		syncWithSelection: true,
		showSearchIcon: false,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T]
	select: [option: ComboboxOption<T>]
	open: []
	close: []
	searchInput: [query: string]
}>()

const isOpen = ref(false)
const searchQuery = ref('')
const userHasTyped = ref(false)
const focusedIndex = ref(-1)
const containerRef = ref<HTMLElement>()
const triggerRef = ref<HTMLElement>()
const searchTriggerRef = ref<InstanceType<typeof StyledInput>>()
const dropdownRef = ref<HTMLElement>()
const optionsContainerRef = ref<HTMLElement>()
const optionRefs = ref<(HTMLElement | null)[]>([])
const rafId = ref<number | null>(null)

const effectiveTriggerEl = computed(() => {
	if (props.searchable && searchTriggerRef.value) {
		return (searchTriggerRef.value as unknown as { $el: HTMLElement }).$el as HTMLElement
	}
	return triggerRef.value
})

const dropdownStyle = ref({
	top: '0px',
	left: '0px',
	width: '0px',
})

const openDirection = ref<'down' | 'up'>('down')

const selectedOption = computed<ComboboxOption<T> | undefined>(() => {
	return props.options.find(
		(opt): opt is ComboboxOption<T> => isDropdownOption(opt) && opt.value === props.modelValue,
	)
})

const triggerText = computed(() => {
	if (props.displayValue !== undefined) return props.displayValue
	if (selectedOption.value) return selectedOption.value.label
	return props.placeholder
})

const optionsWithKeys = computed(() => {
	return props.options.map((opt, index) => ({
		...opt,
		key: isDivider(opt) ? `divider-${index}` : `option-${opt.value}`,
	}))
})

const filteredOptions = computed(() => {
	if (!searchQuery.value || !props.searchable || props.disableSearchFilter || !userHasTyped.value) {
		return optionsWithKeys.value
	}

	const query = searchQuery.value.toLowerCase()
	return optionsWithKeys.value.filter((opt) => {
		if (isDivider(opt)) return false
		return opt.label.toLowerCase().includes(query)
	})
})

function getOptionClasses(item: ComboboxOption<T> & { key: string }, index: number) {
	return [
		item.class,
		{
			'bg-surface-5':
				(props.listbox && item.value === props.modelValue) ||
				(focusedIndex.value === index && !(props.listbox && item.value === props.modelValue)),
			'cursor-not-allowed opacity-50 pointer-events-none': item.disabled,
		},
	]
}

function setOptionRef(el: HTMLElement | null, index: number) {
	optionRefs.value[index] = el
}

function setInitialFocus() {
	focusedIndex.value = props.listbox
		? props.options.findIndex((opt) => isDropdownOption(opt) && opt.value === props.modelValue)
		: -1

	if (focusedIndex.value >= 0 && optionRefs.value[focusedIndex.value]) {
		optionRefs.value[focusedIndex.value]?.scrollIntoView({ block: 'center' })
	}
}

function determineOpenDirection(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	viewportHeight: number,
): 'up' | 'down' {
	if (props.forceDirection) {
		return props.forceDirection
	}

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
	if (!effectiveTriggerEl.value || !dropdownRef.value) return

	await nextTick()

	const triggerRect = effectiveTriggerEl.value.getBoundingClientRect()
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

	setInitialFocus()
	startPositionTracking()
}

function closeDropdown() {
	if (!isOpen.value) return

	stopPositionTracking()
	isOpen.value = false
	userHasTyped.value = false
	focusedIndex.value = -1
	emit('close')

	if (!props.searchable) {
		nextTick(() => {
			triggerRef.value?.focus()
		})
	}
}

function handleTriggerClick(event: MouseEvent) {
	// Ignore synthetic clicks generated by keyboard (Enter/Space on role="button")
	// since handleTriggerKeydown already handles keyboard interaction
	if (event.detail === 0) return

	if (isOpen.value) {
		closeDropdown()
	} else {
		openDropdown()
	}
}

function handleOptionClick(option: ComboboxOption<T>, index: number) {
	if (option.disabled || option.type === 'divider') return

	focusedIndex.value = index

	if (option.action) {
		option.action()
	}

	if (props.listbox && option.value !== undefined) {
		emit('update:modelValue', option.value)
	}

	emit('select', option)

	if (option.type !== 'link') {
		if (props.searchable) {
			searchQuery.value = props.syncWithSelection ? option.label : ''
		}
		closeDropdown()
	}
}

function findNextFocusableOption(currentIndex: number, direction: 'next' | 'previous'): number {
	const length = filteredOptions.value.length
	let index = currentIndex
	let option

	do {
		index = direction === 'next' ? (index + 1) % length : (index - 1 + length) % length
		option = filteredOptions.value[index]
	} while (isDivider(option) || option.disabled)

	return index
}

function focusOption(index: number) {
	if (index < 0 || index >= filteredOptions.value.length) return

	const option = filteredOptions.value[index]
	if (isDivider(option) || option.disabled) return

	focusedIndex.value = index
	optionRefs.value[index]?.scrollIntoView({ block: 'nearest' })
}

function focusNextOption() {
	const nextIndex = findNextFocusableOption(focusedIndex.value, 'next')
	focusOption(nextIndex)
}

function focusPreviousOption() {
	const prevIndex = findNextFocusableOption(focusedIndex.value, 'previous')
	focusOption(prevIndex)
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
			if (focusedIndex.value >= 0) {
				const option = filteredOptions.value[focusedIndex.value]
				if (!isDivider(option)) {
					handleOptionClick(option, focusedIndex.value)
				}
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
		if (!isOpen.value) {
			openDropdown()
		}
		focusNextOption()
	} else if (event.key === 'ArrowUp') {
		event.preventDefault()
		if (!isOpen.value) {
			openDropdown()
		}
		focusPreviousOption()
	} else if (event.key === 'Enter') {
		event.preventDefault()
		if (focusedIndex.value >= 0) {
			const option = filteredOptions.value[focusedIndex.value]
			if (option && !isDivider(option)) {
				handleOptionClick(option, focusedIndex.value)
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
	userHasTyped.value = true
	emit('searchInput', searchQuery.value)
	if (!isOpen.value) {
		openDropdown()
	}
}

function handleSearchFocus() {
	if (!isOpen.value) {
		openDropdown()
	}
}

function handleSearchClick() {
	if (!isOpen.value) {
		openDropdown()
	}
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
	{ ignore: [triggerRef, containerRef] },
)

onMounted(() => {
	window.addEventListener('resize', handleWindowResize)
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
	[() => props.modelValue, () => props.options],
	([val]) => {
		if (props.searchable && props.syncWithSelection && !isOpen.value) {
			const opt = props.options.find((o) => isDropdownOption(o) && o.value === val)
			searchQuery.value = opt && isDropdownOption(opt) ? opt.label : ''
		}
	},
	{ immediate: true },
)
</script>
