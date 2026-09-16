<template>
	<li class="flex min-w-0 items-start gap-2.5">
		<Avatar
			:src="member.user.avatar_url"
			:alt="member.user.username"
			size="1.75rem"
			circle
			no-shadow
		/>
		<div class="min-w-0">
			<div class="flex items-center gap-1.5">
				<NuxtLink
					:to="`/user/${member.user.username}`"
					target="_blank"
					class="truncate font-semibold text-contrast"
					>{{ member.user.username }}</NuxtLink
				>
				<CrownIcon
					v-if="member.is_owner"
					class="size-3 shrink-0"
					:aria-label="formatMessage(messages.owner)"
				/>
			</div>
			<p class="mb-1 mt-0.5 text-xs">{{ member.role }}</p>
			<div class="flex flex-wrap gap-x-2 gap-y-1 text-xs">
				<span
					v-for="stat in visibleStats"
					:key="stat.status"
					:class="{ 'text-red': stat.status === 'rejected' }"
					>{{ formatMessage(stat.message, { count: stat.count }) }}</span
				>
			</div>
		</div>
	</li>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon } from '@modrinth/assets'
import { Avatar, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { projectReviewMessages as messages } from '../../messages'

const props = defineProps<{
	member: Labrinth.Projects.v3.TeamMember
	stats: { status: string; count: number }[]
}>()
const { formatMessage } = useVIntl()
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
}
const visibleStats = computed(() =>
	props.stats.flatMap((stat) => {
		const message = statusMessages[stat.status as keyof typeof statusMessages]
		return message && stat.count ? [{ ...stat, message }] : []
	}),
)
</script>
