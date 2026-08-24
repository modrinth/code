<template>
	<div class="w-full p-2">
		<UserProfilePageLayout
			:user-id="userId"
			:project-type="projectType"
			variant="app"
			site-url="https://modrinth.com"
			project-link-mode="app"
			:edit-profile-link="openProfileSettings"
			external-navigation
		>
			<template #project-actions="{ project }">
				<Button
					v-if="project.minecraft_server == null"
					type="outlined"
					class="!text-brand [&>svg]:!text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]"
					:disabled="isProjectInstalling(project.id)"
					@click.stop="installProject(project)"
				>
					<SpinnerIcon v-if="isProjectInstalling(project.id)" class="animate-spin" />
					<DownloadIcon v-else-if="project.project_types.includes('modpack')" />
					<PlusIcon v-else />
					{{
						formatMessage(
							isProjectInstalling(project.id)
								? commonMessages.installingLabel
								: project.project_types.includes('modpack')
									? commonMessages.installButton
									: messages.installToInstance,
						)
					}}
				</Button>
			</template>
		</UserProfilePageLayout>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { DownloadIcon, PlusIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Button,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	provideUserProfile,
	UserProfilePageLayout,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, inject, ref, watch } from 'vue'
import { onBeforeRouteUpdate, useRoute, useRouter } from 'vue-router'

import {
	block_user,
	change_user_avatar,
	delete_user_avatar,
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
import { injectContentInstall } from '@/providers/content-install'

const route = useRoute()
const router = useRouter()
const openProfileSettings = inject(appSettingsModalOpenProfileKey, () => {})
const queryClient = useQueryClient()
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { install: installVersion } = injectContentInstall()
const installingProjectIds = ref(new Set<string>())

const messages = defineMessages({
	installToInstance: {
		id: 'app.user.project.install-to-instance',
		defaultMessage: 'Install to instance',
	},
})

function isProjectInstalling(projectId: string): boolean {
	return installingProjectIds.value.has(projectId)
}

function setProjectInstalling(projectId: string, installing: boolean): void {
	const next = new Set(installingProjectIds.value)
	if (installing) {
		next.add(projectId)
	} else {
		next.delete(projectId)
	}
	installingProjectIds.value = next
}

async function installProject(project: Labrinth.Projects.v3.Project): Promise<void> {
	if (isProjectInstalling(project.id)) return

	setProjectInstalling(project.id, true)
	try {
		await installVersion(
			project.id,
			null,
			null,
			'UserPage',
			() => setProjectInstalling(project.id, false),
			(instanceId) => router.push(`/instance/${encodeURIComponent(instanceId)}`),
		)
	} catch (error) {
		setProjectInstalling(project.id, false)
		handleError(error)
	}
}

const userProfile = provideUserProfile({
	getUser: get_user_profile,
	getProjects: get_user_projects,
	getOrganizations: get_user_organizations,
	getCollections: get_user_collections,
	patchUser: patch_user,
	changeAvatar: async (userId, file, extension) => {
		await change_user_avatar(userId, new Uint8Array(await file.arrayBuffer()), extension)
	},
	deleteAvatar: delete_user_avatar,
	getBlockedUsers: get_blocked_users,
	blockUser: block_user,
	unblockUser: unblock_user,
})

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
