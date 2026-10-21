<template>
	<ServerStatCard :title="title" :icon="icon">
		<template #value>
			<template v-if="offLabel">{{ offLabel }}</template>
			<template v-else>
				<span class="tabular-nums">{{ used }}</span>
				<span class="ml-1 whitespace-nowrap text-2xl font-semibold">
					{{ formatMessage(messages.slotLimit, { limit }) }}
				</span>
			</template>
		</template>

		<div class="flex flex-col gap-3">
			<div
				class="flex h-3 gap-1"
				role="progressbar"
				:aria-label="title"
				:aria-valuemin="0"
				:aria-valuemax="limit"
				:aria-valuenow="Math.min(used, limit)"
			>
				<div
					v-for="segment in limit"
					:key="segment"
					class="h-full flex-1 rounded-full"
					:class="[color === 'blue' ? 'bg-blue' : 'bg-purple', { 'opacity-30': segment > used }]"
				/>
				<div v-if="limit === 0" class="h-full flex-1 rounded-full bg-surface-5" />
			</div>

			<div class="flex flex-wrap items-center gap-2 font-medium text-secondary">
				<slot />
			</div>
		</div>
	</ServerStatCard>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

import ServerStatCard from '#ui/components/servers/ServerStatCard.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

defineProps<{
	title: string
	icon: Component
	used: number
	limit: number
	color: 'blue' | 'purple'
	offLabel?: string
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	slotLimit: {
		id: 'servers.backups.slots.limit',
		defaultMessage: '/ {limit, plural, one {# slot} other {# slots}}',
	},
})
</script>
