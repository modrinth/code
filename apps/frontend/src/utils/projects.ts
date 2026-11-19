import type { Labrinth } from '@modrinth/api-client'

type ProjectSorting = 'publish_time' | 'queue_time' | 'downloads'
type ProjectStatusPriority = { order: number; sort: ProjectSorting }
// An order can only be the same as another if the sorting type is the same.
const projectStatusPriority: {
	[status in Labrinth.Projects.v2.ProjectStatus]: ProjectStatusPriority
} = {
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
	project: Labrinth.Projects.v3.Project | Labrinth.Projects.v2.Project,
	sorting: ProjectSorting,
): number {
	switch (sorting) {
		case 'publish_time':
			return new Date(project.published).getTime()
		case 'queue_time':
			return new Date(project.queued || project.published).getTime()
		case 'downloads':
			return project.downloads
		default:
			return 0
	}
}

export const projectUserSorting = (
	first: Labrinth.Projects.v3.Project | Labrinth.Projects.v2.Project,
	second: Labrinth.Projects.v3.Project | Labrinth.Projects.v2.Project,
): number => {
	const priority1 = projectStatusPriority[first.status] || projectStatusPriority['unknown']
	const priority2 = projectStatusPriority[second.status] || projectStatusPriority['unknown']

	if (priority1.order !== priority2.order) {
		return priority1.order - priority2.order
	} else if (priority1.sort !== priority2.sort) {
		return 0
	}
	return getProjectSortValue(second, priority2.sort) - getProjectSortValue(first, priority1.sort)
}
