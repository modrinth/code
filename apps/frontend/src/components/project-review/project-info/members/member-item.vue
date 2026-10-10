<template>
	<li class="group/member flex min-w-0 items-start gap-2">
		<button
			v-if="avatarUrl"
			type="button"
			class="shrink-0 cursor-zoom-in rounded-full border-0 bg-transparent p-0"
			:aria-label="
				formatMessage(messages.openMemberAvatar, {
					username: member.user.username,
				})
			"
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
		<ImageViewerEditor
			:key="member.user.id"
			ref="viewer"
			:items="viewerItems"
			editor="disabled"
			:pixelated="pixelated"
		>
			<template #actions="{ item }">
				<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" />
			</template>
		</ImageViewerEditor>
		<div class="min-w-0 flex-1">
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
			<ProjectStatusStats :stats="stats" />
		</div>
		<MemberActions
			:user="member.user"
			class="shrink-0 group-focus-within/member:opacity-100 group-hover/member:opacity-100 aria-expanded:opacity-100 [@media(hover:hover)]:opacity-0"
		/>
	</li>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon } from '@modrinth/assets'
import { Avatar, ImageViewerEditor, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ProjectStatusStats from '~/components/ProjectStatusStats.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import ImageViewerActions from '../../image-viewer-actions.vue'
import { projectReviewMessages as messages } from '../../messages'
import MemberActions from './member-actions.vue'

const props = defineProps<{
	member: Labrinth.Projects.v3.TeamMember
	stats: { status: string; count: number }[]
}>()
const { formatMessage } = useVIntl()
const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const { pixelated } = injectProjectReviewPageContext()
const avatarUrl = computed(() => props.member.user.raw_avatar_url || props.member.user.avatar_url)
const viewerItems = computed(() =>
	avatarUrl.value
		? [
				{
					id: avatarUrl.value,
					src: avatarUrl.value,
					alt: props.member.user.username,
				},
			]
		: [],
)
</script>
