<template>
	<div data-pyro-telepopover-wrapper class="relative">
		<button
			ref="triggerRef"
			v-tooltip="tooltip"
			class="teleport-overflow-menu-trigger"
			:class="btnClass"
			:aria-expanded="isOpen"
			:aria-haspopup="true"
			:aria-label="ariaLabel"
			:disabled="disabled"
			@mousedown="handleMouseDown"
			@mouseenter="handleMouseEnter"
			@mouseleave="handleMouseLeave"
			@click="toggleMenu"
		>
			<slot></slot>
		</button>
		<Teleport to="#teleports">
			<Transition
				enter-active-class="transition duration-125 ease-out"
				enter-from-class="transform scale-75 opacity-0"
				enter-to-class="transform scale-100 opacity-100"
				leave-active-class="transition duration-125 ease-in"
				leave-from-class="transform scale-100 opacity-100"
				leave-to-class="transform scale-75 opacity-0"
			>
				<div
					v-if="isOpen"
					ref="menuRef"
					data-pyro-telepopover-root
					class="fixed isolate z-[9999] flex w-fit flex-col gap-2 overflow-hidden rounded-2xl border-[1px] border-solid border-surface-5 bg-bg-raised p-2 shadow-lg"
					:style="menuStyle"
					role="menu"
					tabindex="-1"
					@mousedown.stop
					@mouseleave="handleMouseLeave"
				>
					<template
						v-for="(option, index) in filteredOptions"
						:key="isDivider(option) ? `divider-${index}` : option.id"
					>
						<div v-if="isDivider(option)" class="h-px w-full bg-surface-5"></div>
						<ButtonStyled
							v-else
							type="transparent"
							role="menuitem"
							:color="optionButtonColor(option)"
						>
							<button
								v-if="typeof option.action === 'function' || option.disabled"
								:ref="
									(el) => {
										if (el) menuItemsRef[index] = el as HTMLElement
									}
								"
								v-tooltip="option.tooltip"
								:disabled="option.disabled"
								class="w-full !justify-start !whitespace-nowrap focus-visible:!outline-none"
								:aria-selected="index === selectedIndex"
								:style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
								@click="(event) => handleItemClick(option, index, event)"
								@focus="selectedIndex = index"
								@mouseover="handleMouseOver(index)"
							>
								<slot :name="option.id">
									<component :is="option.icon" v-if="option.icon" class="size-5" />
									{{ option.label ?? option.id }}
								</slot>
							</button>
							<AutoLink
								v-else-if="optionLink(option)"
								:ref="
									(el) => {
										if (el) menuItemsRef[index] = el as HTMLElement
									}
								"
								:to="optionLink(option)"
								:target="option.external ? '_blank' : undefined"
								:rel="option.external ? 'noopener noreferrer' : undefined"
								class="w-full !justify-start !whitespace-nowrap focus-visible:!outline-none"
								:aria-selected="index === selectedIndex"
								:style="index === selectedIndex ? { background: 'var(--color-button-bg)' } : {}"
								@click="(event) => handleItemClick(option, index, event)"
								@focus="selectedIndex = index"
								@mouseover="handleMouseOver(index)"
							>
								<slot :name="option.id">
									<component :is="option.icon" v-if="option.icon" class="size-5" />
									{{ option.label ?? option.id }}
								</slot>
							</AutoLink>
							<span v-else>
								<slot :name="option.id">
									<component :is="option.icon" v-if="option.icon" class="size-5" />
									{{ option.label ?? option.id }}
								</slot>
							</span>
						</ButtonStyled>
					</template>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>

<script setup lang="ts">
import { onClickOutside, useElementHover } from '@vueuse/core'
import { type Component, computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import AutoLink from './AutoLink.vue'
import ButtonStyled from './ButtonStyled.vue'

type OptionColor =
	| 'standard'
	| 'brand'
	| 'primary'
	| 'danger'
	| 'secondary'
	| 'highlight'
	| 'red'
	| 'orange'
	| 'green'
	| 'blue'
	| 'purple'

export interface Option {
	id: string
	label?: string
	icon?: Component
	action?: ((event?: MouseEvent) => void) | string
	link?: string
	external?: boolean
	shown?: boolean
	color?: OptionColor
	disabled?: boolean
	tooltip?: string
	remainOnClick?: boolean
}

export type Divider = {
	divider?: boolean
	shown?: boolean
}

export type Item = Option | Divider

function isDivider(item: Item): item is Divider {
	return !!(item as Divider).divider
}

const props = withDefaults(
	defineProps<{
		options: Item[]
		hoverable?: boolean
		btnClass?: string | string[] | Record<string, boolean>
		disabled?: boolean
		tooltip?: string
		ariaLabel?: string
	}>(),
	{
		hoverable: false,
		btnClass: undefined,
		disabled: false,
		tooltip: undefined,
		ariaLabel: undefined,
	},
)

const emit = defineEmits<{
	select: [option: Option]
	open: []
}>()

const isOpen = ref(false)
const selectedIndex = ref(-1)
const menuRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLElement | null>(null)
const isMouseDown = ref(false)
const typeAheadBuffer = ref('')
const typeAheadTimeout = ref<number | null>(null)
const menuItemsRef = ref<HTMLElement[]>([])

const hoveringTrigger = useElementHover(triggerRef)
const hoveringMenu = useElementHover(menuRef)

const hovering = computed(() => hoveringTrigger.value || hoveringMenu.value)

const menuStyle = ref({
	top: '0px',
	left: '0px',
})

const filteredOptions = computed(() => props.options.filter((option) => option.shown !== false))

const calculateMenuPosition = () => {
	if (!triggerRef.value || !menuRef.value) return { top: '0px', left: '0px' }

	const triggerRect = triggerRef.value.getBoundingClientRect()
	const menuWidth = menuRef.value.offsetWidth
	const menuHeight = menuRef.value.offsetHeight
	const margin = 8

	let top: number
	let left: number

	if (triggerRect.bottom + menuHeight + margin <= window.innerHeight) {
		top = triggerRect.bottom + margin
	} else if (triggerRect.top - menuHeight - margin >= 0) {
		top = triggerRect.top - menuHeight - margin
	} else {
		top = Math.max(margin, window.innerHeight - menuHeight - margin)
	}

	if (triggerRect.right - menuWidth >= margin) {
		left = triggerRect.right - menuWidth
	} else {
		left = Math.max(margin, triggerRect.left)
	}

	return {
		top: `${top}px`,
		left: `${left}px`,
	}
}

const toggleMenu = (event: MouseEvent) => {
	event.stopPropagation()
	if (props.disabled) return
	if (!props.hoverable) {
		if (isOpen.value) {
			closeMenu()
		} else {
			openMenu()
		}
	}
}

const openMenu = () => {
	if (props.disabled) return
	isOpen.value = true
	emit('open')
	disableBodyScroll()
	nextTick(() => {
		menuStyle.value = calculateMenuPosition()
		document.addEventListener('mousemove', handleMouseMove)
		focusFirstMenuItem()
	})
}

const closeMenu = () => {
	isOpen.value = false
	selectedIndex.value = -1
	enableBodyScroll()
	document.removeEventListener('mousemove', handleMouseMove)
}

const selectOption = (option: Option, event?: MouseEvent) => {
	emit('select', option)
	if (typeof option.action === 'function') {
		option.action(event)
	}
	if (!option.remainOnClick) {
		closeMenu()
	}
}

const handleMouseDown = (event: MouseEvent) => {
	if (props.disabled) return
	event.preventDefault()
	isMouseDown.value = true
}

const handleMouseEnter = () => {
	if (props.hoverable) {
		openMenu()
	}
}

const handleMouseLeave = () => {
	if (props.hoverable) {
		setTimeout(() => {
			if (!hovering.value) {
				closeMenu()
			}
		}, 250)
	}
}

const handleMouseMove = (event: MouseEvent) => {
	if (!isOpen.value || !isMouseDown.value) return

	const menuRect = menuRef.value?.getBoundingClientRect()
	if (!menuRect) return

	const menuItems = menuRef.value?.querySelectorAll('[role="menuitem"]')
	if (!menuItems) return

	for (let i = 0; i < menuItems.length; i++) {
		const itemRect = (menuItems[i] as HTMLElement).getBoundingClientRect()
		if (
			event.clientX >= itemRect.left &&
			event.clientX <= itemRect.right &&
			event.clientY >= itemRect.top &&
			event.clientY <= itemRect.bottom
		) {
			selectedIndex.value = i
			break
		}
	}
}

const handleItemClick = (option: Option, index: number, event?: MouseEvent) => {
	if (option.disabled) return
	selectedIndex.value = index
	selectOption(option, event)
}

const handleMouseOver = (index: number) => {
	selectedIndex.value = index
	menuItemsRef.value[selectedIndex.value]?.focus?.()
}

const disableBodyScroll = () => {
	document.body.style.overflow = 'hidden'
}

const enableBodyScroll = () => {
	document.body.style.overflow = ''
}

const focusFirstMenuItem = () => {
	if (menuItemsRef.value.length > 0) {
		menuItemsRef.value[0]?.focus?.()
	}
}

const handleKeydown = (event: KeyboardEvent) => {
	if (!isOpen.value) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault()
			openMenu()
		}
		return
	}

	switch (event.key) {
		case 'ArrowDown':
			event.preventDefault()
			selectedIndex.value = (selectedIndex.value + 1) % filteredOptions.value.length
			menuItemsRef.value[selectedIndex.value]?.focus?.()
			break
		case 'ArrowUp':
			event.preventDefault()
			selectedIndex.value =
				(selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length
			menuItemsRef.value[selectedIndex.value]?.focus?.()
			break
		case 'Home':
			event.preventDefault()
			if (menuItemsRef.value.length > 0) {
				selectedIndex.value = 0
				menuItemsRef.value[selectedIndex.value]?.focus?.()
			}
			break
		case 'End':
			event.preventDefault()
			if (menuItemsRef.value.length > 0) {
				selectedIndex.value = filteredOptions.value.length - 1
				menuItemsRef.value[selectedIndex.value]?.focus?.()
			}
			break
		case 'Enter':
		case ' ':
			event.preventDefault()
			if (selectedIndex.value >= 0) {
				const option = filteredOptions.value[selectedIndex.value]
				if (isDivider(option)) break
				selectOption(option)
			}
			break
		case 'Escape':
			event.preventDefault()
			closeMenu()
			triggerRef.value?.focus?.()
			break
		case 'Tab':
			event.preventDefault()
			if (menuItemsRef.value.length > 0) {
				if (event.shiftKey) {
					selectedIndex.value =
						(selectedIndex.value - 1 + filteredOptions.value.length) % filteredOptions.value.length
				} else {
					selectedIndex.value = (selectedIndex.value + 1) % filteredOptions.value.length
				}
				menuItemsRef.value[selectedIndex.value]?.focus?.()
			}
			break
		default:
			if (event.key.length === 1) {
				typeAheadBuffer.value += event.key.toLowerCase()
				const matchIndex = filteredOptions.value.findIndex(
					(option) =>
						!isDivider(option) && option.id.toLowerCase().startsWith(typeAheadBuffer.value),
				)
				if (matchIndex !== -1) {
					selectedIndex.value = matchIndex
					menuItemsRef.value[selectedIndex.value]?.focus?.()
				}
				if (typeAheadTimeout.value) {
					clearTimeout(typeAheadTimeout.value)
				}
				typeAheadTimeout.value = setTimeout(() => {
					typeAheadBuffer.value = ''
				}, 1000) as unknown as number
			}
			break
	}
}

const handleResizeOrScroll = () => {
	if (isOpen.value) {
		menuStyle.value = calculateMenuPosition()
	}
}

const throttle = <T extends unknown[]>(
	func: (...args: T) => void,
	limit: number,
): ((...args: T) => void) => {
	let inThrottle: boolean
	return function (...args: T) {
		if (!inThrottle) {
			func(...args)
			inThrottle = true
			setTimeout(() => (inThrottle = false), limit)
		}
	}
}

const throttledHandleResizeOrScroll = throttle(handleResizeOrScroll, 100)

onMounted(() => {
	triggerRef.value?.addEventListener('keydown', handleKeydown)
	window.addEventListener('resize', throttledHandleResizeOrScroll)
	window.addEventListener('scroll', throttledHandleResizeOrScroll)
})

onUnmounted(() => {
	triggerRef.value?.removeEventListener('keydown', handleKeydown)
	window.removeEventListener('resize', throttledHandleResizeOrScroll)
	window.removeEventListener('scroll', throttledHandleResizeOrScroll)
	document.removeEventListener('mousemove', handleMouseMove)
	if (typeAheadTimeout.value) {
		clearTimeout(typeAheadTimeout.value)
	}
	enableBodyScroll()
})

watch(isOpen, (newValue) => {
	if (newValue) {
		nextTick(() => {
			menuRef.value?.addEventListener('keydown', handleKeydown)
		})
	} else {
		menuRef.value?.removeEventListener('keydown', handleKeydown)
	}
})

onClickOutside(menuRef, (event) => {
	if (!triggerRef.value?.contains(event.target as Node)) {
		closeMenu()
	}
})

function optionLink(option: Option) {
	if (typeof option.action === 'string') return option.action
	return option.link
}

function optionButtonColor(option: Option) {
	switch (option.color) {
		case 'primary':
			return 'brand'
		case 'danger':
			return 'red'
		case 'secondary':
		case 'highlight':
			return 'standard'
		default:
			return option.color ?? 'standard'
	}
}
</script>
