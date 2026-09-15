<script setup lang="ts">
import type { Placement } from '@floating-ui/vue'
import { computed, nextTick, ref, useId, useTemplateRef } from 'vue'

import FloatingMenu from '../../floating/FloatingMenu.vue'
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

const menu = useTemplateRef<{ show: () => void; hide: () => void }>('menu')
const triggerButton = ref<ButtonElementHandle | null>(null)
const panelElement = ref<HTMLElement | null>(null)
const isOpen = ref(false)
const panelId = `button-popout-${useId()}`
const triggerComponent = computed(() => (props.iconOnly ? IconButton : Button))
const menuPlacement = computed(() => props.placement.replace(/-center$/, '') as Placement)

function onOpen() {
	isOpen.value = true
	emit('open')
	if (props.autoFocus) {
		nextTick(focusPanel)
	}
}

function onClose() {
	isOpen.value = false
	emit('close')
}

function onPanelKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape') {
		return
	}
	event.preventDefault()
	nextTick(() => triggerButton.value?.element?.focus())
}

function focusPanel() {
	const focusable = panelElement.value?.querySelector<HTMLElement>(
		'button:not([disabled]), a[href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
	)
	;(focusable ?? panelElement.value)?.focus()
}

defineExpose({
	open: () => menu.value?.show(),
	close: () => menu.value?.hide(),
})
</script>

<template>
	<FloatingMenu
		ref="menu"
		bare
		:class="$attrs.class"
		:placement="menuPlacement"
		:disabled="disabled"
		panel-class="rounded-[14px] bg-surface-3 text-primary shadow-lg ring-1 ring-surface-5"
		@open="onOpen"
		@close="onClose"
	>
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
		>
			<slot name="trigger" />
		</component>
		<template #popper="{ hide }">
			<div
				:id="panelId"
				ref="panelElement"
				class="p-4"
				:role="props.panelRole"
				:aria-label="props.label"
				tabindex="-1"
				@keydown="onPanelKeydown"
			>
				<slot name="panel" :close="hide" />
			</div>
		</template>
	</FloatingMenu>
</template>
