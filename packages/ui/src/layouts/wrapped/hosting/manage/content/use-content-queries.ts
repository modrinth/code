import type { Archon } from '@modrinth/api-client'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	addonToggleKey,
	applyPendingAddonToggle,
} from '#ui/composables/server-content-toggle-state'
import { serverKeys } from '#ui/composables/server-queries'
import type { ContentItem } from '#ui/layouts/shared/content-tab/types'
import { injectModrinthClient, injectModrinthServerContext } from '#ui/providers'

const STALE_TIME = 30_000
const PROJECT_METADATA_BATCH_SIZE = 800

export type ContentQueries = ReturnType<typeof useContentQueries>

export function useContentQueries(serverId: string) {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { worldId } = injectModrinthServerContext()

	const queryKey = computed(() => serverKeys.content(serverId))
	const modpackContentQueryKey = computed(
		() => [...serverKeys.content(serverId), 'modpack'] as const,
	)

	async function fetchAddons(fromModpack: boolean) {
		const targetWorldId = worldId.value!
		const content = await client.archon.content_v1.getAddons(serverId, targetWorldId, {
			from_modpack: fromModpack,
		})
		return {
			...content,
			addons:
				content.addons?.map((addon) =>
					applyPendingAddonToggle(queryClient, serverId, targetWorldId, addon),
				) ?? null,
		}
	}

	const contentQuery = useQuery({
		queryKey,
		queryFn: () => fetchAddons(false),
		enabled: computed(() => worldId.value !== null),
		staleTime: STALE_TIME,
	})

	const modpackContentQuery = useQuery({
		queryKey: modpackContentQueryKey,
		queryFn: () => fetchAddons(true),
		enabled: computed(() => worldId.value !== null && !!contentQuery.data.value?.modpack),
		staleTime: STALE_TIME,
	})

	function prefetchModpackContent() {
		if (worldId.value === null) return
		void queryClient.prefetchQuery({
			queryKey: modpackContentQueryKey.value,
			queryFn: () => fetchAddons(true),
			staleTime: STALE_TIME,
		})
	}

	const modpackAddons = ref<Archon.Content.v1.Addon[]>([])

	const addonLookup = computed(
		() =>
			new Map(
				(contentQuery.data.value?.addons ?? []).map((addon) => [addonToggleKey(addon), addon]),
			),
	)
	const modpackAddonLookup = computed(
		() => new Map(modpackAddons.value.map((addon) => [addonToggleKey(addon), addon])),
	)

	function getAddonForItem(item: ContentItem) {
		return item.source_kind === 'modrinth_modpack'
			? modpackAddonLookup.value.get(item.id)
			: addonLookup.value.get(item.id)
	}

	function getAddonQueryKey(addon: Archon.Content.v1.Addon) {
		return addon.from_modpack ? modpackContentQueryKey.value : queryKey.value
	}

	const contentProjectIds = computed(() =>
		[...(contentQuery.data.value?.addons ?? []), ...modpackAddons.value]
			.map((addon) => addon.project_id)
			.filter((id): id is string => !!id)
			.filter((id, index, ids) => ids.indexOf(id) === index)
			.sort(),
	)
	const contentProjectsQuery = useQuery({
		queryKey: computed(() => ['labrinth', 'projects', 'v2', contentProjectIds.value] as const),
		queryFn: async () => {
			const ids = contentProjectIds.value
			const batches = []
			for (let index = 0; index < ids.length; index += PROJECT_METADATA_BATCH_SIZE) {
				batches.push(ids.slice(index, index + PROJECT_METADATA_BATCH_SIZE))
			}
			return (
				await Promise.all(batches.map((batch) => client.labrinth.projects_v2.getMultiple(batch)))
			).flat()
		},
		enabled: computed(() => contentProjectIds.value.length > 0),
	})
	const contentProjectsById = computed(
		() => new Map((contentProjectsQuery.data.value ?? []).map((project) => [project.id, project])),
	)

	async function optimisticallyUpdateAddons(
		update: (addons: Archon.Content.v1.Addon[]) => Archon.Content.v1.Addon[],
	) {
		const targetQueryKey = queryKey.value
		await queryClient.cancelQueries({ queryKey: targetQueryKey, exact: true })
		const previousData = queryClient.getQueryData<Archon.Content.v1.Addons>(targetQueryKey)
		queryClient.setQueryData<Archon.Content.v1.Addons>(targetQueryKey, (current) =>
			current ? { ...current, addons: update(current.addons ?? []) } : current,
		)
		const dataUpdateCount = queryClient.getQueryState(targetQueryKey)?.dataUpdateCount
		return () => {
			if (
				previousData &&
				queryClient.getQueryState(targetQueryKey)?.dataUpdateCount === dataUpdateCount
			) {
				queryClient.setQueryData(targetQueryKey, previousData)
			}
		}
	}

	return {
		queryKey,
		contentQuery,
		modpackContentQuery,
		prefetchModpackContent,
		modpackAddons,
		getAddonForItem,
		getAddonQueryKey,
		contentProjectsById,
		optimisticallyUpdateAddons,
	}
}
