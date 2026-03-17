import type { Archon } from '@modrinth/api-client'
import type { Project } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { $fetch } from 'ofetch'
import { computed, type ComputedRef } from 'vue'

// TODO: Remove and use v1
export function useServerProject(
	upstream: ComputedRef<Archon.Servers.v0.Server['upstream'] | null>,
) {
	return useQuery({
		queryKey: computed(() => ['servers', 'project', upstream.value?.project_id ?? null]),
		queryFn: () =>
			$fetch<Project>(`https://api.modrinth.com/v2/project/${upstream.value!.project_id}`),
		enabled: computed(() => !!upstream.value?.project_id),
	})
}
