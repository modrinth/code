<script setup lang="ts">
import { LockIcon, TriangleAlertIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'

import type { ContentEnabledForState, ContentSide } from '../types'

const messages = defineMessages({
	server: {
		id: 'content.enabled-for.server',
		defaultMessage: 'Server',
	},
	player: {
		id: 'content.enabled-for.player',
		defaultMessage: 'Player',
	},
	locked: {
		id: 'content.enabled-for.locked',
		defaultMessage: 'These choices are locked. Select Unlock from the row menu to change them.',
	},
	requiredHere: {
		id: 'content.enabled-for.required-here',
		defaultMessage: 'This content needs to be enabled here to work correctly.',
	},
	notSupportedHere: {
		id: 'content.enabled-for.not-supported-here',
		defaultMessage: "This content isn't designed to be enabled here.",
	},
})

const props = withDefaults(
	defineProps<{
		modelValue: ContentEnabledForState
		disabled?: boolean
		disabledTooltip?: string | null
	}>(),
	{
		disabled: false,
		disabledTooltip: undefined,
	},
)

const emit = defineEmits<{
	'update:model-value': [side: ContentSide, value: boolean]
}>()

const { formatMessage } = useVIntl()

const sides: ContentSide[] = ['server', 'player']
const disabledSides = computed(() => new Set(props.modelValue.disabledSides ?? []))

function isSelected(side: ContentSide) {
	return props.modelValue[side]
}

function isDisabled(side: ContentSide) {
	return props.disabled || disabledSides.value.has(side)
}

function getDisabledTooltip(side: ContentSide) {
	if (props.disabled) return props.disabledTooltip
	if (!disabledSides.value.has(side)) return undefined
	return formatMessage(isSelected(side) ? messages.requiredHere : messages.notSupportedHere)
}

function toggle(side: ContentSide) {
	if (isDisabled(side)) return
	emit('update:model-value', side, !isSelected(side))
}
</script>

<template>
	<div class="flex w-[200px] shrink-0 items-center gap-1.5">
		<button
			v-for="side in sides"
			:key="side"
			v-tooltip="getDisabledTooltip(side)"
			type="button"
			class="flex h-8 items-center rounded-xl border border-solid px-3 text-sm font-medium transition-[background-color,border-color,color,opacity,transform] duration-100 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow"
			:class="[
				isSelected(side)
					? 'border-brand bg-brand-highlight text-brand'
					: 'border-surface-5 bg-transparent text-primary hover:bg-surface-3',
				isDisabled(side) ? 'cursor-not-allowed opacity-50' : 'cursor-pointer active:scale-[0.97]',
			]"
			:aria-pressed="isSelected(side)"
			:aria-disabled="isDisabled(side)"
			@click="toggle(side)"
		>
			{{ formatMessage(messages[side]) }}
		</button>

		<span
			v-if="modelValue.locked && disabledSides.size > 0"
			v-tooltip="formatMessage(messages.locked)"
			class="inline-flex size-5 shrink-0 cursor-help items-center justify-center text-secondary"
			tabindex="0"
		>
			<LockIcon class="size-5" />
		</span>

		<span
			v-if="modelValue.warningTooltip"
			v-tooltip="modelValue.warningTooltip"
			class="inline-flex size-5 shrink-0 cursor-help items-center justify-center"
			tabindex="0"
		>
			<TriangleAlertIcon class="size-5 text-orange" />
		</span>
	</div>
</template>
