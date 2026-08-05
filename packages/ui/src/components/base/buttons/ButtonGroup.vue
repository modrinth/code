<script setup lang="ts">
withDefaults(
	defineProps<{
		label?: string
	}>(),
	{
		label: undefined,
	},
)
</script>

<template>
	<div
		:role="label ? 'group' : undefined"
		:aria-label="label"
		class="button-group inline-flex [&>[data-button]+[data-button]]:-ms-px [&>[data-button]:active]:z-10 [&>[data-button]:focus-visible]:z-10 [&>[data-button]:not(:disabled):not([aria-disabled=true]):hover]:z-10 [&>[data-button]:not(:first-child)]:rounded-l-none [&>[data-button]:not(:last-child)]:rounded-r-none"
	>
		<slot />
	</div>
</template>

<style scoped>
.button-group
	:deep([data-button].button-frame--colored:first-child:not(:last-child):not(:focus-visible)) {
	clip-path: inset(-4rem var(--button-group-inner-clip, 1px) -4rem -4rem);
}

.button-group
	:deep(
		[data-button].button-frame--colored:not(:first-child):not(:last-child):not(:focus-visible)
	) {
	clip-path: inset(
		-4rem var(--button-group-inner-clip, 1px) -4rem var(--button-group-leading-clip, 0px)
	);
}

.button-group
	:deep([data-button].button-frame--colored:last-child:not(:first-child):not(:focus-visible)) {
	clip-path: inset(-4rem -4rem -4rem var(--button-group-leading-clip, 0px));
}

:dir(rtl)
	.button-group
	:deep([data-button].button-frame--colored:first-child:not(:last-child):not(:focus-visible)) {
	clip-path: inset(-4rem -4rem -4rem var(--button-group-inner-clip, 1px));
}

:dir(rtl)
	.button-group
	:deep(
		[data-button].button-frame--colored:not(:first-child):not(:last-child):not(:focus-visible)
	) {
	clip-path: inset(
		-4rem var(--button-group-leading-clip, 0px) -4rem var(--button-group-inner-clip, 1px)
	);
}

:dir(rtl)
	.button-group
	:deep([data-button].button-frame--colored:last-child:not(:first-child):not(:focus-visible)) {
	clip-path: inset(-4rem var(--button-group-leading-clip, 0px) -4rem -4rem);
}

.button-group :deep([data-button].button-frame--colored:active) {
	--button-group-inner-clip: 0;
}

/*
 * the seam is normally drawn by the button on the right, but hovering over the left one will make it draw the seam instead. when pressed, make the right one draw its own seam again.
 */
.button-group
	:deep([data-button].button-frame--colored:not(:disabled):not([aria-disabled='true']):hover) {
	--button-group-inner-clip: 0;
}

.button-group
	:deep(
		[data-button].button-frame--colored:not(:disabled):not([aria-disabled='true']):hover:not(
				:active
			)
			+ [data-button]
	) {
	--button-group-leading-clip: 1px;
}
</style>
