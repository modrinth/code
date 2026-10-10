import type { AbstractModrinthClient, Archon, UploadProgress } from '@modrinth/api-client'
import type { ContentDiffItem } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { invoke } from '@tauri-apps/api/core'
import { join, tempDir } from '@tauri-apps/api/path'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'

import { config } from '@/config'
import { apiClient } from '@/services/api-client'

import { export_instance_mrpack, get_content_items } from './instance'
import { get_server_status } from './worlds'

export type LinkedServer = Archon.Servers.v0.Server & {
	linked_instance: Archon.Servers.v0.LinkedInstance
}

/** Shared with the Servers page, so both read the same cached list. */
export const serversQueryKey = ['servers'] as const

export function fetchServers(client: AbstractModrinthClient = apiClient) {
	return client.archon.servers_v0.list({ limit: 100 })
}

/**
 * The servers that follow an instance, on the Core this App is connected to. Empty when there
 * is no Core or it cannot be reached. Refreshes on an interval while mounted, so running,
 * offline, and updating stay current.
 */
export function useLinkedServers(instanceId: MaybeRefOrGetter<string | undefined>) {
	const query = useQuery({
		queryKey: serversQueryKey,
		queryFn: () => fetchServers(),
		enabled: () => Boolean(config.coreUrl),
		staleTime: 5_000,
		refetchInterval: 5_000,
		retry: false,
	})
	const servers = computed(() => {
		const id = toValue(instanceId)
		return (query.data.value?.servers ?? []).filter(
			(server): server is LinkedServer => !!id && server.linked_instance?.id === id,
		)
	})
	return { servers, query }
}

/** Whether the server's process is up, including while it starts or stops. */
export function isServerRunning(server: Archon.Servers.v0.Server) {
	return (
		server.power_state === 'running' ||
		server.power_state === 'starting' ||
		server.power_state === 'stopping'
	)
}

export type LinkedServerState = 'running' | 'offline' | 'updating' | 'pending' | 'failed'

/** The one state shown for a linked server. An update in progress wins over power state. */
export function linkedServerState(server: LinkedServer): LinkedServerState {
	const update = server.linked_instance.state
	if (update === 'failed') return 'failed'
	if (update === 'downloading' || update === 'applying' || server.status === 'installing') {
		return 'updating'
	}
	if (update === 'pending' && isServerRunning(server)) return 'pending'
	return isServerRunning(server) ? 'running' : 'offline'
}

export function serverAddress(server: Archon.Servers.v0.Server) {
	return `${server.net.domain || server.net.ip || '127.0.0.1'}:${server.net.port}`
}

/** Players on a running server right now, from a status ping. Zero when it cannot be reached. */
export async function playersOnline(server: Archon.Servers.v0.Server) {
	if (!isServerRunning(server)) return 0
	try {
		return (await get_server_status(serverAddress(server))).players?.online ?? 0
	} catch {
		return 0
	}
}

/**
 * Exports the instance as a `.mrpack` with its mods and config. Modrinth content is listed by
 * reference and anything else is packed in, so custom mods reach the server too.
 */
export async function exportInstancePack(instanceId: string): Promise<File> {
	// One file per instance, overwritten by the next push.
	const path = await join(
		await tempDir(),
		`amberite-server-${instanceId.replace(/[^a-zA-Z0-9-]/g, '_')}.mrpack`,
	)
	await export_instance_mrpack(instanceId, path, ['mods', 'config'], [])
	const bytes = await invoke<number[]>('plugin:files|file_read_dragged_file', { path })
	return new File([new Uint8Array(bytes)], 'instance.mrpack')
}

/**
 * Sends an exported instance to a server on the Core. The first push installs it and links the
 * server; later pushes update it. A running server takes the update on its next stop unless
 * `restart` is set.
 */
export async function pushPackToServer(
	pack: File,
	instance: { id: string; name: string },
	serverId: string,
	options: {
		properties?: Archon.Content.v1.PropertiesFields
		restart?: boolean
		onProgress?: (progress: UploadProgress) => void
	} = {},
	client: AbstractModrinthClient = apiClient,
) {
	// A Core server has one world, with the server's own id.
	const worldId = serverId
	await client.archon.content_v1.uploadInstance(serverId, worldId, pack, instance, {
		properties: options.properties,
		onProgress: options.onProgress,
	}).promise
	if (options.restart) await client.archon.content_v1.updateModpack(serverId, worldId)
}

/**
 * What a push would change on a server: the instance's enabled mods compared with the mods the
 * server last took from it. Config changes are not detected.
 */
export async function diffInstanceAgainstServer(
	instanceId: string,
	server: Archon.Servers.v0.Server,
	client: AbstractModrinthClient = apiClient,
): Promise<ContentDiffItem[]> {
	const [items, content] = await Promise.all([
		get_content_items(instanceId),
		client.archon.content_v1.getAddons(server.server_id, server.server_id, {
			from_modpack: true,
		}),
	])
	const wanted = items.filter((item) => item.project_type === 'mod' && item.enabled)
	const onServer = (content.addons ?? []).filter((addon) => addon.kind === 'mod')
	// Modrinth content is matched by project, so a new version is an update, not add plus remove.
	const key = (projectId: string | null | undefined, fileName: string) => projectId ?? fileName
	const serverByKey = new Map(
		onServer.map((addon) => [key(addon.project_id, addon.filename), addon]),
	)
	const wantedKeys = new Set<string>()
	const diffs: ContentDiffItem[] = []
	for (const item of wanted) {
		const hasProject = item.project.id !== item.file_name
		const itemKey = key(hasProject ? item.project.id : null, item.file_name)
		wantedKeys.add(itemKey)
		const existing = serverByKey.get(itemKey)
		const names = { projectName: item.project.title, fileName: item.file_name }
		if (!existing) {
			diffs.push({ type: 'added', ...names, newVersionName: item.version?.version_number })
		} else if (existing.filename !== item.file_name) {
			diffs.push({
				type: 'updated',
				...names,
				currentVersionName: existing.version?.name ?? existing.filename,
				newVersionName: item.version?.version_number ?? item.file_name,
			})
		}
	}
	for (const addon of onServer) {
		if (wantedKeys.has(key(addon.project_id, addon.filename))) continue
		diffs.push({
			type: 'removed',
			projectName: addon.name ?? addon.filename,
			fileName: addon.filename,
		})
	}
	return diffs
}
