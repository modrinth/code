<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { computed, ref, toRef, useId } from 'vue'

import { useAnchoredTeleport } from '../../../../utils/use-anchored-teleport'
import type {
	ButtonMenuAction,
	ButtonMenuLink,
	ButtonMenuSubmenu,
	TeleportPlacement,
} from '../types'
import {
	buttonMenuItemClasses,
	getButtonMenuItemAttrs,
	isDivider,
	isHeading,
	menuItemSelector,
	menuPanelPadding,
	submenuGap,
	useButtonMenuNavigation,
	useHoverIntent,
	visibleOptions,
} from './button-menu'
import ButtonMenuItem from './ButtonMenuItem.vue'
import ButtonMenuPanel from './ButtonMenuPanel.vue'

const HOVER_CLOSE_DELAY = 200

const props = withDefaults(
	defineProps<{
		option: ButtonMenuSubmenu
		placement?: TeleportPlacement
		distance?: number
	}>(),
	{
		placement: 'right-start',
		distance: submenuGap,
	},
)

const emit = defineEmits<{
	select: [option: ButtonMenuAction | ButtonMenuLink]
}>()

const triggerElement = ref<HTMLElement | null>(null)
const panel = ref<InstanceType<typeof ButtonMenuPanel> | null>(null)
const panelElement = computed(() => panel.value?.element ?? null)
const resolvedPlacement = toRef(props, 'placement')
const resolvedDistance = toRef(props, 'distance')
const alignOffset = ref(-menuPanelPadding)
const panelId = `button-menu-submenu-${useId()}`

const options = computed(() => visibleOptions(props.option.options))
const triggerAttrs = computed(() => getButtonMenuItemAttrs(props.option))

const { isOpen, panelStyle, resolvedSide, expandOrigin, open, close } = useAnchoredTeleport(
	triggerElement,
	panelElement,
	resolvedPlacement,
	resolvedDistance,
	alignOffset,
)

const bridge = computed(() => ({ side: resolvedSide.value, size: resolvedDistance.value }))

const { focusItem, handleNavigationKeydown } = useButtonMenuNavigation(
	panelElement,
	menuItemSelector,
)

const { handleMouseEnter, handleMouseLeave, cancelLeave } = useHoverIntent({
	closeDelay: HOVER_CLOSE_DELAY,
	onEnter: () => openSubmenu(false),
	onLeave: () => closeSubmenu(),
})

async function openSubmenu(focus = true) {
	cancelLeave()
	if (props.option.disabled) return
	if (!isOpen.value) await open()
	if (focus) focusItem(0)
}

function closeSubmenu(restoreFocus = false) {
	cancelLeave()
	close(restoreFocus)
}

function handleTriggerClick(event: MouseEvent) {
	if (event.detail && window.matchMedia('(hover: hover)').matches) return
	toggleSubmenu()
}

function toggleSubmenu() {
	if (isOpen.value) closeSubmenu()
	else openSubmenu()
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (event.key === 'ArrowRight') {
		event.preventDefault()
		openSubmenu()
		return
	}
	if (event.key === 'ArrowLeft' && isOpen.value) {
		event.preventDefault()
		closeSubmenu(true)
		return
	}
	if (event.key !== 'Enter' && event.key !== ' ') return
	event.preventDefault()
	toggleSubmenu()
}

function handleSelect(option: ButtonMenuAction | ButtonMenuLink) {
	emit('select', option)
	if (!option.remainOpen) closeSubmenu()
}

function handlePanelKeydown(event: KeyboardEvent) {
	if (handleNavigationKeydown(event)) {
		event.stopPropagation() // parent menu is listening too
		return
	}

	if (event.key !== 'Escape' && event.key !== 'ArrowLeft') return
	event.preventDefault()
	event.stopPropagation()
	closeSubmenu(true)
}
</script>

<template>
	<button
		ref="triggerElement"
		v-tooltip="props.option.tooltip"
		v-bind="triggerAttrs"
		type="button"
		:aria-disabled="props.option.disabled || undefined"
		:aria-expanded="isOpen"
		:aria-controls="panelId"
		aria-haspopup="menu"
		:class="buttonMenuItemClasses"
		@click="handleTriggerClick"
		@keydown="handleTriggerKeydown"
		@mouseenter="handleMouseEnter"
		@mouseleave="handleMouseLeave"
	>
		<slot name="trigger" :option="props.option">
			<component :is="props.option.icon" v-if="props.option.icon" aria-hidden="true" />
			{{ props.option.label }}
		</slot>
		<ChevronRightIcon aria-hidden="true" class="ml-auto !text-secondary" />
	</button>

	<ButtonMenuPanel
		ref="panel"
		:open="isOpen"
		:panel-id="panelId"
		:label="props.option.label"
		:panel-style="panelStyle"
		:side="resolvedSide"
		:origin="expandOrigin"
		expand="horizontal"
		:bridge="bridge"
		@keydown="handlePanelKeydown"
		@mouseenter="handleMouseEnter"
		@mouseleave="handleMouseLeave"
	>
		<template v-for="(child, index) in options" :key="child.id ?? `${child.type}-${index}`">
			<div v-if="isDivider(child)" role="separator" class="my-1 h-px bg-surface-5" />

			<div
				v-else-if="isHeading(child)"
				class="px-3 pb-1 pt-2 text-xs font-bold uppercase tracking-wide text-secondary first:pt-1"
			>
				{{ child.label }}
			</div>

			<ButtonMenuItem v-else :option="child" submenu-item @select="handleSelect">
				<slot name="item" :option="child">
					<component :is="child.icon" v-if="child.icon" aria-hidden="true" />
					{{ child.label }}
				</slot>
			</ButtonMenuItem>
		</template>
	</ButtonMenuPanel>
</template>
