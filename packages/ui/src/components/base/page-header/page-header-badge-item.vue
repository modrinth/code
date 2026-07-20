<template>
	<AutoLink
		v-if="to && !disabled"
		v-tooltip="tooltip"
		:to="to"
		:aria-label="ariaLabel"
		:link-class="badgeClassString"
		v-bind="$attrs"
	>
		<component
			:is="icon"
			v-if="icon"
			:class="iconClass ?? defaultIconClass"
			aria-hidden="true"
			v-bind="iconProps"
		/>
		<slot />
	</AutoLink>
	<button
		v-else-if="action || disabled"
		v-tooltip="tooltip"
		type="button"
		:disabled="disabled"
		:aria-label="ariaLabel"
		:class="badgeClass"
		v-bind="$attrs"
		@click="handleClick"
	>
		<component
			:is="icon"
			v-if="icon"
			:class="iconClass ?? defaultIconClass"
			aria-hidden="true"
			v-bind="iconProps"
		/>
		<slot />
	</button>
	<div v-else v-tooltip="tooltip" :aria-label="ariaLabel" :class="badgeClass" v-bind="$attrs">
		<component
			:is="icon"
			v-if="icon"
			:class="iconClass ?? defaultIconClass"
			aria-hidden="true"
			v-bind="iconProps"
		/>
		<slot />
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AutoLink from '../AutoLink.vue'
import type { PageHeaderIconProps, PageHeaderInteractiveProps } from './types'

defineOptions({
	inheritAttrs: false,
})

const props = withDefaults(defineProps<PageHeaderIconProps & PageHeaderInteractiveProps>(), {
	icon: undefined,
	iconProps: undefined,
	iconClass: undefined,
	tooltip: undefined,
	ariaLabel: undefined,
	to: undefined,
	action: undefined,
	disabled: false,
})

const badgeClass = computed(() => [
	'inline-flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-button-bg px-2 py-1 text-sm font-semibold leading-none text-secondary text-nowrap',
	props.to || props.action ? 'm-0 cursor-pointer hover:underline' : '',
	props.disabled ? 'cursor-not-allowed opacity-60' : '',
])
const badgeClassString = computed(() => badgeClass.value.filter(Boolean).join(' '))
const defaultIconClass = 'block size-4 shrink-0 text-current'

function handleClick(event: MouseEvent) {
	void props.action?.(event)
}
</script>
