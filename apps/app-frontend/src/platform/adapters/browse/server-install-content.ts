import type { Archon } from '@modrinth/api-client'
import {
	type BrowseTarget,
	injectModrinthClient,
	injectNotificationManager,
	resolveServerPlatform,
	serverKeys,
	type ServerSetupModalHandle,
	useServerInstall,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { config as appConfig } from '@/config'
import { useBreadcrumbServerId } from '@/shared/breadcrumbs'

function readQueryString(value: unknown): string | null {
	if (Array.isArray(value)) return value[0] ?? null
	return typeof value === 'string' && value.length > 0 ? value : null
}

export interface ServerInstallContentOptions {
	setupModal?: Ref<ServerSetupModalHandle | null>
	inheritBreadcrumbServer?: boolean
}

export function useServerInstallContent(options: ServerInstallContentOptions = {}) {
	const breadcrumbServerId = options.inheritBreadcrumbServer ? useBreadcrumbServerId() : null
	const route = useRoute()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()

	const serverId = computed(
		() => readQueryString(route.query.sid) ?? breadcrumbServerId?.value ?? null,
	)
	const worldIdQuery = computed(() => readQueryString(route.query.wid))
	const from = computed(() => readQueryString(route.query.from))
	const isFromWorlds = computed(() => from.value === 'worlds')
	const isServerContext = computed(() => !!serverId.value)

	const serverQuery = useQuery({
		queryKey: computed(() => serverKeys.detail(serverId.value ?? '')),
		queryFn: () => client.archon.servers_v0.get(serverId.value!),
		enabled: isServerContext,
		staleTime: 30_000,
		placeholderData: () =>
			queryClient
				.getQueryData<Archon.Servers.v0.ServerGetResponse>(serverKeys.list())
				?.servers.find((server) => server.server_id === serverId.value),
	})

	const needsWorldLookup = computed(() => isServerContext.value && !worldIdQuery.value)
	const serverDetailQuery = useQuery({
		queryKey: computed(() => serverKeys.detailV1(serverId.value ?? '')),
		queryFn: () => client.archon.servers_v1.get(serverId.value!),
		enabled: needsWorldLookup,
		staleTime: 30_000,
	})

	const worldId = computed(() => {
		if (!serverId.value) return null
		if (worldIdQuery.value) return worldIdQuery.value
		const worlds = serverDetailQuery.data.value?.worlds ?? []
		return (worlds.find((world) => world.is_active) ?? worlds[0])?.id ?? null
	})

	const contentQuery = useQuery({
		queryKey: computed(
			() => [...serverKeys.content(serverId.value ?? ''), 'installed', worldId.value] as const,
		),
		queryFn: () => client.archon.content_v1.getAddons(serverId.value!, worldId.value!),
		enabled: computed(() => !!serverId.value && !!worldId.value),
	})

	watch([serverQuery.error, serverDetailQuery.error, contentQuery.error], (errors) => {
		const error = errors.find(Boolean)
		if (error) handleError(error)
	})

	const serverData = computed(() => (serverId.value ? (serverQuery.data.value ?? null) : null))
	const platform = computed(() => resolveServerPlatform(serverData.value, contentQuery.data.value))

	const optimisticallyInstalledIds = ref(new Set<string>())
	watch(serverId, () => {
		optimisticallyInstalledIds.value = new Set()
	})

	const installedProjectIds = computed(
		() =>
			new Set([
				...(contentQuery.data.value?.addons ?? [])
					.map((addon) => addon.project_id)
					.filter((projectId): projectId is string => !!projectId),
				...optimisticallyInstalledIds.value,
			]),
	)

	function markInstalled(projectIds: string[]) {
		optimisticallyInstalledIds.value = new Set([...optimisticallyInstalledIds.value, ...projectIds])
	}

	const serverInstall = useServerInstall({
		serverId,
		worldId,
		from,
		platform,
		installedProjectIds,
		siteUrl: appConfig.siteUrl,
		setupModal: options.setupModal,
		onInstalled: async (installedServerId, projectIds) => {
			markInstalled(projectIds)
			await queryClient.invalidateQueries({ queryKey: serverKeys.content(installedServerId) })
		},
	})

	const target = computed<BrowseTarget | null>(() => {
		const server = serverData.value
		if (!server) return null
		return {
			name: server.name,
			serverId: serverId.value,
			isMedal: server.is_medal,
			gameVersion: platform.value.gameVersion,
			loader: platform.value.loader,
			heading: serverInstall.heading.value,
			back: serverInstall.back.value,
		}
	})

	async function getServerProjectVersions(projectId: string) {
		const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
		return versions.map((version) => ({ id: version.id }))
	}

	async function ready() {
		if (!serverId.value) return
		await Promise.all([
			serverQuery.suspense(),
			needsWorldLookup.value ? serverDetailQuery.suspense() : undefined,
		])
		if (worldId.value) await contentQuery.suspense()
	}

	return {
		serverId,
		worldId,
		isFromWorlds,
		isServerContext,
		serverData,
		platform,
		installedProjectIds,
		markInstalled,
		serverInstall,
		target,
		getServerProjectVersions,
		ready,
	}
}
