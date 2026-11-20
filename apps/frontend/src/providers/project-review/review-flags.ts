import { createContext, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, type Ref } from 'vue'

export const [injectReviewFlags, provideReviewFlags] =
	createContext<ReturnType<typeof createReviewFlags>>('ProjectReviewFlags')

export function createReviewFlags(projectId: Ref<string | undefined>) {
	const { labrinth } = injectModrinthClient()
	const { data, isPending, isError } = useQuery({
		queryKey: computed(() => ['project', 'flags', projectId.value]),
		queryFn: () => labrinth.project_flags_internal.getProjectFlags(projectId.value!),
		enabled: computed(() => !!projectId.value),
	})
	const flags = computed(() =>
		[...(data.value ?? [])].sort(
			(a, b) =>
				Number(!!a.location) - Number(!!b.location) ||
				(a.name || a.key).localeCompare(b.name || b.key, 'en'),
		),
	)
	return {
		flags,
		isPending,
		isError,
		forPanel: (key: string) =>
			flags.value.filter((flag) => flag.location?.review_panel_key === key),
	}
}
