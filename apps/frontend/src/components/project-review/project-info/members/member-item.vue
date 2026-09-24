<template>
	<li class="flex min-w-0 items-start gap-2">
		<button
			v-if="avatarUrl"
			type="button"
			class="shrink-0 cursor-zoom-in rounded-full border-0 bg-transparent p-0"
			:aria-label="formatMessage(messages.openMemberAvatar, { username: member.user.username })"
			@click="viewer?.show(0)"
		>
			<Avatar
				:src="member.user.avatar_url || avatarUrl"
				:alt="member.user.username"
				size="2rem"
				circle
				no-shadow
			/>
		</button>
		<Avatar v-else :alt="member.user.username" size="2rem" circle no-shadow />
		<ImageViewerEditor :key="member.user.id" ref="viewer" :items="viewerItems" editor="disabled">
			<template #actions="{ item }">
				<ButtonLink
					v-tooltip="formatMessage(messages.openImageInNewTab)"
					type="quiet"
					class="!w-9 !rounded-full !p-0"
					:aria-label="formatMessage(messages.openImageInNewTab)"
					:href="item.src"
					target="_blank"
				>
					<ExternalIcon aria-hidden="true" />
				</ButtonLink>
			</template>
		</ImageViewerEditor>
		<div class="min-w-0">
			<div class="flex items-center gap-1.5">
				<NuxtLink
					:to="`/user/${member.user.username}`"
					target="_blank"
					class="truncate font-medium text-primary hover:underline"
					>{{ member.user.username }}</NuxtLink
				>
				<CrownIcon
					v-if="member.is_owner"
					class="size-3 shrink-0"
					:aria-label="formatMessage(messages.owner)"
				/>
			</div>
			<p class="m-0 text-xs">{{ member.role }}</p>
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
		</div>
	</li>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, CrownIcon, ExternalIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonLink,
	ImageViewerEditor,
	PROJECT_STATUS_ICONS,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { projectReviewMessages as messages } from '../../messages'

const props = defineProps<{
	member: Labrinth.Projects.v3.TeamMember
	stats: { status: string; count: number }[]
}>()
const { formatMessage } = useVIntl()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const avatarUrl = computed(() => props.member.user.raw_avatar_url || props.member.user.avatar_url)
const viewerItems = computed(() =>
	avatarUrl.value
		? [{ id: avatarUrl.value, src: avatarUrl.value, alt: props.member.user.username }]
		: [],
)
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
}
const statusStats = computed(() =>
	Object.entries(statusMessages)
		.map(([key, message]) => {
			const status = key as keyof typeof statusMessages
			return {
				status,
				message,
				count: props.stats.find((stat) => stat.status === status)?.count ?? 0,
				icon: status === 'approved' ? CheckIcon : PROJECT_STATUS_ICONS[status],
				color: statusColors[status],
			}
		})
		.filter((stat) => stat.count > 0),
)
</script>
