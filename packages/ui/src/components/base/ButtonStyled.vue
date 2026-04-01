<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple' | 'medal-promo'
		customColor?: string
		size?: 'standard' | 'large' | 'small'
		circular?: boolean
		type?: 'standard' | 'outlined' | 'transparent' | 'highlight' | 'highlight-colored-text' | 'chip'
		colorFill?: 'auto' | 'background' | 'text' | 'none'
		hoverColorFill?: 'auto' | 'background' | 'text' | 'none'
		highlightedStyle?: 'main-nav-primary' | 'main-nav-secondary'
		highlighted?: boolean
	}>(),
	{
		color: 'standard',
		size: 'standard',
		circular: false,
		type: 'standard',
		colorFill: 'auto',
		hoverColorFill: 'auto',
		highlightedStyle: 'main-nav-primary',
		highlighted: false,
	},
)

const highlightedColorVar = computed(() => {
	if (props.customColor) {
		return null
	}
	switch (props.color) {
		case 'brand':
			return 'var(--color-brand-highlight)'
		case 'red':
			return 'var(--color-red-highlight)'
		case 'orange':
			return 'var(--color-orange-highlight)'
		case 'green':
			return 'var(--color-green-highlight)'
		case 'medal-promo':
		case 'blue':
			return 'var(--color-blue-highlight)'
		case 'purple':
			return 'var(--color-purple-highlight)'
		case 'standard':
		default:
			return null
	}
})

const colorVar = computed(() => {
	if (props.customColor) {
		return props.customColor
	}
	switch (props.color) {
		case 'brand':
			return 'var(--color-brand)'
		case 'red':
			return 'var(--color-red)'
		case 'orange':
			return 'var(--color-orange)'
		case 'green':
			return 'var(--color-green)'
		case 'blue':
			return 'var(--color-blue)'
		case 'purple':
			return 'var(--color-purple)'
		case 'medal-promo':
			return 'var(--medal-promotion-text-orange)'
		case 'standard':
		default:
			return null
	}
})

const height = computed(() => {
	if (props.size === 'large') {
		return '2.5rem'
	} else if (props.size === 'small') {
		return '1.5rem'
	}
	return '2.25rem'
})

const width = computed(() => {
	if (props.size === 'large') {
		return props.circular ? '2.5rem' : 'auto'
	} else if (props.size === 'small') {
		return props.circular ? '1.5rem' : 'auto'
	}
	return props.circular ? '2.25rem' : 'auto'
})

const paddingX = computed(() => {
	let padding = props.circular ? '0.5rem' : '1.25rem'
	if (props.size === 'large') {
		padding = props.circular ? '0.5rem' : '1.25rem'
	} else if (props.size === 'small') {
		padding = props.circular ? '0.125rem' : '0.5rem'
	}
	return `calc(${padding} - 0.125rem)`
})

const paddingY = computed(() => {
	if (props.size === 'large') {
		return '0.5rem'
	}
	return '0.5rem'
})

const gap = computed(() => {
	if (props.size === 'large') {
		return '0.5rem'
	} else if (props.size === 'small') {
		return '0.25rem'
	}
	return '0.375rem'
})

const fontWeight = computed(() => {
	if (props.size === 'large') {
		return '800'
	}
	return '600'
})

const radius = computed(() => {
	return '5px'
})

const iconSize = computed(() => {
	if (props.size === 'large') {
		return '1.25rem'
	} else if (props.size === 'small') {
		return '1rem'
	}
	return '1.25rem'
})

function setColorFill(
	colors: { bg: string; text: string },
	fill: 'background' | 'text' | 'none',
): { bg: string; text: string } {
	if (colorVar.value) {
		if (fill === 'background') {
			if (props.type === 'highlight' && highlightedColorVar.value) {
				colors.bg = highlightedColorVar.value
				colors.text = 'var(--color-contrast)'
			} else if (props.type === 'highlight-colored-text' && highlightedColorVar.value) {
				colors.bg = highlightedColorVar.value
				colors.text = colorVar.value
			} else {
				colors.bg = colorVar.value
				colors.text = 'var(--color-accent-contrast)'
			}
		} else if (fill === 'text') {
			colors.text = colorVar.value
		}
	}
	return colors
}

const colorVariables = computed(() => {
	const defaultShadow =
		props.type === 'standard' || props.type === 'highlight' || props.highlighted
			? 'var(--shadow-button)'
			: 'none'

	if (props.highlighted) {
		const colors = {
			bg:
				props.highlightedStyle === 'main-nav-primary'
					? 'var(--color-button-bg-selected)'
					: 'var(--color-button-bg)',
			text:
				props.highlightedStyle === 'main-nav-primary'
					? 'var(--color-button-text-selected)'
					: 'var(--color-contrast)',
			icon:
				props.type === 'chip'
					? 'var(--color-contrast)'
					: props.highlightedStyle === 'main-nav-primary'
						? 'var(--color-button-text-selected)'
						: 'var(--color-contrast)',
		}
		const hoverColors = JSON.parse(JSON.stringify(colors))
		const boxShadow =
			props.type === 'chip' && colorVar.value ? `0 0 0 2px ${colorVar.value}` : defaultShadow
		return `--_bg: ${colors.bg}; --_text: ${colors.text}; --_icon: ${colors.icon}; --_hover-bg: ${hoverColors.bg}; --_hover-text: ${hoverColors.text}; --_hover-icon: ${hoverColors.icon}; --_box-shadow: ${boxShadow};`
	}

	let colors = {
		bg: 'var(--color-button-bg)',
		text: 'var(--color-base)',
	}
	let hoverColors = JSON.parse(JSON.stringify(colors))

	if (props.type === 'outlined') {
		hoverColors.bg = 'transparent'
	}

	if (props.type === 'outlined') {
		colors.bg = 'transparent'
		colors = setColorFill(colors, props.colorFill === 'auto' ? 'text' : props.colorFill)
		hoverColors = setColorFill(
			hoverColors,
			props.hoverColorFill === 'auto' ? 'text' : props.hoverColorFill,
		)
	} else if (props.type === 'chip') {
		// Chip type uses highlight-colored-text styling when colored
		if (colorVar.value && highlightedColorVar.value) {
			colors.bg = highlightedColorVar.value
			colors.text = colorVar.value
			hoverColors.bg = highlightedColorVar.value
			hoverColors.text = colorVar.value
		}
	} else {
		colors = setColorFill(colors, props.colorFill === 'auto' ? 'background' : props.colorFill)
		hoverColors = setColorFill(
			hoverColors,
			props.hoverColorFill === 'auto' ? 'background' : props.hoverColorFill,
		)
	}

	const boxShadow =
		props.type === 'chip' && colorVar.value ? `0 0 0 2px ${colorVar.value}` : defaultShadow
	return `--_bg: ${colors.bg}; --_text: ${colors.text}; --_hover-bg: ${hoverColors.bg}; --_hover-text: ${hoverColors.text}; --_box-shadow: ${boxShadow};`
})

const fontSize = computed(() => {
	if (props.size === 'small') {
		return 'text-sm'
	}
	return 'text-base'
})
</script>

<template>
	<div
		class="btn-wrapper"
		:class="[{ outline: type === 'outlined', chip: type === 'chip' }, fontSize]"
		:style="`${colorVariables}--_height:${height};--_width:${width};--_radius: ${radius};--_padding-x:${paddingX};--_padding-y:${paddingY};--_gap:${gap};--_font-weight:${fontWeight};--_icon-size:${iconSize};`"
	>
		<slot />
	</div>
</template>

<style scoped lang="scss">
.btn-wrapper {
	display: contents;
}

/* Searches up to 4 children deep for valid button */
.btn-wrapper :deep(:is(button, a, .button-like):first-child),
.btn-wrapper :slotted(:is(button, a, .button-like):first-child),
.btn-wrapper :slotted(*) > :is(button, a, .button-like):first-child,
.btn-wrapper :slotted(*) > *:first-child > :is(button, a, .button-like):first-child,
.btn-wrapper
	:slotted(*)
	> *:first-child
	> *:first-child
	> :is(button, a, .button-like):first-child {
	@apply flex flex-row items-center justify-center border-solid border border-[rgba(0,0,0,0.2)] bg-[--_bg] text-[--_text] h-[--_height] min-w-[--_width] rounded-[4px] px-[--_padding-x] py-[--_padding-y] gap-[--_gap] font-[--_font-weight] whitespace-nowrap;
	position: relative;

	svg:first-child {
		color: var(--_icon, var(--_text));
		flex-shrink: 0;
	}

	&::before {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(
			to bottom,
			rgba(255, 255, 255, 0.8),
			rgba(0, 0, 0, 0),
			rgba(0, 0, 0, 0.3)
		);
		border-radius: 4px;
	}

	&[disabled],
	&[disabled='true'],
	&.disabled,
	&.looks-disabled {
		@apply opacity-50;
	}

	&:not([disabled]):not([disabled='true']):not(.disabled) {
		@apply hover:brightness-[--hover-brightness] focus-visible:brightness-[--hover-brightness] hover:bg-[--_hover-bg] hover:text-[--_hover-text] focus-visible:bg-[--_hover-bg] focus-visible:text-[--_hover-text];

		&:hover svg:first-child,
		&:focus-visible svg:first-child {
			color: var(--_hover-icon, var(--_hover-text));
		}
	}
}

/*noinspection CssUnresolvedCustomProperty*/
.btn-wrapper :deep(:is(button, a, .button-like):first-child) > svg:first-child,
.btn-wrapper :slotted(:is(button, a, .button-like):first-child) > svg:first-child,
.btn-wrapper :slotted(*) > :is(button, a, .button-like):first-child > svg:first-child,
.btn-wrapper
	:slotted(*)
	> *:first-child
	> :is(button, a, .button-like):first-child
	> svg:first-child,
.btn-wrapper
	:slotted(*)
	> *:first-child
	> *:first-child
	> :is(button, a, .button-like):first-child
	> svg:first-child {
	min-width: var(--_icon-size, 1rem);
	min-height: var(--_icon-size, 1rem);
}

.joined-buttons {
	display: flex;
	gap: 1px;

	> .btn-wrapper:not(:first-child) {
		:deep(:is(button, a, .button-like):first-child),
		:slotted(:is(button, a, .button-like):first-child),
		:slotted(*) > :is(button, a, .button-like):first-child,
		:slotted(*) > *:first-child > :is(button, a, .button-like):first-child,
		:slotted(*) > *:first-child > *:first-child > :is(button, a, .button-like):first-child {
			border-top-left-radius: 0;
			border-bottom-left-radius: 0;
		}
	}

	> :not(:last-child) {
		:deep(:is(button, a, .button-like):first-child),
		:slotted(:is(button, a, .button-like):first-child),
		:slotted(*) > :is(button, a, .button-like):first-child,
		:slotted(*) > *:first-child > :is(button, a, .button-like):first-child,
		:slotted(*) > *:first-child > *:first-child > :is(button, a, .button-like):first-child {
			border-top-right-radius: 0;
			border-bottom-right-radius: 0;
		}
	}
}

/* guys, I know this is nuts, I know */
</style>
