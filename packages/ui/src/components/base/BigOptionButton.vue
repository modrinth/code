<template>
	<button
		class="flex w-full items-center gap-3 rounded-[20px] border-none bg-surface-4 p-3 text-left outline-none transition-all hover:brightness-110 active:scale-[0.98]"
		@click="$emit('click')"
	>
		<div
			class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border border-solid border-surface-5"
		>
			<component :is="icon" class="h-7 w-7 text-secondary" stroke-width="1.5" />
		</div>
		<div class="flex flex-1 flex-col gap-1">
			<span class="text-base font-semibold text-contrast">{{ title }}</span>
			<span class="text-sm font-medium text-primary">{{ description }}</span>
		</div>
	</button>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
	icon: Component
	title: string
	description: string
}>()

defineEmits<{
	(e: 'click'): void
}>()
</script>
