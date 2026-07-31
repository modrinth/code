import type { AbstractModrinthClient, Archon } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'
import { onUnmounted, ref, watch } from 'vue'

import { injectModrinthClient } from '../providers'

type ReadableRef<T> = Ref<T> | ComputedRef<T>
type RuntimeUnsubscriber = () => void

type RuntimeReadyWaiter = {
	resolve: () => void
	reject: (error: Error) => void
	timeout: ReturnType<typeof setTimeout>
}

type ServerContextRuntime = {
	client: AbstractModrinthClient
	serverId: string
	leases: number
	socketLeases: number
	syncLeases: number
	releaseTimer: ReturnType<typeof setTimeout> | null
	socketReleaseTimer: ReturnType<typeof setTimeout> | null
	syncReleaseTimer: ReturnType<typeof setTimeout> | null
	connectPromise: Promise<void> | null
	socketUnsubscribers: RuntimeUnsubscriber[]
	installProgressItems: Ref<Archon.Websocket.v0.InstallProgressItem[]>
	isSocketAuthenticated: Ref<boolean>
	isSocketAuthIncorrect: Ref<boolean>
	hasAuthoritativeInstallProgress: Ref<boolean>
	readyWaiters: Set<RuntimeReadyWaiter>
	destroyed: boolean
}

export type ServerContextRuntimeLease = {
	serverId: string
	installProgressItems: Ref<Archon.Websocket.v0.InstallProgressItem[]>
	isSocketAuthenticated: Ref<boolean>
	isSocketAuthIncorrect: Ref<boolean>
	hasAuthoritativeInstallProgress: Ref<boolean>
	waitUntilReady: () => Promise<void>
	release: () => void
}

type RetainServerContextRuntimeOptions = {
	connect?: boolean
	socket?: boolean
	sync?: boolean
}

const runtimeReleaseDelay = 1000
const authoritativeReadinessTimeout = 30000
const runtimesByClient = new WeakMap<AbstractModrinthClient, Map<string, ServerContextRuntime>>()

function getClientRuntimes(client: AbstractModrinthClient) {
	let runtimes = runtimesByClient.get(client)
	if (!runtimes) {
		runtimes = new Map()
		runtimesByClient.set(client, runtimes)
	}
	return runtimes
}

function isRuntimeReady(runtime: ServerContextRuntime) {
	return runtime.isSocketAuthenticated.value && runtime.hasAuthoritativeInstallProgress.value
}

function resolveReadyWaiters(runtime: ServerContextRuntime) {
	if (!isRuntimeReady(runtime)) return

	for (const waiter of runtime.readyWaiters) {
		clearTimeout(waiter.timeout)
		waiter.resolve()
	}
	runtime.readyWaiters.clear()
}

function createServerContextRuntime(
	client: AbstractModrinthClient,
	serverId: string,
): ServerContextRuntime {
	const runtime: ServerContextRuntime = {
		client,
		serverId,
		leases: 0,
		socketLeases: 0,
		syncLeases: 0,
		releaseTimer: null,
		socketReleaseTimer: null,
		syncReleaseTimer: null,
		connectPromise: null,
		socketUnsubscribers: [],
		installProgressItems: ref([]),
		isSocketAuthenticated: ref(false),
		isSocketAuthIncorrect: ref(false),
		hasAuthoritativeInstallProgress: ref(false),
		readyWaiters: new Set(),
		destroyed: false,
	}

	return runtime
}

function attachRuntimeSocketListeners(runtime: ServerContextRuntime) {
	if (runtime.socketUnsubscribers.length > 0) return

	runtime.socketUnsubscribers = [
		runtime.client.archon.sockets.on(runtime.serverId, 'auth-ok', () => {
			runtime.isSocketAuthenticated.value = true
			runtime.isSocketAuthIncorrect.value = false
			runtime.hasAuthoritativeInstallProgress.value = false
		}),
		runtime.client.archon.sockets.on(runtime.serverId, 'auth-incorrect', () => {
			runtime.isSocketAuthenticated.value = false
			runtime.isSocketAuthIncorrect.value = true
			runtime.hasAuthoritativeInstallProgress.value = false
		}),
		runtime.client.archon.sockets.on(runtime.serverId, 'install-progress', (event) => {
			runtime.installProgressItems.value = event.items
			runtime.hasAuthoritativeInstallProgress.value = true
			resolveReadyWaiters(runtime)
		}),
	]
}

function disconnectRuntimeSocket(runtime: ServerContextRuntime) {
	for (const unsubscribe of runtime.socketUnsubscribers) unsubscribe()
	runtime.socketUnsubscribers = []
	runtime.client.archon.sockets.disconnect(runtime.serverId)
	runtime.connectPromise = null
	runtime.isSocketAuthenticated.value = false
	runtime.isSocketAuthIncorrect.value = false
	runtime.hasAuthoritativeInstallProgress.value = false
	for (const waiter of runtime.readyWaiters) {
		clearTimeout(waiter.timeout)
		waiter.reject(new Error(`Node socket for server ${runtime.serverId} was released`))
	}
	runtime.readyWaiters.clear()
}

function disconnectRuntimeSync(runtime: ServerContextRuntime) {
	runtime.client.archon.sync.disconnect(runtime.serverId)
}

async function ensureRuntimeConnections(
	runtime: ServerContextRuntime,
	options: RetainServerContextRuntimeOptions = {},
) {
	if (runtime.destroyed) {
		throw new Error(`Server context runtime for ${runtime.serverId} has been released`)
	}

	const shouldConnectSocket = options.socket !== false
	const shouldConnectSync = options.sync !== false
	const socketStatus = runtime.client.archon.sockets.getStatus(runtime.serverId)
	if (shouldConnectSocket && !socketStatus?.connected) {
		attachRuntimeSocketListeners(runtime)
		runtime.isSocketAuthenticated.value = false
		runtime.hasAuthoritativeInstallProgress.value = false
	}

	if (shouldConnectSync) {
		void runtime.client.archon.sync
			.safeConnectServer(runtime.serverId, { intent: 'all' })
			.catch((error) => {
				console.warn(
					`[server-context-runtime] Failed to connect sync stream for ${runtime.serverId}:`,
					error,
				)
			})
	}

	if (shouldConnectSocket && !runtime.connectPromise) {
		const connectPromise = runtime.client.archon.sockets
			.safeConnect(runtime.serverId)
			.then(() => {
				runtime.isSocketAuthenticated.value = true
			})
			.finally(() => {
				if (runtime.connectPromise === connectPromise) {
					runtime.connectPromise = null
				}
			})
		runtime.connectPromise = connectPromise
	}

	if (runtime.connectPromise) await runtime.connectPromise
}

async function waitUntilRuntimeReady(runtime: ServerContextRuntime) {
	await ensureRuntimeConnections(runtime)
	if (isRuntimeReady(runtime)) return

	await new Promise<void>((resolve, reject) => {
		const waiter: RuntimeReadyWaiter = {
			resolve,
			reject,
			timeout: setTimeout(() => {
				runtime.readyWaiters.delete(waiter)
				reject(
					new Error(
						`Timed out waiting for authoritative install progress for server ${runtime.serverId}`,
					),
				)
			}, authoritativeReadinessTimeout),
		}
		runtime.readyWaiters.add(waiter)
		resolveReadyWaiters(runtime)
	})
}

function destroyRuntime(runtime: ServerContextRuntime) {
	if (runtime.destroyed || runtime.leases > 0) return
	runtime.destroyed = true

	if (runtime.socketReleaseTimer) clearTimeout(runtime.socketReleaseTimer)
	if (runtime.syncReleaseTimer) clearTimeout(runtime.syncReleaseTimer)
	disconnectRuntimeSocket(runtime)
	disconnectRuntimeSync(runtime)

	getClientRuntimes(runtime.client).delete(runtime.serverId)
}

export function retainServerContextRuntime(
	client: AbstractModrinthClient,
	serverId: string,
	options: RetainServerContextRuntimeOptions = {},
): ServerContextRuntimeLease {
	const runtimes = getClientRuntimes(client)
	let runtime = runtimes.get(serverId)
	if (!runtime) {
		runtime = createServerContextRuntime(client, serverId)
		runtimes.set(serverId, runtime)
	}

	if (runtime.releaseTimer) {
		clearTimeout(runtime.releaseTimer)
		runtime.releaseTimer = null
	}
	const retainSocket = options.socket !== false
	const retainSync = options.sync !== false
	if (retainSocket) {
		if (runtime.socketReleaseTimer) {
			clearTimeout(runtime.socketReleaseTimer)
			runtime.socketReleaseTimer = null
		}
		attachRuntimeSocketListeners(runtime)
		runtime.socketLeases += 1
	}
	if (retainSync) {
		if (runtime.syncReleaseTimer) {
			clearTimeout(runtime.syncReleaseTimer)
			runtime.syncReleaseTimer = null
		}
		runtime.syncLeases += 1
	}
	runtime.leases += 1
	if (options.connect !== false) {
		void ensureRuntimeConnections(runtime, options).catch((error) => {
			if (runtime && runtime.leases > 0) {
				console.warn(
					`[server-context-runtime] Failed to connect node socket for ${serverId}:`,
					error,
				)
			}
		})
	}

	let released = false
	return {
		serverId,
		installProgressItems: runtime.installProgressItems,
		isSocketAuthenticated: runtime.isSocketAuthenticated,
		isSocketAuthIncorrect: runtime.isSocketAuthIncorrect,
		hasAuthoritativeInstallProgress: runtime.hasAuthoritativeInstallProgress,
		waitUntilReady: () => waitUntilRuntimeReady(runtime),
		release: () => {
			if (released) return
			released = true
			runtime.leases = Math.max(0, runtime.leases - 1)
			if (retainSocket) {
				runtime.socketLeases = Math.max(0, runtime.socketLeases - 1)
			}
			if (retainSync) {
				runtime.syncLeases = Math.max(0, runtime.syncLeases - 1)
			}

			if (runtime.leases === 0) {
				if (runtime.socketReleaseTimer) clearTimeout(runtime.socketReleaseTimer)
				if (runtime.syncReleaseTimer) clearTimeout(runtime.syncReleaseTimer)
				runtime.socketReleaseTimer = null
				runtime.syncReleaseTimer = null
				runtime.releaseTimer = setTimeout(() => {
					runtime.releaseTimer = null
					destroyRuntime(runtime)
				}, runtimeReleaseDelay)
				return
			}

			if (retainSocket && runtime.socketLeases === 0) {
				runtime.socketReleaseTimer = setTimeout(() => {
					runtime.socketReleaseTimer = null
					if (runtime.socketLeases === 0) disconnectRuntimeSocket(runtime)
				}, runtimeReleaseDelay)
			}
			if (retainSync && runtime.syncLeases === 0) {
				runtime.syncReleaseTimer = setTimeout(() => {
					runtime.syncReleaseTimer = null
					if (runtime.syncLeases === 0) disconnectRuntimeSync(runtime)
				}, runtimeReleaseDelay)
			}
		},
	}
}

export function useServerContextRuntime(serverId: ReadableRef<string | null>) {
	const client = injectModrinthClient()
	let lease: ServerContextRuntimeLease | null = null

	const stop = watch(
		() => serverId.value,
		(nextServerId) => {
			lease?.release()
			lease = null

			if (typeof window !== 'undefined' && nextServerId) {
				lease = retainServerContextRuntime(client, nextServerId)
			}
		},
		{ immediate: true },
	)

	onUnmounted(() => {
		stop()
		lease?.release()
		lease = null
	})
}

export async function waitForServerContextRuntimeReady(
	client: AbstractModrinthClient,
	serverId: string,
) {
	const lease = retainServerContextRuntime(client, serverId)
	try {
		await lease.waitUntilReady()
	} finally {
		lease.release()
	}
}
