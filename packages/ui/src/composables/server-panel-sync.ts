import type { Archon } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'
import { onMounted, onUnmounted, watch } from 'vue'

import { injectModrinthClient } from '#ui/providers'

type ReadableRef<T> = Ref<T> | ComputedRef<T>
type SyncUnsubscriber = () => void

type UseServerPanelSyncOptions = {
	serverId: ReadableRef<string>
	worldId: ReadableRef<string | null>
}

const ACTION_LOG_INVALIDATE_DELAY_MS = 500

export function useServerPanelSync(options: UseServerPanelSyncOptions) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	let activeServerId: string | null = null
	let unsubscribers: SyncUnsubscriber[] = []
	let mounted = false
	let actionLogInvalidateTimer: ReturnType<typeof setTimeout> | null = null

	const legacyServerDetailKey = (serverId: string) => ['servers', 'detail', serverId] as const
	const serverV1DetailKey = (serverId: string) => ['servers', 'v1', 'detail', serverId] as const
	const actionLogBaseKey = (serverId: string) =>
		['servers', 'action-log', 'v1', 'infinite', serverId] as const

	function connect(targetServerId: string) {
		if (!targetServerId || activeServerId === targetServerId) return

		disconnect()
		activeServerId = targetServerId

		if (!client.archon.sync.getStatus(targetServerId)?.lastEventId) {
			void invalidateCorePanelQueries(targetServerId)
		}

		unsubscribers = [
			client.archon.sync.onAny(targetServerId, (event) => handleSyncEvent(targetServerId, event)),
		]

		void client.archon.sync.safeConnectServer(targetServerId, { intent: 'all' }).catch((error) => {
			console.warn(`[server-panel-sync] Failed to connect sync stream for ${targetServerId}:`, error)
		})
	}

	function disconnect() {
		if (actionLogInvalidateTimer) {
			clearTimeout(actionLogInvalidateTimer)
			actionLogInvalidateTimer = null
		}

		for (const unsubscribe of unsubscribers) unsubscribe()
		unsubscribers = []

		if (activeServerId) {
			client.archon.sync.disconnect(activeServerId)
			activeServerId = null
		}
	}

	function handleSyncEvent(serverId: string, event: Archon.Sync.v1.SyncEvent) {
		if (event.type === 'protocol.reset' || event.type === 'protocol.invalid') {
			void invalidateCorePanelQueries(serverId)
			return
		}

		if (event.type === 'protocol.error') {
			console.warn(`[server-panel-sync] Protocol error for ${serverId}: ${event.error}`)
			return
		}

		scheduleActionLogInvalidation(serverId)

		if (event.type.startsWith('backup.')) {
			handleBackupEvent(serverId)
			return
		}

		switch (event.type) {
			case 'server.patch':
				handleServerPatch(serverId, event)
				break
			case 'server.network.patch':
				handleServerNetworkPatch(serverId, event)
				break
			case 'server.transfer.start':
			case 'server.transfer.done':
				void invalidateServerDetails(serverId)
				break
			case 'users.patch':
				handleUsersPatch(serverId)
				break
			case 'world.patch':
				handleWorldPatch(serverId, event)
				break
			case 'world.startup.patch':
				handleWorldStartupPatch(serverId, event)
				break
			case 'world.content.addon.patch':
				handleWorldContentAddonPatch(serverId, event)
				break
			case 'world.content.base.update':
				handleWorldContentBaseUpdate(serverId, event)
				break
		}
	}

	function handleServerPatch(serverId: string, event: Archon.Sync.v1.ServerPatchEvent) {
		queryClient.setQueryData<Archon.Servers.v0.Server>(
			legacyServerDetailKey(serverId),
			(current) =>
				current
					? {
							...current,
							name: event.name,
							net: {
								...current.net,
								domain: event.subdomain,
							},
						}
					: current,
		)
		queryClient.setQueryData<Archon.Servers.v1.ServerFull>(
			serverV1DetailKey(serverId),
			(current) =>
				current
					? {
							...current,
							name: event.name,
							subdomain: event.subdomain,
						}
					: current,
		)
	}

	function handleServerNetworkPatch(
		serverId: string,
		event: Archon.Sync.v1.ServerNetworkPatchEvent,
	) {
		queryClient.setQueryData<Archon.Servers.v0.Allocation[]>(
			['servers', 'allocations', serverId],
			event.ports,
		)
		void invalidateServerDetails(serverId)
	}

	function handleUsersPatch(serverId: string) {
		void queryClient.invalidateQueries({ queryKey: ['servers', 'users', 'v1', serverId] })
		void invalidateServerDetails(serverId)
	}

	function handleWorldPatch(serverId: string, event: Archon.Sync.v1.WorldPatchEvent) {
		patchServerFullWorld(serverId, event.world_id, (world) => ({
			...world,
			name: event.name,
		}))
	}

	function handleWorldStartupPatch(
		serverId: string,
		event: Archon.Sync.v1.WorldStartupPatchEvent,
	) {
		patchServerFullWorld(serverId, event.world_id, (world) =>
			world.content
				? {
						...world,
						content: {
							...world.content,
							java_version: event.java_version,
							invocation: event.invocation,
							original_invocation: event.original_invocation,
						},
					}
				: world,
		)
		void queryClient.invalidateQueries({ queryKey: ['servers', 'startup', 'v1', serverId] })
	}

	function handleWorldContentAddonPatch(
		serverId: string,
		event: Archon.Sync.v1.WorldContentAddonPatchEvent,
	) {
		if (event.world_id !== options.worldId.value) {
			void invalidateContentAndServerDetails(serverId)
			return
		}

		queryClient.setQueryData<Archon.Content.v1.Addons>(
			['content', 'list', 'v1', serverId],
			(current) =>
				current
					? {
							...current,
							addons: event.specs,
						}
					: current,
		)
	}

	function handleWorldContentBaseUpdate(
		serverId: string,
		event: Archon.Sync.v1.WorldContentBaseUpdateEvent,
	) {
		if (event.world_id === options.worldId.value) {
			queryClient.setQueryData<Archon.Content.v1.Addons>(
				['content', 'list', 'v1', serverId],
				(current) => (current ? { ...current, ...event.spec } : event.spec),
			)
		} else {
			void queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] })
		}

		void queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) })
	}

	function handleBackupEvent(serverId: string) {
		void queryClient.invalidateQueries({ queryKey: ['backups', 'queue', serverId] })
		void invalidateServerDetails(serverId)
	}

	function patchServerFullWorld(
		serverId: string,
		worldId: string,
		patch: (world: Archon.Servers.v1.WorldFull) => Archon.Servers.v1.WorldFull,
	) {
		queryClient.setQueryData<Archon.Servers.v1.ServerFull>(serverV1DetailKey(serverId), (current) =>
			current
				? {
						...current,
						worlds: current.worlds.map((world) => (world.id === worldId ? patch(world) : world)),
					}
				: current,
		)
	}

	function scheduleActionLogInvalidation(serverId: string) {
		if (actionLogInvalidateTimer) clearTimeout(actionLogInvalidateTimer)

		actionLogInvalidateTimer = setTimeout(() => {
			actionLogInvalidateTimer = null
			void queryClient.invalidateQueries({ queryKey: actionLogBaseKey(serverId) })
		}, ACTION_LOG_INVALIDATE_DELAY_MS)
	}

	async function invalidateServerDetails(serverId: string) {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: legacyServerDetailKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) }),
		])
	}

	async function invalidateContentAndServerDetails(serverId: string) {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
			invalidateServerDetails(serverId),
		])
	}

	async function invalidateCorePanelQueries(serverId: string) {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: legacyServerDetailKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['backups', 'queue', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'users', 'v1', serverId] }),
			queryClient.invalidateQueries({ queryKey: actionLogBaseKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'startup', 'v1', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] }),
		])
	}

	onMounted(() => {
		mounted = true
		connect(options.serverId.value)
	})

	watch(
		() => options.serverId.value,
		(serverId) => {
			if (!mounted) return
			if (serverId) {
				connect(serverId)
			} else {
				disconnect()
			}
		},
	)

	onUnmounted(() => {
		mounted = false
		disconnect()
	})

	return {
		disconnect,
	}
}
