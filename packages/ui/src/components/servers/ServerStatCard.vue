<script setup lang="ts">
import type { Component } from 'vue'
import { type RouteLocationRaw, RouterLink } from 'vue-router'

defineProps<{
	title: string
	icon: Component
	value?: string
	secondary?: string | null
	to?: RouteLocationRaw | null
}>()
</script>

<template>
	<component
		:is="to ? RouterLink : 'div'"
		:to="to ?? undefined"
		class="@container relative isolate min-h-[145px] w-full overflow-hidden rounded-[20px] border border-solid border-surface-4 bg-surface-3 p-5"
		:class="
			to
				? 'cursor-pointer transition-transform duration-100 hover:brightness-125 active:scale-95'
				: ''
		"
	>
		<div class="relative z-10 flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="stat-drop-shadow flex items-center gap-2 text-lg font-medium text-primary">
					{{ title }}
				</span>
				<component :is="icon" class="stat-drop-shadow relative z-10 size-6" aria-hidden="true" />
			</div>
			<span class="stat-drop-shadow text-3xl font-bold text-contrast @[13rem]:text-4xl">
				<slot name="value">
					<span class="flex flex-wrap items-baseline gap-x-1">
						<span class="whitespace-nowrap">{{ value }}</span>
						<span
							v-if="secondary"
							class="stat-drop-shadow whitespace-nowrap text-sm font-normal text-secondary"
						>
							{{ secondary }}
						</span>
					</span>
				</slot>
			</span>
			<slot />
		</div>
		<slot name="background" />
	</component>
</template>

<style scoped>
.stat-drop-shadow {
	filter: drop-shadow(0 4px 6px var(--surface-3));
}
</style>
