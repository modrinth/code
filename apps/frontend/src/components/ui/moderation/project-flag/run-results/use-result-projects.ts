import type { Labrinth } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { fetchSegmentedWith } from '~/utils/fetch-helpers'

import { useFlagErrorNotification } from '../use-flag-error-notification'

export function useResultProjects(categoryIds: () => string[], enabled: () => boolean) {
	const client = injectModrinthClient()

	const projectTypeIds = computed(() => [...categoryIds()].sort())
	const {
		data: filterProjects,
		isPending: projectTypesPending,
		error: projectTypeError,
		refetch: refetchProjectTypes,
	} = useQuery({
		queryKey: computed(
			() => ['moderation', 'project-flag', 'filter-projects', projectTypeIds.value] as const,
		),
		queryFn: ({ queryKey: [, , , ids] }) =>
			fetchSegmentedWith(ids, (batch) => client.labrinth.projects_v3.getMultiple(batch), 100),
		enabled: computed(() => enabled() && projectTypeIds.value.length > 0),
		staleTime: 60_000,
	})
	const loadingProjectTypes = computed(
		() => enabled() && projectTypeIds.value.length > 0 && projectTypesPending.value,
	)
	const filterProjectsById = computed(
		() => new Map(filterProjects.value?.map((project) => [project.id, project])),
	)
	return {
		filterProjects,
		filterProjectsById,
		loadingProjectTypes,
		projectTypeError,
		refetchProjectTypes,
	}
}

export function useResultProjectOwners(
	visibleIds: () => string[],
	filterByType: () => boolean,
	filteredProject: (id: string) => Labrinth.Projects.v3.Project | undefined,
) {
	const client = injectModrinthClient()
	const projectIds = computed(() => [...visibleIds()].sort())
	const { data: projects, error: projectsError } = useQuery({
		queryKey: computed(
			() => ['moderation', 'project-flag', 'result-projects', projectIds.value] as const,
		),
		queryFn: ({ queryKey: [, , , ids] }) => client.labrinth.projects_v3.getMultiple(ids),
		enabled: computed(() => projectIds.value.length > 0 && !filterByType()),
	})
	const visibleProjects = computed(() =>
		filterByType()
			? projectIds.value.flatMap((id) => {
					const project = filteredProject(id)
					return project ? [project] : []
				})
			: (projects.value ?? []),
	)
	const projectsById = computed(
		() => new Map(visibleProjects.value.map((project) => [project.id, project])),
	)
	const teamIds = computed(() =>
		[
			...new Set(
				visibleProjects.value
					.filter((project) => !project.organization)
					.map((project) => project.team_id),
			),
		].sort(),
	)
	const organizationIds = computed(() =>
		[
			...new Set(
				visibleProjects.value.flatMap((project) =>
					project.organization ? [project.organization] : [],
				),
			),
		].sort(),
	)
	const { data: teams, error: teamsError } = useQuery({
		queryKey: computed(
			() => ['moderation', 'project-flag', 'result-teams', teamIds.value] as const,
		),
		queryFn: ({ queryKey: [, , , ids] }) => client.labrinth.teams_v3.getMultiple(ids),
		enabled: computed(() => teamIds.value.length > 0),
	})
	const { data: organizations, error: organizationsError } = useQuery({
		queryKey: computed(
			() => ['moderation', 'project-flag', 'result-organizations', organizationIds.value] as const,
		),
		queryFn: ({ queryKey: [, , , ids] }) => client.labrinth.organizations_v3.getMultiple(ids),
		enabled: computed(() => organizationIds.value.length > 0),
	})
	useFlagErrorNotification(projectsError, 'Could not load project details')
	useFlagErrorNotification(teamsError, 'Could not load project teams')
	useFlagErrorNotification(organizationsError, 'Could not load project organizations')

	const ownersByProject = computed(() => {
		const usersByTeam = new Map(
			teams.value
				?.flat()
				.filter((member) => member.is_owner)
				.map((member) => [member.team_id, member.user]),
		)
		const organizationsById = new Map(
			organizations.value?.map((organization) => [organization.id, organization]),
		)
		const owners = new Map<
			string,
			{ id: string; name: string; icon?: string | null; kind: 'user' | 'organization' }
		>()
		for (const project of visibleProjects.value) {
			if (project.organization) {
				const organization = organizationsById.get(project.organization)
				if (organization)
					owners.set(project.id, {
						id: organization.id,
						name: organization.name,
						icon: organization.icon_url,
						kind: 'organization',
					})
			} else {
				const user = usersByTeam.get(project.team_id)
				if (user)
					owners.set(project.id, {
						id: user.id,
						name: user.username,
						icon: user.avatar_url,
						kind: 'user',
					})
			}
		}
		return owners
	})

	return { projectsById, ownersByProject }
}
