<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ShieldAlertIcon } from '@modrinth/assets'
import { capitalizeString } from '@modrinth/utils'
import { computed } from 'vue'

import { getHighestSeverity, getSeverityBadgeColor } from './helpers'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	details: Labrinth.TechReview.Internal.ReportIssueDetail[]
}>()

const { getDetailDecision } = injectTechReviewDecisions()

const highestSeverity = computed(() => getHighestSeverity(props.details))
const highestSeverityLabel = computed(() => capitalizeString(highestSeverity.value))
const severityBadgeClass = computed(() => getSeverityBadgeColor(highestSeverity.value))

const markedCount = computed(
	() =>
		props.details.filter((detail) => getDetailDecision(detail.id, detail.status) !== 'pending')
			.length,
)
const failedCount = computed(
	() =>
		props.details.filter((detail) => getDetailDecision(detail.id, detail.status) === 'malware')
			.length,
)
const allFlagsMarked = computed(
	() => props.details.length > 0 && markedCount.value === props.details.length,
)
</script>

<template>
	<div v-if="details.length > 0" class="flex items-center gap-2">
		<div
			class="rounded-full border-solid px-2.5 py-1"
			:class="[
				severityBadgeClass,
				{
					'!bg-transparent opacity-50': allFlagsMarked,
				},
			]"
		>
			<span class="text-sm font-medium">{{ highestSeverityLabel }}</span>
		</div>

		<div
			class="flex items-center gap-1 rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1 text-sm text-primary"
			:class="{
				'bg-transparent opacity-50': allFlagsMarked,
			}"
		>
			{{ markedCount }}/{{ details.length }} done
		</div>

		<div
			v-if="failedCount > 0"
			v-tooltip="`${failedCount} failed`"
			class="flex items-center gap-1 rounded-full border border-solid border-red bg-bg-red p-1 text-sm font-bold text-red"
		>
			<ShieldAlertIcon class="size-5" />
		</div>
	</div>
</template>
