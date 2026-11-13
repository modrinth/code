import type {
	AbstractModrinthClient,
	ProjectFlagRun,
	ProjectFlagRunResult,
} from '@modrinth/api-client'
import { queryOptions } from '@tanstack/vue-query'

const PAGE_LIMIT = 200
const REFRESH_INTERVAL = 10_000

export const projectFlagRunHistoryKey = ['moderation', 'project-flag', 'runs', 'history'] as const

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
		staleTime: done ? 5 * 60_000 : REFRESH_INTERVAL,
		refetchInterval: done ? false : REFRESH_INTERVAL,
	})
}
