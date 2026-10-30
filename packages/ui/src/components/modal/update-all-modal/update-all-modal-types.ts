import type { Labrinth } from '@modrinth/api-client'

export type UpdateAllVersion = Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'> &
	Partial<Pick<Labrinth.Versions.v2.Version, 'version_type' | 'changelog'>>

export interface UpdateAllItem {
	id: string
	project: Pick<Labrinth.Projects.v2.Project, 'id' | 'title' | 'icon_url'>
	currentVersion: Pick<Labrinth.Versions.v2.Version, 'id' | 'version_number'>
	/** Compatible update candidates, in preferred order. */
	versions: UpdateAllVersion[]
	initialVersionId?: string
	initiallySelected?: boolean
}

export interface UpdateAllSelection {
	id: string
	projectId: string
	version: UpdateAllVersion
}
