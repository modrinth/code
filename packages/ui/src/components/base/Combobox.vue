<template>
	<div ref="containerRef" class="relative inline-block w-full">
		<span
			ref="triggerRef"
			role="button"
			tabindex="0"
			class="relative cursor-pointer flex h-11 w-full items-center justify-between overflow-hidden rounded-xl bg-button-bg px-4 py-2.5 text-left outline outline-1 outline-offset-[-1px] outline-button-border transition-all duration-200 text-button-text hover:bg-button-bgHover active:bg-button-bgActive"
			:class="[
				triggerClasses,
				{
					'z-[9999]': isOpen,
					'rounded-b-none': isOpen && openDirection === 'down',
					'rounded-t-none': isOpen && openDirection === 'up',
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
				<span class="text-base font-semibold leading-tight">
					<slot name="selected">{{ triggerText }}</slot>
				</span>
			</div>
			<div class="flex items-center gap-1">
				<slot name="suffix"></slot>
				<ChevronLeftIcon
					v-if="showChevron"
					class="size-5 shrink-0 rotate-90 transition-transform duration-200"
					:class="{ '-rotate-90': isOpen }"
				/>
			</div>
		</span>

		<Teleport to="#teleports">
			<div
				v-if="isOpen"
				ref="dropdownRef"
				class="fixed z-[9999] flex flex-col overflow-hidden rounded-[14px] bg-dropdown-bg outline outline-1 outline-offset-[-1px] outline-button-border"
				:class="openDirection === 'down' ? 'rounded-t-none' : 'rounded-b-none'"
				:style="dropdownStyle"
				:role="listbox ? 'listbox' : 'menu'"
				@mousedown.stop
				@keydown="handleDropdownKeydown"
			>
				<div v-if="searchable" class="p-4">
					<div
						class="flex items-center gap-2 overflow-hidden rounded-xl bg-bg-raised px-4 py-2.5 outline outline-1 outline-offset-[-1px] outline-button-border focus-within:outline-2 focus-within:outline-contrast"
					>
						<SearchIcon class="size-5" />
						<input
							ref="searchInputRef"
							v-model="searchQuery"
							type="text"
							:placeholder="searchPlaceholder"
							class="flex-1 !bg-bg-raised text-sm font-medium leading-[18px] text-dropdown-text placeholder-secondary !shadow-none !outline-none"
							@keydown.stop="handleSearchKeydown"
						/>
					</div>
				</div>

				<div v-if="searchable && filteredOptions.length > 0" class="h-px bg-divider"></div>

				<div
					v-if="filteredOptions.length > 0"
					ref="optionsContainerRef"
					class="flex flex-col gap-2 overflow-y-auto p-3"
					:style="{ maxHeight: `${maxHeight}px` }"
				>
					<template v-for="(item, index) in filteredOptions" :key="item.key">
						<div v-if="item.type === 'divider'" class="h-px bg-divider"></div>
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
							class="flex items-center gap-2.5 cursor-pointer rounded-xl px-4 py-3 text-left transition-colors duration-150 text-dropdown-text hover:bg-button-bgHover focus:bg-button-bgHover"
							:class="[
								item.class,
								{
									'bg-button-bgSelected text-button-textSelected':
										listbox && item.value === modelValue,
									'bg-button-bgHover':
										focusedIndex === index && !(listbox && item.value === modelValue),
									'cursor-not-allowed opacity-50 pointer-events-none': item.disabled,
								},
							]"
							tabindex="-1"
							@click="handleOptionClick(item, index)"
							@mouseenter="!item.disabled && (focusedIndex = index)"
						>
							<slot :name="`option-${item.value}`" :item="item">
								<div class="flex items-center gap-2">
									<component :is="item.icon" v-if="item.icon" class="h-5 w-5" />
									<span class="text-base font-semibold leading-tight">
										{{ item.label }}
									</span>
								</div>
							</slot>
						</component>
					</template>
				</div>

				<div v-else-if="searchQuery" class="p-4 text-center text-sm text-secondary">
					No results found
				</div>
			</div>
		</Teleport>
	</div>
</template>

<script setup lang="ts" generic="T">
import { ChevronLeftIcon, SearchIcon } from '@modrinth/assets'
import { onClickOutside } from '@vueuse/core'
import { computed, nextTick, onMounted, onUnmounted, ref, useSlots, watch } from 'vue'

export interface DropdownOption<T> {
	value: T
	label: string
	icon?: any
	disabled?: boolean
	class?: string
	type?: 'button' | 'link' | 'divider'
	href?: string
	target?: string
	action?: () => void
}

const props = withDefaults(
	defineProps<{
		modelValue?: T
		options: (DropdownOption<T> | { type: 'divider' })[]
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
	}>(),
	{
		placeholder: 'Select an option',
		disabled: false,
		searchable: false,
		searchPlaceholder: 'Search...',
		listbox: true,
		showChevron: true,
		maxHeight: 300,
		extraPosition: 'bottom',
	},
)

const emit = defineEmits<{
	'update:modelValue': [value: T]
	select: [option: DropdownOption<T>]
	open: []
	close: []
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

const selectedOption = computed<DropdownOption<T> | undefined>(() => {
	return props.options.find(
		(opt) =>
			(opt as any).type !== 'divider' && (opt as DropdownOption<T>).value === props.modelValue,
	) as DropdownOption<T> | undefined
})

const triggerText = computed(() => {
	if (props.displayValue !== undefined) return props.displayValue
	if (selectedOption.value) return selectedOption.value.label
	return props.placeholder
})

const filteredOptions = computed(() => {
	const opts = props.options.map((opt, index) => ({
		...opt,
		key: opt.type === 'divider' ? `divider-${index}` : `option-${(opt as DropdownOption<T>).value}`,
	}))

	if (!searchQuery.value || !props.searchable) {
		return opts
	}

	const query = searchQuery.value.toLowerCase()
	return opts.filter((opt) => {
		if (opt.type === 'divider') return false
		return (opt as DropdownOption<T>).label.toLowerCase().includes(query)
	})
})

function setOptionRef(el: HTMLElement | null, index: number) {
	optionRefs.value[index] = el
}

async function updateDropdownPosition() {
	if (!triggerRef.value || !dropdownRef.value) return

	await nextTick()

	const triggerRect = triggerRef.value.getBoundingClientRect()
	const dropdownRect = dropdownRef.value.getBoundingClientRect()
	const viewportHeight = window.innerHeight
	const viewportWidth = window.innerWidth
	const margin = 8

	let top = triggerRect.bottom
	let left = triggerRect.left
	let opensUp = false

	if (triggerRect.bottom + dropdownRect.height + margin > viewportHeight) {
		if (triggerRect.top - dropdownRect.height - margin > 0) {
			top = triggerRect.top - dropdownRect.height
			opensUp = true
		}
	}

	if (left + dropdownRect.width > viewportWidth - margin) {
		left = Math.max(margin, viewportWidth - dropdownRect.width - margin)
	}

	dropdownStyle.value = {
		top: `${top}px`,
		left: `${left}px`,
		width: `${triggerRect.width}px`,
	}

	openDirection.value = opensUp ? 'up' : 'down'
}

async function open() {
	if (props.disabled || isOpen.value) return

	isOpen.value = true
	searchQuery.value = ''
	focusedIndex.value = props.listbox
		? props.options.findIndex((opt) => opt.type !== 'divider' && opt.value === props.modelValue)
		: -1

	emit('open')

	await nextTick()
	await updateDropdownPosition()

	if (props.searchable && searchInputRef.value) {
		searchInputRef.value.focus()
	}
}

function close() {
	if (!isOpen.value) return

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
		close()
	} else {
		open()
	}
}

function handleOptionClick(option: DropdownOption<T>, index: number) {
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
		close()
	}
}

function focusOption(index: number) {
	if (index < 0 || index >= filteredOptions.value.length) return

	const option = filteredOptions.value[index]
	if (option.type === 'divider' || (option as DropdownOption<T>).disabled) return

	focusedIndex.value = index
	optionRefs.value[index]?.focus()
	optionRefs.value[index]?.scrollIntoView({ block: 'nearest' })
}

function focusNextOption() {
	let nextIndex = focusedIndex.value
	do {
		nextIndex = (nextIndex + 1) % filteredOptions.value.length
	} while (
		filteredOptions.value[nextIndex].type === 'divider' ||
		(filteredOptions.value[nextIndex] as DropdownOption<T>).disabled
	)
	focusOption(nextIndex)
}

function focusPreviousOption() {
	let prevIndex = focusedIndex.value
	do {
		prevIndex = (prevIndex - 1 + filteredOptions.value.length) % filteredOptions.value.length
	} while (
		filteredOptions.value[prevIndex].type === 'divider' ||
		(filteredOptions.value[prevIndex] as DropdownOption<T>).disabled
	)
	focusOption(prevIndex)
}

function handleTriggerKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'Enter':
		case ' ':
		case 'ArrowDown':
			event.preventDefault()
			open()
			break
		case 'ArrowUp':
			event.preventDefault()
			open()
			break
	}
}

function handleDropdownKeydown(event: KeyboardEvent) {
	switch (event.key) {
		case 'Escape':
			event.preventDefault()
			close()
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
				if (option.type !== 'divider') {
					handleOptionClick(option as DropdownOption<T>, focusedIndex.value)
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
		close()
	} else if (event.key === 'ArrowDown') {
		event.preventDefault()
		focusNextOption()
	} else if (event.key === 'ArrowUp') {
		event.preventDefault()
		focusPreviousOption()
	}
}

function handleResize() {
	if (isOpen.value) {
		updateDropdownPosition()
	}
}

onClickOutside(
	dropdownRef,
	() => {
		close()
	},
	{ ignore: [triggerRef] },
)

onMounted(() => {
	window.addEventListener('resize', handleResize)
	window.addEventListener('scroll', handleResize, true)
})

onUnmounted(() => {
	window.removeEventListener('resize', handleResize)
	window.removeEventListener('scroll', handleResize, true)
})

watch(isOpen, (value) => {
	if (value) {
		updateDropdownPosition()
	}
})
</script>
