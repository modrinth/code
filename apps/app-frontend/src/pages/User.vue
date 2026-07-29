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
import type { Labrinth } from '@modrinth/api-client'
import { provideUserProfile, UserProfilePageLayout } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, ref, watch } from 'vue'
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
import { useBreadcrumb } from '@/providers/breadcrumbs'

const route = useRoute()
const openProfileSettings = inject(appSettingsModalOpenProfileKey, () => {})
const queryClient = useQueryClient()
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

function getCachedUserSummary(id: string) {
	return queryClient.getQueryData<Labrinth.Users.v3.User>(['users', 'summary', id])
}

const { data: user } = useQuery({
	queryKey: computed(() => ['user', userId.value]),
	queryFn: () => userProfile.getUser(userId.value),
	enabled: false,
	staleTime: 30_000,
})

const breadcrumbUserId = ref(userId.value)
const breadcrumbLabel = ref(getCachedUserSummary(userId.value)?.username ?? userId.value)
const breadcrumbTo = ref(route.fullPath)
watch(
	[userId, user, () => route.fullPath],
	([currentUserId, currentUser, currentPath]) => {
		if (route.name !== 'User') return
		breadcrumbUserId.value = currentUserId
		breadcrumbLabel.value = currentUser?.username ?? currentUserId
		breadcrumbTo.value = currentPath
	},
	{ immediate: true, flush: 'sync' },
)

useBreadcrumb({
	slot: 'user',
	id: () => `user:${breadcrumbUserId.value}`,
	label: breadcrumbLabel,
	to: breadcrumbTo,
	visual: () => ({
		type: 'image',
		src: user.value?.avatar_url ?? getCachedUserSummary(breadcrumbUserId.value)?.avatar_url,
		alt: breadcrumbLabel.value,
		circle: true,
		tintBy: breadcrumbUserId.value,
	}),
})

async function ensureUserProfileData(id: string): Promise<void> {
	if (!id) return

	try {
		await queryClient.ensureQueryData({
			queryKey: ['user', id],
			queryFn: () => userProfile.getUser(id),
			staleTime: 30_000,
		})
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
}

onBeforeRouteUpdate(async (to) => {
	const value = to.params.user
	const id = Array.isArray(value) ? (value[0] ?? '') : (value ?? '')
	await ensureUserProfileData(id)
})

await ensureUserProfileData(userId.value)
</script>
