<template>
	<RestrictUserModal
		v-if="userQuery.data.value"
		ref="restrictModal"
		:user="userQuery.data.value"
		:user-id="user.id"
	/>
	<div class="flex shrink-0 gap-0">
		<IconButton
			v-if="techReviewMode && isStaffViewing && user.role === 'developer'"
			v-tooltip="restrictionLabel"
			:label="restrictionLabel"
			type="quiet"
			color="red"
			:size="size"
			class="shrink-0"
			:class="{
				'group-focus-within/member:opacity-100 group-hover/member:opacity-100 [@media(hover:hover)]:opacity-0':
					hoverOnly,
			}"
			:loading="userQuery.isFetching.value"
			@click="openRestrictions"
		>
			<CircleSlashIcon aria-hidden="true" />
		</IconButton>
		<MemberActions
			:user="user"
			:size="size"
			class="shrink-0"
			:class="{
				'group-focus-within/member:opacity-100 group-hover/member:opacity-100 aria-expanded:opacity-100 [@media(hover:hover)]:opacity-0':
					hoverOnly,
			}"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CircleSlashIcon } from '@modrinth/assets'
import { IconButton, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import RestrictUserModal from '@modrinth/ui/src/layouts/shared/user-profile/components/restrict-user-modal.vue'
import { useQuery } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import { useAuthState } from '~/composables/auth'
import { isStaff } from '~/helpers/users.js'
import { injectReviewSubmission } from '~/providers/project-review/review-submission'

import MemberActions from './member-actions.vue'

const props = withDefaults(
	defineProps<{ user: Labrinth.Users.v3.User; hoverOnly?: boolean; size?: 'xs' | 'sm' }>(),
	{ size: 'sm' },
)

const { techReviewMode } = injectReviewSubmission()
const auth = useAuthState()
const isStaffViewing = computed(() => isStaff(auth.value.user))
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const restrictModal = ref<InstanceType<typeof RestrictUserModal>>()
const restrictionsActivated = ref(false)
const userQuery = useQuery({
	queryKey: computed(() => ['user', props.user.id]),
	queryFn: () => client.labrinth.users_v3.get(props.user.id),
	enabled: restrictionsActivated,
	staleTime: 30_000,
})
const restrictionLabel = computed(() =>
	(userQuery.data.value ?? props.user).restriction
		? `Edit restrictions for ${props.user.username}`
		: `Restrict ${props.user.username}`,
)

async function openRestrictions() {
	if (!techReviewMode.value || !isStaffViewing.value || userQuery.isFetching.value) return
	const id = props.user.id
	restrictionsActivated.value = true
	const result = await userQuery.refetch()
	if (props.user.id !== id || !techReviewMode.value) return
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
</script>
