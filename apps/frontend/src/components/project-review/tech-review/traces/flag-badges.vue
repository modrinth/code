<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ShieldAlertIcon } from '@modrinth/assets'
import { TagItem } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { computed } from 'vue'

import { getHighestSeverity, getSeverityBadgeColor } from '../helpers'
import { injectTechReviewDecisions } from '../verdict/use-decisions'

const props = defineProps<{
	details: Labrinth.TechReview.Internal.ReportIssueDetail[]
	hideSeverity?: boolean
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
		<TagItem
			v-if="!hideSeverity"
			:class="[
				severityBadgeClass,
				{
					'!bg-transparent opacity-50': allFlagsMarked,
				},
			]"
		>
			{{ highestSeverityLabel }}
		</TagItem>

		<div
			class="flex items-center gap-1 text-xs text-secondary"
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
