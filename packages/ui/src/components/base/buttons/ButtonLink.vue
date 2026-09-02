<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

import ButtonFrame from './ButtonFrame.vue'
import type {
	ButtonColor,
	ButtonInteraction,
	ButtonLinkDestination,
	ButtonSize,
	ButtonType,
} from './types'

type ButtonLinkProps = ButtonLinkDestination & {
	type?: ButtonType
	color?: ButtonColor
	size?: ButtonSize
	interaction?: ButtonInteraction
	target?: string
	rel?: string
	download?: string | boolean
	disabled?: boolean
}

const props = withDefaults(defineProps<ButtonLinkProps>(), {
	type: 'base',
	size: 'md',
	target: undefined,
	rel: undefined,
	download: undefined,
	disabled: false,
})

const usesRouter = computed(() => props.to !== undefined && !props.disabled)
const component = computed(() => (usesRouter.value ? RouterLink : 'a'))
const resolvedRel = computed(() => {
	if (props.rel) return props.rel
	return props.target === '_blank' ? 'noopener noreferrer' : undefined
})

// https://github.com/vuejs/vue-router/issues/2005 moment
const linkAttrs = computed(() =>
	usesRouter.value ? { to: props.to } : !props.disabled ? { href: props.href } : {},
)

function handleClick(event: MouseEvent) {
	if (!props.disabled) return
	event.preventDefault()
	event.stopImmediatePropagation()
}
</script>

<template>
	<ButtonFrame
		:as="component"
		:type="props.type"
		:color="props.color"
		:size="props.size"
		:interaction="props.interaction"
		v-bind="linkAttrs"
		:target="props.target"
		:rel="resolvedRel"
		:download="!props.disabled ? props.download : undefined"
		:aria-disabled="props.disabled || undefined"
		:role="props.disabled ? 'link' : undefined"
		:tabindex="props.disabled ? -1 : undefined"
		@click="handleClick"
	>
		<slot />
	</ButtonFrame>
</template>
