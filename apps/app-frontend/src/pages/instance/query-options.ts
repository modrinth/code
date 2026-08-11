import { queryOptions } from '@tanstack/vue-query'

import { get_project_v3 } from '@/helpers/cache.js'
import { get as getInstance } from '@/helpers/instance'
import { loadInstanceContentData } from '@/helpers/instance-content'
import { get_by_instance_id } from '@/helpers/process'
import { refreshWorlds } from '@/helpers/worlds'

export const instanceKeys = {
	all: ['instances'] as const,
	detail: (instanceId: string) => [...instanceKeys.all, 'summary', instanceId] as const,
	processes: (instanceId: string) => [...instanceKeys.all, 'processes', instanceId] as const,
	content: (instanceId: string) => [...instanceKeys.all, 'content', instanceId] as const,
	contentUpdateCheck: (instanceId: string) =>
		[...instanceKeys.all, 'content-update-check', instanceId] as const,
	rootPath: (instanceId: string) => [...instanceKeys.detail(instanceId), 'root-path'] as const,
	files: (instanceId: string, path: string) =>
		[...instanceKeys.detail(instanceId), 'files', path] as const,
	console: (instanceId: string) => [...instanceKeys.detail(instanceId), 'console'] as const,
	logs: (instanceId: string) => [...instanceKeys.detail(instanceId), 'logs'] as const,
	installedProjectIds: (instanceId: string, source: 'content' | 'worlds') =>
		[...instanceKeys.detail(instanceId), 'installed-project-ids', source] as const,
	linkedContent: (instanceId: string) => ['linkedModpackContent', instanceId] as const,
	worlds: (instanceId: string) => ['worlds', instanceId] as const,
	linkedProject: (projectId: string) => ['project', 'v3', projectId] as const,
	sharedEligibility: (userId: string | null | undefined) =>
		['shared-instance-eligibility', userId] as const,
	sharedUpdatePreview: (instanceId: string, userId: string | null | undefined) =>
		[...instanceKeys.detail(instanceId), 'shared-update-preview', userId] as const,
	sharedMembers: (instanceId: string) => ['sharedInstanceUsers', instanceId] as const,
}

export function instanceDetailQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: instanceKeys.detail(instanceId),
		queryFn: async () => {
			const instance = await getInstance(instanceId)
			if (!instance) throw new Error(`Instance ${instanceId} is not managed`)
			return instance
		},
		staleTime: 30_000,
	})
}

export function instanceProcessesQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: instanceKeys.processes(instanceId),
		queryFn: async () => {
			const processes = await get_by_instance_id(instanceId)
			return Array.isArray(processes) ? processes : []
		},
		staleTime: 0,
	})
}

export function instanceLinkedProjectQueryOptions(projectId: string) {
	return queryOptions({
		queryKey: instanceKeys.linkedProject(projectId),
		queryFn: () => get_project_v3(projectId, 'must_revalidate'),
		staleTime: 30_000,
	})
}

export function instanceContentQueryOptions(
	instanceId: string,
	onError?: (error: Error) => unknown,
) {
	return queryOptions({
		queryKey: instanceKeys.content(instanceId),
		queryFn: () => loadInstanceContentData(instanceId, undefined, onError),
		staleTime: 30_000,
	})
}

export function instanceWorldsQueryOptions(instanceId: string) {
	return queryOptions({
		queryKey: instanceKeys.worlds(instanceId),
		queryFn: () => refreshWorlds(instanceId),
		staleTime: 0,
	})
}
