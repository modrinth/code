<script setup lang="ts">
import type { CSSProperties } from 'vue'
import { computed, nextTick, onUnmounted, ref, useId, watch } from 'vue'
import { RouterLink } from 'vue-router'

import { useAnchoredTeleport } from '../../../utils/use-anchored-teleport'
import Button from './Button.vue'
import IconButton from './IconButton.vue'
import type {
	ButtonColor,
	ButtonElementHandle,
	ButtonInteraction,
	ButtonSize,
	ButtonType,
	OverflowMenuAction,
	OverflowMenuLink,
	OverflowMenuOption,
	TeleportPlacement,
} from './types'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		label: string
		options: OverflowMenuOption[]
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		interaction?: ButtonInteraction
		disabled?: boolean
		iconOnly?: boolean
		circular?: boolean
		tooltip?: string
		placement?: TeleportPlacement
		distance?: number
		hoverable?: boolean
	}>(),
	{
		type: 'base',
		size: 'md',
		disabled: false,
		iconOnly: true,
		circular: true,
		placement: 'bottom-end',
		distance: 8,
		hoverable: false,
	},
)

const emit = defineEmits<{
	select: [option: OverflowMenuAction | OverflowMenuLink]
	open: []
	close: []
}>()

const triggerButton = ref<ButtonElementHandle | null>(null)
const triggerElement = computed(() => triggerButton.value?.element ?? null)
const panelElement = ref<HTMLElement | null>(null)
const resolvedPlacement = computed(() => props.placement)
const resolvedDistance = computed(() => props.distance)
const menuId = `button-overflow-${useId()}`
const selectedIndex = ref(-1)
const typeahead = ref('')
let typeaheadTimer: ReturnType<typeof setTimeout> | undefined
let hoverCloseTimer: ReturnType<typeof setTimeout> | undefined
const triggerComponent = computed(() => (props.iconOnly ? IconButton : Button))

const visibleOptions = computed(() => props.options.filter((option) => option.shown !== false))
const menuOptions = computed(() =>
	visibleOptions.value.filter(
		(option): option is OverflowMenuAction | OverflowMenuLink => option.type !== 'divider',
	),
)

const { isOpen, panelStyle, anchorStyle, resolvedSide, open, close } = useAnchoredTeleport(
	triggerElement,
	panelElement,
	resolvedPlacement,
	resolvedDistance,
)

const menuTransformOrigin = computed(() => {
	switch (resolvedSide.value) {
		case 'top':
			return 'bottom center'
		case 'left':
			return 'right center'
		case 'right':
			return 'left center'
		default:
			return 'top center'
	}
})

const menuItemClasses =
	'overflow-menu-item flex min-h-10 w-full items-center gap-2 rounded-[10px] border-0 bg-transparent px-3 py-2 text-left text-base font-semibold leading-5 text-contrast no-underline ' +
	'cursor-pointer whitespace-nowrap hover:bg-surface-4 focus-visible:bg-surface-4 focus-visible:outline-none ' +
	'disabled:cursor-not-allowed disabled:opacity-50 [&[aria-disabled=true]]:cursor-not-allowed [&[aria-disabled=true]]:opacity-50 ' +
	'[&>svg]:size-5 [&>svg]:shrink-0 [&>svg]:text-primary'

const toneVariables: Record<ButtonColor, string> = {
	brand: 'var(--color-brand)',
	red: 'var(--color-red)',
	orange: 'var(--color-orange)',
	green: 'var(--color-green)',
	blue: 'var(--color-blue)',
	purple: 'var(--color-purple)',
	medal_promotion: 'var(--medal-promotion-text-orange, var(--color-orange))',
}

function getMenuItemStyle(option: OverflowMenuAction | OverflowMenuLink) {
	if (!option.tone || option.tone === 'default') return undefined

	return {
		'--overflow-menu-item-tone': toneVariables[option.tone],
	} as CSSProperties
}

function isDivider(
	option: OverflowMenuOption,
): option is Extract<OverflowMenuOption, { type: 'divider' }> {
	return option.type === 'divider'
}

function isLink(option: OverflowMenuOption): option is OverflowMenuLink {
	return option.type === 'link'
}

function getMenuItems() {
	if (!panelElement.value) return []
	return Array.from(panelElement.value.querySelectorAll<HTMLElement>('[role="menuitem"]'))
}

function focusItem(index: number) {
	const items = getMenuItems()
	if (items.length === 0) return
	selectedIndex.value = (index + items.length) % items.length
	items[selectedIndex.value]?.focus()
}

async function openMenu(position: 'first' | 'last' = 'first', focus = true) {
	if (props.disabled || isOpen.value) return
	await open()
	emit('open')
	if (!focus) return
	await nextTick()
	focusItem(position === 'first' ? 0 : getMenuItems().length - 1)
}

function closeMenu(restoreFocus = false) {
	if (!isOpen.value) return
	selectedIndex.value = -1
	close(restoreFocus)
}

async function toggleMenu(event?: MouseEvent) {
	if (props.hoverable && window.matchMedia('(hover: hover)').matches && event?.detail) return
	if (isOpen.value) closeMenu()
	else await openMenu()
}

function clearHoverCloseTimer() {
	if (!hoverCloseTimer) return
	clearTimeout(hoverCloseTimer)
	hoverCloseTimer = undefined
}

function handleMouseEnter() {
	if (!props.hoverable || !window.matchMedia('(hover: hover)').matches) return
	clearHoverCloseTimer()
	openMenu('first', false)
}

function handleMouseLeave() {
	if (!props.hoverable || !window.matchMedia('(hover: hover)').matches) return
	clearHoverCloseTimer()
	hoverCloseTimer = setTimeout(() => closeMenu(), 250)
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
	event.preventDefault()
	openMenu(event.key === 'ArrowDown' ? 'first' : 'last')
}

function handleAction(option: OverflowMenuAction, event: MouseEvent) {
	if (option.disabled) return
	option.action(event)
	emit('select', option)
	if (!option.remainOpen) closeMenu(true)
}

function handleLink(option: OverflowMenuLink, event: MouseEvent) {
	if (option.disabled) {
		event.preventDefault()
		return
	}
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

function handleLinkKeydown(option: OverflowMenuLink, event: KeyboardEvent) {
	if (event.key !== ' ') return
	event.preventDefault()
	if (option.disabled) return
	;(event.currentTarget as HTMLElement).click()
}

function handleMenuKeydown(event: KeyboardEvent) {
	const items = getMenuItems()
	if (items.length === 0) return

	switch (event.key) {
		case 'ArrowDown':
			event.preventDefault()
			focusItem(selectedIndex.value + 1)
			break
		case 'ArrowUp':
			event.preventDefault()
			focusItem(selectedIndex.value - 1)
			break
		case 'Home':
			event.preventDefault()
			focusItem(0)
			break
		case 'End':
			event.preventDefault()
			focusItem(items.length - 1)
			break
		case 'Escape':
			event.preventDefault()
			closeMenu(true)
			break
		case 'Tab':
			triggerElement.value?.focus()
			closeMenu()
			break
		default: {
			if (
				event.key === ' ' ||
				event.key.length !== 1 ||
				event.ctrlKey ||
				event.metaKey ||
				event.altKey
			)
				return

			const character = event.key.toLocaleLowerCase()
			const query = typeahead.value === character ? character : `${typeahead.value}${character}`
			const startIndex = query.length === 1 ? selectedIndex.value + 1 : 0
			typeahead.value = query

			for (let offset = 0; offset < menuOptions.value.length; offset++) {
				const index = (startIndex + offset) % menuOptions.value.length
				if (menuOptions.value[index]?.label.toLocaleLowerCase().startsWith(query)) {
					focusItem(index)
					break
				}
			}

			if (typeaheadTimer) clearTimeout(typeaheadTimer)
			typeaheadTimer = setTimeout(() => {
				typeahead.value = ''
			}, 500)
		}
	}
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
	if (!openState && typeaheadTimer) {
		clearTimeout(typeaheadTimer)
		typeaheadTimer = undefined
		typeahead.value = ''
	}
})

onUnmounted(clearHoverCloseTimer)

defineExpose({ open: openMenu, close: closeMenu })
</script>

<template>
	<component
		:is="triggerComponent"
		ref="triggerButton"
		v-bind="$attrs"
		v-tooltip="props.tooltip"
		:label="props.iconOnly ? props.label : undefined"
		:aria-label="props.iconOnly ? undefined : props.label"
		:circular="props.iconOnly ? props.circular : undefined"
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:interaction="props.interaction"
		:disabled="props.disabled"
		:aria-expanded="isOpen"
		:aria-controls="menuId"
		aria-haspopup="menu"
		@click="toggleMenu"
		@keydown="handleTriggerKeydown"
		@mouseenter="handleMouseEnter"
		@mouseleave="handleMouseLeave"
	>
		<slot />
	</component>

	<Teleport to="body">
		<Transition name="floating-expand">
			<div
				v-if="isOpen"
				:id="menuId"
				ref="panelElement"
				class="fixed isolate z-[9999] rounded-[14px] bg-surface-3 shadow-lg ring-1 ring-surface-5"
				:style="[panelStyle, { transformOrigin: menuTransformOrigin }]"
				role="menu"
				:aria-label="props.label"
				@keydown="handleMenuKeydown"
				@mouseenter="handleMouseEnter"
				@mouseleave="handleMouseLeave"
			>
				<span
					aria-hidden="true"
					class="overflow-menu-arrow"
					:data-side="resolvedSide"
					:style="anchorStyle"
				/>
				<div
					data-anchored-scroll-region
					class="flex min-w-48 flex-col gap-1 overflow-y-auto p-2"
					:style="{ maxHeight: panelStyle.maxHeight }"
				>
					<template
						v-for="(option, index) in visibleOptions"
						:key="option.id ?? `divider-${index}`"
					>
						<div v-if="isDivider(option)" role="separator" class="my-1 h-px bg-surface-5" />

						<RouterLink
							v-else-if="isLink(option) && option.to !== undefined && !option.disabled"
							v-tooltip="option.tooltip"
							:to="option.to"
							:class="menuItemClasses"
							:style="getMenuItemStyle(option)"
							:data-tone="option.tone && option.tone !== 'default' ? option.tone : undefined"
							:data-hover-filled="option.hoverFilled || option.hoverFilledOnly || undefined"
							:data-hover-filled-only="option.hoverFilledOnly || undefined"
							role="menuitem"
							tabindex="-1"
							@click="handleLink(option, $event)"
							@keydown="handleLinkKeydown(option, $event)"
							@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
						>
							<slot :name="option.id" :option="option">
								<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
								{{ option.label }}
							</slot>
						</RouterLink>

						<a
							v-else-if="isLink(option)"
							v-tooltip="option.tooltip"
							:href="option.disabled ? undefined : option.href"
							:target="option.target"
							:rel="option.rel ?? (option.target === '_blank' ? 'noopener noreferrer' : undefined)"
							:download="option.download"
							:aria-disabled="option.disabled || undefined"
							:class="menuItemClasses"
							:style="getMenuItemStyle(option)"
							:data-tone="option.tone && option.tone !== 'default' ? option.tone : undefined"
							:data-hover-filled="option.hoverFilled || option.hoverFilledOnly || undefined"
							:data-hover-filled-only="option.hoverFilledOnly || undefined"
							role="menuitem"
							tabindex="-1"
							@click="handleLink(option, $event)"
							@keydown="handleLinkKeydown(option, $event)"
							@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
						>
							<slot :name="option.id" :option="option">
								<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
								{{ option.label }}
							</slot>
						</a>

						<button
							v-else
							v-tooltip="option.tooltip"
							type="button"
							:aria-disabled="option.disabled || undefined"
							:class="menuItemClasses"
							:style="getMenuItemStyle(option)"
							:data-tone="option.tone && option.tone !== 'default' ? option.tone : undefined"
							:data-hover-filled="option.hoverFilled || option.hoverFilledOnly || undefined"
							:data-hover-filled-only="option.hoverFilledOnly || undefined"
							role="menuitem"
							tabindex="-1"
							@click="handleAction(option, $event)"
							@focus="selectedIndex = getMenuItems().indexOf($event.currentTarget as HTMLElement)"
						>
							<slot :name="option.id" :option="option">
								<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
								{{ option.label }}
							</slot>
						</button>
					</template>
				</div>
			</div>
		</Transition>
	</Teleport>
</template>

<style scoped>
.overflow-menu-arrow {
	position: absolute;
	width: 0;
	height: 0;
	pointer-events: none;
}

.overflow-menu-arrow::before {
	position: absolute;
	width: 10px;
	height: 10px;
	content: '';
	background-color: var(--surface-3);
	transform: translate(-50%, -50%) rotate(45deg);
}

.overflow-menu-arrow[data-side='bottom'] {
	top: 0;
}

.overflow-menu-arrow[data-side='bottom']::before {
	border-top: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
}

.overflow-menu-arrow[data-side='top'] {
	bottom: 0;
}

.overflow-menu-arrow[data-side='top']::before {
	border-right: 1px solid var(--surface-5);
	border-bottom: 1px solid var(--surface-5);
}

.overflow-menu-arrow[data-side='right'] {
	left: 0;
}

.overflow-menu-arrow[data-side='right']::before {
	border-bottom: 1px solid var(--surface-5);
	border-left: 1px solid var(--surface-5);
}

.overflow-menu-arrow[data-side='left'] {
	right: 0;
}

.overflow-menu-arrow[data-side='left']::before {
	border-top: 1px solid var(--surface-5);
	border-right: 1px solid var(--surface-5);
}

.overflow-menu-item[data-tone]:not([data-hover-filled-only]) {
	color: var(--overflow-menu-item-tone);
}

.overflow-menu-item[data-tone]:not([data-hover-filled-only]) :deep(svg) {
	color: var(--overflow-menu-item-tone);
}

.overflow-menu-item[data-tone][data-hover-filled]:hover,
.overflow-menu-item[data-tone][data-hover-filled]:focus-visible {
	color: var(--color-accent-contrast);
	background-color: var(--overflow-menu-item-tone);
}

.overflow-menu-item[data-tone][data-hover-filled]:hover :deep(svg),
.overflow-menu-item[data-tone][data-hover-filled]:focus-visible :deep(svg) {
	color: inherit;
}
</style>
