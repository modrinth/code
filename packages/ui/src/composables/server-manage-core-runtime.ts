import {
	type Archon,
	clearNodeAuthState,
	setNodeAuthState,
	type UploadState,
} from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import type { FileOperation } from '../layouts/shared/files-tab/types'
import { injectModrinthClient, provideModrinthServerContext } from '../providers'
import type { BusyReason, CancelUploadHandler, ServerStats } from '../providers/server-context'
import { defineMessage } from './i18n'
import { useModrinthServersConsole } from './server-console'
import {
	retainServerContextRuntime,
	type ServerContextRuntimeLease,
} from './server-context-runtime'
import { useServerInstallationTracker } from './server-installation-tracker'

type ReadableRef<T> = Ref<T> | ComputedRef<T>
type SocketUnsubscriber = () => void

type ConnectSocketOptions = {
	extraSubscriptions?: (targetServerId: string) => SocketUnsubscriber[]
}

type UseServerManageCoreRuntimeOptions = {
	serverId: ReadableRef<string>
	worldId: ReadableRef<string | null>
	server: ReadableRef<Archon.Servers.v0.Server | null | undefined>
	serverFull?: ReadableRef<Archon.Servers.v1.ServerFull | null | undefined>
	content?: ReadableRef<Archon.Content.v1.Addons | null | undefined>
	extraBusyReasons?: ComputedRef<BusyReason[]>
	setDisconnectedOnAuthIncorrect?: boolean
	syncUptimeFromState?: boolean
	incrementUptimeLocally?: boolean
	eventGuard?: () => boolean
	onStateEvent?: (data: Archon.Websocket.v0.WSStateEvent) => void
}

const createInitialStats = (): ServerStats => ({
	current: {
		cpu_percent: 0,
		ram_usage_bytes: 0,
		ram_total_bytes: 1,
		storage_usage_bytes: 0,
		storage_total_bytes: 0,
	},
	past: {
		cpu_percent: 0,
		ram_usage_bytes: 0,
		ram_total_bytes: 1,
		storage_usage_bytes: 0,
		storage_total_bytes: 0,
	},
	graph: {
		cpu: [],
		ram: [],
	},
})

const appendGraphData = (dataArray: number[], newValue: number): number[] => {
	const updated = [...dataArray, newValue]
	if (updated.length > 10) updated.shift()
	return updated
}

const STALE_STATS_THRESHOLD_MS = 5000
const STALE_STATS_PUSH_INTERVAL_MS = 1000

const mapPowerStateFromStateEvent = (
	data: Archon.Websocket.v0.WSStateEvent,
): Archon.Websocket.v0.PowerState => {
	const powerMap: Record<Archon.Websocket.v0.FlattenedPowerState, Archon.Websocket.v0.PowerState> =
		{
			not_ready: 'stopped',
			starting: 'starting',
			running: 'running',
			stopping: 'stopping',
			idle:
				data.was_oom || (data.exit_code != null && data.exit_code !== 0) ? 'crashed' : 'stopped',
		}
	return powerMap[data.power_variant]
}

export function useServerManageCoreRuntime(options: UseServerManageCoreRuntimeOptions) {
	const client = injectModrinthClient()
	const modrinthServersConsole = useModrinthServersConsole()

	const shouldProcessEvent = () => (options.eventGuard ? options.eventGuard() : true)

	const isConnected = ref(false)
	const isWsAuthIncorrect = ref(false)
	const serverPowerState = ref<Archon.Websocket.v0.PowerState>('stopped')
	const powerStateDetails = ref<{ oom_killed?: boolean; exit_code?: number }>()
	const isServerRunning = computed(() => serverPowerState.value === 'running')
	const stats = ref<ServerStats>(createInitialStats())
	const uptimeSeconds = ref(0)
	const fsAuth = ref<{ url: string; token: string } | null>(null)
	const fsOps = ref<Archon.Websocket.v0.FilesystemOperation[]>([])
	const fsQueuedOps = ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([])
	const {
		begin: beginInstallation,
		cancelOptimistic: cancelOptimisticInstallation,
		dismiss: dismissInstallation,
		handleProgress: handleInstallProgress,
		installation,
		installProgressItems,
		isBlocking: isInstallationBlocking,
		reset: resetInstallation,
	} = useServerInstallationTracker({
		worldId: options.worldId,
		server: options.server,
		content: options.content,
	})
	const connectedSocketServerId = ref<string | null>(null)
	const socketUnsubscribers = ref<SocketUnsubscriber[]>([])
	const cpuData = ref<number[]>([])
	const ramData = ref<number[]>([])
	let serverContextRuntimeLease: ServerContextRuntimeLease | null = null

	let uptimeIntervalId: ReturnType<typeof setInterval> | null = null
	let staleStatsTimeoutId: ReturnType<typeof setTimeout> | null = null
	let staleStatsIntervalId: ReturnType<typeof setInterval> | null = null

	const busyReasons = computed<BusyReason[]>(() => {
		const reasons: BusyReason[] = []
		if (isInstallationBlocking.value) {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.installing',
					defaultMessage: 'Server is installing',
				}),
			})
		}
		if (options.extraBusyReasons) reasons.push(...options.extraBusyReasons.value)
		return reasons
	})

	const stopUptimeTicker = () => {
		if (uptimeIntervalId) {
			clearInterval(uptimeIntervalId)
			uptimeIntervalId = null
		}
	}

	const startUptimeTicker = () => {
		if (!options.incrementUptimeLocally || uptimeIntervalId) return
		uptimeIntervalId = setInterval(() => {
			uptimeSeconds.value += 1
		}, 1000)
	}

	const updateStats = (currentStats: ServerStats['current']) => {
		if (!shouldProcessEvent()) return
		if (!isConnected.value) isConnected.value = true
		cpuData.value = appendGraphData(cpuData.value, currentStats.cpu_percent)
		ramData.value = appendGraphData(
			ramData.value,
			Math.floor((currentStats.ram_usage_bytes / currentStats.ram_total_bytes) * 100),
		)
		stats.value = {
			current: currentStats,
			past: { ...stats.value.current },
			graph: {
				cpu: cpuData.value,
				ram: ramData.value,
			},
		}
	}

	const clearStaleStatsTimers = () => {
		if (staleStatsTimeoutId) {
			clearTimeout(staleStatsTimeoutId)
			staleStatsTimeoutId = null
		}
		if (staleStatsIntervalId) {
			clearInterval(staleStatsIntervalId)
			staleStatsIntervalId = null
		}
	}

	const pushZeroStats = () => {
		if (!shouldProcessEvent()) return
		cpuData.value = appendGraphData(cpuData.value, 0)
		ramData.value = appendGraphData(ramData.value, 0)
		stats.value = {
			current: {
				...stats.value.current,
				cpu_percent: 0,
				ram_usage_bytes: 0,
			},
			past: { ...stats.value.current },
			graph: {
				cpu: cpuData.value,
				ram: ramData.value,
			},
		}
	}

	const armStaleStatsWatchdog = () => {
		clearStaleStatsTimers()
		staleStatsTimeoutId = setTimeout(() => {
			pushZeroStats()
			staleStatsIntervalId = setInterval(pushZeroStats, STALE_STATS_PUSH_INTERVAL_MS)
		}, STALE_STATS_THRESHOLD_MS)
	}

	const updatePowerState = (
		state: Archon.Websocket.v0.PowerState,
		details?: { oom_killed?: boolean; exit_code?: number },
	) => {
		if (!shouldProcessEvent()) return
		serverPowerState.value = state
		powerStateDetails.value = state === 'crashed' ? details : undefined
		if (state === 'stopped' || state === 'crashed') {
			stopUptimeTicker()
			uptimeSeconds.value = 0
		}
	}

	const handleLog = (data: Archon.Websocket.v0.WSLogEvent) => {
		if (!shouldProcessEvent()) return
		modrinthServersConsole.recordWsEvent({ event: 'log', ...data })
		modrinthServersConsole.addLegacyLog(data.message)
	}

	const handleLog4j = (data: Archon.Websocket.v0.WSLog4jEvent) => {
		if (!shouldProcessEvent()) return
		modrinthServersConsole.recordWsEvent({ event: 'log4j', ...data })
		modrinthServersConsole.addLog4jEvent(data)
	}

	const handleStats = (data: Archon.Websocket.v0.WSStatsEvent) => {
		armStaleStatsWatchdog()
		updateStats({
			cpu_percent: data.cpu_percent,
			ram_usage_bytes: data.ram_usage_bytes,
			ram_total_bytes: data.ram_total_bytes,
			storage_usage_bytes: data.storage_usage_bytes,
			storage_total_bytes: data.storage_total_bytes,
		})
	}

	const handlePowerState = (data: Archon.Websocket.v0.WSPowerStateEvent) => {
		if (data.state === 'crashed') {
			updatePowerState(data.state, {
				oom_killed: data.oom_killed,
				exit_code: data.exit_code,
			})
		} else {
			updatePowerState(data.state)
		}
	}

	const handleState = (data: Archon.Websocket.v0.WSStateEvent) => {
		if (!shouldProcessEvent()) return
		options.onStateEvent?.(data)
		updatePowerState(mapPowerStateFromStateEvent(data), {
			exit_code: data.exit_code ?? undefined,
			oom_killed: data.was_oom,
		})

		if (options.syncUptimeFromState && data.uptime > 0) {
			stopUptimeTicker()
			uptimeSeconds.value = data.uptime
			startUptimeTicker()
		}
	}

	const handleUptime = (data: Archon.Websocket.v0.WSUptimeEvent) => {
		if (!shouldProcessEvent()) return
		stopUptimeTicker()
		uptimeSeconds.value = data.uptime
		startUptimeTicker()
	}

	const clearSocketListeners = () => {
		for (const unsub of socketUnsubscribers.value) unsub()
		socketUnsubscribers.value = []
	}

	const disconnectSocket = (targetServerId?: string) => {
		if (!targetServerId && !connectedSocketServerId.value) return

		clearSocketListeners()
		serverContextRuntimeLease?.release()
		serverContextRuntimeLease = null

		stopUptimeTicker()
		clearStaleStatsTimers()
		connectedSocketServerId.value = null
		isConnected.value = false
		isWsAuthIncorrect.value = false
		serverPowerState.value = 'stopped'
		powerStateDetails.value = undefined
		uptimeSeconds.value = 0
		resetInstallation()
	}

	const connectSocket = async (
		targetServerId: string,
		connectOptions: ConnectSocketOptions = {},
	): Promise<boolean> => {
		if (
			connectedSocketServerId.value === targetServerId &&
			(isConnected.value || isWsAuthIncorrect.value)
		) {
			return true
		}

		disconnectSocket(connectedSocketServerId.value ?? undefined)

		try {
			const runtimeLease = retainServerContextRuntime(client, targetServerId, {
				connect: false,
			})
			serverContextRuntimeLease = runtimeLease
			connectedSocketServerId.value = targetServerId

			const baseSubscriptions: SocketUnsubscriber[] = [
				client.archon.sockets.on(targetServerId, 'log', handleLog),
				client.archon.sockets.on(targetServerId, 'log4j', handleLog4j),
				client.archon.sockets.on(targetServerId, 'stats', handleStats),
				client.archon.sockets.on(targetServerId, 'state', handleState),
				client.archon.sockets.on(targetServerId, 'power-state', handlePowerState),
				client.archon.sockets.on(targetServerId, 'uptime', handleUptime),
				watch(
					runtimeLease.installProgressItems,
					(items) => {
						if (shouldProcessEvent()) handleInstallProgress(items)
					},
					{ immediate: true },
				),
				watch(
					runtimeLease.isSocketAuthenticated,
					(authenticated) => {
						if (!shouldProcessEvent()) return
						if (authenticated || options.setDisconnectedOnAuthIncorrect) {
							isConnected.value = authenticated
						}
					},
					{ immediate: true },
				),
				watch(
					runtimeLease.isSocketAuthIncorrect,
					(authIncorrect) => {
						if (shouldProcessEvent()) isWsAuthIncorrect.value = authIncorrect
					},
					{ immediate: true },
				),
			]
			const extraSubscriptions = connectOptions.extraSubscriptions?.(targetServerId) ?? []
			socketUnsubscribers.value = [...baseSubscriptions, ...extraSubscriptions]

			modrinthServersConsole.clear()
			modrinthServersConsole.beginInitialLogHydration()

			await runtimeLease.waitUntilReady()
			isConnected.value = true
			isWsAuthIncorrect.value = false

			return true
		} catch (error) {
			console.error('[hosting/manage] Failed to connect server socket:', error)
			disconnectSocket(targetServerId)
			return false
		}
	}

	const uploadState = ref<UploadState>({
		isUploading: false,
		currentFileName: null,
		currentFileProgress: 0,
		uploadedBytes: 0,
		totalBytes: 0,
		completedFiles: 0,
		totalFiles: 0,
	})
	const cancelUpload = ref<CancelUploadHandler | null>(null)

	type QueuedOpWithState = Archon.Websocket.v0.QueuedFilesystemOp & { state: 'queued' }
	const dismissedOpIds = ref<Set<string>>(new Set())

	const activeOperations = computed<FileOperation[]>(() => [
		...fsQueuedOps.value.map((x) => ({ ...x, state: 'queued' }) satisfies QueuedOpWithState),
		...(fsOps.value.filter((op) => !op.id || !dismissedOpIds.value.has(op.id)) as FileOperation[]),
	])

	async function dismissOperation(opId: string, action: 'dismiss' | 'cancel') {
		if (action === 'dismiss') {
			dismissedOpIds.value = new Set([...dismissedOpIds.value, opId])
		}
		try {
			await client.kyros.files_v0.modifyOperation(opId, action)
		} catch (error) {
			if (action === 'dismiss') return
			console.error(`Failed to ${action} operation:`, error)
		}
	}

	const refreshFsAuth = async () => {
		if (!options.serverId.value) {
			fsAuth.value = null
			return
		}
		fsAuth.value = await client.archon.servers_v0.getFilesystemAuth(options.serverId.value)
	}
	const currentUserPermissions = computed(() => options.server.value?.current_user_permissions ?? 0)
	const serverFull = computed(() => options.serverFull?.value ?? null)

	provideModrinthServerContext({
		get serverId() {
			return options.serverId.value
		},
		worldId: options.worldId as Ref<string | null>,
		server: options.server as Ref<Archon.Servers.v0.Server>,
		serverFull,
		currentUserPermissions,
		isConnected,
		isWsAuthIncorrect,
		powerState: serverPowerState,
		powerStateDetails,
		isServerRunning,
		stats,
		uptimeSeconds,
		installProgressItems,
		installation,
		beginInstallation,
		cancelOptimisticInstallation,
		dismissInstallation,
		busyReasons,
		fsAuth,
		fsOps,
		fsQueuedOps,
		refreshFsAuth,
		uploadState,
		cancelUpload,
		activeOperations,
		dismissOperation,
	})

	setNodeAuthState(() => fsAuth.value, refreshFsAuth)

	const cleanupCoreRuntime = (targetServerId?: string) => {
		disconnectSocket(targetServerId ?? connectedSocketServerId.value ?? undefined)
		clearNodeAuthState()
	}

	return {
		activeOperations,
		beginInstallation,
		busyReasons,
		cancelUpload,
		cancelOptimisticInstallation,
		cleanupCoreRuntime,
		connectSocket,
		connectedSocketServerId,
		cpuData,
		disconnectSocket,
		dismissInstallation,
		dismissOperation,
		fsAuth,
		fsOps,
		fsQueuedOps,
		installation,
		isConnected,
		isServerRunning,
		isWsAuthIncorrect,
		installProgressItems,
		powerStateDetails,
		ramData,
		refreshFsAuth,
		serverPowerState,
		stats,
		uptimeSeconds,
		uploadState,
	}
}
