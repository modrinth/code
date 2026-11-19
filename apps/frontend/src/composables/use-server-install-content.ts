import type { BrowseTarget, ServerSetupModalHandle } from '@modrinth/ui'
import {
	injectModrinthClient,
	injectNotificationManager,
	resolveServerPlatform,
	useServerInstall,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'

import { useRoute, useRuntimeConfig } from '#app'
import { queryAsString } from '~/utils/router'

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

	const { data: serverData } = useQuery({
		queryKey: computed(() => ['servers', 'detail', serverId.value] as const),
		queryFn: () => client.archon.servers_v0.get(serverId.value!),
		enabled: computed(() => !!serverId.value),
	})

	const { data: serverContent, error: serverContentError } = useQuery({
		queryKey: computed(() => ['content', 'list', serverId.value ?? ''] as const),
		queryFn: () => client.archon.content_v1.getAddons(serverId.value!, worldId.value!),
		enabled: computed(() => !!serverId.value && !!worldId.value),
	})

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
