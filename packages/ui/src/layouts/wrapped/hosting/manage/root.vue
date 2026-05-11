<template>
	<div
		v-if="filteredNotices.length > 0"
		class="relative mx-auto mb-4 flex w-full min-w-0 flex-col gap-3 px-6"
		:class="{
			'max-w-[1280px]': isNuxt,
		}"
	>
		<ServerNotice
			v-for="notice in filteredNotices"
			:key="`notice-${notice.id}`"
			:level="notice.level"
			:message="notice.message"
			:dismissable="notice.dismissable"
			:title="notice.title"
			class="w-full"
			@dismiss="() => dismissNotice(notice.id)"
		/>
	</div>
	<div
		v-if="serverData && serverData.node === null && serverData.status !== 'suspended'"
		class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
	>
		<ErrorInformationCard
			title="We're getting your server ready"
			description="Your server's hardware is being prepared and will be available shortly!"
			:icon="TransferIcon"
			icon-color="blue"
			:action="generalErrorAction"
		/>
	</div>
	<div
		v-else-if="serverData?.status === 'suspended' && serverData.suspension_reason === 'upgrading'"
		class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
	>
		<ErrorInformationCard
			title="Server upgrading"
			description="Your server's hardware is currently being upgraded and will be back online shortly!"
			:icon="TransferIcon"
			icon-color="blue"
			:action="generalErrorAction"
		/>
	</div>
	<div
		v-else-if="serverData?.status === 'suspended'"
		class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
	>
		<ErrorInformationCard
			title="Server suspended"
			:description="suspendedDescription"
			:icon="LockIcon"
			icon-color="orange"
			:action="suspendedAction"
		/>
	</div>
	<div
		v-else-if="serverError?.statusCode === 403 || serverError?.statusCode === 404"
		class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
	>
		<ErrorInformationCard
			title="An error occured."
			description="Please contact Modrinth Support."
			:icon="TransferIcon"
			icon-color="orange"
			:error-details="generalErrorDetails"
			:action="generalErrorAction"
		/>
	</div>
	<div
		v-else-if="serverError || !nodeAccessible"
		class="flex min-h-[calc(100vh-4rem)] items-center justify-center text-contrast"
	>
		<ErrorInformationCard
			title="Server Node Unavailable"
			:icon="TriangleAlertIcon"
			icon-color="red"
			:action="nodeUnavailableAction"
			:error-details="nodeUnavailableDetails"
		>
			<template #description>
				<div class="text-md space-y-4">
					<p class="leading-[170%] text-secondary">
						Your server's node, where your Modrinth Server is physically hosted, is not accessible
						at the moment. We are working to resolve the issue as quickly as possible.
					</p>
					<p class="leading-[170%] text-secondary">
						Your data is safe and will not be lost, and your server will be back online as soon as
						the issue is resolved.
					</p>
					<p class="leading-[170%] text-secondary">
						If reloading does not work initially, please contact Modrinth Support via the chat
						bubble in the bottom right corner and we'll be happy to help.
					</p>
				</div>
			</template>
		</ErrorInformationCard>
	</div>
	<!-- SERVER START -->
	<div
		v-else-if="serverData"
		data-pyro-server-manager-root
		class="relative mx-auto box-border flex w-full min-w-0 flex-col gap-4 px-6 transition-all duration-300"
		:style="{
			'--server-bg-image': serverImage
				? `url(${serverImage})`
				: `linear-gradient(180deg, rgba(153,153,153,1) 0%, rgba(87,87,87,1) 100%)`,
		}"
		:class="[
			'server-panel-' + revealState,
			isNuxt ? 'min-h-[100svh] max-w-[1280px] pb-16' : 'min-h-[calc(100svh-100px)] pb-6',
		]"
	>
		<template v-if="revealState !== 'pending' || isOnboarding">
			<ServerManageHeader
				v-if="!isOnboarding"
				class="server-stagger-item"
				:style="{ '--si': 0 }"
				:server="serverData"
				:server-image="serverImage"
				:server-project="serverProject"
				:uptime-seconds="showUptime ? uptimeSeconds : undefined"
			>
				<template #actions>
					<div class="flex gap-2">
						<PanelServerActionButton :disabled="!!installError" />
						<Tooltip
							theme="dismissable-prompt"
							:triggers="[]"
							:shown="showSettingsHint"
							:auto-hide="false"
							placement="bottom-end"
						>
							<ButtonStyled circular size="large">
								<button
									v-tooltip="showSettingsHint ? undefined : 'Server settings'"
									@click="
										() => {
											openServerSettingsModal()
											dismissSettingsHint()
										}
									"
								>
									<SettingsIcon />
								</button>
							</ButtonStyled>
							<template #popper>
								<div class="grid grid-cols-[min-content] gap-1">
									<div class="flex min-w-48 items-center justify-between gap-8">
										<h3 class="m-0 whitespace-nowrap text-base font-bold text-contrast">
											{{ formatMessage(settingsHintMessages.title) }}
										</h3>
										<ButtonStyled size="small" circular>
											<button
												v-tooltip="formatMessage(settingsHintMessages.dismiss)"
												@click="dismissSettingsHint"
											>
												<XIcon aria-hidden="true" />
											</button>
										</ButtonStyled>
									</div>
									<p class="m-0 text-wrap text-sm font-medium leading-tight text-secondary">
										{{ formatMessage(settingsHintMessages.description) }}
									</p>
								</div>
							</template>
						</Tooltip>
						<PanelServerOverflowMenu
							:disabled="!!installError"
							:uptime-seconds="uptimeSeconds"
							:show-copy-id-action="showCopyIdAction"
							:show-debug-info="showAdvancedDebugInfo"
						/>
					</div>
				</template>
			</ServerManageHeader>

			<ServerOnboardingPanelPage v-if="isOnboarding" :browse-modpacks="handleBrowseModpacks" />

			<template v-else>
				<div
					data-pyro-navigation
					class="server-stagger-item isolate flex w-full select-none flex-col justify-between gap-4 overflow-auto md:flex-row md:items-center"
					:style="{ '--si': 1 }"
				>
					<NavTabs :links="navLinks" replace />
				</div>

				<div
					data-pyro-mount
					class="server-stagger-item h-full w-full flex-1"
					:style="{ '--si': 2 }"
				>
					<div
						v-if="installError"
						class="mx-auto mb-4 flex justify-between gap-2 rounded-2xl border-2 border-solid border-red bg-bg-red p-4 font-semibold text-contrast"
					>
						<div class="flex flex-row gap-4">
							<IssuesIcon class="hidden h-8 w-8 shrink-0 text-red sm:block" />
							<div class="flex flex-col gap-2 leading-[150%]">
								<div class="flex items-center gap-3">
									<IssuesIcon class="flex h-8 w-8 shrink-0 text-red sm:hidden" />
									<div class="flex gap-2 text-2xl font-bold">{{ errorTitle }}</div>
								</div>

								<div
									v-if="errorTitle.toLocaleLowerCase() === 'installation error'"
									class="font-normal"
								>
									<div
										v-if="
											errorMessage.toLocaleLowerCase() === 'the specified version may be incorrect'
										"
									>
										An invalid loader or Minecraft version was specified and could not be installed.
										<ul class="m-0 mt-4 p-0 pl-4">
											<li>
												If this version of Minecraft was released recently, please check if Modrinth
												Hosting supports it.
											</li>
											<li>
												If you've installed a modpack, it may have been packaged incorrectly or may
												not be compatible with the loader.
											</li>
											<li>
												Your server may need to be reinstalled with a valid mod loader and version.
												You can change the loader by clicking the "Change Loader" button.
											</li>
											<li>
												If you're stuck, please contact Modrinth Support with the information below:
											</li>
										</ul>
										<ButtonStyled>
											<button class="mt-2" @click="copyServerDebugInfo">
												<CopyIcon v-if="!copied" />
												<CheckIcon v-else />
												Copy Debug Info
											</button>
										</ButtonStyled>
									</div>
									<div v-if="errorMessage.toLocaleLowerCase() === 'internal error'">
										An internal error occurred while installing your server. Don't fret — try
										reinstalling your server, and if the problem persists, please contact Modrinth
										support with your server's debug information.
									</div>
									<div
										v-if="errorMessage.toLocaleLowerCase() === 'this version is not yet supported'"
									>
										An error occurred while installing your server because Modrinth Hosting does not
										support the version of Minecraft or the loader you specified. Try reinstalling
										your server with a different version or loader, and if the problem persists,
										please contact Modrinth Support with your server's debug information.
									</div>

									<div
										v-if="errorTitle === 'Installation error'"
										class="mt-2 flex flex-col gap-4 sm:flex-row"
									>
										<ButtonStyled v-if="errorLog">
											<button @click="openInstallLog"><FileIcon />Open Installation Log</button>
										</ButtonStyled>
										<ButtonStyled>
											<button @click="copyServerDebugInfo">
												<CopyIcon v-if="!copied" />
												<CheckIcon v-else />
												Copy Debug Info
											</button>
										</ButtonStyled>
										<ButtonStyled color="red" type="standard">
											<button
												class="whitespace-pre"
												@click="openServerSettingsModal('installation')"
											>
												<RightArrowIcon />
												Change Loader
											</button>
										</ButtonStyled>
									</div>
								</div>
							</div>
						</div>
					</div>

					<div v-if="serverData.is_medal" class="mb-4">
						<MedalServerCountdown
							:server-id="serverId"
							:stripe-publishable-key="stripePublishableKey"
							:site-url="siteUrl"
							:products="products"
						/>
					</div>

					<div
						v-if="!isConnected && !isReconnecting && !isLoading"
						data-pyro-server-ws-error
						class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-red p-4 text-contrast"
					>
						<IssuesIcon class="size-5 text-red" />
						Something went wrong...
					</div>

					<div
						v-if="isReconnecting"
						data-pyro-server-ws-reconnecting
						class="mb-4 flex w-full flex-row items-center gap-4 rounded-2xl bg-bg-orange p-4 text-sm text-contrast"
					>
						<LoaderCircleIcon class="h-5 w-5 animate-spin" />
						Hang on, we're reconnecting to your server.
					</div>

					<ServerPanelAdmonitions
						class="mb-4"
						:sync-progress="syncProgress"
						:content-error="contentError"
						@content-retry="handleContentRetry"
					/>
					<slot :on-reinstall="onReinstall" :on-reinstall-failed="onReinstallFailed" />
				</div>
			</template>
		</template>
	</div>
	<div
		v-if="showAdvancedDebugInfo"
		class="relative mx-auto mt-6 box-border w-full min-w-0 max-w-[1280px] px-6"
	>
		<h2 class="m-0 text-lg font-extrabold text-contrast">Server data</h2>
		<pre class="markdown-body w-full overflow-auto rounded-2xl bg-bg-raised p-4 text-sm">{{
			safeStringify(serverData)
		}}</pre>
	</div>
	<Suspense>
		<ServerSettingsModal
			ref="serverSettingsModal"
			:resolve-viewer="resolveViewer"
			:browse-modpacks="handleBrowseModpacks"
		/>
	</Suspense>
	<ConfirmLeaveModal
		ref="confirmLeaveModal"
		:header="formatMessage(leaveMessages.uploadInProgress)"
		:body="formatMessage(leaveMessages.leavePageBody)"
		admonition-type="critical"
	/>
</template>

<script setup lang="ts">
import { Intercom, shutdown } from '@intercom/messenger-js-sdk'
import type { Archon, Labrinth } from '@modrinth/api-client'
import { ModrinthApiError, NuxtModrinthClient } from '@modrinth/api-client'
import {
	BoxesIcon,
	CheckIcon,
	CopyIcon,
	DatabaseBackupIcon,
	FileIcon,
	FolderOpenIcon,
	IssuesIcon,
	LayoutTemplateIcon,
	LoaderCircleIcon,
	LockIcon,
	RightArrowIcon,
	SettingsIcon,
	TransferIcon,
	TriangleAlertIcon,
	XIcon,
} from '@modrinth/assets'
import type { Stats } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage, useTimeoutFn } from '@vueuse/core'
import DOMPurify from 'dompurify'
import { Tooltip } from 'floating-vue'
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router'

import ButtonStyled from '#ui/components/base/ButtonStyled.vue'
import ErrorInformationCard from '#ui/components/base/ErrorInformationCard.vue'
import NavTabs from '#ui/components/base/NavTabs.vue'
import ServerNotice from '#ui/components/base/ServerNotice.vue'
import ConfirmLeaveModal from '#ui/components/modal/ConfirmLeaveModal.vue'
import ServerPanelAdmonitions from '#ui/components/servers/admonitions/ServerPanelAdmonitions.vue'
import MedalServerCountdown from '#ui/components/servers/marketing/MedalServerCountdown.vue'
import {
	PanelServerActionButton,
	PanelServerOverflowMenu,
	ServerManageHeader,
} from '#ui/components/servers/server-header'
import ServerSettingsModal from '#ui/components/servers/ServerSettingsModal.vue'
import {
	useDebugLogger,
	useLoadingBarToken,
	useModrinthServersConsole,
	useReadyState,
	useServerImage,
	useServerProject,
} from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerBackupsQueue } from '#ui/composables/server-backups-queue'
import { useServerManageCoreRuntime } from '#ui/composables/server-manage-core-runtime'
import type { LogLine } from '#ui/layouts/shared/console'
import type { ServerSettingsTabId } from '#ui/layouts/shared/server-settings'
import {
	injectModrinthClient,
	injectNotificationManager,
	provideServerSettingsModal,
} from '#ui/providers'
import { formatLoaderLabel } from '#ui/utils/loaders'
import {
	pendingServerContentInstallsEvent,
	readPendingServerContentInstalls,
	writePendingServerContentInstalls,
} from '#ui/utils/server-content-installing'

import ServerOnboardingPanelPage from './[id]/onboarding.vue'

interface Tab {
	label: string
	href: string
	icon?: object
	subpages?: string[]
}

const props = withDefaults(
	defineProps<{
		serverId: string
		reloadPage: () => void
		resolveViewer: () => Promise<{ userId: string | null; userRole: string | null }>
		showCopyIdAction?: boolean
		showAdvancedDebugInfo?: boolean
		showUptime?: boolean
		additionalTabs?: Tab[]
		stripePublishableKey?: string
		siteUrl?: string
		products?: Labrinth.Billing.Internal.Product[]
		authUser?: { id: string; username: string; email: string; created: string }
		fetchIntercomToken?: () => Promise<{ token: string }>
		intercomAppId?: string
		navigateToBilling?: () => void
		navigateToServers?: () => void
		browseModpacks?: (args: {
			serverId: string
			worldId: string | null
			from: 'reset-server' | 'onboarding'
		}) => void | Promise<void>
		browseContent?: (args: {
			serverId: string
			worldId: string | null
			type: 'mod' | 'plugin' | 'datapack'
		}) => void | Promise<void>
	}>(),
	{
		showCopyIdAction: false,
		showAdvancedDebugInfo: false,
		showUptime: true,
		additionalTabs: () => [],
		stripePublishableKey: undefined,
		siteUrl: undefined,
		products: () => [],
		authUser: undefined,
		fetchIntercomToken: undefined,
		intercomAppId: 'ykeritl9',
		navigateToBilling: undefined,
		navigateToServers: undefined,
		browseModpacks: undefined,
		browseContent: undefined,
	},
)

const { formatMessage } = useVIntl()

const leaveMessages = defineMessages({
	uploadInProgress: {
		id: 'servers.manage.confirm-leave.upload-in-progress',
		defaultMessage: 'Upload in progress',
	},
	leavePageBody: {
		id: 'servers.manage.confirm-leave.body',
		defaultMessage: 'A file upload is in progress. Leaving this page will cancel the upload.',
	},
})

const settingsHintMessages = defineMessages({
	title: {
		id: 'servers.manage.settings-hint.title',
		defaultMessage: 'Your server settings have moved',
	},
	description: {
		id: 'servers.manage.settings-hint.description',
		defaultMessage: 'They can now be found here!',
	},
	dismiss: {
		id: 'servers.manage.settings-hint.dismiss',
		defaultMessage: "Don't show again",
	},
})

// disabled, keeping the animation logic cos it's really nice and we might want to re-enable in future
const DISABLE_LOADING_ANIM = true

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const isNuxt = computed(() => client instanceof NuxtModrinthClient)
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()
const debug = useDebugLogger('ServerManage')

const isReconnecting = ref(false)
const isLoading = ref(true)
const isMounted = ref(true)
const copied = ref(false)
const installError = ref<Error | null>(null)
const errorTitle = ref('Error')
const errorMessage = ref('An unexpected error occurred.')
const errorLog = ref('')
const errorLogFile = ref('')
const isOnboarding = computed(() => serverData.value?.flows?.intro)

const SETTINGS_HINT_KEY = 'server-panel-settings-hint-dismissed'
const settingsHintDismissed = useStorage(SETTINGS_HINT_KEY, false)
const showSettingsHint = ref(!settingsHintDismissed.value)
function dismissSettingsHint() {
	showSettingsHint.value = false
	settingsHintDismissed.value = true
}

const serverSettingsModal = ref<InstanceType<typeof ServerSettingsModal> | null>(null)
const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal>>()

const {
	data: serverData,
	error: serverQueryError,
	isLoading: serverLoading,
} = useQuery({
	queryKey: ['servers', 'detail', props.serverId],
	queryFn: () => client.archon.servers_v0.get(props.serverId)!,
})

useLoadingBarToken(useReadyState({ isLoading: serverLoading, data: serverData }))

function updateServerData(patch: Partial<Archon.Servers.v0.Server>) {
	if (!serverData.value) return
	queryClient.setQueryData(['servers', 'detail', props.serverId], {
		...serverData.value,
		...patch,
	})
}

const serverError = computed(() => {
	const err = serverQueryError.value
	if (err instanceof ModrinthApiError) return err
	return err ? ModrinthApiError.fromUnknown(err) : null
})

const { data: serverFull } = useQuery({
	queryKey: ['servers', 'v1', 'detail', props.serverId],
	queryFn: () => client.archon.servers_v1.get(props.serverId),
})

const worldId = computed(() => {
	if (!serverFull.value) return null
	const activeWorld = serverFull.value.worlds.find((w) => w.is_active)
	return activeWorld?.id ?? serverFull.value.worlds[0]?.id ?? null
})

const { handleWsBackupProgress, busyReasons: backupsBusy } = useServerBackupsQueue(
	computed(() => props.serverId),
	worldId,
)

const { image: serverImage } = useServerImage(
	props.serverId,
	computed(() => serverData.value?.upstream ?? null),
)
const { data: serverProject } = useServerProject(computed(() => serverData.value?.upstream ?? null))

const syncProgress = ref<Archon.Websocket.v0.SyncContentProgress | null>(null)
const contentError = ref<Archon.Websocket.v0.SyncContentError | null>(null)
const syncProgressActive = ref(false)
const hasPendingServerContentInstalls = ref(false)
const hasSeenPendingServerContentSync = ref(false)
const isAwaitingPostInstallRefresh = ref(false)
const { start: startSyncHide, stop: cancelSyncHide } = useTimeoutFn(
	() => (syncProgressActive.value = false),
	1000,
	{ immediate: false },
)

watch(syncProgress, (progress) => {
	if (progress != null) {
		cancelSyncHide()
		syncProgressActive.value = true
		if (progress.phase !== 'Analyzing' && hasPendingServerContentInstalls.value) {
			hasSeenPendingServerContentSync.value = true
		}
	} else if (syncProgressActive.value) {
		startSyncHide()
		if (hasSeenPendingServerContentSync.value) {
			writePendingServerContentInstalls(props.serverId, worldId.value, [])
			hasSeenPendingServerContentSync.value = false
		}
	}
})

watch(contentError, (error) => {
	if (!error || !hasPendingServerContentInstalls.value) return
	writePendingServerContentInstalls(props.serverId, worldId.value, [])
	hasSeenPendingServerContentSync.value = false
})

const isSyncingContent = computed(
	() =>
		syncProgressActive.value ||
		isAwaitingPostInstallRefresh.value ||
		hasPendingServerContentInstalls.value,
)

function syncPendingServerContentInstalls() {
	hasPendingServerContentInstalls.value =
		readPendingServerContentInstalls(props.serverId, worldId.value).length > 0
}

function handlePendingServerContentInstallsChanged(event: Event) {
	const detail = (event as CustomEvent<{ serverId?: string | null; worldId?: string | null }>)
		.detail
	if (detail?.serverId !== props.serverId || detail?.worldId !== worldId.value) return
	syncPendingServerContentInstalls()
}

watch(worldId, syncPendingServerContentInstalls, { immediate: true })

let hasSeenInstallProgress = false

const onStateEvent = (data: Archon.Websocket.v0.WSStateEvent) => {
	debug('[root.vue] handleState received:', {
		power_variant: data.power_variant,
		progress: data.progress,
		serverStatus: serverData.value?.status,
	})
	hasReceivedWsData.value = true
	syncProgress.value = data.progress
	contentError.value = data.content_error

	if (serverData.value) {
		if (data.progress != null && serverData.value.status !== 'installing') {
			debug('[root.vue] handleState: progress != null, setting status to installing')
			hasSeenInstallProgress = true
			updateServerData({ status: 'installing' })
		} else if (data.progress != null) {
			hasSeenInstallProgress = true
		} else if (
			data.progress == null &&
			data.content_error == null &&
			serverData.value.status === 'installing' &&
			hasSeenInstallProgress
		) {
			debug('[root.vue] handleState: progress null + was installing, applying optimistic update')
			hasSeenInstallProgress = false
			applyOptimisticCompletion()
			invalidateAfterInstall()
		}
	}
}

const {
	cancelUpload,
	cleanupCoreRuntime,
	connectSocket,
	cpuData,
	fsOps,
	fsQueuedOps,
	isConnected,
	ramData,
	serverPowerState,
	stats,
	uptimeSeconds,
	uploadState,
} = useServerManageCoreRuntime({
	serverId: computed(() => props.serverId),
	worldId,
	server: serverData,
	isSyncingContent,
	extraBusyReasons: backupsBusy,
	setDisconnectedOnAuthIncorrect: false,
	syncUptimeFromState: true,
	incrementUptimeLocally: true,
	eventGuard: () => isMounted.value,
	onStateEvent,
})

const isUploading = computed(() => uploadState.value.isUploading)

function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (isUploading.value) {
		e.preventDefault()
		return ''
	}
}

if (typeof window !== 'undefined') {
	watch(isUploading, (uploading) => {
		if (uploading) {
			window.addEventListener('beforeunload', handleBeforeUnload)
		} else {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		}
	})

	onBeforeUnmount(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload)
	})

	onBeforeRouteLeave(async () => {
		if (isUploading.value) {
			const shouldLeave = (await confirmLeaveModal.value?.prompt()) ?? false
			if (shouldLeave) cancelUpload.value?.()
			return shouldLeave
		}
		return true
	})
}

type CachedWsState = {
	stats: Stats
	cpuData: number[]
	ramData: number[]
	powerState: Archon.Websocket.v0.PowerState
	uptimeSeconds: number
	consoleLines: LogLine[]
}

const modrinthServersConsole = useModrinthServersConsole()
const wsStateCacheKey = ['servers', 'ws-state', props.serverId] as const
const cachedWsState = queryClient.getQueryData<CachedWsState>(wsStateCacheKey)
if (cachedWsState) {
	stats.value = cachedWsState.stats
	cpuData.value = cachedWsState.cpuData
	ramData.value = cachedWsState.ramData
	serverPowerState.value = cachedWsState.powerState
	uptimeSeconds.value = cachedWsState.uptimeSeconds
}

const log = useDebugLogger('server-panel-reveal')

const hasReceivedWsData = ref(!!cachedWsState)
log('init', {
	hasCachedWsState: !!cachedWsState,
	hasReceivedWsData: hasReceivedWsData.value,
	isConnected: isConnected.value,
	serverData: !!serverData.value,
})

const saveWsStateToCache = () => {
	if (!hasReceivedWsData.value) return
	queryClient.setQueryData(wsStateCacheKey, {
		stats: stats.value,
		cpuData: cpuData.value,
		ramData: ramData.value,
		powerState: serverPowerState.value,
		uptimeSeconds: uptimeSeconds.value,
		consoleLines: modrinthServersConsole.output.value,
	} satisfies CachedWsState)
}

watch([stats, serverPowerState], () => {
	if (!isConnected.value) return
	hasReceivedWsData.value = true
})

const canReveal = computed(() => serverData.value && hasReceivedWsData.value)
log('canReveal initial', {
	canReveal: canReveal.value,
	serverData: !!serverData.value,
	hasReceivedWsData: hasReceivedWsData.value,
})

const revealState = ref<'pending' | 'revealing' | 'visible'>(
	DISABLE_LOADING_ANIM || canReveal.value ? 'visible' : 'pending',
)
log('revealState initial', revealState.value)

const REVEAL_TOTAL_MS = 2 * 80 + 400

watch(canReveal, (ready) => {
	log('canReveal changed', { ready, revealState: revealState.value })
	if (ready && revealState.value === 'pending') {
		if (DISABLE_LOADING_ANIM) {
			revealState.value = 'visible'
		} else {
			revealState.value = 'revealing'
			setTimeout(() => {
				revealState.value = 'visible'
				log('revealState -> visible')
			}, REVEAL_TOTAL_MS)
		}
	}
})

watch(isConnected, (connected) => {
	log('isConnected changed', connected)
})

watch(serverData, (data) => {
	log('serverData changed', !!data)
})

const navLinks = computed<Tab[]>(() => [
	{
		label: 'Overview',
		href: `/hosting/manage/${props.serverId}`,
		icon: LayoutTemplateIcon,
		subpages: [],
	},
	{
		label: 'Content',
		href: `/hosting/manage/${props.serverId}/content`,
		icon: BoxesIcon,
		subpages: ['mods', 'datapacks'],
	},
	{
		label: 'Files',
		href: `/hosting/manage/${props.serverId}/files`,
		icon: FolderOpenIcon,
		subpages: [],
	},
	{
		label: 'Backups',
		href: `/hosting/manage/${props.serverId}/backups`,
		icon: DatabaseBackupIcon,
		subpages: [],
	},
	...props.additionalTabs,
])

const filteredNotices = computed(
	() => serverData.value?.notices?.filter((n) => n.level !== 'survey') ?? [],
)
const surveyNotice = computed(() => serverData.value?.notices?.find((n) => n.level === 'survey'))

async function dismissNotice(noticeId: number) {
	await client.archon.servers_v0.dismissNotice(props.serverId, noticeId).catch((err) => {
		addNotification({
			title: 'Error dismissing notice',
			text: err,
			type: 'error',
		})
	})
	await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
}

async function dismissSurvey() {
	const noticeId = surveyNotice.value?.id
	if (noticeId === undefined) return
	await dismissNotice(noticeId)
}

type TallyPopupOptions = {
	key?: string
	layout?: 'default' | 'modal'
	width?: number
	alignLeft?: boolean
	hideTitle?: boolean
	overlay?: boolean
	emoji?: {
		text: string
		animation:
			| 'none'
			| 'wave'
			| 'tada'
			| 'heart-beat'
			| 'spin'
			| 'flash'
			| 'bounce'
			| 'rubber-band'
			| 'head-shake'
	}
	autoClose?: number
	showOnce?: boolean
	doNotShowAfterSubmit?: boolean
	customFormUrl?: string
	hiddenFields?: { [key: string]: unknown }
	onOpen?: () => void
	onClose?: () => void
	onPageView?: (page: number) => void
	onSubmit?: (payload: unknown) => void
}

const popupOptions = computed(
	() =>
		({
			layout: 'default',
			width: 400,
			autoClose: 2000,
			hideTitle: true,
			hiddenFields: {
				username: props.authUser?.username,
				user_id: props.authUser?.id,
				user_email: props.authUser?.email,
				server_id: serverData.value?.server_id,
				loader: serverData.value?.loader,
				game_version: serverData.value?.mc_version,
				modpack_id: serverProject.value?.id,
				modpack_name: serverProject.value?.title,
			},
			onOpen: () => console.log(`Opened survey notice: ${surveyNotice.value?.id}`),
			onClose: async () => await dismissSurvey(),
			onSubmit: (payload: unknown) => {
				console.log('Form submitted:', payload)
			},
		}) satisfies TallyPopupOptions,
)

function getTally(): { openPopup?: (id: string, opts: TallyPopupOptions) => void } | undefined {
	return (
		window as Window & { Tally?: { openPopup?: (id: string, opts: TallyPopupOptions) => void } }
	).Tally
}

function showSurvey() {
	if (!surveyNotice.value) return

	try {
		const tally = getTally()
		if (tally?.openPopup) {
			tally.openPopup(surveyNotice.value.message, popupOptions.value)
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
	}
}

function loadTallyScript() {
	if (document.querySelector('script[src*="tally.so"]')) return
	const script = document.createElement('script')
	script.src = 'https://tally.so/widgets/embed.js'
	script.defer = true
	document.head.appendChild(script)
}

async function handleContentRetry() {
	if (!worldId.value) return
	try {
		await client.archon.content_v1.repair(props.serverId, worldId.value)
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to retry installation',
		})
	}
}

const handleBackupProgress = (data: Archon.Websocket.v0.WSBackupProgressEvent) => {
	handleWsBackupProgress(data)
}

const handleFilesystemOps = (data: Archon.Websocket.v0.WSFilesystemOpsEvent) => {
	const allOps = data.all

	if (JSON.stringify(fsOps.value) !== JSON.stringify(allOps)) {
		fsOps.value = allOps
	}

	fsQueuedOps.value = fsQueuedOps.value.filter(
		(queuedOp) => !allOps.some((x) => x.src === queuedOp.src),
	)

	const cancelled = allOps.filter((x) => x.state === 'cancelled')
	Promise.all(
		cancelled.map((x) =>
			client.kyros.files_v0.modifyOperation(x.id, 'dismiss').catch((error) => {
				console.error('Failed to dismiss cancelled operation:', error)
			}),
		),
	)
}

const handleNewMod = () => {
	queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
}

const handleInstallationResult = async (data: Archon.Websocket.v0.WSInstallationResultEvent) => {
	debug('[root.vue] handleInstallationResult received:', data)
	switch (data.result) {
		case 'ok': {
			debug('[root.vue] handleInstallationResult: ok received')
			if (!serverData.value) break

			applyOptimisticCompletion()
			installError.value = null
			invalidateAfterInstall()

			break
		}
		case 'err': {
			console.log('failed to install')
			console.log(data)
			errorTitle.value = 'Installation error'
			errorMessage.value = data.reason ?? 'Unknown error'
			installError.value = new Error(data.reason ?? 'Unknown error')

			try {
				let files = await client.kyros.files_v0.listDirectory('/', 1, 100)
				if (files && files.total > 1) {
					for (let i = 2; i <= files.total; i++) {
						const nextFiles = await client.kyros.files_v0.listDirectory('/', i, 100)
						if (nextFiles?.items?.length === 0) break
						if (nextFiles) files = nextFiles
					}
				}
				const fileName = files?.items?.find((file) =>
					file.name.startsWith('modrinth-installation'),
				)?.name
				errorLogFile.value = fileName ?? ''
				if (fileName) {
					const content = await client.kyros.files_v0.downloadFile(fileName)
					errorLog.value = await content.text()
				}
			} catch (err) {
				console.error('Failed to fetch installation log:', err)
			}
			break
		}
	}
}

const newLoader = ref<string | null>(null)
const newLoaderVersion = ref<string | null>(null)
const newMCVersion = ref<string | null>(null)

const onReinstall = async (
	potentialArgs: { loader?: string; lVersion?: string; mVersion?: string } | undefined,
) => {
	debug('[root.vue] onReinstall called with:', potentialArgs)

	if (serverData.value?.flows?.intro) {
		await client.archon.servers_v1.endIntro(props.serverId)
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
	}

	if (!serverData.value) return

	debug('[root.vue] onReinstall: setting serverData.status to installing')
	hasSeenInstallProgress = false
	updateServerData({ status: 'installing' })

	if (potentialArgs?.loader) {
		newLoader.value = potentialArgs.loader
	}
	if (potentialArgs?.lVersion) {
		newLoaderVersion.value = potentialArgs.lVersion
	}
	if (potentialArgs?.mVersion) {
		newMCVersion.value = potentialArgs.mVersion
	}

	installError.value = null
	errorTitle.value = 'Error'
	errorMessage.value = 'An unexpected error occurred.'

	modrinthServersConsole.clear()

	debug('[root.vue] onReinstall: triggering immediate invalidation')
	queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
	queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
}

const onReinstallFailed = () => {
	debug('[root.vue] onReinstallFailed: reverting status to available')
	updateServerData({ status: 'available' })
	newLoader.value = null
	newLoaderVersion.value = null
	newMCVersion.value = null
}

function applyOptimisticCompletion() {
	const patch: Partial<Archon.Servers.v0.Server> = { status: 'available' }
	if (newLoader.value) patch.loader = formatLoaderLabel(newLoader.value) as Archon.Servers.v0.Loader
	if (newLoaderVersion.value) patch.loader_version = newLoaderVersion.value
	if (newMCVersion.value) patch.mc_version = newMCVersion.value

	debug('[root.vue] applyOptimisticCompletion: patch:', patch)
	updateServerData(patch)

	const addonsQueries = queryClient.getQueriesData<Archon.Content.v1.Addons>({
		queryKey: ['content', 'list', 'v1', props.serverId],
	})
	for (const [key, data] of addonsQueries) {
		if (!data) continue
		const addonsPatch: Record<string, string> = {}
		if (newLoader.value) addonsPatch.modloader = newLoader.value
		if (newLoaderVersion.value) addonsPatch.modloader_version = newLoaderVersion.value
		if (newMCVersion.value) addonsPatch.game_version = newMCVersion.value
		if (Object.keys(addonsPatch).length > 0) {
			queryClient.setQueryData(key, { ...data, ...addonsPatch })
		}
	}

	newLoader.value = null
	newLoaderVersion.value = null
	newMCVersion.value = null
}

async function invalidateAfterInstall() {
	debug('[root.vue] invalidateAfterInstall: scheduling 2s delayed invalidation')
	isAwaitingPostInstallRefresh.value = true
	setTimeout(async () => {
		try {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] }),
				queryClient.invalidateQueries({
					queryKey: ['servers', 'startup', 'v1', props.serverId],
				}),
				queryClient.invalidateQueries({ queryKey: ['content', 'list'] }),
			])
		} catch (err: unknown) {
			console.error('Error refreshing data after installation:', err)
		} finally {
			isAwaitingPostInstallRefresh.value = false
		}
	}, 2000)
}

const nodeAccessible = ref(true)

const nodeUnavailableDetails = computed(() => [
	{
		label: 'Server ID',
		value: props.serverId,
		type: 'inline' as const,
	},
	{
		label: 'Node',
		value:
			(serverError.value?.responseData as { hostname?: string } | undefined)?.hostname ??
			serverData.value?.datacenter ??
			'Unknown',
		type: 'inline' as const,
	},
	{
		label: 'Error message',
		value: nodeAccessible.value
			? (serverError.value?.message ?? 'Unknown')
			: 'Unable to reach node. Ping test failed.',
		type: 'block' as const,
	},
])

const suspendedDescription = computed(() => {
	if (serverData.value?.suspension_reason === 'cancelled') {
		return 'Your subscription has been cancelled.\nContact Modrinth Support if you believe this is an error.'
	}
	if (serverData.value?.suspension_reason) {
		return `Your server has been suspended: ${serverData.value.suspension_reason}\nContact Modrinth Support if you believe this is an error.`
	}
	return 'Your server has been suspended.\nContact Modrinth Support if you believe this is an error.'
})

const generalErrorDetails = computed(() => [
	{
		label: 'Server ID',
		value: props.serverId,
		type: 'inline' as const,
	},
	{
		label: 'Timestamp',
		value: String(new Date().toISOString()),
		type: 'inline' as const,
	},
	{
		label: 'Error Name',
		value: serverError.value?.name,
		type: 'inline' as const,
	},
	{
		label: 'Error Message',
		value: serverError.value?.message,
		type: 'block' as const,
	},
	...(serverError.value?.originalError
		? [
				{
					label: 'Original Error',
					value: String(serverError.value.originalError),
					type: 'hidden' as const,
				},
			]
		: []),
	...(serverError.value?.stack
		? [
				{
					label: 'Stack Trace',
					value: serverError.value.stack,
					type: 'hidden' as const,
				},
			]
		: []),
])

const suspendedAction = computed(() => ({
	label: 'Go to billing settings',
	onClick: () => props.navigateToBilling?.(),
	color: 'brand' as const,
}))

const generalErrorAction = computed(() => ({
	label: 'Go back to all servers',
	onClick: () => props.navigateToServers?.(),
	color: 'brand' as const,
}))

const nodeUnavailableAction = computed(() => ({
	label: 'Reload',
	onClick: () => props.reloadPage(),
	color: 'brand' as const,
	disabled: false,
}))

const copyServerDebugInfo = () => {
	const debugInfo = `Server ID: ${serverData.value?.server_id}\nError: ${errorMessage.value}\nKind: ${serverData.value?.upstream?.kind}\nProject ID: ${serverData.value?.upstream?.project_id}\nVersion ID: ${serverData.value?.upstream?.version_id}\nLog: ${errorLog.value}`
	navigator.clipboard.writeText(debugInfo)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 5000)
}

const openInstallLog = () => {
	const url = `/hosting/manage/${props.serverId}/files?editing=${encodeURIComponent(errorLogFile.value)}`
	window.history.pushState({}, '', url)
	window.dispatchEvent(new PopStateEvent('popstate'))
}

function openServerSettingsModal(tabId?: ServerSettingsTabId) {
	if (!props.serverId) return
	serverSettingsModal.value?.show({ serverId: props.serverId, tabId })
}

function handleBrowseModpacks(args: {
	serverId: string
	worldId: string | null
	from: 'reset-server' | 'onboarding'
}) {
	props.browseModpacks?.(args)
}

function handleBrowseContent(args: {
	serverId: string
	worldId: string | null
	type: 'mod' | 'plugin' | 'datapack'
}) {
	props.browseContent?.(args)
}

provideServerSettingsModal({
	openServerSettings: (options) => openServerSettingsModal(options?.tabId),
	browseServerContent: (args) => handleBrowseContent(args),
})

function safeStringify(obj: unknown, indent = ' '): string {
	const seen = new WeakSet()
	return JSON.stringify(
		obj,
		(_key, value) => {
			if (typeof value === 'object' && value !== null) {
				if (seen.has(value)) {
					return '[Circular]'
				}
				seen.add(value)
			}
			return value
		},
		indent,
	)
}

async function testNodeReachability(): Promise<boolean> {
	const nodeInstance = serverData.value?.node?.instance
	if (!nodeInstance) return false

	const wsUrl = `wss://${nodeInstance}/pingtest`

	try {
		return await new Promise((resolve) => {
			const socket = new WebSocket(wsUrl)
			const timeout = setTimeout(() => {
				socket.close()
				resolve(false)
			}, 5000)

			socket.onopen = () => {
				clearTimeout(timeout)
				socket.send(performance.now().toString())
			}

			socket.onmessage = () => {
				clearTimeout(timeout)
				socket.close()
				resolve(true)
			}

			socket.onerror = () => {
				clearTimeout(timeout)
				resolve(false)
			}
		})
	} catch (error) {
		console.error(`Failed to ping node ${wsUrl}:`, error)
		return false
	}
}

function initializeServer() {
	if (serverData.value?.status === 'suspended') {
		isLoading.value = false
		return
	}

	if (serverData.value?.node === null) {
		isLoading.value = false
		return
	}

	testNodeReachability()
		.then((result) => {
			nodeAccessible.value = result
			if (!nodeAccessible.value) {
				isLoading.value = false
			}
		})
		.catch((err) => {
			console.error('Error testing node reachability:', err)
			nodeAccessible.value = false
			isLoading.value = false
		})

	if (serverError.value) {
		isLoading.value = false
	} else {
		void connectSocket(props.serverId, {
			extraSubscriptions: (targetServerId) => [
				client.archon.sockets.on(targetServerId, 'installation-result', handleInstallationResult),
				client.archon.sockets.on(targetServerId, 'backup-progress', handleBackupProgress),
				client.archon.sockets.on(targetServerId, 'filesystem-ops', handleFilesystemOps),
				client.archon.sockets.on(targetServerId, 'new-mod', handleNewMod),
			],
		})
			.then((connected) => {
				if (connected && cachedWsState?.consoleLines?.length) {
					modrinthServersConsole.clear()
					modrinthServersConsole.addLines(cachedWsState.consoleLines)
				}
			})
			.finally(() => {
				isLoading.value = false
			})
	}

	if (serverData.value?.flows?.intro && serverProject.value) {
		client.archon.servers_v1.endIntro(props.serverId).then(() => {
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
		})
	}
}

let intercomInitialized = false

const cleanup = () => {
	isMounted.value = false

	saveWsStateToCache()

	if (intercomInitialized) {
		shutdown()
		intercomInitialized = false
	}

	cleanupCoreRuntime(props.serverId)

	isReconnecting.value = false
	isLoading.value = true

	DOMPurify.removeHook('afterSanitizeAttributes')
}

onMounted(() => {
	isMounted.value = true
	syncPendingServerContentInstalls()
	window.addEventListener(
		pendingServerContentInstallsEvent,
		handlePendingServerContentInstallsChanged,
	)

	if (serverData.value) {
		initializeServer()
	} else {
		const stopWatch = watch(serverData, (data) => {
			if (data) {
				stopWatch()
				initializeServer()
			}
		})
	}

	const tryInitIntercom = () => {
		if (intercomInitialized) return
		if (!props.authUser || !props.fetchIntercomToken) {
			console.debug('[PYROSERVERS][INTERCOM] waiting for auth user and token fetcher', {
				hasAuthUser: !!props.authUser,
				hasFetchIntercomToken: !!props.fetchIntercomToken,
			})
			return
		}
		intercomInitialized = true
		console.debug('[PYROSERVERS][INTERCOM] initializing secure support chat')
		props
			.fetchIntercomToken()
			.then(({ token }) => {
				console.debug('[PYROSERVERS][INTERCOM] fetched messenger JWT, booting widget')
				Intercom({
					app_id: props.intercomAppId!,
					intercom_user_jwt: token,
					session_duration: 1000 * 60 * 60 * 24,
				})
				window.setTimeout(() => {
					const hasWidget = !!document.querySelector(
						'.intercom-lightweight-app, #intercom-container, #intercom-frame',
					)
					if (!hasWidget) {
						console.warn(
							'[PYROSERVERS][INTERCOM] boot completed but no Intercom widget was detected',
						)
					}
				}, 2500)
			})
			.catch((error) => {
				intercomInitialized = false
				console.warn('[PYROSERVERS][INTERCOM] failed to initialize secure support chat', error)
			})
	}
	tryInitIntercom()
	const stopIntercomWatch = watch(
		() => props.authUser,
		(user) => {
			if (user) {
				tryInitIntercom()
				stopIntercomWatch()
			}
		},
	)

	DOMPurify.addHook(
		'afterSanitizeAttributes',
		(node: {
			tagName: string
			getAttribute: (arg0: string) => string | null
			setAttribute: (arg0: string, arg1: string) => void
		}) => {
			if (node.tagName === 'A' && node.getAttribute('target')) {
				node.setAttribute('rel', 'noopener noreferrer')
			}
		},
	)

	loadTallyScript()
	if (surveyNotice.value) {
		showSurvey()
	}

	if (route.query.openSettings) {
		const tabId = route.query.openSettings as ServerSettingsTabId
		router.replace({ query: { ...route.query, openSettings: undefined } })
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', props.serverId] })
		queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', props.serverId] })
		queryClient.invalidateQueries({ queryKey: ['servers', 'startup', 'v1', props.serverId] })
		nextTick(() => openServerSettingsModal(tabId))
	}
})

onUnmounted(() => {
	window.removeEventListener(
		pendingServerContentInstallsEvent,
		handlePendingServerContentInstallsChanged,
	)
	cleanup()
})
</script>

<style>
@keyframes server-action-buttons-anim {
	0% {
		opacity: 0;
		transform: translateX(1rem);
	}

	100% {
		opacity: 1;
		transform: none;
	}
}

.server-action-buttons-anim {
	animation: server-action-buttons-anim 0.2s ease-out;
}

.server-panel-pending .server-stagger-item {
	opacity: 0;
}

.server-panel-revealing .server-stagger-item {
	animation: serverReveal 0.4s ease-out both;
	animation-delay: calc(var(--si) * 80ms);
}

@keyframes serverReveal {
	from {
		opacity: 0;
		transform: translateY(12px);
	}
	to {
		opacity: 1;
		transform: translateY(0);
	}
}
</style>
