<template>
	<div
		v-if="filteredNotices.length > 0"
		class="experimental-styles-within relative mx-auto mb-4 flex w-full min-w-0 max-w-[1280px] flex-col gap-3 px-6"
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
			:icon="PanelErrorIcon"
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
		class="experimental-styles-within mobile-blurred-servericon relative mx-auto mb-12 box-border flex min-h-screen w-full min-w-0 max-w-[1280px] flex-col gap-6 px-6 transition-all duration-300"
		:style="{
			'--server-bg-image': serverImage
				? `url(${serverImage})`
				: `linear-gradient(180deg, rgba(153,153,153,1) 0%, rgba(87,87,87,1) 100%)`,
		}"
	>
		<ServerManageHeader
			:server="serverData"
			:server-image="serverImage"
			:server-project="serverProject"
			:uptime-seconds="uptimeSeconds"
		>
			<template #actions>
				<div v-if="isConnected && !serverData.flows?.intro" class="flex gap-2">
					<PanelServerActionButton :disabled="!!error" />
					<PanelServerOverflowMenu
						:disabled="!!error"
						:uptime-seconds="uptimeSeconds"
						:show-copy-id-action="flags.developerMode"
						:show-debug-info="flags.advancedDebugInfo"
					/>
				</div>
			</template>
		</ServerManageHeader>

		<ServerOnboardingPanelPage v-if="serverData.flows?.intro" />

		<template v-else>
			<div
				data-pyro-navigation
				class="isolate flex w-full select-none flex-col justify-between gap-4 overflow-auto md:flex-row md:items-center"
			>
				<NavTabs :links="navLinks" replace />
			</div>

			<div data-pyro-mount class="h-full w-full flex-1">
				<div
					v-if="error"
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
										<NuxtLink
											class="whitespace-pre"
											:to="`/hosting/manage/${serverId}/options/loader`"
										>
											<RightArrowIcon />
											Change Loader
										</NuxtLink>
									</ButtonStyled>
								</div>
							</div>
						</div>
					</div>
				</div>

				<div v-if="serverData.is_medal" class="mb-4">
					<MedalServerCountdown :server-id="serverId" />
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
					<PanelSpinner />
					Hang on, we're reconnecting to your server.
				</div>

				<Transition
					enter-active-class="transition-all duration-300 ease-out overflow-hidden"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-40"
					leave-active-class="transition-all duration-200 ease-in overflow-hidden"
					leave-from-class="opacity-100 max-h-40"
					leave-to-class="opacity-0 max-h-0"
				>
					<InstallingBanner
						v-if="
							(serverData.status === 'installing' || isSyncingContent || contentError) &&
							syncProgress?.phase !== 'Analyzing'
						"
						data-pyro-server-installing
						class="mb-4"
						:progress="syncProgress"
						:content-error="contentError"
						@retry="handleContentRetry"
					>
						<template #icon>
							<ServerIcon :image="serverImage" class="!h-6 !w-6" />
						</template>
					</InstallingBanner>
				</Transition>
				<BackupProgressAdmonitions class="mb-4" />
				<NuxtPage
					:route="route"
					:is-connected="isConnected"
					:is-ws-auth-incorrect="isWSAuthIncorrect"
					:stats="stats"
					:server-power-state="serverPowerState"
					:power-state-details="powerStateDetails"
					@reinstall="onReinstall"
					@reinstall-failed="onReinstallFailed"
				/>
			</div>
		</template>
	</div>
	<div
		v-if="flags.advancedDebugInfo"
		class="experimental-styles-within relative mx-auto mt-6 box-border w-full min-w-0 max-w-[1280px] px-6"
	>
		<h2 class="m-0 text-lg font-extrabold text-contrast">Server data</h2>
		<pre class="markdown-body w-full overflow-auto rounded-2xl bg-bg-raised p-4 text-sm">{{
			safeStringify(serverData)
		}}</pre>
	</div>
</template>

<script setup lang="ts">
import { Intercom, shutdown } from '@intercom/messenger-js-sdk'
import type { Archon } from '@modrinth/api-client'
import { ModrinthApiError } from '@modrinth/api-client'
import {
	BoxesIcon,
	CheckIcon,
	CopyIcon,
	DatabaseBackupIcon,
	FileIcon,
	FolderOpenIcon,
	IssuesIcon,
	LayoutTemplateIcon,
	LockIcon,
	RightArrowIcon,
	SettingsIcon,
	TransferIcon,
} from '@modrinth/assets'
import {
	BackupProgressAdmonitions,
	ButtonStyled,
	ErrorInformationCard,
	formatLoaderLabel,
	injectModrinthClient,
	injectNotificationManager,
	InstallingBanner,
	NavTabs,
	PanelServerActionButton,
	PanelServerOverflowMenu,
	ServerIcon,
	ServerManageHeader,
	ServerNotice,
	ServerOnboardingPanelPage,
	useDebugLogger,
	useServerManageCoreRuntime,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useTimeoutFn } from '@vueuse/core'
import DOMPurify from 'dompurify'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { reloadNuxtApp } from '#app'
import PanelErrorIcon from '~/components/ui/servers/icons/PanelErrorIcon.vue'
import MedalServerCountdown from '~/components/ui/servers/marketing/MedalServerCountdown.vue'
import PanelSpinner from '~/components/ui/servers/PanelSpinner.vue'
import { useServerImage } from '~/composables/servers/use-server-image.ts'
import { useServerProject } from '~/composables/servers/use-server-project.ts'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()

const isReconnecting = ref(false)
const isLoading = ref(true)
const isMounted = ref(true)
const flags = useFeatureFlags()

const INTERCOM_APP_ID = ref('ykeritl9')
const auth = (await useAuth()) as unknown as {
	value: { user: { id: string; username: string; email: string; created: string } }
}
const userId = ref(auth.value?.user?.id ?? null)
const username = ref(auth.value?.user?.username ?? null)
const email = ref(auth.value?.user?.email ?? null)
const createdAt = ref(
	auth.value?.user?.created ? Math.floor(new Date(auth.value.user.created).getTime() / 1000) : null,
)

const debug = useDebugLogger('ServerManage')
const route = useNativeRoute()
const router = useRouter()
const serverId = route.params.id as string

const { data: serverData, error: serverQueryError } = useQuery({
	queryKey: ['servers', 'detail', serverId],
	queryFn: () => client.archon.servers_v0.get(serverId)!,
})

function updateServerData(patch: Partial<Archon.Servers.v0.Server>) {
	if (!serverData.value) return
	queryClient.setQueryData(['servers', 'detail', serverId], {
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
	queryKey: ['servers', 'v1', 'detail', serverId],
	queryFn: () => client.archon.servers_v1.get(serverId),
})

const worldId = computed(() => {
	if (!serverFull.value) return null
	const activeWorld = serverFull.value.worlds.find((w) => w.is_active)
	return activeWorld?.id ?? serverFull.value.worlds[0]?.id ?? null
})

const serverImage = useServerImage(
	serverId,
	computed(() => serverData.value?.upstream ?? null),
)
const { data: serverProject } = useServerProject(computed(() => serverData.value?.upstream ?? null))

const errorTitle = ref('Error')
const errorMessage = ref('An unexpected error occurred.')
const errorLog = ref('')
const errorLogFile = ref('')

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

const queryClient = useQueryClient()
const cancelledBackups = new Set<string>()

const markBackupCancelled = (backupId: string) => {
	cancelledBackups.add(backupId)
}

// Parthenon state event
const syncProgress = ref<Archon.Websocket.v0.SyncContentProgress | null>(null)
const contentError = ref<Archon.Websocket.v0.SyncContentError | null>(null)
const syncProgressActive = ref(false)
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
	} else if (syncProgressActive.value) {
		startSyncHide()
	}
})

const isSyncingContent = computed(
	() => syncProgressActive.value || isAwaitingPostInstallRefresh.value,
)

let hasSeenInstallProgress = false

const onStateEvent = (data: Archon.Websocket.v0.WSStateEvent) => {
	debug('[id.vue] handleState received:', {
		power_variant: data.power_variant,
		progress: data.progress,
		serverStatus: serverData.value?.status,
	})
	syncProgress.value = data.progress
	contentError.value = data.content_error

	if (serverData.value) {
		if (data.progress != null && serverData.value.status !== 'installing') {
			debug('[id.vue] handleState: progress != null, setting status to installing')
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
			debug('[id.vue] handleState: progress null + was installing, applying optimistic update')
			hasSeenInstallProgress = false
			applyOptimisticCompletion()
			invalidateAfterInstall()
		}
	}
}

const {
	backupsState,
	cleanupCoreRuntime,
	connectSocket,
	fsOps,
	fsQueuedOps,
	isConnected,
	isWsAuthIncorrect: isWSAuthIncorrect,
	powerStateDetails,
	serverPowerState,
	stats,
	uptimeSeconds,
} = useServerManageCoreRuntime({
	serverId: computed(() => serverId),
	worldId,
	server: serverData,
	isSyncingContent,
	markBackupCancelled,
	includeBackupBusyReasons: true,
	setDisconnectedOnAuthIncorrect: false,
	syncUptimeFromState: true,
	incrementUptimeLocally: true,
	eventGuard: () => isMounted.value,
	onStateEvent,
})

const copied = ref(false)
const error = ref<Error | null>(null)

const navLinks = [
	{
		label: 'Overview',
		href: `/hosting/manage/${serverId}`,
		icon: LayoutTemplateIcon,
		subpages: [],
	},
	{
		label: 'Content',
		href: `/hosting/manage/${serverId}/content`,
		icon: BoxesIcon,
		subpages: ['mods', 'datapacks'],
	},
	{ label: 'Files', href: `/hosting/manage/${serverId}/files`, icon: FolderOpenIcon, subpages: [] },
	{
		label: 'Backups',
		href: `/hosting/manage/${serverId}/backups`,
		icon: DatabaseBackupIcon,
		subpages: [],
	},
	{
		label: 'Options',
		href: `/hosting/manage/${serverId}/options`,
		icon: SettingsIcon,
		subpages: ['startup', 'network', 'properties', 'info'],
	},
]

const filteredNotices = computed(
	() => serverData.value?.notices?.filter((n) => n.level !== 'survey') ?? [],
)
const surveyNotice = computed(() => serverData.value?.notices?.find((n) => n.level === 'survey'))

async function dismissSurvey() {
	const noticeId = surveyNotice.value?.id
	if (noticeId === undefined) {
		console.warn('No survey notice to dismiss')
		return
	}
	await dismissNotice(noticeId)
	console.log(`Dismissed survey notice ${noticeId}`)
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
	hiddenFields?: {
		[key: string]: unknown
	}
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
				username: auth.value?.user?.username,
				user_id: auth.value?.user?.id,
				user_email: auth.value?.user?.email,
				server_id: serverData.value?.server_id,
				loader: serverData.value?.loader,
				game_version: serverData.value?.mc_version,
				modpack_id: serverProject.value?.id,
				modpack_name: serverProject.value?.title,
			},
			onOpen: () => console.log(`Opened survey notice: ${surveyNotice.value?.id}`),
			onClose: async () => await dismissSurvey(),
			onSubmit: (payload: any) => {
				console.log('Form submitted:', payload)
			},
		}) satisfies TallyPopupOptions,
)

function showSurvey() {
	if (!surveyNotice.value) {
		console.warn('No survey notice to open')
		return
	}

	try {
		if ((window as any).Tally?.openPopup) {
			console.log(
				`Opening Tally popup for survey notice ${surveyNotice.value?.id} (form ID: ${surveyNotice.value?.message})`,
			)
			;(window as any).Tally.openPopup(surveyNotice.value?.message, popupOptions.value)
		} else {
			console.warn('Tally script not yet loaded')
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
	}
}

async function handleContentRetry() {
	if (!worldId.value) return
	try {
		await client.archon.content_v1.repair(serverId, worldId.value)
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to retry installation',
		})
	}
}

const handleBackupProgress = (data: Archon.Websocket.v0.WSBackupProgressEvent) => {
	if (data.task === 'file') return

	const backupId = data.id

	if (cancelledBackups.has(backupId)) return

	const current = backupsState.get(backupId) ?? {}
	const currentTaskState = current[data.task]?.state
	const isIncomingTerminal =
		data.state === 'done' || data.state === 'failed' || data.state === 'cancelled'

	// Skip duplicate terminal events, but allow retries (terminal → ongoing)
	if (currentTaskState === data.state && isIncomingTerminal) return

	const previousProgress = current[data.task]?.progress
	if (currentTaskState !== data.state || previousProgress !== data.progress) {
		backupsState.set(backupId, {
			...current,
			[data.task]: {
				progress: data.progress,
				state: data.state,
			},
		})
	}

	if (isIncomingTerminal) {
		const attemptCleanup = (attempt: number = 1) => {
			queryClient.invalidateQueries({ queryKey: ['backups', 'list', serverId] }).then(() => {
				const backupData = queryClient.getQueryData<Archon.Backups.v1.Backup[]>([
					'backups',
					'list',
					serverId,
				])
				const backup = backupData?.find((b) => b.id === backupId)
				const isStillActive =
					backup && (backup.status === 'in_progress' || backup.status === 'pending')

				if (isStillActive && attempt < 6) {
					setTimeout(() => attemptCleanup(attempt + 1), 1000 * Math.pow(2, attempt - 1))
					return
				}

				if (isStillActive) {
					queryClient.setQueryData<Archon.Backups.v1.Backup[]>(
						['backups', 'list', serverId],
						(old) =>
							old?.map((b) => {
								if (b.id !== backupId) return b
								return {
									...b,
									status: data.state === 'done' ? ('done' as const) : ('error' as const),
									ongoing: false,
									interrupted: data.state === 'failed',
								}
							}),
					)
				}

				backupsState.delete(backupId)
			})
		}

		attemptCleanup()
	}
}

const opsQueuedForModification = ref<string[]>([])

const handleFilesystemOps = (data: Archon.Websocket.v0.WSFilesystemOpsEvent) => {
	const allOps = data.all

	if (JSON.stringify(fsOps.value) !== JSON.stringify(allOps)) {
		fsOps.value = allOps
	}

	fsQueuedOps.value = fsQueuedOps.value.filter(
		(queuedOp) => !allOps.some((x) => x.src === queuedOp.src),
	)

	const dismissOp = async (opId: string) => {
		try {
			await client.kyros.files_v0.modifyOperation(opId, 'dismiss')
		} catch (error) {
			console.error('Failed to dismiss operation:', error)
		}
	}

	const cancelled = allOps.filter((x) => x.state === 'cancelled')
	Promise.all(cancelled.map((x) => dismissOp(x.id)))

	const completed = allOps.filter((x) => x.state === 'done')
	if (completed.length > 0) {
		setTimeout(
			async () =>
				await Promise.all(
					completed.map((x) => {
						if (!opsQueuedForModification.value.includes(x.id)) {
							opsQueuedForModification.value.push(x.id)
							return dismissOp(x.id)
						}
						return Promise.resolve()
					}),
				),
			3000,
		)
	}
}

const handleNewMod = () => {
	queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
}

const newLoader = ref<string | null>(null)
const newLoaderVersion = ref<string | null>(null)
const newMCVersion = ref<string | null>(null)

const onReinstall = async (potentialArgs: any) => {
	debug('[id.vue] onReinstall called with:', potentialArgs)

	if (serverData.value?.flows?.intro) {
		await client.archon.servers_v1.endIntro(serverId)
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	}

	if (!serverData.value) return

	debug('[id.vue] onReinstall: setting serverData.status to installing')
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

	debug('[id.vue] onReinstall: stored refs:', {
		newLoader: newLoader.value,
		newLoaderVersion: newLoaderVersion.value,
		newMCVersion: newMCVersion.value,
	})

	error.value = null
	errorTitle.value = 'Error'
	errorMessage.value = 'An unexpected error occurred.'

	// Immediately refetch so loader.vue has fresh data (buttons stay locked via isSyncingContent)
	debug('[id.vue] onReinstall: triggering immediate invalidation for loader.vue')
	queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
}

const onReinstallFailed = () => {
	debug('[id.vue] onReinstallFailed: reverting status to available')
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

	debug('[id.vue] applyOptimisticCompletion: patch:', patch)
	updateServerData(patch)

	const addonsQueries = queryClient.getQueriesData<Archon.Content.v1.Addons>({
		queryKey: ['content', 'list', 'v1', serverId],
	})
	debug(
		'[id.vue] applyOptimisticCompletion: found',
		addonsQueries.length,
		'addons queries to patch',
	)
	for (const [key, data] of addonsQueries) {
		if (!data) continue
		const addonsPatch: Record<string, string> = {}
		if (newLoader.value) addonsPatch.modloader = newLoader.value
		if (newLoaderVersion.value) addonsPatch.modloader_version = newLoaderVersion.value
		if (newMCVersion.value) addonsPatch.game_version = newMCVersion.value
		if (Object.keys(addonsPatch).length > 0) {
			debug('[id.vue] applyOptimisticCompletion: patching addons cache:', addonsPatch)
			queryClient.setQueryData(key, { ...data, ...addonsPatch })
		}
	}

	newLoader.value = null
	newLoaderVersion.value = null
	newMCVersion.value = null
}

async function invalidateAfterInstall() {
	debug(
		'[id.vue] invalidateAfterInstall: setting isAwaitingPostInstallRefresh=true, scheduling 2s delayed invalidation',
	)
	isAwaitingPostInstallRefresh.value = true
	setTimeout(async () => {
		debug('[id.vue] invalidateAfterInstall: delayed invalidation firing now')
		try {
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
				queryClient.invalidateQueries({ queryKey: ['servers', 'startup', 'v1', serverId] }),
				queryClient.invalidateQueries({ queryKey: ['content', 'list'] }),
			])
			debug('[id.vue] invalidateAfterInstall: delayed invalidation complete')
		} catch (err: unknown) {
			console.error('Error refreshing data after installation:', err)
		} finally {
			debug('[id.vue] invalidateAfterInstall: setting isAwaitingPostInstallRefresh=false')
			isAwaitingPostInstallRefresh.value = false
		}
	}, 2000)
}

const handleInstallationResult = async (data: Archon.Websocket.v0.WSInstallationResultEvent) => {
	debug('[id.vue] handleInstallationResult received:', data)
	switch (data.result) {
		case 'ok': {
			debug('[id.vue] handleInstallationResult: ok received')
			if (!serverData.value) break

			debug('[id.vue] handleInstallationResult: stored refs:', {
				newLoader: newLoader.value,
				newLoaderVersion: newLoaderVersion.value,
				newMCVersion: newMCVersion.value,
			})
			debug('[id.vue] handleInstallationResult: current serverData:', {
				status: serverData.value.status,
				loader: serverData.value.loader,
				loader_version: serverData.value.loader_version,
				mc_version: serverData.value.mc_version,
			})

			applyOptimisticCompletion()
			error.value = null
			invalidateAfterInstall()

			break
		}
		case 'err': {
			console.log('failed to install')
			console.log(data)
			errorTitle.value = 'Installation error'
			errorMessage.value = data.reason ?? 'Unknown error'
			error.value = new Error(data.reason ?? 'Unknown error')

			// Fetch installation log if available
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

const nodeUnavailableDetails = computed(() => [
	{
		label: 'Server ID',
		value: serverId,
		type: 'inline' as const,
	},
	{
		label: 'Node',
		value:
			(serverError.value?.responseData as any)?.hostname ??
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
		value: serverId,
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
	onClick: () => router.push('/settings/billing'),
	color: 'brand' as const,
}))

const generalErrorAction = computed(() => ({
	label: 'Go back to all servers',
	onClick: () => router.push('/hosting/manage'),
	color: 'brand' as const,
}))

const nodeUnavailableAction = computed(() => ({
	label: 'Reload',
	onClick: () => reloadNuxtApp(),
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
	router.replace({
		path: `/hosting/manage/${serverId}/files`,
		query: { ...route.query, editing: errorLogFile.value },
	})
}

const cleanup = () => {
	isMounted.value = false

	shutdown()

	cleanupCoreRuntime(serverId)

	isReconnecting.value = false
	isLoading.value = true

	cancelledBackups.clear()

	DOMPurify.removeHook('afterSanitizeAttributes')
}

async function dismissNotice(noticeId: number) {
	await client.archon.servers_v0.dismissNotice(serverId, noticeId).catch((err) => {
		addNotification({
			title: 'Error dismissing notice',
			text: err,
			type: 'error',
		})
	})
	await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
}

const nodeAccessible = ref(true)

async function testNodeReachability(): Promise<boolean> {
	const nodeInstance = serverData.value?.node?.instance
	if (!nodeInstance) {
		console.warn('No node instance available for ping test')
		return false
	}

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

	// Skip node test if node is null (upgrading/provisioning)
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
		void connectSocket(serverId, {
			extraSubscriptions: (targetServerId) => [
				client.archon.sockets.on(targetServerId, 'installation-result', handleInstallationResult),
				client.archon.sockets.on(targetServerId, 'backup-progress', handleBackupProgress),
				client.archon.sockets.on(targetServerId, 'filesystem-ops', handleFilesystemOps),
				client.archon.sockets.on(targetServerId, 'new-mod', handleNewMod),
			],
		}).finally(() => {
			isLoading.value = false
		})
	}

	if (serverData.value?.flows?.intro && serverProject.value) {
		client.archon.servers_v1.endIntro(serverId).then(() => {
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
		})
	}
}

onMounted(() => {
	isMounted.value = true

	// serverData comes from useQuery and may not be available yet at mount time.
	// Wait for it before initializing WebSocket, node reachability, etc.
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

	if (username.value && email.value && userId.value && createdAt.value) {
		const currentUser = auth.value?.user as any
		const matches =
			username.value === currentUser?.username &&
			email.value === currentUser?.email &&
			userId.value === currentUser?.id &&
			createdAt.value === Math.floor(new Date(currentUser?.created).getTime() / 1000)

		if (matches) {
			Intercom({
				app_id: INTERCOM_APP_ID.value,
				userId: userId.value,
				name: username.value,
				email: email.value,
				created_at: createdAt.value,
			})
		} else {
			console.warn('[PYROSERVERS][INTERCOM] mismatch')
		}
	}

	DOMPurify.addHook(
		'afterSanitizeAttributes',
		(node: {
			tagName: string
			getAttribute: (arg0: string) => any
			setAttribute: (arg0: string, arg1: string) => void
		}) => {
			if (node.tagName === 'A' && node.getAttribute('target')) {
				node.setAttribute('rel', 'noopener noreferrer')
			}
		},
	)

	if (surveyNotice.value) {
		showSurvey()
	}
})

onUnmounted(() => {
	cleanup()
})

definePageMeta({
	middleware: 'auth',
})

useHead({
	script: [
		{
			src: 'https://tally.so/widgets/embed.js',
			defer: true,
		},
	],
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

.mobile-blurred-servericon::before {
	position: absolute;
	left: 0;
	top: 0;
	display: block;
	height: 9rem;
	width: 100%;
	background-size: cover;
	background-position: center;
	background-repeat: no-repeat;
	filter: blur(1rem);
	content: '';
	background-image:
		linear-gradient(
			to bottom,
			rgba(from var(--color-raised-bg) r g b / 0.2),
			rgb(from var(--color-raised-bg) r g b / 0.8)
		),
		var(--server-bg-image);
}

@media screen and (min-width: 640px) {
	.mobile-blurred-servericon::before {
		display: none;
	}
}
</style>
