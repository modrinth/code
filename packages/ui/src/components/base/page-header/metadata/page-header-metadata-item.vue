<template>
	<div :class="rootClass" data-page-header-metadata-item v-bind="$attrs">
		<span
			class="page-header-metadata-item-divider absolute right-full flex h-full w-[1.625rem] items-center justify-center"
		>
			<BulletDivider class="shrink-0" />
		</span>
		<AutoLink
			v-if="to && !disabled"
			v-tooltip="tooltip"
			:to="to"
			:aria-label="ariaLabel"
			:link-class="contentClassString"
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
			:class="contentClass"
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
		<div v-else v-tooltip="tooltip" :aria-label="ariaLabel" :class="contentClass">
			<component
				:is="icon"
				v-if="icon"
				:class="iconClass ?? defaultIconClass"
				aria-hidden="true"
				v-bind="iconProps"
			/>
			<slot />
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AutoLink from '../../AutoLink.vue'
import BulletDivider from '../../BulletDivider.vue'
import type { PageHeaderMetadataItemProps } from '../types'

defineOptions({
	inheritAttrs: false,
})

const props = withDefaults(defineProps<PageHeaderMetadataItemProps>(), {
	icon: undefined,
	iconProps: undefined,
	iconClass: undefined,
	tooltip: undefined,
	ariaLabel: undefined,
	to: undefined,
	action: undefined,
	disabled: false,
})

const defaultIconClass = 'block size-5 shrink-0 text-current'
const baseClass =
	'relative flex min-w-0 items-center font-medium leading-none text-secondary text-nowrap'
const contentBaseClass = 'inline-flex min-w-0 items-center gap-2 text-inherit'
const interactiveClass = 'm-0 cursor-pointer border-0 bg-transparent p-0 hover:underline'

const rootClass = computed(() => [baseClass, props.disabled ? 'cursor-not-allowed opacity-60' : ''])
const contentClass = computed(() => [
	contentBaseClass,
	props.to || props.action ? interactiveClass : '',
])
const contentClassString = computed(() => contentClass.value.filter(Boolean).join(' '))

function handleClick(event: MouseEvent) {
	void props.action?.(event)
}
</script>
