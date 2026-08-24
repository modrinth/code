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
	project: Labrinth.Projects.v3.Project,
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
	first: Labrinth.Projects.v3.Project,
	second: Labrinth.Projects.v3.Project,
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
