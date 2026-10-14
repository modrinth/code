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
				<template v-if="project.minecraft_server == null && serverContext">
					<Button
						v-if="project.project_types.includes('modpack')"
						type="outlined"
						class="!text-brand [&>svg]:!text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]"
						@click.stop="router.push(`/project/${project.id}`)"
					>
						<DownloadIcon />
						{{ formatMessage(commonMessages.installButton) }}
					</Button>
					<Button
						v-else-if="getServerContentType(project)"
						type="outlined"
						class="!text-brand [&>svg]:!text-brand !shadow-[inset_0_0_0_1px_var(--color-brand)]"
						:disabled="
							isProjectInstalling(project.id) ||
							serverInstallContent.serverContentProjectIds.value.has(project.id)
						"
						@click.stop="toggleServerProject(project)"
					>
						<SpinnerIcon v-if="isProjectInstalling(project.id)" class="animate-spin" />
						<CheckIcon
							v-else-if="
								serverInstallContent.serverContentProjectIds.value.has(project.id) ||
								isServerProjectSelected(project.id)
							"
						/>
						<PlusIcon v-else />
						{{
							formatMessage(
								serverInstallContent.serverContentProjectIds.value.has(project.id)
									? commonMessages.installedLabel
									: isServerProjectSelected(project.id)
										? commonMessages.selectedLabel
										: commonMessages.installButton,
							)
						}}
					</Button>
				</template>
				<Button
					v-else-if="project.minecraft_server == null"
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
		<SelectedProjectsFloatingBar v-if="serverContext" :install-context="serverContext" />
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, DownloadIcon, PlusIcon, SpinnerIcon } from '@modrinth/assets'
import {
	type BrowseInstallContentType,
	Button,
	commonMessages,
	defineMessages,
	getTargetInstallPreferences,
	injectNotificationManager,
	provideUserProfile,
	requestInstall,
	SelectedProjectsFloatingBar,
	UserProfilePageLayout,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, watch } from 'vue'
import { onBeforeRouteUpdate, useRoute, useRouter } from 'vue-router'

import { injectSettingsActions } from '@/features/settings/settings-actions'
import { get_project, get_version_many } from '@/helpers/cache.js'
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
import { useServerInstallContent } from '@/platform/adapters/browse/server-install-content'
import { injectContentInstall } from '@/providers/content-install'
import { useBreadcrumb } from '@/shared/breadcrumbs'

const route = useRoute()
const router = useRouter()
const { openProfile: openProfileSettings } = injectSettingsActions()
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

const SERVER_INSTALLABLE_CONTENT_TYPES: BrowseInstallContentType[] = [
	'mod',
	'plugin',
	'datapack',
	'resourcepack',
	'shader',
]

const serverInstallContent = useServerInstallContent({
	serverSetupModalRef: ref(null),
	inheritBreadcrumbServer: true,
})
serverInstallContent.watchServerContextChanges()

/** Install context for the server panel this page was reached from, if any. */
const serverContext = computed(() => {
	const server = serverInstallContent.serverContextServerData.value
	if (!server || serverInstallContent.isSetupServerContext.value) return null
	return {
		name: server.name,
		loader: server.loader ?? '',
		gameVersion: server.mc_version ?? '',
		serverId: serverInstallContent.serverIdQuery.value,
		backUrl: serverInstallContent.serverBackUrl.value,
		backLabel: serverInstallContent.serverBackLabel.value,
		heading: serverInstallContent.serverBrowseHeading.value,
		selectedProjects: serverInstallContent.selectedServerInstallProjects.value,
		isInstallingSelected: serverInstallContent.isInstallingQueuedServerInstalls.value,
		installProgress: serverInstallContent.queuedInstallProgress.value,
		clearQueued: serverInstallContent.clearQueuedServerInstalls,
		clearSelected: serverInstallContent.clearQueuedServerInstalls,
		discardSelectedAndBack: serverInstallContent.discardQueuedServerInstallsAndBack,
		installSelected: serverInstallContent.installQueuedServerInstallsAndBack,
	}
})

function getServerContentType(project: Labrinth.Projects.v3.Project) {
	return SERVER_INSTALLABLE_CONTENT_TYPES.find((type) => project.project_types.includes(type))
}

function isServerProjectSelected(projectId: string): boolean {
	return serverInstallContent.queuedServerInstallRootProjectIds.value.has(projectId)
}

async function getInstallProjectVersions(projectId: string) {
	const project = await get_project(projectId, 'must_revalidate')
	return (await get_version_many(
		project.versions,
		'must_revalidate',
	)) as Labrinth.Versions.v2.Version[]
}

async function toggleServerProject(project: Labrinth.Projects.v3.Project): Promise<void> {
	if (isServerProjectSelected(project.id)) {
		serverInstallContent.removeQueuedServerInstall(project.id)
		return
	}

	const contentType = getServerContentType(project)
	const server = serverInstallContent.serverContextServerData.value
	if (!contentType || !server || isProjectInstalling(project.id)) return

	setProjectInstalling(project.id, true)
	try {
		const plan = await requestInstall({
			project: { project_id: project.id, name: project.name, icon_url: project.icon_url },
			contentType,
			mode: 'queue',
			selectedFilters: [],
			providedFilters: [],
			overriddenProvidedFilterTypes: [],
			targetPreferences: getTargetInstallPreferences(
				{ gameVersion: server.mc_version, loader: server.loader },
				contentType,
			),
			getProjectVersions: getInstallProjectVersions,
			queue: {
				get: serverInstallContent.getQueuedServerInstallPlans,
				set: serverInstallContent.setQueuedServerInstallPlans,
			},
		})
		await serverInstallContent.resolveQueuedServerInstallPlan(plan)
	} catch (error) {
		serverInstallContent.removeQueuedServerInstall(project.id)
		handleError(error as Error)
	} finally {
		setProjectInstalling(project.id, false)
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

await Promise.all([ensureUserProfileData(userId.value), serverInstallContent.initServerContext()])
</script>
