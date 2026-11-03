<template>
	<div class="flex flex-col gap-2">
		<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
			<span class="font-semibold text-contrast">
				{{ formatMessage(messages.storageTitle) }}
			</span>
			<span
				class="font-semibold tabular-nums"
				:class="overLimit ? 'text-red' : 'text-contrast'"
				aria-live="polite"
			>
				{{
					formatMessage(messages.storageUsage, {
						used: formatBytes(used, 1),
						limit: formatBytes(limit, 0),
					})
				}}
			</span>
		</div>
		<div
			role="meter"
			:aria-label="formatMessage(messages.storageTitle)"
			:aria-valuenow="used"
			:aria-valuemin="0"
			:aria-valuemax="limit"
			class="relative h-3.5 overflow-hidden rounded-full bg-surface-4"
		>
			<div
				class="absolute inset-y-0 left-0 bg-gradient-to-b from-white/10 to-black/10 motion-safe:transition-[width,background-color] motion-safe:duration-300"
				:style="{ width: `${withinLimitPercent}%`, backgroundColor: barColor }"
			/>
			<div
				v-if="overLimit"
				class="absolute inset-y-0 right-0 bg-red bg-gradient-to-b from-white/10 to-black/10 motion-safe:transition-[left] motion-safe:duration-300"
				:style="{ left: `calc(${limitMarkerPercent}% + 2px)` }"
			/>
			<div
				v-if="overLimit"
				class="absolute inset-y-0 w-0 border-0 border-l-2 border-dotted border-contrast"
				:style="{ left: `${limitMarkerPercent}%` }"
				aria-hidden="true"
			/>
		</div>
		<p v-if="overLimit" class="m-0 text-sm text-red">
			{{ formatMessage(messages.storageOverLimit, { size: formatBytes(used - limit, 1) }) }}
		</p>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useFormatBytes } from '#ui/composables/format-bytes'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

const WARNING_RATIO = 0.75

const props = defineProps<{
	used: number
	limit: number
}>()

const { formatMessage } = useVIntl()
const formatBytes = useFormatBytes()

const overLimit = computed(() => props.used > props.limit)
const barScale = computed(() => Math.max(props.used, props.limit))
const withinLimitPercent = computed(
	() => (Math.min(props.used, props.limit) / barScale.value) * 100,
)
const limitMarkerPercent = computed(() => (props.limit / barScale.value) * 100)
const barColor = computed(() => {
	const ratio = Math.min(props.used / props.limit, 1)
	if (ratio <= WARNING_RATIO) {
		const orangeWeight = (ratio / WARNING_RATIO) * 100
		return `color-mix(in oklch, var(--color-green), var(--color-orange) ${orangeWeight}%)`
	}
	const redWeight = ((ratio - WARNING_RATIO) / (1 - WARNING_RATIO)) * 100
	return `color-mix(in oklch, var(--color-orange), var(--color-red) ${redWeight}%)`
})

const messages = defineMessages({
	storageTitle: {
		id: 'instance.shared-instance.publish-review.storage-title',
		defaultMessage: 'Storage allowance',
	},
	storageUsage: {
		id: 'instance.shared-instance.publish-review.storage-usage',
		defaultMessage: '{used} of {limit}',
	},
	storageOverLimit: {
		id: 'instance.shared-instance.publish-review.storage-over-limit',
		defaultMessage: '{size} over the limit. Deselect some files to push this update.',
	},
})
</script>
