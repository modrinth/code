<script setup lang="ts">
defineProps<{
	open: boolean
}>()
</script>

<template>
	<div
		class="fixed bottom-[calc(var(--size-mobile-navbar-height)-var(--size-rounded-card))] left-0 z-[11] box-border w-full rounded-t-2xl border border-b-0 border-solid border-surface-5 bg-surface-3 pb-[--size-rounded-card] transition-[transform,box-shadow] duration-[400ms] ease-[cubic-bezier(0.54,0.84,0.42,1)]"
		:class="
			open
				? 'translate-y-0 shadow-[0_0_20px_2px_rgba(0,0,0,0.3)]'
				: 'translate-y-[calc(100%+env(safe-area-inset-bottom))]'
		"
	>
		<slot />
	</div>
</template>
