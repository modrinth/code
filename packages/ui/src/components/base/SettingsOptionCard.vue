<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
	title: string
	icon?: Component
	description?: string
}>()
</script>

<template>
	<div
		class="grid w-full grid-cols-[1fr_auto] items-center gap-6 p-4 rounded-2xl border border-solid border-surface-4 bg-surface-3"
	>
		<div>
			<h3 class="mb-1 mt-0 flex items-center gap-2 text-lg font-semibold text-contrast">
				<component :is="icon" v-if="icon" class="size-5 shrink-0 text-primary" aria-hidden="true" />
				{{ title }}
			</h3>
			<div
				v-if="description || $slots.default"
				class="flex flex-col gap-2 leading-normal [&>p]:m-0"
			>
				<p v-if="description">{{ description }}</p>
				<slot />
			</div>
		</div>
		<div>
			<slot name="actions" />
		</div>
	</div>
</template>
