import type { Labrinth } from '@modrinth/api-client'

type ProjectSorting = 'publish_time' | 'queue_time' | 'downloads'
type ProjectStatusPriority = { order: number; sort: ProjectSorting }

const projectStatusPriority: Record<Labrinth.Projects.v2.ProjectStatus, ProjectStatusPriority> = {
	approved: { order: 1, sort: 'downloads' },
	scheduled: { order: 1, sort: 'downloads' },
	archived: { order: 2, sort: 'downloads' },
	unlisted: { order: 3, sort: 'downloads' },
	private: { order: 4, sort: 'downloads' },
	processing: { order: 5, sort: 'queue_time' },
	withheld: { order: 6, sort: 'publish_time' },
	rejected: { order: 7, sort: 'publish_time' },
	draft: { order: 8, sort: 'publish_time' },
	unknown: { order: 9, sort: 'publish_time' },
}

function getProjectSortValue(
	project: Labrinth.Projects.v2.Project,
	sorting: ProjectSorting,
): number {
	switch (sorting) {
		case 'publish_time':
			return new Date(project.published).getTime()
		case 'queue_time':
			return new Date(project.queued || project.published).getTime()
		case 'downloads':
			return project.downloads
	}
}

export function projectUserSorting(
	first: Labrinth.Projects.v2.Project,
	second: Labrinth.Projects.v2.Project,
): number {
	const firstPriority = projectStatusPriority[first.status] ?? projectStatusPriority.unknown
	const secondPriority = projectStatusPriority[second.status] ?? projectStatusPriority.unknown

	if (firstPriority.order !== secondPriority.order) {
		return firstPriority.order - secondPriority.order
	}
	if (firstPriority.sort !== secondPriority.sort) {
		return 0
	}

	return (
		getProjectSortValue(second, secondPriority.sort) -
		getProjectSortValue(first, firstPriority.sort)
	)
}

export function resolveProjectType(
	project: Labrinth.Projects.v2.Project,
	loaders: Labrinth.Tags.v2.Loader[],
): string {
	if (project.project_type !== 'mod') {
		return project.project_type
	}

	const projectLoaders = new Set(project.loaders)
	const supportsType = (type: string) =>
		loaders.some(
			(loader) =>
				projectLoaders.has(loader.name) && loader.supported_project_types.includes(type),
		)

	if (supportsType('datapack')) return 'datapack'
	if (supportsType('plugin')) return 'plugin'
	return 'mod'
}

const PRIDE_26_MIDAS_DURATION_MS = 30 * 24 * 60 * 60 * 1000

export function hasPride26Badge(user?: Labrinth.Users.v3.User | null): boolean {
	return user?.campaigns?.pride_26?.has_badge === true
}

export function hasActivePride26Midas(
	user?: Labrinth.Users.v3.User | null,
	now = Date.now(),
): boolean {
	const campaign = user?.campaigns?.pride_26
	if (!campaign?.has_midas) return false

	const donatedAt = Date.parse(campaign.last_donated_at)
	return Number.isFinite(donatedAt) && donatedAt + PRIDE_26_MIDAS_DURATION_MS > now
}
