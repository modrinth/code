<template>
	<button
		class="flex w-full hover:cursor-pointer items-center gap-3 rounded-[20px] border border-solid p-3 text-left transition-all hover:brightness-110 brightness-90 active:scale-[0.98]"
		:class="selected ? 'border-brand bg-brand-highlight' : 'border-surface-5 bg-surface-4'"
		@click="$emit('click')"
	>
		<div
			class="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border border-solid"
			:class="selected ? 'border-brand' : 'border-surface-5'"
		>
			<component
				:is="icon"
				class="size-8 text-secondary"
				:class="selected ? '!stroke-brand' : ''"
				stroke-width="1.5"
			/>
		</div>
		<div class="flex flex-1 flex-col gap-1">
			<span class="text-base font-semibold text-contrast">{{ title }}</span>
			<span class="text-sm font-medium text-primary">{{ description }}</span>
		</div>
		<ChevronRightIcon class="size-5 shrink-0 text-secondary" />
	</button>
</template>

<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import type { Component } from 'vue'

defineProps<{
	icon: Component
	title: string
	description: string
	selected?: boolean
}>()

defineEmits<{
	(e: 'click'): void
}>()
</script>
