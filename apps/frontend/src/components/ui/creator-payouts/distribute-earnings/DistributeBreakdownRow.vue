<template>
	<div class="mb-4 flex items-start justify-between gap-6 last:mb-0">
		<div class="flex min-w-0 flex-col gap-1">
			<span
				class="font-base text-base"
				:class="strong ? 'font-medium text-contrast' : 'text-primary'"
			>
				{{ label }}
			</span>
			<span v-if="description" class="text-sm text-secondary">
				{{ description }}
			</span>
		</div>
		<span
			class="font-base shrink-0 text-right text-base"
			:class="[
				value === emptyValue
					? 'text-primary opacity-60'
					: strong
						? 'text-lg font-medium text-contrast'
						: valueToneClass,
			]"
		>
			{{ value }}
		</span>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
	label: string
	value: string
	description?: string
	negative?: boolean
	strong?: boolean
	tone?: 'positive' | 'negative' | 'neutral'
}>()

const emptyValue = '—'
const valueToneClass = computed(() => {
	if (props.tone === 'positive') {
		return 'text-green'
	}

	if (props.tone === 'negative' || props.negative) {
		return 'text-red'
	}

	return 'text-primary'
})
</script>
