<template>
	<div v-if="instance" :class="{ 'flex h-full flex-col': isFixedRender }">
		<div
			:class="['p-6 pr-2 pb-4', { 'shrink-0': isFixedRender }]"
			@contextmenu.prevent.stop="(event) => handleRightClick(event)"
		>
			<ExportModal v-if="!instance.quarantined" ref="exportModal" :instance="instance" />
			<ConfirmDeleteInstanceModal ref="deleteConfirmModal" @delete="deleteSelectedInstance" />
			<InstanceSettingsModal
				:key="instance.id"
				ref="settingsModal"
				:instance="instance"
				:offline="offline"
				@unlinked="fetchInstance"
			/>
			<UpdateToPlayModal ref="updateToPlayModal" :instance="instance" />
			<SharedInstanceUpdateModal
				ref="sharedInstanceUpdateModal"
				@shared-instance-unavailable="handleSharedInstanceUnavailable"
				@report="(event) => reportSharedInstance(event, true)"
			/>
			<SharedInstanceInstallModal
				ref="sharedInstanceReportModal"
				@reported="handleSharedInstanceReported"
			/>
			<InstancePageHeader
				:instance="instance"
				:icon-src="icon"
				:is-server-instance="isServerInstance"
				:show-instance-play-time="showInstancePlayTime"
				:time-played="timePlayed"
				:playing="playing"
				:loading="loading"
				:stopping="stopping"
				:loading-server-ping="loadingServerPing"
				:players-online="playersOnline"
				:status-online="statusOnline"
				:recent-plays="recentPlays"
				:ping="ping"
				:minecraft-server="minecraftServer"
				:linked-project-v3="linkedProjectV3"
				:shared-instance-manager="sharedInstanceManager"
				@repair="() => repairInstance()"
				@stop="() => stopInstance('InstancePage')"
				@play="() => startInstance('InstancePage')"
				@play-server="() => handlePlayServer()"
				@settings="() => settingsModal?.show()"
				@open-folder="() => instance && showInstanceInFolder(instance.id)"
				@export="() => !instance.quarantined && exportModal?.show()"
				@create-shortcut="() => createShortcut()"
				@report="reportSharedInstance"
			/>
		</div>
		<div :class="['px-6', { 'shrink-0': isFixedRender }]">
			<NavTabs :links="tabs" />
			<InstanceAdmonitions
				class="mt-4"
				:instance="instance"
				:shared-instance-unavailable-reason="sharedInstanceUnavailableReason"
				:shared-instance-unavailable-manager="sharedInstanceUnavailableManager"
				:shared-instance-wrong-account="sharedInstanceWrongAccount"
				:shared-instance-expected-user-id="sharedInstanceExpectedUserId"
				:shared-instance-role="instance.shared_instance?.role"
				:shared-instance-signed-out="sharedInstanceSignedOut"
				@published="fetchInstance"
				@delete="requestInstanceDeletion"
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
							v-bind="contentSubpageProps"
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
	EditIcon,
	ExternalIcon,
	EyeIcon,
	FolderOpenIcon,
	GlobeIcon,
	HashIcon,
	PlayIcon,
	PlusIcon,
	StopCircleIcon,
	TerminalSquareIcon,
	UpdatedIcon,
	UserPlusIcon,
	XIcon,
} from '@modrinth/assets'
import { injectNotificationManager, NavTabs, useLoadingBarToken } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, onUnmounted, ref, shallowRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import InstanceAdmonitions from '@/components/ui/instance/instance-admonitions/index.vue'
import InstancePageHeader from '@/components/ui/instance-page-header/index.vue'
import ConfirmDeleteInstanceModal from '@/components/ui/modal/ConfirmDeleteInstanceModal.vue'
import InstanceSettingsModal from '@/components/ui/modal/InstanceSettingsModal.vue'
import UpdateToPlayModal from '@/components/ui/modal/UpdateToPlayModal.vue'
import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import SharedInstanceUpdateModal from '@/components/ui/shared-instances/SharedInstanceUpdateModal.vue'
import {
	fetchCachedServerStatus,
	getFreshCachedServerStatus,
} from '@/composables/instances/use-server-status-query'
import { useInstanceConsole } from '@/composables/useInstanceConsole'
import { trackEvent } from '@/helpers/analytics'
import { get_project_v3 } from '@/helpers/cache.js'
import { instance_listener, process_listener } from '@/helpers/events'
import {
	getSharedInstanceUnavailableReason,
	install_existing_instance,
	install_get_shared_instance_preview,
	install_pack_to_existing_instance,
	isSharedInstanceUnavailableError,
	type SharedInstanceUnavailableReason,
} from '@/helpers/install'
import { get, get_full_path, kill, remove, run } from '@/helpers/instance'
import { type InstanceContentData, loadInstanceContentData } from '@/helpers/instance-content'
import { get_by_instance_id } from '@/helpers/process'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'
import { createInstanceShortcut, showInstanceInFolder } from '@/helpers/utils.js'
import { refreshWorlds, type ServerStatus } from '@/helpers/worlds'
import { injectServerInstall } from '@/providers/server-install'
import { handleSevereError } from '@/store/error.js'
import { useBreadcrumbs, useTheming } from '@/store/state'

import { provideSharedInstanceState, useSharedInstanceState } from './use-shared-instance-state'

dayjs.extend(relativeTime)

const { addNotification, handleError } = injectNotificationManager()
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
const checkingSharedInstanceLaunch = ref(false)
const subpagePending = ref(false)
const stopping = ref(false)
const exportModal = ref<InstanceType<typeof ExportModal>>()
const updateToPlayModal = ref<InstanceType<typeof UpdateToPlayModal>>()
const sharedInstanceUpdateModal = ref<InstanceType<typeof SharedInstanceUpdateModal>>()
const sharedInstanceReportModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const deleteConfirmModal = ref<InstanceType<typeof ConfirmDeleteInstanceModal>>()
const selectedInstanceToDelete = ref<GameInstance | null>(null)

const { notifySharedInstanceError, notifySharedInstanceUnavailable } = useSharedInstanceErrors()

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
const sharedInstanceState = useSharedInstanceState(instance, offline, notifySharedInstanceError)
provideSharedInstanceState(sharedInstanceState)
const {
	actionsLocked: sharedInstanceActionsLocked,
	expectedUserId: sharedInstanceExpectedUserId,
	manager: sharedInstanceManager,
	refreshUpdatePreview: refreshSharedInstanceUpdatePreview,
	setUnavailable: setSharedInstanceUnavailable,
	signedOut: sharedInstanceSignedOut,
	unavailableManager: sharedInstanceUnavailableManager,
	unavailableReason: sharedInstanceUnavailableReason,
	wrongAccount: sharedInstanceWrongAccount,
} = sharedInstanceState

watch(
	() => router.currentRoute.value,
	(nextRoute) => {
		if (
			nextRoute.path.startsWith('/instance') &&
			(!instance.value || nextRoute.params.id === instance.value.id)
		) {
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
	const requestedInstanceId = route.params.id as string
	const requestedRouteName = route.name

	const nextInstance = await get(requestedInstanceId).catch(handleError)
	let nextLinkedProjectV3: Labrinth.Projects.v3.Project | undefined
	let nextIsServerInstance = false

	const contentPreloadPromise =
		nextInstance && isContentSubpageRoute(requestedRouteName)
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

	let nextPreloadedContent = await contentPreloadPromise
	let nextRoute = router.currentRoute.value
	if (nextRoute.params.id !== requestedInstanceId) return

	if (nextInstance && isContentSubpageRoute(nextRoute.name) && !nextPreloadedContent) {
		nextPreloadedContent = await loadInstanceContentData(nextInstance.id, undefined, handleError)
		nextRoute = router.currentRoute.value
		if (nextRoute.params.id !== requestedInstanceId) return
	}

	instance.value = nextInstance ?? undefined
	displayedInstanceRoute.value = nextRoute
	sharedInstanceState.reset()
	sharedInstanceState.refreshAvailability()
	linkedProjectV3.value = nextLinkedProjectV3
	isServerInstance.value = nextIsServerInstance
	preloadedContent.value = nextPreloadedContent
	activeInstanceId.value = nextInstance?.id
	resetServerStatus()

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
const showShareTab = computed(() => {
	const linkType = instance.value?.link?.type

	return (
		!instance.value?.quarantined &&
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

watch(
	() => ({
		quarantined: instance.value?.quarantined ?? false,
		routeName: router.currentRoute.value.name,
	}),
	({ quarantined, routeName }) => {
		if (quarantined && routeName === 'InstanceShare') {
			void router.replace(basePath.value)
		}
	},
	{ immediate: true },
)

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
	if (!instance.value || instance.value.quarantined) return
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

async function handleSharedInstanceUnavailable(
	reason: SharedInstanceUnavailableReason | null = null,
) {
	notifySharedInstanceUnavailable(reason, sharedInstanceUnavailableManager.value)
	await fetchInstance()
	setSharedInstanceUnavailable(reason)
}

const startInstance = async (context: string) => {
	if (!instance.value || instance.value.quarantined) return
	if (checkingSharedInstanceLaunch.value || loading.value || playing.value) return

	const instanceId = instance.value.id
	const isSharedInstanceMember = instance.value.shared_instance?.role === 'member'
	const canCheckSharedInstanceUpdate =
		!!instance.value.shared_instance && !sharedInstanceActionsLocked.value && !offline.value

	if (canCheckSharedInstanceUpdate) {
		let preview: Awaited<ReturnType<typeof refreshSharedInstanceUpdatePreview>>
		checkingSharedInstanceLaunch.value = true
		try {
			preview = await refreshSharedInstanceUpdatePreview()
		} catch (error) {
			if (isSharedInstanceUnavailableError(error)) {
				await handleSharedInstanceUnavailable(getSharedInstanceUnavailableReason(error))
			} else {
				notifySharedInstanceError(error)
			}
			return
		} finally {
			checkingSharedInstanceLaunch.value = false
		}

		if (instance.value?.id !== instanceId) return

		if (preview?.updateAvailable && sharedInstanceUpdateModal.value) {
			sharedInstanceUpdateModal.value.show(instance.value, preview, async () => {
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
	if (!instance.value?.link?.project_id || instance.value.quarantined) return
	loading.value = true
	try {
		await playServerProject(instance.value.link.project_id)
	} finally {
		await updatePlayState()
		loading.value = false
	}
}

const repairInstance = async () => {
	if (instance.value.quarantined) return
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
	if (!instance.value || instance.value.quarantined) return
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

async function reportSharedInstance(event?: MouseEvent, closeUpdateModal = false) {
	const reportInstance = instance.value
	const sharedInstance = reportInstance?.shared_instance
	if (!reportInstance || sharedInstance?.role !== 'member') return

	try {
		const preview = await install_get_shared_instance_preview(
			sharedInstance.id,
			reportInstance.name,
		)
		if (instance.value?.id !== reportInstance.id) return
		if (closeUpdateModal) sharedInstanceUpdateModal.value?.hide()
		sharedInstanceReportModal.value?.showReport(preview, event)
	} catch (error) {
		handleError(error as Error)
	}
}

function handleSharedInstanceReported(deleteInstance: boolean) {
	if (!deleteInstance || !instance.value) return
	requestInstanceDeletion()
}

function requestInstanceDeletion() {
	if (!instance.value) return
	selectedInstanceToDelete.value = instance.value
	deleteConfirmModal.value?.show()
}

async function deleteSelectedInstance() {
	const selectedInstance = selectedInstanceToDelete.value
	selectedInstanceToDelete.value = null
	if (!selectedInstance) return

	trackEvent('InstanceRemove', {
		loader: selectedInstance.loader,
		game_version: selectedInstance.game_version,
	})
	await router.push({ path: '/' })
	await remove(selectedInstance.id).catch(handleError)
}

const handleRightClick = (event: MouseEvent) => {
	const baseOptions = [
		...(instance.value?.quarantined ? [] : [{ name: 'add_content' }, { type: 'divider' }]),
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
					...(instance.value?.quarantined
						? []
						: [
								{
									name: 'play',
									color: 'primary',
								},
							]),
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
