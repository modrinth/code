<template>
	<button
		class="group flex w-full enabled:hover:cursor-pointer items-center gap-3 rounded-[20px] p-3 text-left transition-[background-color,box-shadow,filter,opacity,transform] duration-150 ease-out enabled:hover:brightness-[--hover-brightness] enabled:active:scale-[0.98] focus-visible:outline-none enabled:focus-visible:brightness-[--hover-brightness] enabled:focus-visible:ring-4 enabled:focus-visible:ring-brand-shadow disabled:cursor-not-allowed disabled:opacity-50 border-none"
		:class="
			selected
				? 'bg-brand-highlight shadow-[inset_0_0_0_1px_var(--color-brand),0_1px_1px_rgba(0,0,0,0.12)]'
				: 'bg-surface-4 shadow-[inset_0_0_0_1px_var(--surface-5),0_1px_1px_rgba(0,0,0,0.12)]'
		"
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
