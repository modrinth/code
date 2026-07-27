<template>
	<div class="w-full pt-2">
		<UserProfilePageLayout
			:user-id="userId"
			:project-type="projectType"
			site-url="https://modrinth.com"
			project-link-mode="app"
			external-navigation
		/>
	</div>
</template>

<script setup lang="ts">
import {
	provideUserProfile,
	UserProfilePageLayout,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, watch } from 'vue'
import { useRoute } from 'vue-router'

import {
	get_user_collections,
	get_user_organizations,
	get_user_profile,
	get_user_projects,
	patch_user,
} from '@/helpers/users'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const route = useRoute()
const queryClient = useQueryClient()
const breadcrumbs = useBreadcrumbs()
const userProfile = provideUserProfile({
	getUser: get_user_profile,
	getProjects: get_user_projects,
	getOrganizations: get_user_organizations,
	getCollections: get_user_collections,
	patchUser: patch_user,
})

const userId = computed(() => {
	const value = route.params.user
	return Array.isArray(value) ? (value[0] ?? '') : (value ?? '')
})
const projectType = computed(() => {
	const value = route.params.projectType
	return Array.isArray(value) ? value[0] : value
})

if (userId.value) {
	try {
		await queryClient.ensureQueryData({
			queryKey: ['user', userId.value],
			queryFn: () => userProfile.getUser(userId.value),
			staleTime: 30_000,
		})
	} catch {
		// Let the mounted layout's useQuery surface errors; do not fail route setup.
	}

	await Promise.allSettled([
		queryClient.ensureQueryData({
			queryKey: ['user', userId.value, 'projects'],
			queryFn: () => userProfile.getProjects(userId.value),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['user', userId.value, 'organizations'],
			queryFn: () => userProfile.getOrganizations(userId.value),
			staleTime: 30_000,
		}),
		queryClient.ensureQueryData({
			queryKey: ['user', userId.value, 'collections'],
			queryFn: () => userProfile.getCollections(userId.value),
			staleTime: 30_000,
		}),
	])
}

const { data: user } = useQuery({
	queryKey: computed(() => ['user', userId.value]),
	queryFn: () => userProfile.getUser(userId.value),
	enabled: false,
	staleTime: 30_000,
})

watch(
	user,
	(value) => {
		if (value?.username) {
			breadcrumbs.setName('User', value.username)
		}
	},
	{ immediate: true },
)
</script>
