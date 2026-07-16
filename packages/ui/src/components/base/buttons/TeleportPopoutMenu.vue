<script setup lang="ts">
import { computed, nextTick, ref, useId, watch } from 'vue'

import Button from './Button.vue'
import IconButton from './IconButton.vue'
import type {
	ButtonElementHandle,
	ButtonSize,
	ButtonTone,
	ButtonVariant,
	TeleportPlacement,
} from './types'
import { useAnchoredTeleport } from './useAnchoredTeleport'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		label: string
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		disabled?: boolean
		iconOnly?: boolean
		placement?: TeleportPlacement
		panelRole?: 'dialog' | 'region'
		focusOnOpen?: boolean
	}>(),
	{
		variant: 'base',
		size: 'default',
		disabled: false,
		iconOnly: false,
		placement: 'bottom-end',
		panelRole: 'dialog',
		focusOnOpen: true,
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
	if (props.focusOnOpen) await nextTick(focusPanel)
}

function closeMenu(restoreFocus = false) {
	if (!isOpen.value) return
	close(restoreFocus)
}

async function toggleMenu() {
	if (isOpen.value) closeMenu()
	else await openMenu()
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
		:label="props.iconOnly ? props.label : undefined"
		:aria-label="props.iconOnly ? undefined : props.label"
		:variant="props.variant"
		:tone="props.tone"
		:size="props.size"
		:disabled="props.disabled"
		:aria-expanded="isOpen"
		:aria-controls="panelId"
		aria-haspopup="dialog"
		@click="toggleMenu"
	>
		<slot name="trigger" />
	</component>

	<Teleport to="body">
		<Transition
			enter-active-class="transition duration-125 ease-out"
			enter-from-class="scale-95 opacity-0"
			enter-to-class="scale-100 opacity-100"
			leave-active-class="transition duration-100 ease-in"
			leave-from-class="scale-100 opacity-100"
			leave-to-class="scale-95 opacity-0"
		>
			<div
				v-if="isOpen"
				:id="panelId"
				ref="panelElement"
				class="fixed isolate z-[9999] rounded-[14px] bg-surface-3 p-4 text-primary shadow-lg ring-1 ring-surface-5"
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
