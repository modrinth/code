<script setup lang="ts">
import { computed, nextTick, onMounted, ref, toRef, useId, watch } from 'vue'

import { useAnchoredTeleport } from '../../../utils/use-anchored-teleport'
import Button from './Button.vue'
import {
	isDivider,
	isHeading,
	isLink,
	isMenuRow,
	isSubmenu,
	useHoverIntent,
	useMenuKeyboard,
	visibleOptions,
} from './button-menu/button-menu'
import ButtonMenuItem from './button-menu/ButtonMenuItem.vue'
import ButtonMenuPanel from './button-menu/ButtonMenuPanel.vue'
import ButtonMenuSubmenu from './button-menu/ButtonMenuSubmenu.vue'
import IconButton from './IconButton.vue'
import type {
	ButtonColor,
	ButtonElementHandle,
	ButtonInteraction,
	ButtonMenuAction,
	ButtonMenuLink,
	ButtonMenuOption,
	ButtonSize,
	ButtonType,
	TeleportPlacement,
} from './types'

const HOVER_CLOSE_DELAY = 250

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		label: string
		options: ButtonMenuOption[]
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
	select: [option: ButtonMenuAction | ButtonMenuLink]
	open: []
	close: []
}>()

const triggerButton = ref<ButtonElementHandle | null>(null)
const triggerElement = computed(() => triggerButton.value?.element ?? null)
const panel = ref<InstanceType<typeof ButtonMenuPanel> | null>(null)
const panelElement = computed(() => panel.value?.element ?? null)
const resolvedPlacement = toRef(props, 'placement')
const resolvedDistance = toRef(props, 'distance')
const menuId = `button-overflow-${useId()}`
const triggerComponent = computed(() => (props.iconOnly ? IconButton : Button))

const options = computed(() => visibleOptions(props.options))
const rowOptions = computed(() => options.value.filter(isMenuRow))

const { isOpen, panelStyle, anchorStyle, resolvedSide, expandOrigin, open, close } =
	useAnchoredTeleport(triggerElement, panelElement, resolvedPlacement, resolvedDistance)

const { focusedIndex, getItems, focusItem, handleKeydown, reset } = useMenuKeyboard({
	panel: panelElement,
	rows: () => rowOptions.value,
	onEscape: () => closeMenu(true),
	onTab: () => {
		triggerElement.value?.focus()
		closeMenu()
	},
})

const { handleMouseEnter, handleMouseLeave, cancelLeave } = useHoverIntent({
	closeDelay: HOVER_CLOSE_DELAY,
	enabled: () => props.hoverable,
	onEnter: () => openMenu('first', false),
	onLeave: () => closeMenu(),
})

async function openMenu(position: 'first' | 'last' = 'first', focus = true) {
	if (props.disabled || isOpen.value) return
	cancelLeave()
	await open()
	emit('open')
	if (!focus) return
	await nextTick()
	focusItem(position === 'first' ? 0 : getItems().length - 1)
}

function closeMenu(restoreFocus = false) {
	if (!isOpen.value) return
	reset()
	close(restoreFocus)
}

async function toggleMenu(event?: MouseEvent) {
	if (props.hoverable && window.matchMedia('(hover: hover)').matches && event?.detail) return
	if (isOpen.value) closeMenu()
	else await openMenu()
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
	event.preventDefault()
	openMenu(event.key === 'ArrowDown' ? 'first' : 'last')
}

function handleItemSelect(option: ButtonMenuAction | ButtonMenuLink) {
	emit('select', option)
	if (!option.remainOpen) closeMenu(!isLink(option)) // don't steal focus from a navigating link
}

function handleSubmenuSelect(option: ButtonMenuAction | ButtonMenuLink) {
	emit('select', option)
	if (!option.remainOpen) closeMenu()
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
})

const isClient = ref(false)
onMounted(() => {
	isClient.value = true
})

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

	<Teleport v-if="isClient" to="body">
		<ButtonMenuPanel
			ref="panel"
			:open="isOpen"
			:panel-id="menuId"
			:label="props.label"
			:panel-style="panelStyle"
			:side="resolvedSide"
			:anchor-style="anchorStyle"
			:origin="expandOrigin"
			@keydown="handleKeydown"
			@mouseenter="handleMouseEnter"
			@mouseleave="handleMouseLeave"
		>
			<template v-for="(option, index) in options" :key="option.id ?? `${option.type}-${index}`">
				<div v-if="isDivider(option)" role="separator" class="my-1 h-px bg-surface-5" />

				<div
					v-else-if="isHeading(option)"
					class="px-3 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary first:pt-1"
				>
					{{ option.label }}
				</div>

				<ButtonMenuSubmenu
					v-else-if="isSubmenu(option)"
					:option="option"
					@select="handleSubmenuSelect"
				>
					<template #trigger>
						<slot :name="option.id" :option="option">
							<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
							{{ option.label }}
						</slot>
					</template>
					<template #item="{ option: child }">
						<slot :name="child.id" :option="child">
							<component :is="child.icon" v-if="child.icon" aria-hidden="true" />
							{{ child.label }}
						</slot>
					</template>
				</ButtonMenuSubmenu>

				<ButtonMenuItem
					v-else
					:option="option"
					@select="handleItemSelect"
					@focus="focusedIndex = getItems().indexOf($event)"
				>
					<slot :name="option.id" :option="option">
						<component :is="option.icon" v-if="option.icon" aria-hidden="true" />
						{{ option.label }}
					</slot>
				</ButtonMenuItem>
			</template>
		</ButtonMenuPanel>
	</Teleport>
</template>
