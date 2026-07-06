<template>
	<AutoLink
		v-if="to && !disabled"
		v-tooltip="tooltip"
		:to="to"
		:aria-label="ariaLabel"
		:class="resolvedClass"
		:style="style"
		@click="emit('click', $event)"
	>
		<slot />
	</AutoLink>
	<button
		v-else-if="isInteractive"
		v-tooltip="tooltip"
		type="button"
		:disabled="disabled"
		:aria-label="ariaLabel"
		:class="resolvedClass"
		:style="style"
		@click="emit('click', $event)"
	>
		<slot />
	</button>
	<div v-else v-tooltip="tooltip" :class="resolvedClass" :style="style">
		<slot />
	</div>
</template>

<script setup lang="ts">
import type { StyleValue } from 'vue'
import { computed } from 'vue'

import AutoLink from '../AutoLink.vue'
import type { PageHeaderClass, PageHeaderTarget } from './types'

const props = withDefaults(
	defineProps<{
		to?: PageHeaderTarget
		clickable?: boolean
		disabled?: boolean
		tooltip?: string
		ariaLabel?: string
		baseClass: PageHeaderClass
		interactiveClass?: PageHeaderClass
		passiveClass?: PageHeaderClass
		disabledClass?: PageHeaderClass
		style?: StyleValue
	}>(),
	{
		to: undefined,
		clickable: false,
		disabled: false,
		tooltip: undefined,
		ariaLabel: undefined,
		interactiveClass: undefined,
		passiveClass: undefined,
		disabledClass: 'cursor-not-allowed opacity-60',
		style: undefined,
	},
)

const emit = defineEmits<{
	click: [event: MouseEvent]
}>()

const isInteractive = computed(() => !!props.to || props.clickable || props.disabled)

const resolvedClass = computed(() => [
	props.baseClass,
	isInteractive.value ? props.interactiveClass : props.passiveClass,
	props.disabled ? props.disabledClass : '',
])
</script>
