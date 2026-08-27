<template>
	<AutoLink
		v-if="!loading && !notFound"
		:to="to"
		target="_blank"
		class="flex items-center gap-3 p-3 rounded-2xl border border-solid border-surface-4 bg-surface-3 no-underline hover:bg-surface-4 transition-colors max-w-full"
	>
		<Avatar :src="iconUrl" size="48px" />
		<div class="flex flex-col gap-1 min-w-0">
			<span class="text-contrast font-semibold truncate">{{ title }}</span>
			<span v-if="description" class="text-sm text-secondary line-clamp-1">{{ description }}</span>
		</div>
		<span v-if="stat" class="ml-auto text-sm text-secondary shrink-0 pl-2">{{ stat }}</span>
	</AutoLink>
	<div
		v-else-if="loading"
		class="flex items-center gap-3 p-3 rounded-2xl border border-solid border-surface-4 bg-surface-3"
	>
		<div class="size-[48px] rounded-full bg-surface-4 animate-pulse shrink-0"></div>
		<div class="flex flex-col gap-2">
			<div class="w-32 h-4 rounded-full bg-surface-4 animate-pulse"></div>
			<div class="w-48 h-3 rounded-full bg-surface-4 animate-pulse"></div>
		</div>
	</div>
	<div
		v-else
		class="flex items-center gap-3 p-3 rounded-2xl border border-solid border-surface-4 bg-surface-3 text-secondary"
	>
		{{ notFoundMessage }}
	</div>
</template>

<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from '../AutoLink.vue'
import Avatar from '../Avatar.vue'

defineProps<{
	to: string | RouteLocationRaw
	iconUrl?: string | null
	title?: string
	description?: string | null
	stat?: string
	loading?: boolean
	notFound?: boolean
	notFoundMessage: string
}>()
</script>
