<script setup lang="ts">
import type { Component, CSSProperties } from 'vue'
import { computed, ref } from 'vue'

import type { ButtonSize, ButtonTone, ButtonVariant } from './types'

const baseClasses = [
	'relative inline-flex min-w-0 shrink-0 touch-manipulation items-center justify-center',
	'whitespace-nowrap border-0 no-underline',
	'cursor-pointer select-none transition-[background-color,color,box-shadow,filter,opacity,transform] duration-150 ease-out',
	'hover:brightness-[--hover-brightness] focus-visible:brightness-[--hover-brightness]',
	'focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow',
	'enabled:active:scale-[0.97] disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50',
	'[&[aria-disabled=true]]:pointer-events-none [&[aria-disabled=true]]:cursor-not-allowed [&[aria-disabled=true]]:opacity-50',
].join(' ')

const sizeClasses: Record<ButtonSize, string> = {
	sm: 'h-6 gap-1 rounded-lg px-1.5 text-sm font-semibold leading-5 [&>svg]:size-4 [&>svg]:min-h-4 [&>svg]:min-w-4 [&>svg]:shrink-0',
	default:
		'h-9 gap-1.5 rounded-xl px-2.5 text-base font-semibold leading-5 [&>svg]:size-5 [&>svg]:min-h-5 [&>svg]:min-w-5 [&>svg]:shrink-0',
	md: 'h-10 gap-2 rounded-[14px] px-4 text-base font-semibold leading-5 [&>svg]:size-5 [&>svg]:min-h-5 [&>svg]:min-w-5 [&>svg]:shrink-0',
	lg: 'h-12 gap-2 rounded-2xl px-3.5 text-base font-extrabold leading-5 [&>svg]:size-6 [&>svg]:min-h-6 [&>svg]:min-w-6 [&>svg]:shrink-0',
}

const iconOnlySizeClasses: Record<ButtonSize, string> = {
	sm: 'w-6 px-0',
	default: 'w-9 px-0',
	md: 'w-10 px-0',
	lg: 'w-12 px-0',
}

const variantClasses: Record<ButtonVariant, string> = {
	base: 'button-frame--base bg-surface-4 text-contrast [&>svg]:text-primary',
	colored: 'button-frame--colored bg-[--button-tone] text-[rgba(0,0,0,0.9)] [&>svg]:text-inherit',
	outlined: 'button-frame--outlined bg-transparent text-contrast [&>svg]:text-primary',
	quiet:
		'button-frame--quiet bg-transparent hover:bg-surface-4 focus-visible:bg-surface-4 [&>svg]:text-inherit',
}

const toneVariables: Record<ButtonTone, string> = {
	brand: 'var(--color-brand)',
	red: 'var(--color-red)',
	orange: 'var(--color-orange)',
	green: 'var(--color-green)',
	blue: 'var(--color-blue)',
	purple: 'var(--color-purple)',
	promotion: 'var(--medal-promotion-text-orange, var(--color-orange))',
}

const props = withDefaults(
	defineProps<{
		as: string | Component
		variant?: ButtonVariant
		tone?: ButtonTone
		size?: ButtonSize
		iconOnly?: boolean
	}>(),
	{
		variant: 'base',
		size: 'default',
		iconOnly: false,
	},
)

const element = ref<HTMLElement | null>(null)
const classes = computed(() => [
	baseClasses,
	variantClasses[props.variant],
	sizeClasses[props.size],
	props.iconOnly ? iconOnlySizeClasses[props.size] : '',
])
const style = computed((): CSSProperties | undefined => {
	if (props.variant === 'quiet' && !props.tone) return undefined
	if (props.variant !== 'colored' && props.variant !== 'quiet') return undefined

	return {
		'--button-tone': toneVariables[props.tone ?? 'brand'],
	} as CSSProperties
})

defineExpose({ element })
</script>

<template>
	<component :is="as" ref="element" data-button :class="classes" :style="style">
		<slot />
	</component>
</template>

<style scoped>
.button-frame--base {
	box-shadow:
		inset 0 0 0 1px var(--surface-5),
		0 1px 1px rgba(0, 0, 0, 0.12);
}

.button-frame--colored {
	box-shadow:
		0 0 0 1px color-mix(in srgb, var(--button-tone) 30%, transparent),
		0 2px 4px rgba(0, 0, 0, 0.04),
		0 5px 8px rgba(0, 0, 0, 0.04),
		0 10px 18px rgba(0, 0, 0, 0.03),
		0 24px 48px rgba(0, 0, 0, 0.03);
}

.button-frame--colored::before {
	position: absolute;
	inset: 0;
	padding: 1px;
	pointer-events: none;
	content: '';
	border-radius: inherit;
	background: linear-gradient(180deg, rgba(255, 255, 255, 0.3), rgba(255, 255, 255, 0));
	-webkit-mask:
		linear-gradient(#000 0 0) content-box,
		linear-gradient(#000 0 0);
	-webkit-mask-composite: xor;
	mask-composite: exclude;
}

.button-frame--outlined {
	box-shadow: inset 0 0 0 1px var(--surface-5);
}

.button-frame--quiet {
	color: var(--button-tone, var(--color-base));
}
</style>
