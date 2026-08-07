<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'

import { useAnchoredTeleport } from '../../../utils/use-anchored-teleport'
import Button from './Button.vue'
import IconButton from './IconButton.vue'
import type {
	ButtonColor,
	ButtonElementHandle,
	ButtonInteraction,
	ButtonSize,
	ButtonType,
	TeleportPlacement,
} from './types'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		label: string
		type?: ButtonType
		color?: ButtonColor
		size?: ButtonSize
		interaction?: ButtonInteraction
		disabled?: boolean
		iconOnly?: boolean
		tooltip?: string
		autoFocus?: boolean
		placement?: TeleportPlacement
		panelRole?: 'dialog' | 'region'
	}>(),
	{
		type: 'base',
		size: 'md',
		disabled: false,
		iconOnly: false,
		autoFocus: true,
		placement: 'bottom-end',
		panelRole: 'dialog',
	},
)

const emit = defineEmits<{
	open: []
	close: []
}>()

const triggerButton = ref<ButtonElementHandle | null>(null)
const triggerElement = computed(() => triggerButton.value?.element ?? null)
const panelElement = ref<HTMLElement | null>(null)
const resolvedPlacement = computed(() => props.placement)
const panelId = `button-popout-${useId()}`
const triggerComponent = computed(() => (props.iconOnly ? IconButton : Button))

const { isOpen, panelStyle, open, close } = useAnchoredTeleport(
	triggerElement,
	panelElement,
	resolvedPlacement,
)

function focusPanel() {
	const focusable = panelElement.value?.querySelector<HTMLElement>(
		'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
	)
	;(focusable ?? panelElement.value)?.focus()
}

async function openMenu() {
	if (props.disabled || isOpen.value) return
	await open()
	emit('open')
	if (props.autoFocus) await nextTick(focusPanel)
}

function closeMenu(restoreFocus = true) {
	if (!isOpen.value) return
	close(restoreFocus)
}

async function toggleMenu() {
	if (isOpen.value) closeMenu()
	else await openMenu()
}

function handleTriggerKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape' || !isOpen.value) return
	event.preventDefault()
	closeMenu()
}

function handlePanelKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape') return
	event.preventDefault()
	closeMenu(true)
}

watch(isOpen, (openState, previousOpenState) => {
	if (!openState && previousOpenState) emit('close')
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
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:interaction="props.interaction"
		:disabled="props.disabled"
		:aria-expanded="isOpen"
		:aria-controls="panelId"
		:aria-haspopup="props.panelRole === 'dialog' ? 'dialog' : undefined"
		@click="toggleMenu"
		@keydown="handleTriggerKeydown"
	>
		<slot name="trigger" />
	</component>

	<Teleport to="body">
		<Transition name="floating-expand">
			<div
				v-if="isOpen"
				:id="panelId"
				ref="panelElement"
				class="fixed isolate z-[9999] overflow-y-auto rounded-[14px] bg-surface-3 p-4 text-primary shadow-lg ring-1 ring-surface-5"
				:style="panelStyle"
				:role="props.panelRole"
				:aria-label="props.label"
				tabindex="-1"
				@keydown="handlePanelKeydown"
			>
				<slot name="panel" :close="closeMenu" />
			</div>
		</Transition>
	</Teleport>
</template>
