<template>
	<div class="smart-clickable" :class="{ 'smart-clickable--has-clickable': !!$slots.clickable }">
		<slot name="clickable" />
		<div
			v-bind="$attrs"
			class="smart-clickable__contents"
			:class="{
				'pointer-events-none': !!$slots.clickable,
			}"
		>
			<slot />
		</div>
	</div>
</template>

<script setup lang="ts">
defineOptions({
	inheritAttrs: false,
})
</script>
<style scoped lang="scss">
.smart-clickable {
	display: grid;

	> * {
		grid-area: 1 / 1;
	}

	.smart-clickable__contents {
		// Utility classes for contents
		:deep(.smart-clickable\:allow-pointer-events) {
			pointer-events: all;
		}

		:deep(.ease-brightness) {
			opacity: 1;
			transition: opacity 0.125s ease-out;
		}
	}
}

// Only apply effects when a clickable is present
.smart-clickable.smart-clickable--has-clickable {
	// Setup base styles for contents
	.smart-clickable__contents {
		transition: scale 0.125s ease-out;
	}

	// When clickable is being hovered or focus-visible, give contents an effect
	:first-child:hover + .smart-clickable__contents,
	:first-child:focus-visible + .smart-clickable__contents {
		// Utility classes for contents
		:deep(.smart-clickable\:underline-on-hover) {
			text-decoration: underline;
		}
		:deep(.smart-clickable\:highlight-on-hover) {
			filter: brightness(var(--hover-brightness, 1.25));
		}
		:deep(.smart-clickable\:surface-4-on-hover) {
			@apply bg-surface-4;
		}
		:deep(.smart-clickable\:surface-5-on-hover) {
			@apply bg-surface-5;
		}
		:deep(.ease-brightness) {
			opacity: 0.85;
			transition: opacity 0.125s ease-out;
		}
	}

	:first-child:focus-visible + .smart-clickable__contents {
		// Utility classes for contents
		:deep(.smart-clickable\:outline-on-focus) {
			outline: 0.25rem solid var(--color-focus-ring);
		}
	}

	// When clickable is being clicked, give contents an effect
	:first-child:active + .smart-clickable__contents {
		scale: 0.97;
	}
}
</style>
