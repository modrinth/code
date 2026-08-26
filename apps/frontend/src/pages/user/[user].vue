<template>
	<ProjectCreateModal ref="projectCreateModal" />
	<CollectionCreateModal ref="collectionCreateModal" />
	<UserProfilePageLayout
		:user-id="userId"
		:project-type="projectType"
		:display-mode="cosmetics.searchDisplayMode.user"
		:sidebar-position="cosmetics.leftContentLayout ? 'left' : 'right'"
		:site-url="config.public.siteUrl"
		:on-create-project="openProjectCreateModal"
		:on-create-collection="openCollectionCreateModal"
	>
		<template #sidebar>
			<AdPlaceholder v-if="!auth.user" />
		</template>
	</UserProfilePageLayout>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient, provideUserProfile, UserProfilePageLayout } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import ProjectCreateModal from '~/components/ui/create/ProjectCreateModal.vue'
import { warmProjectCheckCaches } from '~/composables/queries/project'

const route = useNativeRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const userProfile = provideUserProfile({
	getUser: (userId) => client.labrinth.users_v3.get(userId),
	getProjects: (userId) => client.labrinth.users_v3.getProjects(userId),
	getOrganizations: (userId) => client.labrinth.users_v2.getOrganizations(userId),
	getCollections: (userId) => client.labrinth.users_v2.getCollections(userId),
	patchUser: (userId, patch) => client.labrinth.users_v2.patch(userId, patch),
	changeAvatar: (userId, file, extension) =>
		client.labrinth.users_v2.changeIcon(userId, file, extension),
	deleteAvatar: (userId) => client.labrinth.users_v2.deleteIcon(userId),
	getBlockedUsers: () => client.labrinth.blocked_users_v3.list(),
	blockUser: (userId) => client.labrinth.blocked_users_v3.block(userId),
	unblockUser: (userId) => client.labrinth.blocked_users_v3.unblock(userId),
})
const auth = await useAuth()
const cosmetics = useCosmetics()
const config = useRuntimeConfig()

function readRouteParam(param: unknown): string | undefined {
	const value = Array.isArray(param) ? param[0] : param
	return typeof value === 'string' && value.length > 0 ? value : undefined
}

const userId = ref(readRouteParam(route.params.user) ?? '')
watch(
	() => readRouteParam(route.params.user),
	(next) => {
		if (next) userId.value = next
	},
)
const projectType = computed(() => readRouteParam(route.params.projectType))

let prefetchedUser: Labrinth.Users.v3.User | undefined
try {
	prefetchedUser = await queryClient.ensureQueryData({
		queryKey: ['user', userId.value],
		queryFn: () => userProfile.getUser(userId.value),
		staleTime: 30_000,
	})
} catch {
	// Let the mounted layout's useQuery surface errors; do not fail route setup.
}

const [projectsResult] = await Promise.allSettled([
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

if (projectsResult.status === 'fulfilled') {
	warmProjectCheckCaches(queryClient, projectsResult.value)
}
const title = computed(() =>
	prefetchedUser ? `${prefetchedUser.username} - Modrinth` : 'User not found',
)
const description = computed(() => {
	if (!prefetchedUser) {
		return `There's no user here, check that you have the right link!`
	}
	return prefetchedUser.bio
		? `${prefetchedUser.bio} - Download ${prefetchedUser.username}'s projects on Modrinth`
		: `Download ${prefetchedUser.username}'s projects on Modrinth`
})

useSeoMeta({
	title: () => title.value,
	description: () => description.value,
	ogTitle: () => title.value,
	ogDescription: () => description.value,
	ogImage: () =>
		prefetchedUser
			? (prefetchedUser?.avatar_url ?? 'https://cdn-raw.modrinth.com/placeholder-circle.png')
			: 'https://cdn-raw.modrinth.com/not-found-circle.png',
})

const projectCreateModal = ref<InstanceType<typeof ProjectCreateModal> | null>(null)
const collectionCreateModal = ref<InstanceType<typeof CollectionCreateModal> | null>(null)

function openProjectCreateModal(event?: MouseEvent) {
	projectCreateModal.value?.show(event)
}

function openCollectionCreateModal(event?: MouseEvent) {
	collectionCreateModal.value?.show(event)
}
</script>
