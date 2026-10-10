<template>
	<div class="mt-1 flex flex-wrap gap-x-2 gap-y-1 text-xs">
		<Tooltip
			v-for="stat in statusStats"
			:key="stat.status"
			:text="formatMessage(stat.message, { count: stat.count })"
			:aria-label="formatMessage(stat.message, { count: stat.count })"
			class="flex items-center gap-0.5 font-medium tabular-nums"
			:class="stat.color"
		>
			<component :is="stat.icon" class="size-[12px] shrink-0" aria-hidden="true" />
			<span aria-hidden="true">{{ stat.count }}</span>
		</Tooltip>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, ShieldAlertIcon } from '@modrinth/assets'
import { defineMessages, PROJECT_STATUS_ICONS, Tooltip, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{
	stats: { status: string; count: number }[]
}>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	approvedCount: {
		id: 'moderation.project-review.approvedCount',
		defaultMessage: '{count} approved',
	},
	archivedCount: {
		id: 'moderation.project-review.archivedCount',
		defaultMessage: '{count} archived',
	},
	unlistedCount: {
		id: 'moderation.project-review.unlistedCount',
		defaultMessage: '{count} unlisted',
	},
	withheldCount: {
		id: 'moderation.project-review.withheldCount',
		defaultMessage: '{count} withheld',
	},
	processingCount: {
		id: 'moderation.project-review.processingCount',
		defaultMessage: '{count} under review',
	},
	draftCount: {
		id: 'moderation.project-review.draftCount',
		defaultMessage: '{count} draft',
	},
	rejectedCount: {
		id: 'moderation.project-review.rejectedCount',
		defaultMessage: '{count} rejected',
	},
	techReviewFailedCount: {
		id: 'moderation.project-review.techReviewFailedCount',
		defaultMessage: '{count} failed technical review',
	},
	privateCount: {
		id: 'moderation.project-review.privateCount',
		defaultMessage: '{count} private',
	},
	scheduledCount: {
		id: 'moderation.project-review.scheduledCount',
		defaultMessage: '{count} scheduled',
	},
	unknownCount: {
		id: 'moderation.project-review.unknownCount',
		defaultMessage: '{count} unknown',
	},
})
const statusMessages = {
	approved: messages.approvedCount,
	archived: messages.archivedCount,
	unlisted: messages.unlistedCount,
	withheld: messages.withheldCount,
	processing: messages.processingCount,
	draft: messages.draftCount,
	rejected: messages.rejectedCount,
	private: messages.privateCount,
	scheduled: messages.scheduledCount,
	unknown: messages.unknownCount,
	tech_review_failed: messages.techReviewFailedCount,
}
const statusColors = {
	approved: 'text-green',
	archived: 'text-purple',
	unlisted: 'text-purple',
	withheld: 'text-red',
	processing: 'text-orange',
	draft: 'text-blue',
	rejected: 'text-red',
	private: 'text-purple',
	scheduled: 'text-orange',
	unknown: 'text-orange',
	tech_review_failed: 'text-red',
}
const statusStats = computed(() =>
	Object.entries(statusMessages)
		.map(([key, message]) => {
			const status = key as keyof typeof statusMessages
			return {
				status,
				message,
				count: props.stats.find((stat) => stat.status === status)?.count ?? 0,
				icon:
					status === 'tech_review_failed'
						? ShieldAlertIcon
						: status === 'approved'
							? CheckIcon
							: PROJECT_STATUS_ICONS[status],
				color: statusColors[status],
			}
		})
		.filter((stat) => stat.count > 0),
)
</script>
