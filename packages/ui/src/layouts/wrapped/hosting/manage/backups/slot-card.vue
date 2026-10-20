<template>
	<div
		class="flex flex-col gap-4 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-5"
	>
		<div class="flex items-start justify-between gap-4">
			<div class="flex min-w-0 flex-col gap-2">
				<span class="text-lg font-medium text-secondary">{{ title }}</span>
				<span v-if="offLabel" class="text-3xl font-semibold text-contrast">{{ offLabel }}</span>
				<span v-else class="text-2xl font-semibold text-contrast">
					<IntlFormatted :message-id="messages.slotsUsed" :values="{ used, limit }">
						<template #used="{ children }">
							<span class="text-4xl font-bold tabular-nums">
								<component :is="() => children" />
							</span>
						</template>
					</IntlFormatted>
				</span>
			</div>
			<component :is="icon" class="size-8 shrink-0 text-secondary" aria-hidden="true" />
		</div>

		<div
			class="flex h-2 gap-1.5"
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
			<div v-if="limit === 0" class="h-full flex-1 rounded-full bg-surface-4" />
		</div>

		<div class="flex flex-wrap items-center gap-2 font-medium text-secondary">
			<slot />
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'

import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { defineMessages } from '#ui/composables/i18n'

defineProps<{
	title: string
	icon: Component
	used: number
	limit: number
	color: 'blue' | 'purple'
	offLabel?: string
}>()

const messages = defineMessages({
	slotsUsed: {
		id: 'servers.backups.slots.used',
		defaultMessage: '<used>{used}</used> / {limit, plural, one {# slot} other {# slots}}',
	},
})
</script>
