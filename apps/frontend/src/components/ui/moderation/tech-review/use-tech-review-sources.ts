import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { type MaybeRefOrGetter, reactive, toValue } from 'vue'

const CACHE_TTL = 24 * 60 * 60 * 1000
const CACHE_KEY_PREFIX = 'tech_review_source_'

type CachedSource = {
	source: string
	timestamp: number
}

function getCachedSource(detailId: string): string | null {
	try {
		const cached = localStorage.getItem(`${CACHE_KEY_PREFIX}${detailId}`)
		if (!cached) return null

		const data: CachedSource = JSON.parse(cached)
		const now = Date.now()

		if (now - data.timestamp > CACHE_TTL) {
			localStorage.removeItem(`${CACHE_KEY_PREFIX}${detailId}`)
			return null
		}

		return data.source
	} catch {
		return null
	}
}

function setCachedSource(detailId: string, source: string): void {
	try {
		const data: CachedSource = {
			source,
			timestamp: Date.now(),
		}
		localStorage.setItem(`${CACHE_KEY_PREFIX}${detailId}`, JSON.stringify(data))
	} catch (error) {
		console.error('Failed to cache source:', error)
	}
}

function clearExpiredCache(): void {
	try {
		const now = Date.now()
		const keys = Object.keys(localStorage)

		for (const key of keys) {
			if (key.startsWith(CACHE_KEY_PREFIX)) {
				const cached = localStorage.getItem(key)
				if (cached) {
					const data: CachedSource = JSON.parse(cached)
					if (now - data.timestamp > CACHE_TTL) {
						localStorage.removeItem(key)
					}
				}
			}
		}
	} catch (error) {
		console.error('Failed to clear expired cache:', error)
	}
}

export function useTechReviewSources(
	issues: MaybeRefOrGetter<Labrinth.TechReview.Internal.FileIssue[]>,
) {
	const client = injectModrinthClient()

	if (import.meta.client) {
		clearExpiredCache()
	}

	const loadingIssues = reactive<Set<string>>(new Set())
	const decompiledSources = reactive<Map<string, string>>(new Map())
	const loadedIssues = reactive<Set<string>>(new Set())

	async function loadIssueSource(issueId: string): Promise<void> {
		if (loadingIssues.has(issueId) || loadedIssues.has(issueId)) return

		loadingIssues.add(issueId)

		try {
			const issueData = await client.labrinth.tech_review_internal.getIssue(issueId)

			for (const detail of issueData.details) {
				if (detail.decompiled_source) {
					decompiledSources.set(detail.id, detail.decompiled_source)
					setCachedSource(detail.id, detail.decompiled_source)
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
			for (const detail of issue.details) {
				if (!decompiledSources.has(detail.id)) {
					const cached = getCachedSource(detail.id)
					if (cached) {
						decompiledSources.set(detail.id, cached)
					}
				}
			}

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
