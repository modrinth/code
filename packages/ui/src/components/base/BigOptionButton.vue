<template>
	<button
		class="group flex w-full enabled:hover:cursor-pointer items-center gap-3 rounded-[20px] p-3 text-left transition-all enabled:hover:brightness-110 enabled:active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50 border-none"
		:class="selected ? 'bg-brand-highlight' : 'bg-surface-4'"
		:disabled="disabled"
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
		<ChevronRightIcon
			class="size-5 shrink-0 text-secondary opacity-0 transition-opacity duration-100"
			:class="{ 'group-hover:opacity-100': !disabled }"
		/>
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
	disabled?: boolean
}>()

defineEmits<{
	(e: 'click'): void
}>()
</script>
