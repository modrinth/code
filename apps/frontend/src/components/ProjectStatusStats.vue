<template>
	<div class="mt-1 flex flex-wrap gap-x-2 gap-y-1 text-xs">
		<Tooltip
			v-for="stat in statusStats"
			:key="stat.status"
			:text="stat.tooltip"
			:aria-label="stat.tooltip"
			:panel-class="
				stat.status === 'account_locked'
					? 'max-w-[300px] whitespace-pre-line break-words text-left'
					: undefined
			"
			tabindex="0"
			class="flex items-center gap-0.5 font-medium tabular-nums"
			:class="stat.color"
		>
			<component :is="stat.icon" class="size-[12px] shrink-0" aria-hidden="true" />
			<span v-if="!stat.account" aria-hidden="true">{{ stat.count }}</span>
			<template v-if="stat.items.length" #popper>
				<div v-if="stat.status === 'account_locked'" class="flex flex-col gap-1.5 text-left">
					<div>Account locked</div>
					<div>Reason: {{ stat.items.join('; ') }}</div>
				</div>
				<div v-else class="text-left">
					<div>Account restricted</div>
					<ul class="m-0 mt-1 list-disc pl-4">
						<li v-for="(item, index) in stat.items" :key="index">{{ item }}</li>
					</ul>
				</div>
			</template>
		</Tooltip>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, CheckIcon, ShieldAlertIcon, UserLockIcon, UserRoundXIcon } from '@modrinth/assets'
import { defineMessages, PROJECT_STATUS_ICONS, Tooltip, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps<{
	stats: { status: string; count: number; details?: string | string[] }[]
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
const accountStatuses = {
	shared_instances_banned: { label: 'Banned from shared instances', icon: BanIcon },
	account_locked: { label: 'Account locked', icon: UserLockIcon },
	account_restricted: { label: 'Account restricted', icon: UserRoundXIcon },
}
const statusMessages = {
	tech_review_failed: messages.techReviewFailedCount,
	approved: messages.approvedCount,
	unlisted: messages.unlistedCount,
	private: messages.privateCount,
	withheld: messages.withheldCount,
	rejected: messages.rejectedCount,
	draft: messages.draftCount,
	processing: messages.processingCount,
	archived: messages.archivedCount,
	scheduled: messages.scheduledCount,
	unknown: messages.unknownCount,
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
const statusOrder = [
	'tech_review_failed',
	'shared_instances_banned',
	'account_restricted',
	'account_locked',
	'approved',
	'unlisted',
	'private',
	'withheld',
	'rejected',
	'draft',
	'processing',
	'archived',
	'scheduled',
	'unknown',
]
const statusStats = computed(() =>
	[
		...Object.entries(statusMessages)
			.map(([key, message]) => {
				const status = key as keyof typeof statusMessages
				const count = props.stats.find((stat) => stat.status === status)?.count ?? 0
				return {
					status,
					tooltip: formatMessage(message, { count }),
					account: false,
					items: [] as string[],
					count,
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
		...Object.entries(accountStatuses).flatMap(([status, { label, icon }]) => {
			const stat = props.stats.find((stat) => stat.status === status && stat.count > 0)
			const items =
				(status === 'account_restricted' || status === 'account_locked') && stat?.details
					? Array.isArray(stat.details)
						? stat.details
						: [stat.details]
					: []
			const details = Array.isArray(stat?.details) ? stat.details.join('; ') : stat?.details
			return stat
				? [
						{
							status,
							count: stat.count,
							icon,
							color: 'text-red',
							account: true,
							items,
							tooltip: details
								? status === 'account_locked'
									? `${label}\nReason: ${details}`
									: `${label}: ${details}`
								: label,
						},
					]
				: []
		}),
	].toSorted((a, b) => statusOrder.indexOf(a.status) - statusOrder.indexOf(b.status)),
)
</script>
