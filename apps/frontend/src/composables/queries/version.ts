import type { AbstractModrinthClient } from '@modrinth/api-client'
import { STALE_TIME } from './project'

export const versionQueryOptions = {
	v3: (versionId: string, client: AbstractModrinthClient) => ({
		queryKey: ['version', 'v3', versionId] as const,
		queryFn: () => client.labrinth.versions_v3.getVersion(versionId),
		staleTime: STALE_TIME,
	}),
}
