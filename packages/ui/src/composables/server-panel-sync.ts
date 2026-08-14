import type { Archon } from '@modrinth/api-client'
import { useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef, Ref } from 'vue'
import { onMounted, onUnmounted, watch } from 'vue'

import { injectModrinthClient } from '#ui/providers'

import {
	retainServerContextRuntime,
	type ServerContextRuntimeLease,
} from './server-context-runtime'

type ReadableRef<T> = Ref<T> | ComputedRef<T>
type SyncUnsubscriber = () => void

type UseServerPanelSyncOptions = {
	serverId: ReadableRef<string | null>
	worldId: ReadableRef<string | null>
}

const ACTION_LOG_INVALIDATE_DELAY_MS = 500

export function useServerPanelSync(options: UseServerPanelSyncOptions) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()

	let activeServerId: string | null = null
	let runtimeLease: ServerContextRuntimeLease | null = null
	let unsubscribers: SyncUnsubscriber[] = []
	let mounted = false
	let actionLogInvalidateTimer: ReturnType<typeof setTimeout> | null = null

	const legacyServerDetailKey = (serverId: string) => ['servers', 'detail', serverId] as const
	const serverV1DetailKey = (serverId: string) => ['servers', 'v1', 'detail', serverId] as const
	const contentListKey = (serverId: string) => ['content', 'list', 'v1', serverId] as const
	const modpackContentListKey = (serverId: string) =>
		['content', 'list', 'v1', serverId, 'modpack'] as const
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
		runtimeLease = retainServerContextRuntime(client, targetServerId, {
			socket: false,
			sync: true,
		})
	}

	function disconnect() {
		if (actionLogInvalidateTimer) {
			clearTimeout(actionLogInvalidateTimer)
			actionLogInvalidateTimer = null
		}

		for (const unsubscribe of unsubscribers) unsubscribe()
		unsubscribers = []

		runtimeLease?.release()
		runtimeLease = null
		activeServerId = null
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
			case 'world.content.update':
				handleWorldContentUpdate(serverId, event)
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

	function handleWorldStartupPatch(serverId: string, event: Archon.Sync.v1.WorldStartupPatchEvent) {
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

		queryClient.setQueryData<Archon.Content.v1.Addons>(contentListKey(serverId), (current) =>
			current
				? {
						...current,
						addons: mergeAddonSpecs(current.addons ?? [], event.specs),
					}
				: current,
		)
	}

	function handleWorldContentBaseUpdate(
		serverId: string,
		event: Archon.Sync.v1.WorldContentBaseUpdateEvent,
	) {
		if (event.world_id === options.worldId.value) {
			queryClient.setQueryData<Archon.Content.v1.Addons>(contentListKey(serverId), (current) =>
				current ? { ...current, ...event.spec } : event.spec,
			)
		} else {
			void queryClient.invalidateQueries({ queryKey: contentListKey(serverId) })
		}

		void queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) })
	}

	function handleWorldContentUpdate(
		serverId: string,
		event: Archon.Sync.v1.WorldContentUpdateEvent,
	) {
		if (event.world_id !== options.worldId.value) {
			void invalidateContentAndServerDetails(serverId)
			return
		}

		const content = worldContentUpdateToAddons(event)
		queryClient.setQueryData<Archon.Content.v1.Addons>(contentListKey(serverId), {
			...content,
			addons: content.addons?.filter((addon) => !addon.from_modpack) ?? null,
		})
		queryClient.setQueryData<Archon.Content.v1.Addons>(modpackContentListKey(serverId), {
			...content,
			addons: content.addons?.filter((addon) => addon.from_modpack) ?? null,
		})
		void queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) })
	}

	function worldContentUpdateToAddons(
		event: Archon.Sync.v1.WorldContentUpdateEvent,
	): Archon.Content.v1.Addons {
		return {
			modloader: event.platform_data?.platform ?? null,
			modloader_version: event.platform_data?.platform_version ?? null,
			game_version: event.platform_data?.game_version ?? null,
			modpack: worldContentModpackToModpackFields(event.linked_modpack),
			installing: event.installing,
			error: event.error,
			addons: event.content.map(worldContentItemToAddon),
		}
	}

	function worldContentModpackToModpackFields(
		modpack: Archon.Sync.v1.WorldContentModpack | null,
	): Archon.Content.v1.ModpackFields | null {
		if (!modpack || modpack.spec === 'CurseForge') return null

		const spec: Archon.Content.v1.ModpackSpec =
			'Modrinth' in modpack.spec
				? {
						platform: 'modrinth',
						project_id: modpack.spec.Modrinth.project_id,
						version_id: modpack.spec.Modrinth.version_id,
					}
				: {
						platform: 'local_file',
						filename: modpack.spec.LocalMrPackFile.path,
						name: modpack.spec.LocalMrPackFile.name,
						description: modpack.spec.LocalMrPackFile.description,
					}

		return {
			spec,
			has_update: modpack.has_update,
			title: modpack.title,
			description: modpack.description,
			icon_url: modpack.icon_url,
			owner: modpack.owner,
			version_number: modpack.version_number,
			date_published: modpack.date_published,
			downloads: modpack.downloads,
			followers: modpack.followers,
		}
	}

	function worldContentItemToAddon(item: Archon.Sync.v1.WorldContentItem): Archon.Content.v1.Addon {
		return {
			id: item.version?.id ?? item.version_id ?? item.file_sha1 ?? item.filename,
			filename: item.filename,
			filesize: item.filesize ?? 0,
			btime: item.btime,
			disabled: item.filename.endsWith('.disabled'),
			kind: parentDirectoryToAddonKind(item.parent_directory),
			from_modpack: item.from_modpack,
			status: item.status,
			pack_client_retained: item.pack_client_retained,
			pack_client_depends: item.pack_client_depends,
			has_update: item.has_update,
			name: item.name,
			project_id: item.project_id,
			version: item.version,
			owner: item.owner,
			icon_url: item.icon_url,
		}
	}

	function parentDirectoryToAddonKind(parentDirectory: string): Archon.Content.v1.AddonKind {
		switch (parentDirectory) {
			case 'plugins':
				return 'plugin'
			case 'datapacks':
				return 'datapack'
			default:
				return 'mod'
		}
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
		queryClient.setQueryData<Archon.Servers.v1.ServerFull>(
			serverV1DetailKey(serverId),
			(current) =>
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
			queryClient.invalidateQueries({ queryKey: contentListKey(serverId) }),
			invalidateServerDetails(serverId),
		])
	}

	async function invalidateCorePanelQueries(serverId: string) {
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: legacyServerDetailKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: serverV1DetailKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: contentListKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: ['backups', 'queue', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'users', 'v1', serverId] }),
			queryClient.invalidateQueries({ queryKey: actionLogBaseKey(serverId) }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'startup', 'v1', serverId] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'allocations', serverId] }),
		])
	}

	function mergeAddonSpecs(
		currentAddons: Archon.Content.v1.Addon[],
		incomingAddons: Archon.Content.v1.Addon[],
	): Archon.Content.v1.Addon[] {
		const currentByFilename = new Map(
			currentAddons.map((addon) => [normalizeAddonFilename(addon.filename), addon] as const),
		)

		return incomingAddons.map((incoming) =>
			mergeAddonSpec(currentByFilename.get(normalizeAddonFilename(incoming.filename)), incoming),
		)
	}

	function mergeAddonSpec(
		current: Archon.Content.v1.Addon | undefined,
		incoming: Archon.Content.v1.Addon,
	): Archon.Content.v1.Addon {
		if (!current) return incoming

		return {
			...current,
			...incoming,
			status: incoming.status ?? current.status,
			filesize: incoming.filesize || current.filesize,
			btime: incoming.btime ?? current.btime,
			name: incoming.name ?? current.name,
			owner: incoming.owner ?? current.owner,
			icon_url: incoming.icon_url ?? current.icon_url,
			has_update: incoming.has_update ?? current.has_update,
			project_id: incoming.project_id ?? current.project_id,
			version: incoming.version
				? {
						...incoming.version,
						name: incoming.version.name ?? current.version?.name ?? null,
						environment: incoming.version.environment ?? current.version?.environment ?? null,
					}
				: current.version,
		}
	}

	function normalizeAddonFilename(filename: string): string {
		return filename.endsWith('.disabled') ? filename.slice(0, -'.disabled'.length) : filename
	}

	onMounted(() => {
		mounted = true
		if (options.serverId.value) connect(options.serverId.value)
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
