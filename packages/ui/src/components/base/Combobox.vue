<template>
	<div ref="containerRef" class="relative inline-block w-full">
		<!-- Searchable mode: input trigger -->
		<div v-if="searchable" class="relative w-full rounded-xl bg-surface-4">
			<!--
				Selection mirror: horizontal padding must match StyledInput (filled + left icon uses `pl-10`,
				else `pl-3`) and `searchableInputClass` when the chevron is shown (`!pr-9`), or the overlay
				text will not line up with the transparent input text / caret.
			-->
			<div
				v-if="searchSelectionOverlayVisible"
				class="pointer-events-none absolute inset-y-0 left-0 right-0 z-0 flex min-w-0 items-center gap-2 font-medium text-primary"
				:class="[showSearchIcon ? 'pl-10' : 'pl-3', showChevron ? 'pr-9' : 'pr-3']"
				aria-hidden="true"
			>
				<span class="min-w-0 truncate">{{ searchQuery }}</span>
				<slot name="search-selection-affix" :option="selectedOption" />
			</div>
			<StyledInput
				ref="searchTriggerRef"
				v-model="searchQuery"
				:icon="showSearchIcon ? SearchIcon : undefined"
				type="text"
				:placeholder="searchPlaceholder || placeholder"
				:disabled="disabled"
				wrapper-class="w-full !bg-transparent"
				:input-class="searchableInputClass"
				class="relative z-[1]"
				@input="handleSearchInput"
				@keydown="handleSearchKeydown"
				@focusin="handleSearchFocus"
				@focusout="handleSearchFocusout"
				@click="handleSearchClick"
			>
				<template v-if="showChevron" #right>
					<ChevronLeftIcon
						class="pointer-events-none absolute right-3 top-1/2 size-5 -translate-y-1/2 text-secondary transition-transform duration-150"
						:class="isOpen ? (openDirection === 'down' ? 'rotate-90' : '-rotate-90') : '-rotate-90'"
					/>
				</template>
			</StyledInput>
		</div>

		<!-- Standard mode: button trigger -->
		<span
			v-else
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative flex min-h-5 w-full items-center justify-between overflow-hidden rounded-xl bg-surface-4 px-4 py-2 text-left transition-all duration-200 text-button-text gap-2.5"
			:class="[
				props.triggerClass,
				{
					'z-[9999]': isOpen,
					'cursor-not-allowed opacity-50': disabled,
					'cursor-pointer hover:brightness-[115%] active:brightness-[115%]': !disabled,
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
					v-if="shouldRenderDropdown"
					ref="dropdownRef"
					class="fixed z-[9999] flex flex-col overflow-hidden rounded-[14px] bg-surface-4 border border-solid border-surface-5"
					:class="[
						props.dropdownClass,
						openDirection === 'up' ? 'shadow-[0_-25px_50px_-12px_rgb(0,0,0,0.25)]' : 'shadow-2xl',
					]"
					:style="dropdownStyle"
					:role="listbox ? 'listbox' : 'menu'"
					@mousedown.stop
					@keydown="handleDropdownKeydown"
				>
					<div
						v-if="filteredOptions.length > 0"
						ref="optionsScrollbarRef"
						class="combobox-options-scrollbar bg-surface-4"
						data-overlayscrollbars-initialize
					>
						<div
							ref="optionsContainerRef"
							class="overflow-y-auto"
							:style="{ maxHeight: `${maxHeight}px` }"
							data-overlayscrollbars-viewport
						>
							<div ref="optionsListRef" class="flex flex-col">
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
										class="group/option flex items-center gap-2.5 cursor-pointer px-4 py-3 text-left transition-all duration-150"
										:class="getOptionClasses(item, index)"
										tabindex="-1"
										@mousedown.prevent
										@click="handleOptionClick(item, index)"
										@mouseenter="handleOptionMouseEnter(item, index)"
									>
										<slot
											name="option"
											:item="item"
											:index="index"
											:is-selected="!!(listbox && item.value === modelValue)"
										>
											<div class="flex w-full items-center justify-between gap-2">
												<div class="flex items-center gap-2">
													<component
														:is="item.icon"
														v-if="item.icon"
														class="h-5 w-5"
														:class="item.value === modelValue ? 'text-green' : 'text-primary'"
													/>
													<div class="flex flex-col gap-1.5">
														<span
															class="font-semibold leading-tight"
															:class="item.value === modelValue ? 'text-green' : 'text-primary'"
														>
															{{ item.label }}
														</span>
														<span
															v-if="item.subLabel"
															class="text-sm"
															:class="item.value === modelValue ? 'text-green' : 'text-secondary'"
														>
															{{ item.subLabel }}
														</span>
													</div>
												</div>
												<slot name="option-suffix" :item="item"></slot>
											</div>
										</slot>
									</component>
								</template>
							</div>
						</div>
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
import 'overlayscrollbars/overlayscrollbars.css'

import { ChevronLeftIcon, SearchIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import { OverlayScrollbars, type PartialOptions } from 'overlayscrollbars'
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

import StyledInput from './StyledInput.vue'

export interface ComboboxOption<T> {
	value: T
	label: string
	subLabel?: string
	icon?: Component
	disabled?: boolean
	class?: string
	type?: 'button' | 'link' | 'divider'
	href?: string
	target?: string
	action?: () => void
	searchTerms?: string[]
}

type OverlayScrollbarsInstance = NonNullable<ReturnType<typeof OverlayScrollbars>>

const DROPDOWN_VIEWPORT_MARGIN = 8
const DROPDOWN_GAP = 8
const DEFAULT_MAX_HEIGHT = 300
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
		searchValue?: string
		triggerClass?: string
		dropdownClass?: string
		/** Width for the teleported dropdown; defaults to the trigger/input width */
		dropdownWidth?: string | number
		/** Minimum width for the teleported dropdown */
		dropdownMinWidth?: string | number
		forceDirection?: 'up' | 'down'
		noOptionsMessage?: string
		disableSearchFilter?: boolean
		/** Keep the selected option's label in the input after selection, and show all options on focus */
		syncWithSelection?: boolean
		/** Select the searchable input text when the field receives focus */
		selectSearchTextOnFocus?: boolean
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
		selectSearchTextOnFocus: false,
		showSearchIcon: false,
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T]
	select: [option: ComboboxOption<T>]
	'option-hover': [option: ComboboxOption<T>]
	open: []
	close: []
	searchInput: [query: string]
	searchBlur: [query: string]
}>()

const slots = useSlots()

const isOpen = ref(false)
const searchQuery = ref('')
const userHasTyped = ref(false)
const focusedIndex = ref(-1)
const containerRef = ref<HTMLElement>()
const triggerRef = ref<HTMLElement>()
const searchTriggerRef = ref<InstanceType<typeof StyledInput>>()
const dropdownRef = ref<HTMLElement>()
const optionsScrollbarRef = ref<HTMLElement>()
const optionsContainerRef = ref<HTMLElement>()
const optionsListRef = ref<HTMLElement>()
const optionRefs = ref<(HTMLElement | null)[]>([])
const rafId = ref<number | null>(null)
const optionsOverlayScrollbars = ref<OverlayScrollbarsInstance | null>(null)

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
	minWidth: '0px',
})

const openDirection = ref<'down' | 'up'>('down')

const selectedOption = computed<ComboboxOption<T> | undefined>(() => {
	return props.options.find(
		(opt): opt is ComboboxOption<T> => isDropdownOption(opt) && opt.value === props.modelValue,
	)
})

/** Extra content (e.g. channel pill) next to the label while the search field is idle */
const searchSelectionOverlayVisible = computed(() => {
	if (!props.searchable || !props.syncWithSelection || !selectedOption.value) return false
	if (!slots['search-selection-affix']) return false
	if (isOpen.value || userHasTyped.value) return false
	return true
})

const searchableInputClass = computed(() => {
	const parts = ['!bg-transparent']
	if (props.showChevron) parts.push('!pr-9')
	if (searchSelectionOverlayVisible.value) {
		parts.push('!text-transparent [caret-color:var(--color-text-primary)] selection:bg-transparent')
	}
	return parts.join(' ')
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
		if (opt.label.toLowerCase().includes(query)) return true
		if (opt.searchTerms?.some((term) => term.toLowerCase().includes(query))) return true
		return false
	})
})

const hasDropdownContent = computed(() => {
	return filteredOptions.value.length > 0 || !!searchQuery.value || !!slots['dropdown-footer']
})

const shouldRenderDropdown = computed(() => {
	return isOpen.value && hasDropdownContent.value
})

function getOptionClasses(item: ComboboxOption<T> & { key: string }, _index: number) {
	const isSelected = props.listbox && item.value === props.modelValue

	return [
		item.class,
		{
			'bg-surface-4 text-contrast hover:brightness-[115%] focus:brightness-[115%]': !isSelected,
			'bg-highlight-green text-green !cursor-default hover:bg-highlight-green focus:bg-highlight-green':
				isSelected,
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
	if (!effectiveTriggerEl.value || !dropdownRef.value) return

	await nextTick()

	const triggerRect = effectiveTriggerEl.value.getBoundingClientRect()
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

async function initializeOptionsOverlayScrollbars() {
	await nextTick()

	if (!isOpen.value || filteredOptions.value.length === 0) {
		destroyOptionsOverlayScrollbars()
		return
	}

	if (!optionsScrollbarRef.value || !optionsContainerRef.value || !optionsListRef.value) {
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
				content: optionsListRef.value,
			},
		},
		OPTIONS_OVERLAY_SCROLLBARS_OPTIONS,
	)
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

async function openDropdown() {
	if (props.disabled || isOpen.value || !hasDropdownContent.value) return

	isOpen.value = true
	emit('open')

	await nextTick()
	await updateDropdownPosition()
	await initializeOptionsOverlayScrollbars()

	setInitialFocus()
	startPositionTracking()
}

function closeDropdown() {
	if (!isOpen.value) return

	stopPositionTracking()
	destroyOptionsOverlayScrollbars()
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
	const isSelected = props.listbox && option.value === props.modelValue
	if (isSelected) return

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

function handleOptionMouseEnter(option: ComboboxOption<T>, index: number) {
	if (option.disabled) return
	focusedIndex.value = index
	emit('option-hover', option)
}

function findNextFocusableOption(currentIndex: number, direction: 'next' | 'previous'): number {
	const length = filteredOptions.value.length
	if (length === 0) return -1

	let index = currentIndex

	for (let i = 0; i < length; i++) {
		index = direction === 'next' ? (index + 1) % length : (index - 1 + length) % length
		const option = filteredOptions.value[index]

		if (!isDivider(option) && !option.disabled) {
			return index
		}
	}

	return -1
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

function handleSearchFocus(event: FocusEvent) {
	const target = event.target
	if (props.selectSearchTextOnFocus && target instanceof HTMLInputElement) {
		window.setTimeout(() => {
			if (document.activeElement === target) {
				target.select()
			}
		})
	}

	if (!isOpen.value) {
		openDropdown()
	}
}

function handleSearchFocusout(event: FocusEvent) {
	const nextTarget = event.relatedTarget
	if (nextTarget instanceof Node && containerRef.value?.contains(nextTarget)) return
	if (nextTarget instanceof Node && dropdownRef.value?.contains(nextTarget)) return

	emit('searchBlur', searchQuery.value)
	if (props.searchValue !== undefined) {
		searchQuery.value = props.searchValue
	}
	closeDropdown()
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
	destroyOptionsOverlayScrollbars()
})

watch(isOpen, (value) => {
	if (value) {
		updateDropdownPosition()
	}
})

watch(shouldRenderDropdown, (value) => {
	if (value) {
		updateDropdownPosition()
		initializeOptionsOverlayScrollbars()
	}
})

watch(filteredOptions, () => {
	if (isOpen.value) {
		updateDropdownPosition()
		if (filteredOptions.value.length > 0) {
			initializeOptionsOverlayScrollbars()
		} else {
			destroyOptionsOverlayScrollbars()
		}
	}
})

watch(hasDropdownContent, (value) => {
	if (!value && isOpen.value) {
		closeDropdown()
	}
})

watch(
	[() => props.modelValue, () => props.options],
	([val]) => {
		if (props.searchable && props.syncWithSelection && !isOpen.value) {
			const opt = props.options.find((o) => isDropdownOption(o) && o.value === val)
			searchQuery.value = opt && isDropdownOption(opt) ? opt.label : ''
		}
		if (isOpen.value) {
			updateOptionsOverlayScrollbars()
		}
	},
	{ immediate: true },
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
.combobox-options-scrollbar :deep(.os-theme-modrinth) {
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
