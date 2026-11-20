import { useQueryClient } from '@tanstack/vue-query'

import { injectModrinthClient } from '#ui/providers/api-client'

import {
	contentVersionQueryOptions,
	getRecommendedVersionBatches,
	getUpdateCandidates,
} from '../queries'
import type { ContentItem } from '../types'

export function useContentPrefetch() {
	const queryClient = useQueryClient()
	const client = injectModrinthClient()

	function prefetchProjectVersions(projectId: string) {
		return queryClient.prefetchQuery(contentVersionQueryOptions.projectVersions(projectId, client))
	}

	function prefetchVersion(versionId: string) {
		return queryClient.prefetchQuery(contentVersionQueryOptions.version(versionId, client))
	}

	function prefetchItem(item: ContentItem) {
		if (item.external || !item.project?.id || !item.version?.id) return
		void prefetchProjectVersions(item.project.id)
		if (item.has_update && item.update_version_id) void prefetchVersion(item.update_version_id)
	}

	function prefetchUpdates(items: ContentItem[]) {
		const candidates = getUpdateCandidates(items)
		if (candidates.length === 0) return
		if (candidates.length === 1) {
			prefetchItem(candidates[0])
			return
		}
		for (const batch of getRecommendedVersionBatches(candidates)) {
			void queryClient.prefetchQuery(contentVersionQueryOptions.versionBatch(batch, client))
		}
		for (const projectId of new Set(candidates.map((item) => item.project!.id))) {
			void prefetchProjectVersions(projectId)
		}
	}

	return { prefetchItem, prefetchUpdates, prefetchProjectVersions, prefetchVersion }
}
