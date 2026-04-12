import type { Archon } from '@modrinth/api-client'
import { useQuery } from '@tanstack/vue-query'
import { computed, type ComputedRef } from 'vue'

import { injectModrinthClient } from '#ui/providers'

// TODO: Remove and use v1
export function useServerProject(
	upstream: ComputedRef<Archon.Servers.v0.Server['upstream'] | null>,
) {
	const client = injectModrinthClient()

	return useQuery({
		queryKey: computed(() => ['servers', 'project', upstream.value?.project_id ?? null]),
		queryFn: () => client.labrinth.projects_v2.get(upstream.value!.project_id!),
		enabled: computed(() => !!upstream.value?.project_id),
	})
}
