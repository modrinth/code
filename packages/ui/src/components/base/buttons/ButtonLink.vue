<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, type RouteLocationRaw } from 'vue-router'

import ButtonFrame from './ButtonFrame.vue'
import type { ButtonSize, ButtonTone, ButtonVariant } from './types'

const props = withDefaults(
	defineProps<{
		to?: RouteLocationRaw
		href?: string
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		target?: string
		rel?: string
		download?: string | boolean
		disabled?: boolean
	}>(),
	{
		to: undefined,
		href: undefined,
		variant: 'base',
		size: 'default',
		target: undefined,
		rel: undefined,
		download: undefined,
		disabled: false,
	},
)

const usesRouter = computed(() => props.to !== undefined && !props.disabled)
const component = computed(() => (usesRouter.value ? RouterLink : 'a'))
const resolvedRel = computed(() => {
	if (props.rel) return props.rel
	return props.target === '_blank' ? 'noopener noreferrer' : undefined
})

function handleClick(event: MouseEvent) {
	if (!props.disabled) return
	event.preventDefault()
	event.stopImmediatePropagation()
}
</script>

<template>
	<ButtonFrame
		:as="component"
		:variant="props.variant"
		:tone="props.tone"
		:size="props.size"
		:to="usesRouter ? props.to : undefined"
		:href="!usesRouter && !props.disabled ? props.href : undefined"
		:target="props.target"
		:rel="resolvedRel"
		:download="!props.disabled ? props.download : undefined"
		:aria-disabled="props.disabled || undefined"
		:tabindex="props.disabled ? -1 : undefined"
		@click="handleClick"
	>
		<slot />
	</ButtonFrame>
</template>
