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
		queryFn: async ({ signal }) => {
			const runs = new Map<string, ProjectFlagRun>()
			for (let offset = 0; ; offset += PAGE_LIMIT) {
				signal.throwIfAborted()
				const page = await client.labrinth.project_flags_internal.listRuns({
					limit: PAGE_LIMIT,
					offset,
				})
				for (const run of page) runs.set(run.id, run)
				if (page.length < PAGE_LIMIT) return [...runs.values()]
			}
		},
		refetchInterval: (query) =>
			query.state.data?.some((run) => run.status !== 'done') ? REFRESH_INTERVAL : false,
	})
}

export function projectFlagResultCategory(result: ProjectFlagRunResult) {
	const rules = Object.values(result.rules)
	if (result.error !== null || rules.some((rule) => rule.error !== null)) return 'errors'
	return rules.some((rule) => rule.flags.length > 0) ? 'flagged' : 'approvable'
}

export function projectFlagRunResultsPageOptions(
	client: AbstractModrinthClient,
	run: Pick<ProjectFlagRun, 'id' | 'status'>,
	limit: number,
	offset: number,
) {
	const done = run.status === 'done'
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
	const done = run.status === 'done'
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
