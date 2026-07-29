import { computed } from 'vue'

import {
	type AnalyticsDashboardContextValue,
	doesProjectStatusMatchFilters,
} from '~/providers/analytics/analytics'

export function useAnalyticsChartProjects(
	context: Pick<
		AnalyticsDashboardContextValue,
		'displayedSelectedProjectIds' | 'projects' | 'displayedSelectedFilters'
	>,
) {
	const selectedProjectIdSet = computed(() => new Set(context.displayedSelectedProjectIds.value))
	const hasAvailableProjects = computed(() => context.projects.value.length > 0)

	const selectedProjects = computed(() =>
		context.projects.value.filter(
			(project) =>
				selectedProjectIdSet.value.has(project.id) &&
				doesProjectStatusMatchFilters(project.status, context.displayedSelectedFilters.value),
		),
	)
	const selectedProjectNameById = computed(
		() => new Map(selectedProjects.value.map((project) => [project.id, project.name])),
	)
	const selectedProjectEventIdSet = computed(
		() => new Set(selectedProjects.value.map((project) => project.id)),
	)

	return {
		selectedProjectIdSet,
		hasAvailableProjects,
		selectedProjects,
		selectedProjectNameById,
		selectedProjectEventIdSet,
	}
}
