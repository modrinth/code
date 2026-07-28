<template>
	<div class="w-full px-2 pt-2">
		<UserProfilePageLayout
			:user-id="userId"
			:project-type="projectType"
			variant="app"
			site-url="https://modrinth.com"
			project-link-mode="app"
			:edit-profile-link="openProfileSettings"
			external-navigation
		/>
	</div>
</template>

<script setup lang="ts">
import { provideUserProfile, UserProfilePageLayout } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, watch } from 'vue'
import { onBeforeRouteUpdate, useRoute } from 'vue-router'

import {
	block_user,
	get_blocked_users,
	get_user_collections,
	get_user_organizations,
	get_user_profile,
	get_user_projects,
	patch_user,
	unblock_user,
} from '@/helpers/users'
import { appSettingsModalOpenProfileKey } from '@/providers/app-settings-modal'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const route = useRoute()
const openProfileSettings = inject(appSettingsModalOpenProfileKey, () => {})
const queryClient = useQueryClient()
const breadcrumbs = useBreadcrumbs()
const userProfile = provideUserProfile({
	getUser: get_user_profile,
	getProjects: get_user_projects,
	getOrganizations: get_user_organizations,
	getCollections: get_user_collections,
	patchUser: patch_user,
	getBlockedUsers: get_blocked_users,
	blockUser: block_user,
	unblockUser: unblock_user,
})

const userId = computed(() => {
	const value = route.params.user
	return Array.isArray(value) ? (value[0] ?? '') : (value ?? '')
})
const projectType = computed(() => {
	const value = route.params.projectType
	return Array.isArray(value) ? value[0] : value
})

async function ensureUserProfileData(id: string): Promise<void> {
	if (!id) return

	let breadcrumbName = id
	try {
		const user = await queryClient.ensureQueryData({
			queryKey: ['user', id],
			queryFn: () => userProfile.getUser(id),
			staleTime: 30_000,
		})
		breadcrumbName = user.username
	} catch {
		// Let the mounted layout's useQuery surface errors; do not fail route setup.
	}

	await Promise.allSettled([
		queryClient.ensureQueryData({
			queryKey: ['user', id, 'projects'],
			queryFn: () => userProfile.getProjects(id),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['user', id, 'organizations'],
			queryFn: () => userProfile.getOrganizations(id),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['user', id, 'collections'],
			queryFn: () => userProfile.getCollections(id),
			staleTime: 30_000,
		}),
	])

	breadcrumbs.setName('User', breadcrumbName)
}

onBeforeRouteUpdate(async (to) => {
	const value = to.params.user
	const id = Array.isArray(value) ? (value[0] ?? '') : (value ?? '')
	await ensureUserProfileData(id)
})

breadcrumbs.setName('User', userId.value)
await ensureUserProfileData(userId.value)

const { data: user } = useQuery({
	queryKey: computed(() => ['user', userId.value]),
	queryFn: () => userProfile.getUser(userId.value),
	enabled: false,
	staleTime: 30_000,
})

watch(
	[userId, user],
	([currentUserId, value]) => {
		breadcrumbs.setName('User', value?.username ?? currentUserId)
	},
	{ immediate: true },
)
</script>
