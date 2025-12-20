<template>
	<div ref="containerRef" class="relative inline-block w-full">
		<span
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative cursor-pointer flex min-h-5 w-full items-center justify-between overflow-hidden rounded-xl bg-button-bg px-4 py-2.5 text-left transition-all duration-200 text-button-text hover:bg-button-bgHover active:bg-button-bgActive"
			:class="[
				triggerClasses,
				{
					'z-[9999]': isOpen,
					'rounded-b-none': shouldRoundBottomCorners,
					'rounded-t-none': shouldRoundTopCorners,
					'cursor-not-allowed opacity-50': disabled,
				},
			]"
			:aria-expanded="isOpen"
			:aria-haspopup="listbox ? 'listbox' : 'menu'"
			:aria-disabled="disabled || undefined"
			@click="handleTriggerClick"
			@keydown="handleTriggerKeydown"
		>
			<div class="flex items-center gap-2">
				<slot name="prefix"></slot>
				<span class="text-primary font-semibold leading-tight">
					<slot name="selected">{{ triggerText }}</slot>
				</span>
			</div>
			<div class="flex items-center gap-1">
				<slot name="suffix"></slot>
				<ChevronLeftIcon
					v-if="showChevron"
					class="size-5 shrink-0 transition-transform duration-300"
					:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
				/>
			</div>
		</span>

		<Teleport to="#teleports">
			<div
				v-if="isOpen"
				ref="dropdownRef"
				class="fixed z-[9999] flex flex-col overflow-hidden rounded-[14px] bg-surface-4 !border-solid border-0 shadow-2xl"
				:class="[
					shouldRoundBottomCorners
						? 'rounded-t-none !border-t-[1px] !border-t-surface-5'
						: 'rounded-b-none !border-b-[1px] !border-b-surface-5',
				]"
				:style="dropdownStyle"
				:role="listbox ? 'listbox' : 'menu'"
				@mousedown.stop
				@keydown="handleDropdownKeydown"
			>
				<div v-if="searchable" class="p-4">
					<div class="iconified-input w-full border-surface-5 border-[1px] border-solid rounded-xl">
						<SearchIcon aria-hidden="true" />
						<input
							ref="searchInputRef"
							v-model="searchQuery"
							type="text"
							:placeholder="searchPlaceholder"
							class=""
							@keydown.stop="handleSearchKeydown"
							@input="emit('searchInput', searchQuery)"
						/>
					</div>
				</div>

				<div v-if="searchable && filteredOptions.length > 0" class="h-px bg-surface-5"></div>

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
			</div>
		</Teleport>
	</div>
</template>

<script setup lang="ts" generic="T">
import { ChevronLeftIcon, SearchIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import {
	type Component,
	computed,
	nextTick,
	onMounted,
	onUnmounted,
	ref,
	useSlots,
	watch,
} from 'vue'

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
		maxHeight?: number
		displayValue?: string
		extraPosition?: 'top' | 'bottom'
		triggerClass?: string
		forceDirection?: 'up' | 'down'
		noOptionsMessage?: string
		disableSearchFilter?: boolean
	}>(),
	{
		placeholder: 'Select an option',
		disabled: false,
		searchable: false,
		searchPlaceholder: 'Search...',
		listbox: true,
		showChevron: true,
		maxHeight: DEFAULT_MAX_HEIGHT,
		extraPosition: 'bottom',
		noOptionsMessage: 'No results found',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T]
	select: [option: ComboboxOption<T>]
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
const searchInputRef = ref<HTMLInputElement>()
const optionsContainerRef = ref<HTMLElement>()
const optionRefs = ref<(HTMLElement | null)[]>([])
const rafId = ref<number | null>(null)

const dropdownStyle = ref({
	top: '0px',
	left: '0px',
	width: '0px',
})

const openDirection = ref<'down' | 'up'>('down')

const triggerClasses = computed(() => {
	const classes = [props.triggerClass]
	if (isOpen.value) {
		if (props.extraPosition === 'bottom' && slots?.extra) {
			classes.push('!rounded-b-none')
		} else if (props.extraPosition === 'top' && slots?.extra) {
			classes.push('!rounded-t-none')
		}
	}
	return classes
})

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
	if (!searchQuery.value || !props.searchable || props.disableSearchFilter) {
		return optionsWithKeys.value
	}

	const query = searchQuery.value.toLowerCase()
	return optionsWithKeys.value.filter((opt) => {
		if (isDivider(opt)) return false
		return opt.label.toLowerCase().includes(query)
	})
})

const shouldRoundBottomCorners = computed(() => isOpen.value && openDirection.value === 'down')
const shouldRoundTopCorners = computed(() => isOpen.value && openDirection.value === 'up')

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

function focusSearchInput() {
	if (props.searchable && searchInputRef.value) {
		searchInputRef.value.focus()
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
		triggerRect.bottom + dropdownRect.height + DROPDOWN_VIEWPORT_MARGIN <= viewportHeight
	const hasSpaceAbove = triggerRect.top - dropdownRect.height - DROPDOWN_VIEWPORT_MARGIN > 0

	return !hasSpaceBelow && hasSpaceAbove ? 'up' : 'down'
}

function calculateVerticalPosition(
	triggerRect: DOMRect,
	dropdownRect: DOMRect,
	direction: 'up' | 'down',
): number {
	return direction === 'up' ? triggerRect.top - dropdownRect.height : triggerRect.bottom
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
	searchQuery.value = ''

	emit('open')

	await nextTick()
	await updateDropdownPosition()

	setInitialFocus()
	focusSearchInput()
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

function handleTriggerClick() {
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
	optionRefs.value[index]?.focus()
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
	switch (event.key) {
		case 'Enter':
		case ' ':
		case 'ArrowDown':
			event.preventDefault()
			openDropdown()
			break
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
		focusNextOption()
	} else if (event.key === 'ArrowUp') {
		event.preventDefault()
		focusPreviousOption()
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
	{ ignore: [triggerRef] },
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
</script>
