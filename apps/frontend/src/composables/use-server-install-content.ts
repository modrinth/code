import type { AbstractModrinthClient } from '@modrinth/api-client'
import type { BrowseTarget, ServerSetupModalHandle } from '@modrinth/ui'
import {
	injectModrinthClient,
	injectNotificationManager,
	resolveServerPlatform,
	serverDetailQueryOptions,
	useServerInstall,
} from '@modrinth/ui'
import { type QueryClient, useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { useRoute, useRuntimeConfig } from '#app'
import { queryAsString } from '~/utils/router'

function serverContentQueryOptions(
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string,
) {
	return {
		queryKey: ['content', 'list', serverId] as const,
		queryFn: () => client.archon.content_v1.getAddons(serverId, worldId),
		staleTime: 30_000,
	}
}

export async function prefetchServerInstallContent(
	queryClient: QueryClient,
	client: AbstractModrinthClient,
	serverId: string,
	worldId: string | null,
) {
	await Promise.allSettled([
		queryClient.ensureQueryData(serverDetailQueryOptions(client, serverId)),
		worldId
			? queryClient.ensureQueryData(serverContentQueryOptions(client, serverId, worldId))
			: undefined,
	])
}

export function useServerInstallContentPrefetch() {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const route = useRoute()

	return async () => {
		const serverId = queryAsString(route.query.sid)
		if (!serverId) return
		await prefetchServerInstallContent(
			queryClient,
			client,
			serverId,
			queryAsString(route.query.wid) || null,
		)
	}
}

export function useServerInstallContent({
	setupModal,
}: {
	setupModal?: Ref<ServerSetupModalHandle | null>
} = {}) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const route = useRoute()

	const serverId = computed(() => queryAsString(route.query.sid) || null)
	const worldId = computed(() => queryAsString(route.query.wid) || null)
	const from = computed(() => queryAsString(route.query.from) || null)

	const { data: serverData } = useQuery(
		computed(() => ({
			...serverDetailQueryOptions(client, serverId.value ?? ''),
			enabled: !!serverId.value,
		})),
	)

	const { data: serverContent, error: serverContentError } = useQuery(
		computed(() => ({
			...serverContentQueryOptions(client, serverId.value ?? '', worldId.value ?? ''),
			enabled: !!serverId.value && !!worldId.value,
		})),
	)

	watch(serverContentError, (error) => {
		if (error) handleError(error)
	})

	const platform = computed(() => resolveServerPlatform(serverData.value, serverContent.value))

	const optimisticallyInstalledIds = ref(new Set<string>())
	const installedProjectIds = computed(
		() =>
			new Set([
				...(serverContent.value?.addons ?? [])
					.map((addon) => addon.project_id)
					.filter((projectId): projectId is string => !!projectId),
				...optimisticallyInstalledIds.value,
			]),
	)

	const serverInstall = useServerInstall({
		serverId,
		worldId,
		from,
		platform,
		installedProjectIds,
		siteUrl: useRuntimeConfig().public.siteUrl as string,
		setupModal,
		onInstalled: async (_, projectIds) => {
			optimisticallyInstalledIds.value = new Set([
				...optimisticallyInstalledIds.value,
				...projectIds,
			])
			await queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
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

	return {
		serverId,
		worldId,
		serverData,
		platform,
		installedProjectIds,
		serverInstall,
		target,
	}
}
