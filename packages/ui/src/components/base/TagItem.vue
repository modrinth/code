<template>
	<button v-if="action" :class="[baseClass, 'cursor-pointer hover:underline']" @click="action">
		<slot />
	</button>
	<div v-else :class="baseClass">
		<slot />
	</div>
</template>
<script setup lang="ts">
defineProps<{
	action?: (event: MouseEvent) => void
}>()

const baseClass =
	'text-nowrap leading-none opacity-90 text-secondary p-0 bg-transparent font-normal inline-flex items-center gap-1 [&>svg]:shrink-0 [&>svg]:h-4 [&>svg]:w-4'
</script>
