<template>
	<li class="group/member flex min-w-0 items-start gap-2">
		<RestrictUserModal
			v-if="userQuery.data.value"
			ref="restrictModal"
			:user="userQuery.data.value"
			:user-id="member.user.id"
		/>
		<button
			v-if="avatarUrl"
			type="button"
			class="shrink-0 cursor-zoom-in rounded-full border-0 bg-transparent p-0"
			:aria-label="`View enlarged avatar for ${member.user.username}`"
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
			<template #actions="{ item, metadata }">
				<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" :metadata="metadata" />
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
				<CrownIcon v-if="member.is_owner" class="size-3 shrink-0" aria-label="Owner" />
			</div>
			<p class="m-0 text-xs">{{ member.role }}</p>
			<ProjectStatusStats :stats="stats" />
		</div>
		<div class="flex shrink-0 gap-0">
			<IconButton
				v-if="techReviewMode && isStaffViewing && member.user.role === 'developer'"
				v-tooltip="restrictionLabel"
				:label="restrictionLabel"
				type="quiet"
				color="red"
				size="sm"
				class="shrink-0 group-focus-within/member:opacity-100 group-hover/member:opacity-100 [@media(hover:hover)]:opacity-0"
				:loading="userQuery.isFetching.value"
				@click="openRestrictions"
			>
				<ShieldAlertIcon aria-hidden="true" />
			</IconButton>
			<MemberActions
				:user="member.user"
				class="shrink-0 group-focus-within/member:opacity-100 group-hover/member:opacity-100 aria-expanded:opacity-100 [@media(hover:hover)]:opacity-0"
			/>
		</div>
	</li>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon, ShieldAlertIcon } from '@modrinth/assets'
import {
	Avatar,
	IconButton,
	ImageViewerEditor,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import RestrictUserModal from '@modrinth/ui/src/layouts/shared/user-profile/components/restrict-user-modal.vue'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import ProjectStatusStats from '~/components/ProjectStatusStats.vue'
import { useAuthState } from '~/composables/auth'
import { isStaff } from '~/helpers/users.js'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import ImageViewerActions from '../../image-viewer-actions.vue'
import MemberActions from './member-actions.vue'

const props = defineProps<{
	member: Labrinth.Projects.v3.TeamMember
	stats: { status: string; count: number }[]
}>()

const viewer = ref<InstanceType<typeof ImageViewerEditor>>()
const { pixelated } = injectProjectReviewPageContext()
const { techReviewMode } = injectReviewSubmission()
const auth = useAuthState()
const isStaffViewing = computed(() => isStaff(auth.value.user))
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const restrictModal = ref<InstanceType<typeof RestrictUserModal>>()
const restrictionsActivated = ref(false)
const userQuery = useQuery({
	queryKey: computed(() => ['user', props.member.user.id]),
	queryFn: () => client.labrinth.users_v3.get(props.member.user.id),
	enabled: restrictionsActivated,
	staleTime: 30_000,
})
const restrictionLabel = computed(() =>
	(userQuery.data.value ?? props.member.user).restriction
		? `Edit restrictions for ${props.member.user.username}`
		: `Restrict ${props.member.user.username}`,
)

async function openRestrictions() {
	if (!techReviewMode.value || !isStaffViewing.value || userQuery.isFetching.value) return
	const id = props.member.user.id
	restrictionsActivated.value = true
	const result = await userQuery.refetch()
	if (props.member.user.id !== id || !techReviewMode.value) return
	if (result.isError || !result.data) {
		addNotification({
			title: 'Could not load user restrictions',
			text: 'Try opening the restriction dialog again.',
			type: 'error',
		})
		return
	}
	await nextTick()
	restrictModal.value?.show()
}

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
