import {
	type Archon,
	clearNodeAuthState,
	setNodeAuthState,
	type UploadState,
} from '@modrinth/api-client'
import type { Stats } from '@modrinth/utils'
import type { ComputedRef, Ref } from 'vue'
import { computed, reactive, ref, watch } from 'vue'

import type { FileOperation } from '../layouts/shared/files-tab/types'
import { injectModrinthClient, provideModrinthServerContext } from '../providers'
import type { BusyReason } from '../providers/server-context'
import { defineMessage } from './i18n'
import { useModrinthServersConsole } from './server-console'

type ReadableRef<T> = Ref<T> | ComputedRef<T>
type SocketUnsubscriber = () => void

type ConnectSocketOptions = {
	force?: boolean
	extraSubscriptions?: (targetServerId: string) => SocketUnsubscriber[]
}

type UseServerManageCoreRuntimeOptions = {
	serverId: ReadableRef<string>
	worldId: ReadableRef<string | null>
	server: ReadableRef<Archon.Servers.v0.Server | null | undefined>
	isSyncingContent: ReadableRef<boolean>
	markBackupCancelled?: (backupId: string) => void
	includeBackupBusyReasons?: boolean
	setDisconnectedOnAuthIncorrect?: boolean
	syncUptimeFromState?: boolean
	incrementUptimeLocally?: boolean
	eventGuard?: () => boolean
	onStateEvent?: (data: Archon.Websocket.v0.WSStateEvent) => void
}

const createInitialStats = (): Stats => ({
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
	const stats = ref<Stats>(createInitialStats())
	const uptimeSeconds = ref(0)
	const backupsState = reactive(new Map())
	const fsAuth = ref<{ url: string; token: string } | null>(null)
	const fsOps = ref<Archon.Websocket.v0.FilesystemOperation[]>([])
	const fsQueuedOps = ref<Archon.Websocket.v0.QueuedFilesystemOp[]>([])
	const connectedSocketServerId = ref<string | null>(null)
	const socketUnsubscribers = ref<SocketUnsubscriber[]>([])
	const cpuData = ref<number[]>([])
	const ramData = ref<number[]>([])

	let uptimeIntervalId: ReturnType<typeof setInterval> | null = null
	let staleStatsTimeoutId: ReturnType<typeof setTimeout> | null = null
	let staleStatsIntervalId: ReturnType<typeof setInterval> | null = null

	const markBackupCancelled =
		options.markBackupCancelled ??
		((backupId: string) => {
			backupsState.delete(backupId)
		})

	const busyReasons = computed<BusyReason[]>(() => {
		const reasons: BusyReason[] = []
		if (options.server.value?.status === 'installing') {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.installing',
					defaultMessage: 'Server is installing',
				}),
			})
		}
		if (options.isSyncingContent.value) {
			reasons.push({
				reason: defineMessage({
					id: 'servers.busy.syncing-content',
					defaultMessage: 'Content sync in progress',
				}),
			})
		}
		if (options.includeBackupBusyReasons) {
			for (const entry of backupsState.values()) {
				if (entry.create?.state === 'ongoing') {
					reasons.push({
						reason: defineMessage({
							id: 'servers.busy.backup-creating',
							defaultMessage: 'Backup creation in progress',
						}),
					})
					break
				}
				if (entry.restore?.state === 'ongoing') {
					reasons.push({
						reason: defineMessage({
							id: 'servers.busy.backup-restoring',
							defaultMessage: 'Backup restore in progress',
						}),
					})
					break
				}
			}
		}
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

	const updateStats = (currentStats: Stats['current']) => {
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

	const handleAuthIncorrect = () => {
		if (!shouldProcessEvent()) return
		isWsAuthIncorrect.value = true
		if (options.setDisconnectedOnAuthIncorrect) {
			isConnected.value = false
		}
	}

	const handleAuthOk = () => {
		if (!shouldProcessEvent()) return
		isWsAuthIncorrect.value = false
		isConnected.value = true
	}

	const clearSocketListeners = () => {
		for (const unsub of socketUnsubscribers.value) unsub()
		socketUnsubscribers.value = []
	}

	const disconnectSocket = (targetServerId?: string) => {
		if (!targetServerId && !connectedSocketServerId.value) return

		clearSocketListeners()

		if (targetServerId) {
			client.archon.sockets.disconnect(targetServerId)
		}

		stopUptimeTicker()
		clearStaleStatsTimers()
		connectedSocketServerId.value = null
		isConnected.value = false
		isWsAuthIncorrect.value = false
		serverPowerState.value = 'stopped'
		powerStateDetails.value = undefined
		uptimeSeconds.value = 0
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
			const safeConnectOptions = connectOptions.force ? { force: true } : undefined
			await client.archon.sockets.safeConnect(targetServerId, safeConnectOptions)
			connectedSocketServerId.value = targetServerId
			isConnected.value = true
			isWsAuthIncorrect.value = false

			modrinthServersConsole.clear()

			const baseSubscriptions: SocketUnsubscriber[] = [
				client.archon.sockets.on(targetServerId, 'log', handleLog),
				client.archon.sockets.on(targetServerId, 'log4j', handleLog4j),
				client.archon.sockets.on(targetServerId, 'stats', handleStats),
				client.archon.sockets.on(targetServerId, 'state', handleState),
				client.archon.sockets.on(targetServerId, 'power-state', handlePowerState),
				client.archon.sockets.on(targetServerId, 'uptime', handleUptime),
				client.archon.sockets.on(targetServerId, 'auth-incorrect', handleAuthIncorrect),
				client.archon.sockets.on(targetServerId, 'auth-ok', handleAuthOk),
			]
			const extraSubscriptions = connectOptions.extraSubscriptions?.(targetServerId) ?? []
			socketUnsubscribers.value = [...baseSubscriptions, ...extraSubscriptions]
			return true
		} catch (error) {
			console.error('[hosting/manage] Failed to connect server socket:', error)
			isConnected.value = false
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
	const cancelUpload = ref<(() => void) | null>(null)

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

	watch(
		() => fsOps.value,
		(newOps) => {
			for (const op of newOps) {
				if (op.state === 'done' && op.id && !dismissedOpIds.value.has(op.id)) {
					setTimeout(() => {
						dismissOperation(op.id!, 'dismiss')
					}, 3000)
				}
			}
		},
		{ deep: true },
	)

	const refreshFsAuth = async () => {
		if (!options.serverId.value) {
			fsAuth.value = null
			return
		}
		fsAuth.value = await client.archon.servers_v0.getFilesystemAuth(options.serverId.value)
	}

	provideModrinthServerContext({
		get serverId() {
			return options.serverId.value
		},
		worldId: options.worldId as Ref<string | null>,
		server: options.server as Ref<Archon.Servers.v0.Server>,
		isConnected,
		isWsAuthIncorrect,
		powerState: serverPowerState,
		powerStateDetails,
		isServerRunning,
		stats,
		uptimeSeconds,
		backupsState,
		markBackupCancelled,
		isSyncingContent: options.isSyncingContent as Ref<boolean>,
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
		backupsState,
		busyReasons,
		cancelUpload,
		cleanupCoreRuntime,
		connectSocket,
		connectedSocketServerId,
		cpuData,
		disconnectSocket,
		dismissOperation,
		fsAuth,
		fsOps,
		fsQueuedOps,
		isConnected,
		isServerRunning,
		isWsAuthIncorrect,
		powerStateDetails,
		ramData,
		refreshFsAuth,
		serverPowerState,
		stats,
		uptimeSeconds,
		uploadState,
	}
}
