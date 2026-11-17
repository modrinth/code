import type { Labrinth } from '@modrinth/api-client'
import {
	type ContentItem,
	contentVersionQueryOptions,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { type Ref, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import {
	switch_project_version_with_dependencies,
	update_managed_modrinth_version,
} from '@/helpers/instance'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'

import { analyticsProjectType } from './content-items'
import type { ContentOperations } from './use-content-operations'

export interface ContentUpdaterOptions {
	instance: Ref<GameInstance>
	canUpdateProject: (item: ContentItem) => boolean
	canChangeContentVersion: (item: ContentItem) => boolean
	operations: ContentOperations
	refetchContent: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	refreshContentState: (cacheBehaviour?: CacheBehaviour) => Promise<void>
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
	const isModpackUpdating = ref(false)

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

	async function installVersion(item: ContentItem, version: Labrinth.Versions.v2.Version) {
		const isUpdate = item.has_update && item.update_version_id === version.id
		if (isUpdate ? !canUpdateProject(item) || item.locked : !canChangeContentVersion(item)) return
		if (!item.file_path) return
		const operation = operations.begin(item)
		if (!operation) return

		try {
			await switch_project_version_with_dependencies(instance.value.id, item.file_path, version.id)

			trackEvent('InstanceProjectUpdate', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
				id: item.project?.id,
				name: item.project?.title ?? item.file_name,
				project_type: analyticsProjectType(item.project_type),
			})
		} catch (err) {
			handleError(err as Error)
		} finally {
			await refreshContentState('must_revalidate')
			operations.finish(operation)
		}
	}

	async function installModpackVersion(version: Labrinth.Versions.v2.Version) {
		if (!instance.value?.id) return
		isModpackUpdating.value = true
		try {
			await update_managed_modrinth_version(instance.value.id, version.id)
			await refetchContent()
		} catch (err) {
			handleError(err as Error)
		} finally {
			isModpackUpdating.value = false
		}
	}

	return {
		isModpackUpdating,
		fetchProjectVersions,
		fetchVersion,
		installVersion,
		installModpackVersion,
	}
}
