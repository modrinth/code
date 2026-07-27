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
import {
	injectModrinthClient,
	provideUserProfile,
	UserProfilePageLayout,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import ProjectCreateModal from '~/components/ui/create/ProjectCreateModal.vue'

const route = useNativeRoute()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const userProfile = provideUserProfile({
	getUser: (userId) => client.labrinth.users_v3.get(userId),
	getProjects: (userId) => client.labrinth.users_v2.getProjects(userId),
	getOrganizations: (userId) => client.labrinth.users_v2.getOrganizations(userId),
	getCollections: (userId) => client.labrinth.users_v2.getCollections(userId),
	patchUser: (userId, patch) => client.labrinth.users_v2.patch(userId, patch),
})
const auth = await useAuth()
const cosmetics = useCosmetics()
const config = useRuntimeConfig()

const userId = computed(() => String(route.params.user))
const projectType = computed(() => {
	const value = route.params.projectType
	return Array.isArray(value) ? value[0] : value
})

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

const title = computed(() =>
	prefetchedUser ? `${prefetchedUser.username} - Modrinth` : 'Modrinth',
)
const description = computed(() => {
	if (!prefetchedUser) return ''
	return prefetchedUser.bio
		? `${prefetchedUser.bio} - Download ${prefetchedUser.username}'s projects on Modrinth`
		: `Download ${prefetchedUser.username}'s projects on Modrinth`
})

useSeoMeta({
	title: () => title.value,
	description: () => description.value,
	ogTitle: () => title.value,
	ogDescription: () => description.value,
	ogImage: () => prefetchedUser?.avatar_url ?? 'https://cdn.modrinth.com/placeholder.png',
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
