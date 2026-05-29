import {
	getProjectStatusFilterValue,
	type ProjectStatusFilterValue,
} from '~/components/analytics-dashboard/query-builder/query-filter'

import type {
	AnalyticsDashboardProject,
	AnalyticsDashboardProjectSource,
	ProjectTypeMetadata,
} from './analytics-types'
import type { AnalyticsSelectedFilters } from './query-builder-url'

const MINECRAFT_JAVA_SERVER_PROJECT_TYPE = 'minecraft_java_server'

export const UNKNOWN_ORGANIZATION_NAME = 'Organization'

function isServerProject(project: ProjectTypeMetadata): boolean {
	if (project.project_type === MINECRAFT_JAVA_SERVER_PROJECT_TYPE) {
		return true
	}

	return project.project_types?.includes(MINECRAFT_JAVA_SERVER_PROJECT_TYPE) ?? false
}

export function isAnalyticsEligibleProject(
	project: ProjectTypeMetadata & { status?: string | null },
): boolean {
	return !isServerProject(project) && getProjectStatusFilterValue(project.status) !== 'draft'
}

export function getSingleQueryValue(value: unknown): string | undefined {
	if (typeof value !== 'string') {
		return undefined
	}

	const normalizedValue = value.trim()
	return normalizedValue.length > 0 ? normalizedValue : undefined
}

export function toAnalyticsDashboardProject(
	project: AnalyticsDashboardProjectSource,
): AnalyticsDashboardProject {
	return {
		id: project.id,
		name: project.name ?? project.title ?? project.id,
		iconUrl: project.icon_url ?? undefined,
		downloads: project.downloads ?? 0,
		status: getProjectStatusFilterValue(project.status),
	}
}

export function getUniqueAnalyticsDashboardProjects(
	projects: AnalyticsDashboardProjectSource[],
	seenProjectIds: Set<string>,
): AnalyticsDashboardProject[] {
	const analyticsProjects: AnalyticsDashboardProject[] = []

	for (const project of projects) {
		if (seenProjectIds.has(project.id) || !isAnalyticsEligibleProject(project)) {
			continue
		}

		seenProjectIds.add(project.id)
		analyticsProjects.push(toAnalyticsDashboardProject(project))
	}

	return analyticsProjects
}

export function getProjectOrganizationId(
	project: AnalyticsDashboardProjectSource,
): string | undefined {
	return typeof project.organization === 'string' && project.organization.trim().length > 0
		? project.organization
		: undefined
}

export function doesProjectStatusMatchFilters(
	status: string | null | undefined,
	filters: AnalyticsSelectedFilters,
): boolean {
	if (filters.project_status.length === 0) {
		return true
	}

	return filters.project_status.includes(getProjectStatusFilterValue(status))
}

export function getProjectIdsMatchingStatusFilter(
	projectIds: string[],
	projectStatusById: Map<string, ProjectStatusFilterValue>,
	filters: AnalyticsSelectedFilters,
): string[] {
	if (filters.project_status.length === 0) {
		return projectIds
	}

	return projectIds.filter((projectId) =>
		doesProjectStatusMatchFilters(projectStatusById.get(projectId), filters),
	)
}
