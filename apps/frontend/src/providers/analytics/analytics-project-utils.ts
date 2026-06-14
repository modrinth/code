import {
	getProjectStatusFilterValue,
	type ProjectStatusFilterValue,
} from '~/components/analytics-dashboard/query-builder/query-filter-utils'

import type {
	AnalyticsDashboardProject,
	AnalyticsDashboardProjectSource,
	AnalyticsSelectedFilters,
	ProjectTypeMetadata,
} from './analytics-types'

const MINECRAFT_JAVA_SERVER_PROJECT_TYPE = 'minecraft_java_server'
const PLUGIN_PROJECT_TYPE = 'plugin'

export const UNKNOWN_ORGANIZATION_NAME = 'Organization'

export function getProjectTypes(project: ProjectTypeMetadata): string[] {
	const projectTypes = new Set<string>()
	const projectType = project.project_type?.trim()
	if (projectType) {
		projectTypes.add(projectType)
	}

	for (const types of [project.project_types, project.projectTypes]) {
		for (const type of types ?? []) {
			const projectType = type.trim()
			if (projectType) {
				projectTypes.add(projectType)
			}
		}
	}

	return [...projectTypes]
}

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

export function isPluginProject(project: ProjectTypeMetadata): boolean {
	const projectTypes = getProjectTypes(project)
	return projectTypes.length > 0 && projectTypes.every((type) => type === PLUGIN_PROJECT_TYPE)
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
		publishedAt: project.published ?? undefined,
		projectTypes: getProjectTypes(project),
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
