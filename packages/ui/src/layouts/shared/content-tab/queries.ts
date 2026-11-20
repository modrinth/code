import type { AbstractModrinthClient } from '@modrinth/api-client'
import { chunk } from 'es-toolkit'

import type { ContentItem } from './types'

export const CONTENT_VERSIONS_STALE_TIME = 60_000

export const contentVersionQueryOptions = {
	projectVersions: (projectId: string, client: AbstractModrinthClient) => ({
		queryKey: ['labrinth', 'versions', 'v2', projectId] as const,
		queryFn: () =>
			client.labrinth.versions_v2.getProjectVersions(projectId, { include_changelog: false }),
		staleTime: CONTENT_VERSIONS_STALE_TIME,
	}),
	version: (versionId: string, client: AbstractModrinthClient) => ({
		queryKey: ['labrinth', 'version', 'v2', versionId] as const,
		queryFn: () => client.labrinth.versions_v2.getVersion(versionId),
		staleTime: CONTENT_VERSIONS_STALE_TIME,
	}),
	versionBatch: (versionIds: string[], client: AbstractModrinthClient) => ({
		queryKey: ['labrinth', 'versions', 'v2', 'recommended', versionIds] as const,
		queryFn: () =>
			client.labrinth.versions_v2.getVersions(versionIds, { include_changelog: false }),
		staleTime: CONTENT_VERSIONS_STALE_TIME,
	}),
}

export function getUpdateCandidates(items: ContentItem[]) {
	return items.filter(
		(item) =>
			item.has_update &&
			!item.locked &&
			!item.installing &&
			item.project?.id &&
			item.version?.id &&
			item.update_version_id,
	)
}

export function getRecommendedVersionBatches(items: ContentItem[]) {
	return chunk([...new Set(items.map((item) => item.update_version_id!))].sort(), 100)
}
