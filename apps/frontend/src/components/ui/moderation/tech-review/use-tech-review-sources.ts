import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { type MaybeRefOrGetter, reactive, toValue } from 'vue'

export function useTechReviewSources(
	issues: MaybeRefOrGetter<Labrinth.TechReview.Internal.FileIssue[]>,
	includeHidden: MaybeRefOrGetter<boolean> = false,
) {
	const client = injectModrinthClient()

	const loadingIssues = reactive<Set<string>>(new Set())
	const decompiledSources = reactive<Map<string, string>>(new Map())
	const loadedIssues = reactive<Set<string>>(new Set())

	async function loadIssueSource(issueId: string): Promise<void> {
		if (loadingIssues.has(issueId) || loadedIssues.has(issueId)) return

		loadingIssues.add(issueId)

		try {
			const issueData = await client.labrinth.tech_review_internal.getIssue(issueId, {
				include_hidden: toValue(includeHidden),
			})

			for (const detail of issueData.details) {
				if (detail.decompiled_source) {
					decompiledSources.set(detail.id, detail.decompiled_source)
				}
			}
			loadedIssues.add(issueId)
		} catch (error) {
			console.error('Failed to load issue source:', error)
		} finally {
			loadingIssues.delete(issueId)
		}
	}

	function handleLoadIssueSources(issueIds: string[]): void {
		const uniqueIssueIds = new Set(issueIds)
		const matchedIssues = toValue(issues).filter((issue) => uniqueIssueIds.has(issue.id))

		for (const issue of matchedIssues) {
			const hasUncached = issue.details.some((detail) => !decompiledSources.has(detail.id))
			if (hasUncached) {
				loadIssueSource(issue.id)
			}
		}
	}

	return {
		loadingIssues,
		decompiledSources,
		handleLoadIssueSources,
	}
}
