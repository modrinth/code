import type {
	AbstractModrinthClient,
	ProjectFlagRun,
	ProjectFlagRunResult,
} from '@modrinth/api-client'
import { queryOptions } from '@tanstack/vue-query'

const PAGE_LIMIT = 200
const REFRESH_INTERVAL = 10_000
const COMPLETED_RESULTS_CACHE_TIME = 3 * 60 * 60_000

export const projectFlagRunHistoryKey = ['moderation', 'project-flag', 'runs', 'history'] as const
export const projectFlagRunActivityKey = ['moderation', 'project-flag', 'runs', 'activity'] as const
export const projectFlagHiddenRunIdsKey = ['moderation', 'project-flag', 'hidden-run-ids'] as const

export function projectFlagRunActivityOptions(client: AbstractModrinthClient) {
	return queryOptions({
		queryKey: projectFlagRunActivityKey,
		queryFn: () => client.labrinth.project_flags_internal.getRunActivity(),
		refetchInterval: REFRESH_INTERVAL,
	})
}

export function projectFlagRunHistoryOptions(client: AbstractModrinthClient) {
	return queryOptions({
		queryKey: projectFlagRunHistoryKey,
		queryFn: async ({ client: queryClient, queryKey, signal }) => {
			const api = client.labrinth.project_flags_internal
			const runs = new Map<string, ProjectFlagRun>()
			for (let offset = 0; ; offset += PAGE_LIMIT) {
				signal.throwIfAborted()
				const page = await api.listRuns({ limit: PAGE_LIMIT, offset })
				for (const run of page) runs.set(run.id, run)
				if (page.length < PAGE_LIMIT) break
			}

			// Temporal can describe a new run before it appears in the visibility index.
			signal.throwIfAborted()
			const cachedRuns = queryClient.getQueryData<ProjectFlagRun[]>(queryKey) ?? []
			const missingRuns = cachedRuns.filter((run) => !runs.has(run.id))
			const describedRuns = await Promise.all(missingRuns.map(({ id }) => api.getRun(id)))
			for (const run of describedRuns) {
				if (run) runs.set(run.id, run)
			}
			// Temporal can still list a deleted run while its visibility index catches up.
			for (const id of queryClient.getQueryData<string[]>(projectFlagHiddenRunIdsKey) ?? []) {
				runs.delete(id)
			}
			return [...runs.values()].sort((a, b) => Date.parse(b.started_at) - Date.parse(a.started_at))
		},
		refetchInterval: (query) =>
			query.state.data?.some((run) => typeof run.status === 'object') ? REFRESH_INTERVAL : false,
	})
}

export function projectFlagRunDetailOptions(client: AbstractModrinthClient, id: string) {
	return queryOptions({
		queryKey: ['moderation', 'project-flag', 'runs', id, 'detail'] as const,
		queryFn: () => client.labrinth.project_flags_internal.getRun(id),
		refetchInterval: REFRESH_INTERVAL,
	})
}

export function projectFlagResultCategory(result: ProjectFlagRunResult) {
	const rules = Object.values(result.rules)
	if (result.error !== null || rules.some((rule) => rule.error !== null)) return 'errors'
	return result.num_flags > 0 ? 'flagged' : 'approvable'
}

export function projectFlagRunResultsPageOptions(
	client: AbstractModrinthClient,
	run: Pick<ProjectFlagRun, 'id' | 'status'>,
	limit: number,
	offset: number,
) {
	const done = typeof run.status === 'string'
	return queryOptions({
		queryKey: [
			'moderation',
			'project-flag',
			'runs',
			run.id,
			'results-page',
			done,
			limit,
			offset,
		] as const,
		queryFn: () => client.labrinth.project_flags_internal.getRunResults(run.id, { limit, offset }),
		staleTime: done ? COMPLETED_RESULTS_CACHE_TIME : REFRESH_INTERVAL,
		gcTime: done ? COMPLETED_RESULTS_CACHE_TIME : undefined,
		refetchInterval: done ? false : REFRESH_INTERVAL,
	})
}

export function projectFlagRunResultsOptions(
	client: AbstractModrinthClient,
	run: Pick<ProjectFlagRun, 'id' | 'status'>,
) {
	const done = typeof run.status === 'string'
	return queryOptions({
		queryKey: ['moderation', 'project-flag', 'runs', run.id, 'results', done] as const,
		queryFn: async ({ signal }) => {
			const results = new Map<string, ProjectFlagRunResult>()
			let total = 0
			let offset = 0
			do {
				signal.throwIfAborted()
				const page = await client.labrinth.project_flags_internal.getRunResults(run.id, {
					limit: PAGE_LIMIT,
					offset,
				})
				if (offset === 0) total = page.total
				for (const result of page.results) results.set(result.project.id, result)
				offset += PAGE_LIMIT
			} while (offset < total)
			return [...results.values()]
		},
		staleTime: done ? COMPLETED_RESULTS_CACHE_TIME : REFRESH_INTERVAL,
		gcTime: done ? COMPLETED_RESULTS_CACHE_TIME : undefined,
		refetchInterval: done ? false : REFRESH_INTERVAL,
	})
}
