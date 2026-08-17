<template>
	<div
		v-bind="$attrs"
		class="group/input min-w-0 touch-manipulation border border-solid font-medium text-primary shadow-none transition-[background-color,border-color,box-shadow,color] focus-within:text-contrast focus-within:ring-4 focus-within:ring-brand-shadow"
		:class="[
			multiline ? 'flex w-full items-start gap-2 rounded-xl px-3 py-2' : sizeClass,
			appearanceClass,
			disabled ? 'cursor-not-allowed opacity-50' : '',
		]"
		:data-disabled="disabled || undefined"
		:data-invalid="invalid || undefined"
	>
		<slot name="leading" />
		<slot />
		<slot name="trailing" />
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { InputAppearance, InputSize } from './types'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
	defineProps<{
		appearance?: InputAppearance
		disabled?: boolean
		invalid?: boolean
		multiline?: boolean
		size?: InputSize
	}>(),
	{
		appearance: 'surface',
		disabled: false,
		invalid: false,
		multiline: false,
		size: 'standard',
	},
)

const sizeClass = computed(
	() =>
		({
			small: 'inline-flex h-8 items-center gap-1.5 rounded-xl px-3',
			standard: 'inline-flex h-9 items-center gap-2 rounded-xl px-3',
			medium: 'inline-flex h-10 items-center gap-2 rounded-[14px] px-4',
			large: 'inline-flex h-12 items-center gap-2 rounded-[14px] px-4',
		})[props.size],
)

const appearanceClass = computed(() => {
	if (props.invalid) return 'border-red bg-highlight-red'
	if (props.appearance === 'transparent') return 'border-transparent bg-transparent'
	if (props.appearance === 'button') return 'border-transparent bg-button-bg'
	return 'border-surface-5 bg-surface-4'
})
</script>
