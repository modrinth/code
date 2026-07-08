<template>
	<div v-if="instance" :class="{ 'flex h-full flex-col': isFixedRender }">
		<div
			:class="['p-6 pr-2 pb-4', { 'shrink-0': isFixedRender }]"
			@contextmenu.prevent.stop="(event) => handleRightClick(event)"
		>
			<ExportModal ref="exportModal" :instance="instance" />
			<InstanceSettingsModal
				:key="instance.id"
				ref="settingsModal"
				:instance="instance"
				:offline="offline"
				@unlinked="fetchInstance"
			/>
			<UpdateToPlayModal
				ref="updateToPlayModal"
				:instance="instance"
				@shared-instance-unavailable="handleSharedInstanceUnavailable"
			/>
			<ContentPageHeader>
				<template #icon>
					<Avatar
						:src="icon ? icon : undefined"
						:alt="instance.name"
						size="64px"
						:tint-by="instance.id"
					/>
				</template>
				<template #title>
					{{ instance.name }}
				</template>
				<template v-if="instance.shared_instance" #title-suffix>
					<div
						class="inline-flex h-7 items-center gap-1 rounded-full border border-solid border-blue bg-highlight-blue px-2.5 !text-base !font-normal leading-none text-blue"
					>
						Shared
						<UnknownIcon
							v-tooltip="'This instance is being shared to other users.'"
							class="size-4 cursor-help"
							aria-label="Shared instance information"
						/>
					</div>
				</template>
				<template #stats>
					<div class="flex items-center flex-wrap gap-2">
						<template v-if="!isServerInstance">
							<div class="flex items-center gap-2 capitalize font-medium">
								{{ instance.loader }} {{ instance.game_version }}
							</div>

							<template v-if="showInstancePlayTime">
								<div class="w-1.5 h-1.5 rounded-full bg-surface-5"></div>

								<div class="flex items-center gap-2 font-medium">
									<template v-if="timePlayed > 0">
										{{ timePlayedHumanized }}
									</template>
									<template v-else> Never played </template>
								</div>
							</template>
						</template>

						<template v-else>
							<template v-if="loadingServerPing">
								<ServerOnlinePlayers
									v-if="playersOnline !== undefined"
									:online="playersOnline"
									:status-online="statusOnline"
									hide-label
								/>
								<ServerRecentPlays :recent-plays="recentPlays ?? 0" hide-label />
								<div
									v-if="
										(playersOnline !== undefined || recentPlays !== undefined) &&
										(minecraftServer?.region || ping)
									"
									class="w-1.5 h-1.5 rounded-full bg-surface-5"
								></div>
								<ServerPing v-if="ping" :ping="ping" />
							</template>

							<ServerRegion v-if="minecraftServer?.region" :region="minecraftServer?.region" />

							<div
								v-if="minecraftServer?.region || ping"
								class="w-1.5 h-1.5 rounded-full bg-surface-5"
							></div>

							<div
								v-if="linkedProjectV3"
								class="flex gap-1.5 items-center font-medium text-primary"
							>
								Linked to
								<Avatar
									:src="linkedProjectV3.icon_url"
									:alt="linkedProjectV3.name"
									:tint-by="instance.id"
									size="24px"
								/>
								<router-link
									:to="`/project/${linkedProjectV3.slug ?? linkedProjectV3.id}`"
									class="hover:underline text-primary truncate"
								>
									{{ linkedProjectV3.name }}
								</router-link>
							</div>
						</template>

						<template v-if="sharedInstanceManager">
							<div class="w-1.5 h-1.5 rounded-full bg-surface-5"></div>

							<div class="flex min-w-0 items-center gap-[5px] font-medium">
								Managed by
								<Avatar
									:src="sharedInstanceManager.avatarUrl"
									:alt="sharedInstanceManager.username"
									:tint-by="sharedInstanceManager.tintBy"
									size="24px"
									circle
									no-shadow
								/>
								<span class="min-w-0 truncate">{{ sharedInstanceManager.username }}</span>
							</div>
						</template>
					</div>
				</template>
				<template #actions>
					<div class="flex gap-2">
						<ButtonStyled
							v-if="
								[
									'installing',
									'pack_installing',
									'pack_installed',
									'not_installed',
									'minecraft_installing',
								].includes(instance.install_stage)
							"
							color="brand"
							size="large"
						>
							<button disabled>Installing...</button>
						</ButtonStyled>
						<ButtonStyled
							v-else-if="instance.install_stage !== 'installed'"
							color="brand"
							size="large"
						>
							<button @click="repairInstance()">
								<DownloadIcon />
								Repair
							</button>
						</ButtonStyled>
						<ButtonStyled v-else-if="playing === true" color="red" size="large">
							<button :disabled="stopping" @click="stopInstance('InstancePage')">
								<StopCircleIcon />
								{{ stopping ? 'Stopping...' : 'Stop' }}
							</button>
						</ButtonStyled>
						<ButtonStyled
							v-else-if="playing === false && loading === false && !isServerInstance"
							color="brand"
							size="large"
						>
							<button @click="startInstance('InstancePage')">
								<PlayIcon />
								Play
							</button>
						</ButtonStyled>
						<div
							v-else-if="playing === false && loading === false && isServerInstance"
							class="joined-buttons"
						>
							<ButtonStyled color="brand" size="large">
								<button @click="handlePlayServer()">
									<PlayIcon />
									Play
								</button>
							</ButtonStyled>
							<ButtonStyled color="brand" size="large">
								<OverflowMenu
									:options="[
										{
											id: 'join_server',
											action: () => handlePlayServer(),
										},
										{
											id: 'launch_instance',
											action: () => startInstance('InstancePage'),
										},
									]"
								>
									<div class="w-0 text-xl relative top-0.5 right-2.5">
										<DropdownIcon />
									</div>

									<template #join_server>
										<PlayIcon />
										Join server
									</template>
									<template #launch_instance>
										<PlayIcon />
										Launch instance
									</template>
								</OverflowMenu>
							</ButtonStyled>
						</div>
						<ButtonStyled
							v-else-if="loading === true && playing === false"
							color="brand"
							size="large"
						>
							<button disabled>Starting...</button>
						</ButtonStyled>
						<ButtonStyled circular size="large">
							<button v-tooltip="'Instance settings'" @click="settingsModal?.show()">
								<SettingsIcon />
							</button>
						</ButtonStyled>
						<ButtonStyled type="transparent" circular size="large">
							<OverflowMenu
								:options="[
									{
										id: 'open-folder',
										action: () => {
											if (instance) showInstanceInFolder(instance.id)
										},
									},
									{
										id: 'export-mrpack',
										action: () => exportModal?.show(),
									},
									{
										id: 'create-shortcut',
										action: () => createShortcut(),
									},
								]"
							>
								<MoreVerticalIcon />
								<template #share-instance> <UserPlusIcon /> Share instance </template>
								<template #host-a-server> <ServerIcon /> Create a server </template>
								<template #open-folder> <FolderOpenIcon /> Open folder </template>
								<template #export-mrpack> <PackageIcon /> Export modpack </template>
								<template #create-shortcut> <ExternalIcon /> Create shortcut </template>
							</OverflowMenu>
						</ButtonStyled>
					</div>
				</template>
			</ContentPageHeader>
		</div>
		<div :class="['px-6', { 'shrink-0': isFixedRender }]">
			<NavTabs :links="tabs" />
			<InstanceAdmonitions
				class="mt-4"
				:instance="instance"
				:shared-instance-unavailable-reason="sharedInstanceUnavailableReason"
				:shared-instance-unavailable-manager="sharedInstanceManager?.username"
				:shared-instance-wrong-account="sharedInstanceWrongAccount"
				:shared-instance-expected-username="sharedInstanceExpectedUsername"
				:shared-instance-role="instance.shared_instance?.role"
				:shared-instance-signed-out="sharedInstanceSignedOut"
				@published="fetchInstance"
			/>
		</div>
		<div :class="['p-6 pt-4', { 'min-h-0 flex-1 overflow-y-auto': isFixedRender }]">
			<RouterView v-slot="{ Component }" :key="instance.id" :route="displayedInstanceRoute">
				<template v-if="Component">
					<Suspense
						:key="instance.id"
						@pending="subpagePending = true"
						@resolve="subpagePending = false"
					>
						<component
							:is="Component"
							:instance="instance"
							:options="options"
							:offline="offline"
							:playing="playing"
							:installed="instance.install_stage !== 'installed'"
							:is-server-instance="isServerInstance"
							:open-settings="() => settingsModal?.show(1)"
							v-bind="instanceSubpageProps"
							@play="updatePlayState"
							@stop="() => stopInstance('InstanceSubpage')"
						></component>
					</Suspense>
				</template>
			</RouterView>
		</div>
		<ContextMenu ref="options" @option-clicked="handleOptionsClick">
			<template #play> <PlayIcon /> Play </template>
			<template #stop> <StopCircleIcon /> Stop </template>
			<template #add_content> <PlusIcon /> Add content </template>
			<template #edit> <EditIcon /> Edit </template>
			<template #copy_path> <ClipboardCopyIcon /> Copy path </template>
			<template #open_folder> <FolderOpenIcon /> Open folder </template>
			<template #copy_link> <ClipboardCopyIcon /> Copy link </template>
			<template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
			<template #copy_names><EditIcon />Copy names</template>
			<template #copy_slugs><HashIcon />Copy slugs</template>
			<template #copy_links><GlobeIcon />Copy links</template>
			<template #toggle><EditIcon />Toggle selected</template>
			<template #disable><XIcon />Disable selected</template>
			<template #enable><CheckCircleIcon />Enable selected</template>
			<template #hide_show><EyeIcon />Show/Hide unselected</template>
			<template #update_all
				><UpdatedIcon />Update {{ selected.length > 0 ? 'selected' : 'all' }}</template
			>
			<template #filter_update><UpdatedIcon />Select Updatable</template>
		</ContextMenu>
	</div>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BoxesIcon,
	CheckCircleIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	DropdownIcon,
	EditIcon,
	ExternalIcon,
	EyeIcon,
	FolderOpenIcon,
	GlobeIcon,
	HashIcon,
	MoreVerticalIcon,
	PackageIcon,
	PlayIcon,
	PlusIcon,
	ServerIcon,
	SettingsIcon,
	StopCircleIcon,
	TerminalSquareIcon,
	UnknownIcon,
	UpdatedIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ContentPageHeader,
	defineMessages,
	injectAuth,
	injectNotificationManager,
	NavTabs,
	OverflowMenu,
	ServerOnlinePlayers,
	ServerPing,
	ServerRecentPlays,
	ServerRegion,
	useLoadingBarToken,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import duration from 'dayjs/plugin/duration'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import InstanceAdmonitions from '@/components/ui/instance/InstanceAdmonitions.vue'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'
import UpdateToPlayModal from '@/components/ui/modal/UpdateToPlayModal.vue'
import {
	fetchCachedServerStatus,
	getFreshCachedServerStatus,
} from '@/composables/instances/use-server-status-query'
import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { trackEvent } from '@/helpers/analytics'
import { get_project_v3, get_user } from '@/helpers/cache.js'
import { instance_listener, process_listener } from '@/helpers/events'
import {
	getErrorMessage,
	getSharedInstanceUnavailableReason,
	install_existing_instance,
	install_get_shared_instance_update_preview,
	install_pack_to_existing_instance,
	isSharedInstanceUnavailableError,
	type SharedInstanceUpdatePreview,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import { get, get_full_path, kill, run } from '@/helpers/instance'
import { type InstanceContentData, loadInstanceContentData } from '@/helpers/instance-content'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { createInstanceShortcut, showInstanceInFolder } from '@/helpers/utils.js'
import { refreshWorlds, type ServerStatus } from '@/helpers/worlds'
import { injectServerInstall } from '@/providers/server-install'
import { handleSevereError } from '@/store/error.js'
import { useBreadcrumbs, useTheming } from '@/store/state'

dayjs.extend(duration)
dayjs.extend(relativeTime)

const { addNotification, handleError } = injectNotificationManager()
const auth = injectAuth()
const { playServerProject } = injectServerInstall()
const queryClient = useQueryClient()
const route = useRoute()

const router = useRouter()
const displayedInstanceRoute = shallowRef(router.currentRoute.value)
const breadcrumbs = useBreadcrumbs()
const themeStore = useTheming()
const showInstancePlayTime = computed(() => themeStore.getFeatureFlag('show_instance_play_time'))
const contentSubpageRouteNames = new Set(['Mods', 'ModsFilter'])

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const instance = ref<GameInstance>()
const preloadedContent = ref<InstanceContentData | null>(null)
const playing = ref(false)
const loading = ref(false)
const subpagePending = ref(false)
const stopping = ref(false)
const exportModal = ref<InstanceType<typeof ExportModal>>()
const updateToPlayModal = ref<InstanceType<typeof UpdateToPlayModal>>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	sharedInstanceUnavailableTitle: {
		id: 'instance.shared-instance.unavailable.title',
		defaultMessage: 'Shared instance no longer available',
	},
	sharedInstanceUnavailableText: {
		id: 'instance.shared-instance.unavailable.text',
		defaultMessage:
			'The shared instance has been deleted or your access has been revoked. Contact {manager} for more information.',
	},
	sharedInstanceDeletedText: {
		id: 'instance.shared-instance.unavailable.deleted-text',
		defaultMessage: 'The shared instance has been deleted. Contact {manager} for more information.',
	},
	sharedInstanceAccessRevokedText: {
		id: 'instance.shared-instance.unavailable.access-revoked-text',
		defaultMessage:
			'Your access to this shared instance has been revoked. Contact {manager} for more information.',
	},
	sharedInstanceUnavailableFallbackManager: {
		id: 'instance.shared-instance.unavailable.manager-fallback',
		defaultMessage: 'the instance manager',
	},
	sharedInstanceErrorTitle: {
		id: 'instance.shared-instance.error.title',
		defaultMessage: 'Something has gone wrong',
	},
})

useLoadingBarToken(subpagePending)

const isServerInstance = ref(false)
const linkedProjectV3 = ref<Labrinth.Projects.v3.Project>()
const selected = ref<unknown[]>([])

const minecraftServer = computed(() => linkedProjectV3.value?.minecraft_server)
const javaServerPingData = computed(() => linkedProjectV3.value?.minecraft_java_server?.ping?.data)
const liveServerStatusOnline = ref(false)
const statusOnline = computed(() => liveServerStatusOnline.value || !!javaServerPingData.value)
const recentPlays = computed(
	() => linkedProjectV3.value?.minecraft_java_server?.verified_plays_2w ?? undefined,
)
const playersOnline = ref<number | undefined>(undefined)
const ping = ref<number | undefined>(undefined)
const loadingServerPing = ref(false)
const activeInstanceId = ref<string>()
const sharedInstanceUpdatePreview = ref<SharedInstanceUpdatePreview | null>(null)
const sharedInstanceUnavailableReason = ref<SharedInstanceUnavailableReason | null>(null)
const sharedInstanceAvailabilityCheckKey = ref<string | null>(null)
const sharedInstanceManagerUser = ref<{
	id: string
	username: string
	avatar_url?: string
} | null>(null)
const sharedInstanceManagerUserId = computed(() => {
	const attachment = instance.value?.shared_instance
	if (!attachment) return null

	if (attachment.role === 'owner') {
		if (!sharedInstanceActionsLocked.value) return null

		return attachment.linked_user_id ?? null
	}

	return attachment.manager_id ?? null
})

const sharedInstanceManager = computed(() => {
	if (!instance.value?.shared_instance) return null

	if (instance.value.shared_instance.role === 'owner') {
		if (!sharedInstanceActionsLocked.value) return null

		const linkedUserId = instance.value.shared_instance.linked_user_id
		const linkedUser = sharedInstanceManagerUser.value
		if (!linkedUserId || !linkedUser) return null

		return {
			username: linkedUser.username,
			avatarUrl: linkedUser.avatar_url ?? undefined,
			tintBy: linkedUser.id,
		}
	}

	const user = sharedInstanceManagerUser.value
	if (!user) return null

	return {
		username: user.username,
		avatarUrl: user.avatar_url ?? undefined,
		tintBy: user.id,
	}
})

const sharedInstanceExpectedUserId = computed(
	() => instance.value?.shared_instance?.linked_user_id ?? null,
)
const sharedInstanceExpectedUsername = computed(() => sharedInstanceExpectedUserId.value ?? '')
const sharedInstanceWrongAccount = computed(() => {
	if (auth.isReady && !auth.isReady.value) return false

	const expectedUserId = sharedInstanceExpectedUserId.value
	if (!expectedUserId) return false

	return auth.user.value?.id !== expectedUserId
})
const sharedInstanceActionsLocked = computed(() => sharedInstanceWrongAccount.value)
const sharedInstanceSignedOut = computed(() => !auth.session_token.value)

watch(
	() => sharedInstanceManagerUserId.value,
	async (userId) => {
		sharedInstanceManagerUser.value = null

		if (!userId) return

		const user = await get_user(userId, 'bypass').catch(() => null)
		if (sharedInstanceManagerUserId.value !== userId) return

		sharedInstanceManagerUser.value = user
	},
	{ immediate: true },
)

watch(
	() => router.currentRoute.value,
	(nextRoute) => {
		if (nextRoute.path.startsWith('/instance')) {
			displayedInstanceRoute.value = nextRoute
		}
	},
	{ immediate: true },
)

function applyServerStatus(status: ServerStatus) {
	playersOnline.value = status.players?.online
	ping.value = status.ping
	liveServerStatusOnline.value = true
	loadingServerPing.value = true
}

function resetServerStatus() {
	ping.value = undefined
	playersOnline.value = undefined
	liveServerStatusOnline.value = false
	loadingServerPing.value = false
}

function isContentSubpageRoute(routeName = displayedInstanceRoute.value.name) {
	return typeof routeName === 'string' && contentSubpageRouteNames.has(routeName)
}

async function fetchInstance() {
	isServerInstance.value = false
	linkedProjectV3.value = undefined
	preloadedContent.value = null
	sharedInstanceUpdatePreview.value = null
	sharedInstanceUnavailableReason.value = null
	sharedInstanceAvailabilityCheckKey.value = null
	resetServerStatus()

	const nextInstance = await get(route.params.id as string).catch(handleError)
	let nextLinkedProjectV3: Labrinth.Projects.v3.Project | undefined
	let nextIsServerInstance = false

	const contentPreloadPromise =
		nextInstance && isContentSubpageRoute()
			? loadInstanceContentData(nextInstance.id, undefined, handleError)
			: Promise.resolve(null)

	if (!offline.value && nextInstance?.link && nextInstance.link.project_id) {
		try {
			nextLinkedProjectV3 = await get_project_v3(nextInstance.link.project_id, 'must_revalidate')

			if (nextLinkedProjectV3?.minecraft_server != null) {
				nextIsServerInstance = true
			}
		} catch (error) {
			handleError(error as Error)
		}
	}

	const nextPreloadedContent = await contentPreloadPromise

	instance.value = nextInstance ?? undefined
	linkedProjectV3.value = nextLinkedProjectV3
	isServerInstance.value = nextIsServerInstance
	preloadedContent.value = nextPreloadedContent
	activeInstanceId.value = nextInstance?.id

	fetchDeferredData(nextInstance?.id)

	if (nextInstance) {
		queryClient.prefetchQuery({
			queryKey: ['worlds', nextInstance.id],
			queryFn: () => refreshWorlds(nextInstance.id),
			staleTime: 30_000,
		})
	}
}

function fetchDeferredData(instanceId?: string) {
	const serverAddress = linkedProjectV3.value?.minecraft_java_server?.address
	if (isServerInstance.value && serverAddress) {
		const cachedStatus = getFreshCachedServerStatus(queryClient, serverAddress)
		if (cachedStatus) {
			applyServerStatus(cachedStatus)
		} else {
			playersOnline.value = undefined
			ping.value = undefined
			loadingServerPing.value = false
		}

		fetchCachedServerStatus(queryClient, serverAddress)
			.then((status) => {
				if (
					activeInstanceId.value !== instanceId ||
					linkedProjectV3.value?.minecraft_java_server?.address !== serverAddress
				)
					return
				applyServerStatus(status)
			})
			.catch((error) => {
				console.error(`Failed to fetch server status for ${serverAddress}:`, error)
			})
			.finally(() => {
				if (activeInstanceId.value !== instanceId) return
				loadingServerPing.value = true
			})
	} else {
		loadingServerPing.value = true
	}

	updatePlayState()
}

async function updatePlayState() {
	if (!route.params.id) return
	const runningProcesses = await get_by_instance_id(route.params.id as string).catch(handleError)

	playing.value = Array.isArray(runningProcesses) && runningProcesses.length > 0
}

await fetchInstance()
watch(
	() => route.params.id,
	async () => {
		if (route.params.id && route.path.startsWith('/instance')) {
			await fetchInstance()
		}
	},
)

async function checkSharedInstanceAvailability(instanceId: string) {
	try {
		const preview = await install_get_shared_instance_update_preview(instanceId)
		if (instance.value?.id !== instanceId) return

		sharedInstanceUpdatePreview.value = preview
		sharedInstanceUnavailableReason.value = null
	} catch (error) {
		if (instance.value?.id !== instanceId) return

		if (isSharedInstanceUnavailableError(error)) {
			sharedInstanceUpdatePreview.value = null
			sharedInstanceUnavailableReason.value = getSharedInstanceUnavailableReason(error)
			return
		}

		addNotification({
			type: 'error',
			title: formatMessage(messages.sharedInstanceErrorTitle),
			text: getErrorMessage(error),
		})
	}
}

watch(
	() => ({
		instanceId: instance.value?.id,
		role: instance.value?.shared_instance?.role,
		locked: sharedInstanceActionsLocked.value,
		offline: offline.value,
		userId: auth.user.value?.id ?? null,
		authReady: auth.isReady?.value ?? true,
	}),
	async ({ instanceId, role, locked, offline, userId, authReady }) => {
		if (!instanceId || role !== 'member' || locked || offline || !authReady) {
			sharedInstanceUpdatePreview.value = null
			sharedInstanceUnavailableReason.value = null
			sharedInstanceAvailabilityCheckKey.value = null
			return
		}

		const key = `${instanceId}:${userId ?? 'signed-out'}`
		if (sharedInstanceAvailabilityCheckKey.value === key) return

		sharedInstanceAvailabilityCheckKey.value = key
		await checkSharedInstanceAvailability(instanceId)
	},
	{ immediate: true },
)

const basePath = computed(
	() => `/instance/${encodeURIComponent(displayedInstanceRoute.value.params.id as string)}`,
)

/**
 * Per-route layout mode.
 * - `'scroll'` (default): the whole instance page scrolls inside `.app-viewport`. This lets
 *   `position: sticky` children (and the viewport-rooted `IntersectionObserver` used by
 *   `useStickyObserver`) work correctly.
 * - `'fixed'`: the header + tabs are pinned and only the tab body scrolls in its own container.
 *   Used by tabs whose content (e.g. the log console) needs a bounded height to resolve `h-full`.
 */
const renderMode = computed<'scroll' | 'fixed'>(() =>
	displayedInstanceRoute.value.meta.renderMode === 'fixed' ? 'fixed' : 'scroll',
)
const isFixedRender = computed(() => renderMode.value === 'fixed')
const contentSubpageProps = computed(() =>
	isContentSubpageRoute() ? { preloadedContent: preloadedContent.value } : {},
)
const instanceSubpageProps = computed(() => ({
	...contentSubpageProps.value,
	...(displayedInstanceRoute.value.name === 'InstanceShare'
		? { sharedInstanceActionsLocked: sharedInstanceActionsLocked.value }
		: {}),
}))
const showShareTab = computed(() => {
	const linkType = instance.value?.link?.type

	return (
		instance.value?.shared_instance?.role !== 'member' &&
		linkType !== 'server_project' &&
		linkType !== 'server_project_modpack'
	)
})

const tabs = computed(() => {
	const instanceTabs = [
		{
			label: 'Content',
			href: `${basePath.value}`,
			icon: BoxesIcon,
		},
		{
			label: 'Files',
			href: `${basePath.value}/files`,
			icon: FolderOpenIcon,
		},
		{
			label: 'Worlds',
			href: `${basePath.value}/worlds`,
			icon: GlobeIcon,
		},
		{
			label: 'Logs',
			href: `${basePath.value}/logs`,
			icon: TerminalSquareIcon,
		},
	]

	if (showShareTab.value) {
		instanceTabs.push({
			label: 'Share',
			href: `${basePath.value}/share`,
			icon: UserPlusIcon,
		})
	}

	return instanceTabs
})

if (instance.value) {
	breadcrumbs.setName(
		'Instance',
		instance.value.name.length > 40
			? instance.value.name.substring(0, 40) + '...'
			: instance.value.name,
	)
	breadcrumbs.setContext({
		name: instance.value.name,
		link: displayedInstanceRoute.value.path,
		query: displayedInstanceRoute.value.query,
	})
}

const options = ref<InstanceType<typeof ContextMenu> | null>(null)

const launchInstance = async (context: string) => {
	loading.value = true
	try {
		await run(route.params.id as string)
		playing.value = true
	} catch (err) {
		handleSevereError(err, { instanceId: route.params.id as string })
	}
	loading.value = false

	if (!instance.value) return
	trackEvent('InstanceStart', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

function sharedInstanceUnavailableTextMessage(reason: SharedInstanceUnavailableReason | null) {
	if (reason === 'deleted') return messages.sharedInstanceDeletedText
	if (reason === 'access_revoked') return messages.sharedInstanceAccessRevokedText
	return messages.sharedInstanceUnavailableText
}

async function handleSharedInstanceUnavailable(
	reason: SharedInstanceUnavailableReason | null = null,
) {
	const manager =
		sharedInstanceManager.value?.username ??
		formatMessage(messages.sharedInstanceUnavailableFallbackManager)
	addNotification({
		type: 'warning',
		title: formatMessage(messages.sharedInstanceUnavailableTitle),
		text: formatMessage(sharedInstanceUnavailableTextMessage(reason), { manager }),
	})
	await fetchInstance()
	sharedInstanceUpdatePreview.value = null
	sharedInstanceUnavailableReason.value = reason
}

const startInstance = async (context: string) => {
	if (!instance.value) return
	if (loading.value || playing.value) return

	const isSharedInstanceMember = instance.value.shared_instance?.role === 'member'
	const canCheckSharedInstanceUpdate =
		isSharedInstanceMember && !sharedInstanceActionsLocked.value && !offline.value

	if (canCheckSharedInstanceUpdate) {
		const preview = sharedInstanceUpdatePreview.value

		if (preview?.updateAvailable && updateToPlayModal.value) {
			updateToPlayModal.value.showSharedInstance(instance.value, preview, async () => {
				await fetchInstance()
				await launchInstance(context)
			})
			return
		}
	}

	if (updateToPlayModal.value?.hasUpdate) {
		if (isSharedInstanceMember) {
			updateToPlayModal.value.show(instance.value, null, async () => {
				await fetchInstance()
				await launchInstance(context)
			})
		} else {
			updateToPlayModal.value.show(instance.value)
		}
		return
	}

	await launchInstance(context)
}

const stopInstance = async (context: string) => {
	stopping.value = true
	await kill(route.params.id as string).catch(handleError)
	stopping.value = false
	playing.value = false

	if (!instance.value) return
	trackEvent('InstanceStop', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		source: context,
	})
}

const handlePlayServer = async () => {
	if (!instance.value?.link?.project_id) return
	loading.value = true
	try {
		await playServerProject(instance.value.link.project_id)
	} finally {
		await updatePlayState()
		loading.value = false
	}
}

const repairInstance = async () => {
	if (
		instance.value.install_stage !== 'pack_installed' &&
		(instance.value.link?.type === 'modrinth_modpack' ||
			instance.value.link?.type === 'server_project_modpack')
	) {
		await install_pack_to_existing_instance(instance.value.id, {
			type: 'fromVersionId',
			project_id: instance.value.link.project_id ?? instance.value.link.server_project_id ?? '',
			version_id: instance.value.link.version_id ?? instance.value.link.content_version_id ?? '',
			title: instance.value.name,
		}).catch(handleError)
	} else {
		await install_existing_instance(instance.value.id, false).catch(handleError)
	}
}

const createShortcut = async () => {
	if (!instance.value) return
	try {
		const shortcutPath = await createInstanceShortcut(instance.value.name, instance.value.id)
		if (!shortcutPath) return

		addNotification({
			type: 'success',
			title: 'Shortcut created',
		})
	} catch (error: unknown) {
		addNotification({
			type: 'error',
			title: `Error creating shortcut`,
			text: `${error}`,
		})
	}
}

const handleRightClick = (event: MouseEvent) => {
	const baseOptions = [
		{ name: 'add_content' },
		{ type: 'divider' },
		{ name: 'edit' },
		{ name: 'open_folder' },
		{ name: 'copy_path' },
	]

	options.value?.showMenu(
		event,
		instance.value,
		playing.value
			? [
					{
						name: 'stop',
						color: 'danger',
					},
					...baseOptions,
				]
			: [
					{
						name: 'play',
						color: 'primary',
					},
					...baseOptions,
				],
	)
}

const handleOptionsClick = async (args: { option: string; item: unknown }) => {
	switch (args.option) {
		case 'play':
			await startInstance('InstancePageContextMenu')
			break
		case 'stop':
			await stopInstance('InstancePageContextMenu')
			break
		case 'add_content':
			await router.push({
				path: `/browse/${instance.value?.loader === 'vanilla' ? 'datapack' : 'mod'}`,
				query: { i: route.params.id },
			})
			break
		case 'edit':
			await router.push({
				path: `/instance/${encodeURIComponent(route.params.id as string)}/options`,
			})
			break
		case 'open_folder':
			if (instance.value) await showInstanceInFolder(instance.value.id)
			break
		case 'copy_path': {
			if (instance.value) {
				const fullPath = await get_full_path(instance.value.id)
				await navigator.clipboard.writeText(fullPath)
			}
			break
		}
	}
}

const unlistenInstances = await instance_listener(
	async (event: { instance_id: string; event: string }) => {
		if (event.instance_id !== route.params.id) return
		if (event.event === 'removed' || route.path === '/') {
			if (route.path !== '/') {
				await router.push({ path: '/' })
			}
			return
		}
		instance.value = await get(route.params.id as string).catch((err) => {
			if (String(err).includes('not managed')) {
				router.push({ path: '/' })
				return undefined
			}
			return handleError(err)
		})
		if (!instance.value?.link?.project_id) {
			linkedProjectV3.value = undefined
			isServerInstance.value = false
		}
	},
)

const unlistenProcesses = await process_listener((e: { event: string; instance_id: string }) => {
	if (e.event === 'finished' && e.instance_id === route.params.id) {
		playing.value = false
	}
})

const icon = computed(() =>
	instance.value?.icon_path ? convertFileSrc(instance.value.icon_path) : null,
)

const settingsModal = ref<InstanceType<typeof InstanceSettingsModal>>()

const timePlayed = computed(() => {
	return instance.value
		? instance.value.recent_time_played + instance.value.submitted_time_played
		: 0
})

const timePlayedHumanized = computed(() => {
	const duration = dayjs.duration(timePlayed.value, 'seconds')
	const hours = Math.floor(duration.asHours())
	if (hours >= 1) {
		return hours + ' hour' + (hours > 1 ? 's' : '')
	}

	const minutes = Math.floor(duration.asMinutes())
	if (minutes >= 1) {
		return minutes + ' minute' + (minutes > 1 ? 's' : '')
	}

	const seconds = Math.floor(duration.asSeconds())
	return seconds + ' second' + (seconds > 1 ? 's' : '')
})

onUnmounted(() => {
	unlistenProcesses()
	unlistenInstances()
	const instanceId = displayedInstanceRoute.value.params.id
	if (instanceId) {
		const { destroy } = useInstanceConsole(instanceId)
		destroy()
	}
})
</script>

<style scoped lang="scss">
.instance-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}

Button {
	width: 100%;
}

.button-group {
	display: flex;
	flex-direction: row;
	gap: 0.5rem;
}

.side-cards {
	position: fixed;
	width: 300px;
	display: flex;
	flex-direction: column;

	min-height: calc(100vh - 3.25rem);
	max-height: calc(100vh - 3.25rem);
	overflow-y: auto;
	-ms-overflow-style: none;
	scrollbar-width: none;

	&::-webkit-scrollbar {
		width: 0;
		background: transparent;
	}

	.card {
		min-height: unset;
		margin-bottom: 0;
	}
}

.instance-nav {
	display: flex;
	flex-direction: column;
	align-items: flex-start;
	justify-content: center;
	padding: 1rem;
	gap: 0.5rem;
	background: var(--color-raised-bg);
	height: 100%;
}

.name {
	font-size: 1.25rem;
	color: var(--color-contrast);
	overflow: hidden;
	text-overflow: ellipsis;
}

.metadata {
	text-transform: capitalize;
}

.instance-container {
	display: flex;
	flex-direction: row;
	overflow: auto;
	gap: 1rem;
	min-height: 100%;
	padding: 1rem;
}

.instance-info {
	display: flex;
	flex-direction: column;
	width: 100%;
}

.badge {
	display: flex;
	align-items: center;
	font-weight: bold;
	width: fit-content;
	color: var(--color-orange);
}

.pages-list {
	display: flex;
	flex-direction: column;
	gap: var(--gap-xs);

	.btn {
		font-size: 100%;
		font-weight: 400;
		background: inherit;
		transition: all ease-in-out 0.1s;
		width: 100%;
		color: var(--color-primary);
		box-shadow: none;

		&.router-link-exact-active {
			box-shadow: var(--shadow-inset-lg);
			background: var(--color-button-bg);
			color: var(--color-contrast);
		}

		&:hover {
			background-color: var(--color-button-bg);
			color: var(--color-contrast);
			box-shadow: var(--shadow-inset-lg);
			text-decoration: none;
		}

		svg {
			width: 1.3rem;
			height: 1.3rem;
		}
	}
}

.instance-nav {
	display: flex;
	flex-direction: row;
	align-items: flex-start;
	justify-content: left;
	padding: 1rem;
	gap: 0.5rem;
	height: min-content;
	width: 100%;
}

.instance-button {
	width: fit-content;
}

.actions {
	display: flex;
	flex-direction: column;
	justify-content: flex-start;
	gap: 0.5rem;
}

.content {
	margin: 0 1rem 0.5rem 20rem;
	width: calc(100% - 20rem);
	display: flex;
	flex-direction: column;
	overflow: auto;
}

.stats {
	grid-area: stats;
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	gap: var(--gap-md);

	.stat {
		display: flex;
		flex-direction: row;
		align-items: center;
		width: fit-content;
		gap: var(--gap-xs);
		--stat-strong-size: 1.25rem;

		strong {
			font-size: var(--stat-strong-size);
		}

		p {
			margin: 0;
		}

		svg {
			height: var(--stat-strong-size);
			width: var(--stat-strong-size);
		}
	}

	.date {
		margin-top: auto;
	}

	@media screen and (max-width: 750px) {
		flex-direction: row;
		column-gap: var(--gap-md);
		margin-top: var(--gap-xs);
	}

	@media screen and (max-width: 600px) {
		margin-top: 0;

		.stat-label {
			display: none;
		}
	}
}
</style>
