<template>
	<div class="flex gap-4 items-center">
		<ButtonStyled
			v-if="hasActiveLoadingBars && !hasVisibleActiveDownloadToasts"
			color="brand"
			type="transparent"
			circular
		>
			<button v-tooltip="formatMessage(messages.viewActiveDownloads)" @click="openDownloadToast()">
				<DownloadIcon />
			</button>
		</ButtonStyled>
		<div v-if="offline" class="flex items-center gap-1">
			<UnplugIcon class="text-secondary" />
			<span class="text-sm text-contrast"> {{ formatMessage(messages.offline) }} </span>
		</div>
		<div
			class="flex border-solid border-surface-5 text-sm items-center gap-2 py-1.5 px-3 rounded-xl border"
		>
			<template v-if="selectedProcess">
				<OnlineIndicatorIcon />
				<div class="text-contrast flex items-center gap-2">
					<router-link
						v-tooltip="formatMessage(messages.viewInstance)"
						:to="`/instance/${encodeURIComponent(selectedProcess.profile.path)}`"
						class="hover:underline"
					>
						{{ selectedProcess.profile.name }}
					</router-link>
					<Dropdown
						v-if="currentProcesses.length > 1"
						placement="bottom"
						:triggers="['click']"
						:hide-triggers="['click']"
						@show="showProfiles = true"
						@hide="showProfiles = false"
					>
						<ButtonStyled type="transparent" circular size="small">
							<button
								v-tooltip="
									showProfiles
										? formatMessage(messages.hideMoreRunningInstances)
										: formatMessage(messages.showMoreRunningInstances)
								"
							>
								<DropdownIcon :class="{ 'rotate-180': !!showProfiles }" />
							</button>
						</ButtonStyled>
						<template #popper>
							<div class="flex w-[20rem] max-h-[24rem] flex-col gap-2 overflow-auto">
								<div
									v-for="process in currentProcesses"
									:key="process.uuid"
									class="flex w-full items-center gap-2 rounded-xl bg-surface-4 p-2 text-sm"
								>
									<button
										v-tooltip.left="
											process.uuid === selectedProcess.uuid
												? formatMessage(messages.primaryInstance)
												: formatMessage(messages.makePrimaryInstance)
										"
										class="flex flex-grow items-center gap-2"
										:class="{
											'active:scale-95 transition-transform': process.uuid !== selectedProcess.uuid,
										}"
										:disabled="process.uuid === selectedProcess.uuid"
										@click="selectProcess(process)"
									>
										<OnlineIndicatorIcon />
										<span class="mr-auto text-contrast flex items-center gap-2">
											{{ process.profile.name }}
											<StarIcon v-if="process.uuid === selectedProcess.uuid" class="text-orange" />
										</span>
									</button>
									<button
										v-tooltip="formatMessage(messages.stopInstance)"
										class="active:scale-95 flex"
										@click.stop="stop(process)"
									>
										<StopCircleIcon class="text-red size-5" />
									</button>
									<button
										v-tooltip="formatMessage(messages.viewLogs)"
										class="active:scale-95 flex"
										@click.stop="goToTerminal(process.profile.path)"
									>
										<TerminalSquareIcon class="text-secondary size-5" />
									</button>
								</div>
							</div>
						</template>
					</Dropdown>
				</div>
				<button
					v-tooltip="formatMessage(messages.stopInstance)"
					class="active:scale-95 flex"
					@click="stop(selectedProcess)"
				>
					<StopCircleIcon class="text-red size-5" />
				</button>
				<button
					v-tooltip="formatMessage(messages.viewLogs)"
					class="active:scale-95 flex"
					@click="goToTerminal()"
				>
					<TerminalSquareIcon class="text-secondary size-5" />
				</button>
			</template>
			<template v-else>
				<span class="size-2 rounded-full bg-secondary" />
				<span class="text-secondary"> {{ formatMessage(messages.noInstancesRunning) }} </span>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import {
	DownloadIcon,
	DropdownIcon,
	OnlineIndicatorIcon,
	StarIcon,
	StopCircleIcon,
	TerminalSquareIcon,
	UnplugIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	injectPopupNotificationManager,
	type PopupNotification,
	type PopupNotificationProgressItem,
	useVIntl,
} from '@modrinth/ui'
import { Dropdown } from 'floating-vue'
import { computed, onBeforeUnmount, ref } from 'vue'
import { useRouter } from 'vue-router'

import { trackEvent } from '@/helpers/analytics'
import { loading_listener, process_listener } from '@/helpers/events'
import { get_all as getRunningProcesses, kill as killProcess } from '@/helpers/process'
import { get_many as getInstances } from '@/helpers/profile.js'
import type { LoadingBar } from '@/helpers/state'
import { progress_bars_list } from '@/helpers/state'
import type { GameInstance } from '@/helpers/types'

const { handleError } = injectNotificationManager()
const popupNotificationManager = injectPopupNotificationManager()
const { formatMessage } = useVIntl()

const router = useRouter()

const showProfiles = ref(false)

interface RunningProcess {
	uuid: string
	profile_path: string
	profile: GameInstance
}

const messages = defineMessages({
	offline: {
		id: 'app.action-bar.offline',
		defaultMessage: 'Offline',
	},
	viewInstance: {
		id: 'app.action-bar.view-instance',
		defaultMessage: 'View instance',
	},
	showMoreRunningInstances: {
		id: 'app.action-bar.show-more-running-instances',
		defaultMessage: 'Show more running instances',
	},
	hideMoreRunningInstances: {
		id: 'app.action-bar.hide-more-running-instances',
		defaultMessage: 'Hide more running instances',
	},
	primaryInstance: {
		id: 'app.action-bar.primary-instance',
		defaultMessage: 'Primary instance',
	},
	makePrimaryInstance: {
		id: 'app.action-bar.make-primary-instance',
		defaultMessage: 'Make primary instance',
	},
	stopInstance: {
		id: 'app.action-bar.stop-instance',
		defaultMessage: 'Stop instance',
	},
	viewLogs: {
		id: 'app.action-bar.view-logs',
		defaultMessage: 'View logs',
	},
	noInstancesRunning: {
		id: 'app.action-bar.no-instances-running',
		defaultMessage: 'No instances running',
	},
	downloadingJava: {
		id: 'app.action-bar.downloading-java',
		defaultMessage: 'Downloading Java {version}',
	},
	downloads: {
		id: 'app.action-bar.downloads',
		defaultMessage: 'Downloads',
	},
	viewActiveDownloads: {
		id: 'app.action-bar.view-active-downloads',
		defaultMessage: 'View active downloads',
	},
})

const currentProcesses = ref<RunningProcess[]>([])
const selectedProcess = ref<RunningProcess | undefined>()

const refresh = async () => {
	const processes = ((await getRunningProcesses().catch((error) => {
		handleError(error)
		return []
	})) ?? []) as Array<{ uuid: string; profile_path: string }>
	const paths = processes.map((process) => process.profile_path)
	const profiles: GameInstance[] = await getInstances(paths).catch((error) => {
		handleError(error)
		return []
	})

	currentProcesses.value = processes
		.map((process) => {
			const profile = profiles.find((item) => process.profile_path === item.path)
			if (!profile) {
				return null
			}
			return {
				...process,
				profile,
			}
		})
		.filter((process): process is RunningProcess => process !== null)
	if (!selectedProcess.value || !currentProcesses.value.includes(selectedProcess.value)) {
		selectedProcess.value = currentProcesses.value[0]
	}
}

await refresh()

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const unlistenProcess = await process_listener(async () => {
	await refresh()
})

const stop = async (process: RunningProcess) => {
	try {
		await killProcess(process.uuid).catch(handleError)

		trackEvent('InstanceStop', {
			loader: process.profile.loader,
			game_version: process.profile.game_version,
			source: 'AppBar',
		})
	} catch (e) {
		console.error(e)
	}
	await refresh()
}

function goToTerminal(path?: string) {
	const selectedPath = path ?? selectedProcess.value?.profile.path
	if (!selectedPath) {
		return
	}
	router.push(`/instance/${encodeURIComponent(selectedPath)}/logs`)
}

const currentLoadingBars = ref<LoadingBar[]>([])
const notificationId = ref<string | number | null>(null)
const dismissed = ref(false)

function getLoadingBarKey(loadingBar: LoadingBar): string {
	return `${loadingBar.loading_bar_uuid ?? loadingBar.id}`
}

function getLoadingProgress(loadingBar: LoadingBar): number {
	if (!loadingBar.total || loadingBar.total <= 0) {
		return 0
	}
	return Math.max(0, Math.min(1, (loadingBar.current ?? 0) / (loadingBar.total ?? 0)))
}

function getLoadingText(loadingBar: LoadingBar): string {
	const percent = Math.floor(getLoadingProgress(loadingBar) * 100)
	return loadingBar.message ? `${percent}% ${loadingBar.message}` : `${percent}%`
}

function getNotification(): PopupNotification | null {
	if (!notificationId.value) {
		return null
	}
	const notification = popupNotificationManager
		.getNotifications()
		.find((notification) => notification.id === notificationId.value)
	return notification ?? null
}

function removeNotification(): void {
	if (!notificationId.value) {
		return
	}
	popupNotificationManager.removeNotification(notificationId.value)
	notificationId.value = null
}

function buildDownloadItems(): PopupNotificationProgressItem[] {
	return currentLoadingBars.value.map((bar) => ({
		id: getLoadingBarKey(bar),
		title: bar.title ?? '',
		text: getLoadingText(bar),
		progress: getLoadingProgress(bar),
		waiting: !bar.total || bar.total <= 0,
	}))
}

const hasVisibleActiveDownloadToasts = computed(() => !!getNotification())
const hasActiveLoadingBars = computed(() => currentLoadingBars.value.length > 0)

function updateNotification(resummon = false): void {
	if (resummon) {
		dismissed.value = false
	}

	if (currentLoadingBars.value.length === 0) {
		removeNotification()
		dismissed.value = false
		return
	}

	if (notificationId.value && !getNotification()) {
		notificationId.value = null
		dismissed.value = true
	}

	if (dismissed.value && !resummon) {
		return
	}

	let notif = getNotification()
	const progressItems = buildDownloadItems()

	if (notif) {
		notif.title = formatMessage(messages.downloads)
		notif.text = undefined
		notif.progressItems = progressItems
		notif.progress = undefined
		notif.waiting = undefined
	} else {
		notif = popupNotificationManager.addPopupNotification({
			title: formatMessage(messages.downloads),
			type: 'download',
			autoCloseMs: null,
			progressItems,
		})
		notificationId.value = notif.id
	}
}

function formatLoadingBars(loadingBar: LoadingBar): LoadingBar {
	const formatted = { ...loadingBar }
	if (formatted.bar_type?.type === 'java_download') {
		formatted.title = formatMessage(messages.downloadingJava, {
			version: formatted.bar_type.version,
		})
	}
	if (formatted.bar_type?.profile_path) {
		formatted.title = formatted.bar_type.profile_path
	}
	if (formatted.bar_type?.pack_name) {
		formatted.title = formatted.bar_type.pack_name
	}
	return formatted
}

async function refreshLoadingBars() {
	const bars: Record<string, LoadingBar> = await progress_bars_list().catch((error) => {
		handleError(error)
		return {}
	})

	currentLoadingBars.value = Object.values(bars)
		.map(formatLoadingBars)
		.filter((bar) => bar?.bar_type?.type !== 'launcher_update')

	currentLoadingBars.value.sort((a, b) => {
		const aKey = `${a.loading_bar_uuid ?? a.id ?? ''}`
		const bKey = `${b.loading_bar_uuid ?? b.id ?? ''}`
		return aKey.localeCompare(bKey)
	})

	updateNotification()
}

await refreshLoadingBars()

const unlistenLoading = await loading_listener(async () => {
	await refreshLoadingBars()
})

function openDownloadToast() {
	updateNotification(true)
}

function selectProcess(process: RunningProcess) {
	selectedProcess.value = process
}

onBeforeUnmount(() => {
	removeNotification()
	dismissed.value = false
	unlistenProcess()
	unlistenLoading()
})
</script>
