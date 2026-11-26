import type { Labrinth } from '@modrinth/api-client'
import {
	type ContentItem,
	contentVersionQueryOptions,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import {
	switch_project_version_with_dependencies,
	update_managed_modrinth_version,
} from '@/helpers/instance'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'

import { analyticsProjectType } from './content-items'
import type { ContentOperation, ContentOperations } from './use-content-operations'

export interface ContentUpdaterOptions {
	instance: Ref<GameInstance>
	canUpdateProject: (item: ContentItem) => boolean
	canChangeContentVersion: (item: ContentItem) => boolean
	operations: ContentOperations
	refetchContent: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	refreshContentState: (cacheBehaviour?: CacheBehaviour) => Promise<void>
}

interface InstallVersionVariables {
	item: ContentItem
	filePath: string
	versionId: string
	operation: ContentOperation
}

export function useContentUpdater({
	instance,
	canUpdateProject,
	canChangeContentVersion,
	operations,
	refetchContent,
	refreshContentState,
}: ContentUpdaterOptions) {
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const client = injectModrinthClient()

	async function fetchProjectVersions(projectId: string) {
		try {
			return await queryClient.fetchQuery(
				contentVersionQueryOptions.projectVersions(projectId, client),
			)
		} catch (error) {
			const cached = (await get_project_versions(projectId).catch(() => null)) as
				| Labrinth.Versions.v2.Version[]
				| null
			if (!cached) throw error
			return cached
		}
	}

	function fetchVersion(versionId: string) {
		return queryClient
			.fetchQuery(contentVersionQueryOptions.version(versionId, client))
			.catch(() => get_version(versionId) as Promise<Labrinth.Versions.v2.Version>)
	}

	const installVersionMutation = useMutation({
		networkMode: 'always',
		mutationFn: ({ filePath, versionId }: InstallVersionVariables) =>
			switch_project_version_with_dependencies(instance.value.id, filePath, versionId),
		onSuccess: (_result, { item }) => {
			trackEvent('InstanceProjectUpdate', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
				id: item.project?.id,
				name: item.project?.title ?? item.file_name,
				project_type: analyticsProjectType(item.project_type),
			})
		},
		onError: (error) => handleError(error),
		onSettled: async (_result, _error, { operation }) => {
			await refreshContentState('must_revalidate').catch(handleError)
			operations.finish(operation)
		},
	})

	const modpackVersionMutation = useMutation({
		networkMode: 'always',
		mutationFn: ({ instanceId, versionId }: { instanceId: string; versionId: string }) =>
			update_managed_modrinth_version(instanceId, versionId),
		onSuccess: () => refetchContent(),
		onError: (error) => handleError(error),
	})

	async function installVersion(item: ContentItem, version: Labrinth.Versions.v2.Version) {
		const isUpdate = item.has_update && item.update_version_id === version.id
		if (isUpdate ? !canUpdateProject(item) || item.locked : !canChangeContentVersion(item)) return
		if (!item.file_path) return
		const operation = operations.begin(item)
		if (!operation) return

		await installVersionMutation
			.mutateAsync({ item, filePath: item.file_path, versionId: version.id, operation })
			.catch(() => undefined)
	}

	async function installModpackVersion(version: Labrinth.Versions.v2.Version) {
		if (!instance.value?.id) return
		await modpackVersionMutation
			.mutateAsync({ instanceId: instance.value.id, versionId: version.id })
			.catch(() => undefined)
	}

	return {
		isModpackUpdating: modpackVersionMutation.isPending,
		fetchProjectVersions,
		fetchVersion,
		installVersion,
		installModpackVersion,
	}
}
