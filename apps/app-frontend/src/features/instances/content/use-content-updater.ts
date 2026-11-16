import type { Labrinth } from '@modrinth/api-client'
import {
	type ConfirmModpackUpdateModal,
	type ContentItem,
	type ContentUpdaterModal,
	contentVersionQueryOptions,
	injectModrinthClient,
	injectNotificationManager,
	useDebugLogger,
	versionChangesGameVersion,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { nextTick, type Ref, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import {
	switch_project_version_with_dependencies,
	update_managed_modrinth_version,
} from '@/helpers/instance'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'

import { analyticsProjectType, contentKey } from './content-items'
import type { ContentOperations } from './use-content-operations'

export interface ContentUpdaterOptions {
	instance: Ref<GameInstance>
	canUpdateProject: (item: ContentItem) => boolean
	canChangeContentVersion: (item: ContentItem) => boolean
	operations: ContentOperations
	refetchContent: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	refreshContentState: (cacheBehaviour?: CacheBehaviour) => Promise<void>
	skipNonEssentialWarnings: Readonly<Ref<boolean>>
	linkedModpackUpdateVersionId: Readonly<Ref<string | null>>
}

export function useContentUpdater({
	instance,
	canUpdateProject,
	canChangeContentVersion,
	operations,
	refetchContent,
	refreshContentState,
	skipNonEssentialWarnings,
	linkedModpackUpdateVersionId,
}: ContentUpdaterOptions) {
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()
	const client = injectModrinthClient()
	const debug = useDebugLogger('Mods:ContentUpdate')
	const { begin: beginContentOperation, finish: finishContentOperation } = operations

	const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
	const modpackUpdateConfirmModal = ref<InstanceType<typeof ConfirmModpackUpdateModal> | null>()
	const updatingProject = ref<ContentItem | null>(null)
	const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
	const loadingVersions = ref(false)
	const loadingChangelog = ref(false)
	const updatingModpack = ref(false)
	const pendingModpackUpdateVersion = ref<Labrinth.Versions.v2.Version | null>(null)
	const isModpackUpdateDowngrade = ref(false)
	const isModpackUpdating = ref(false)

	let updateRequestId = 0
	const activeUpdateRequestId = ref(0)

	function beginUpdateRequest() {
		updateRequestId++
		activeUpdateRequestId.value = updateRequestId
		return updateRequestId
	}

	function isActiveUpdateRequest(requestId: number) {
		return activeUpdateRequestId.value === requestId
	}

	function sortVersionsByPublishedDate(versions: Labrinth.Versions.v2.Version[]) {
		return [...versions].sort(
			(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
		)
	}

	function mergeVersionIntoList(
		versions: Labrinth.Versions.v2.Version[],
		version: Labrinth.Versions.v2.Version,
	) {
		const existingIndex = versions.findIndex((v) => v.id === version.id)
		if (existingIndex === -1) {
			return sortVersionsByPublishedDate([version, ...versions])
		}

		const mergedVersions = [...versions]
		mergedVersions[existingIndex] = version
		return sortVersionsByPublishedDate(mergedVersions)
	}

	async function getUpdaterProjectVersions(projectId: string, pinnedVersionId?: string) {
		let fetchError: unknown = null
		let versions = await queryClient
			.fetchQuery(contentVersionQueryOptions.projectVersions(projectId, client))
			.catch((err) => {
				fetchError = err
				return null
			})

		if (!versions) {
			versions = (await get_project_versions(projectId).catch(() => null)) as
				| Labrinth.Versions.v2.Version[]
				| null
		}

		if (!versions && fetchError) {
			handleError(fetchError as Error)
		}

		let mergedVersions = sortVersionsByPublishedDate(versions ?? [])

		if (pinnedVersionId && !mergedVersions.some((version) => version.id === pinnedVersionId)) {
			const pinnedVersion = await fetchVersion(pinnedVersionId).catch(() => null)

			if (pinnedVersion) {
				mergedVersions = mergeVersionIntoList(mergedVersions, pinnedVersion)
			}
		}

		return mergedVersions
	}

	async function updateProject(mod: ContentItem) {
		if (!canUpdateProject(mod) || mod.locked) return
		const operation = beginContentOperation(mod)
		if (!operation) return

		try {
			const updateVersionId = mod.update_version_id!
			await switch_project_version_with_dependencies(
				instance.value.id,
				mod.file_path,
				updateVersionId,
			)

			trackEvent('InstanceProjectUpdate', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
				id: mod.project?.id,
				name: mod.project?.title ?? mod.file_name,
				project_type: analyticsProjectType(mod.project_type),
			})
		} catch (err) {
			handleError(err as Error)
			throw err
		} finally {
			await refreshContentState('must_revalidate')
			finishContentOperation(operation)
		}
	}

	async function switchProjectVersion(mod: ContentItem, version: Labrinth.Versions.v2.Version) {
		if (!canChangeContentVersion(mod)) return
		if (!mod.file_path) return
		const operation = beginContentOperation(mod)
		if (!operation) return

		const oldPath = mod.file_path

		try {
			await switch_project_version_with_dependencies(instance.value.id, oldPath, version.id)

			trackEvent('InstanceProjectUpdate', {
				loader: instance.value.loader,
				game_version: instance.value.game_version,
				id: mod.project?.id,
				name: mod.project?.title ?? mod.file_name,
				project_type: analyticsProjectType(mod.project_type),
			})
		} catch (err) {
			handleError(err as Error)
		} finally {
			await refreshContentState('must_revalidate')
			finishContentOperation(operation)
		}
	}

	async function handleUpdate(item: ContentItem) {
		if (item.locked || !canUpdateProject(item) || !item.project?.id || !item.version?.id) return

		const requestId = beginUpdateRequest()
		const itemId = contentKey(item)

		debug('handleUpdate triggered', {
			fileName: item.file_name,
			projectType: item.project_type,
			projectId: item.project.id,
			projectTitle: item.project.title,
			currentVersionId: item.version.id,
			currentVersionNumber: item.version.version_number,
			updateVersionId: item.update_version_id,
			instanceGameVersion: instance.value.game_version,
			instanceLoader: instance.value.loader,
		})

		updatingModpack.value = false
		updatingProject.value = item
		updatingProjectVersions.value = []
		loadingVersions.value = true
		loadingChangelog.value = false

		await nextTick()

		const initialVersionId = item.update_version_id ?? undefined
		debug('handleUpdate: opening content updater modal', {
			type: 'content',
			initialVersionId,
			item: {
				id: item.id,
				fileName: item.file_name,
				projectType: item.project_type,
				projectId: item.project.id,
				projectTitle: item.project.title,
				currentVersionId: item.version.id,
				currentVersionNumber: item.version.version_number,
				updateVersionId: item.update_version_id,
			},
			instance: {
				path: instance.value.id,
				name: instance.value.name,
				gameVersion: instance.value.game_version,
				loader: instance.value.loader,
				link: instance.value.link,
			},
			modalStateBeforeFetch: {
				updatingModpack: updatingModpack.value,
				updatingProjectId: updatingProject.value?.id,
				updatingProjectVersions: updatingProjectVersions.value.map((version) => ({
					id: version.id,
					versionNumber: version.version_number,
					gameVersions: version.game_versions,
					loaders: version.loaders,
					datePublished: version.date_published,
				})),
			},
		})
		contentUpdaterModal.value?.show(initialVersionId)

		const versions = await getUpdaterProjectVersions(item.project.id, initialVersionId)

		if (!isActiveUpdateRequest(requestId) || contentKey(updatingProject.value) !== itemId) return

		loadingVersions.value = false

		if (versions.length === 0) {
			debug('handleUpdate: no versions returned', { projectId: item.project.id })
			return
		}

		debug('handleUpdate: fetched versions', {
			projectId: item.project.id,
			projectType: item.project_type,
			totalVersions: versions.length,
			versionSample: versions.slice(0, 5).map((v) => ({
				id: v.id,
				number: v.version_number,
				loaders: v.loaders,
				gameVersions: v.game_versions,
			})),
			currentVersionInList: versions.some((v) => v.id === item.version?.id),
			updateVersionInList: versions.some((v) => v.id === item.update_version_id),
		})

		const preselectedVersion =
			versions.find((version) => version.id === initialVersionId) ?? versions[0] ?? null
		debug('handleUpdate: resolved content updater preselection', {
			type: 'content',
			initialVersionId,
			foundInitialVersion: versions.some((version) => version.id === initialVersionId),
			preselectedVersion: preselectedVersion
				? {
						id: preselectedVersion.id,
						versionNumber: preselectedVersion.version_number,
						gameVersions: preselectedVersion.game_versions,
						loaders: preselectedVersion.loaders,
						datePublished: preselectedVersion.date_published,
					}
				: null,
			versionCount: versions.length,
			currentVersionId: item.version.id,
			updateVersionId: item.update_version_id,
		})

		updatingProjectVersions.value = versions
	}

	async function handleSwitchVersion(item: ContentItem) {
		if (!canChangeContentVersion(item)) return
		if (!item.project?.id || !item.version?.id) return

		const requestId = beginUpdateRequest()
		const itemId = contentKey(item)

		updatingModpack.value = false
		updatingProject.value = item
		updatingProjectVersions.value = []
		loadingVersions.value = true
		loadingChangelog.value = false

		await nextTick()

		const initialVersionId = item.version.id
		contentUpdaterModal.value?.show(initialVersionId, { switchMode: true })

		const versions = await getUpdaterProjectVersions(item.project.id, initialVersionId)

		if (!isActiveUpdateRequest(requestId) || contentKey(updatingProject.value) !== itemId) return

		loadingVersions.value = false

		updatingProjectVersions.value = versions
	}

	async function handleModpackUpdate() {
		if (!instance.value?.link?.project_id) return

		const requestId = beginUpdateRequest()

		updatingModpack.value = true
		updatingProject.value = null
		updatingProjectVersions.value = []
		loadingVersions.value = true
		loadingChangelog.value = false

		await nextTick()

		const initialVersionId =
			linkedModpackUpdateVersionId.value ?? instance.value?.link?.version_id ?? undefined
		debug('handleModpackUpdate: opening modpack updater modal', {
			type: 'modpack',
			initialVersionId,
			linkedModpackUpdateVersionId: linkedModpackUpdateVersionId.value,
			linkedModpackProject: linkedModpackProject.value,
			linkedModpackVersion: linkedModpackVersion.value,
			instance: {
				path: instance.value.id,
				name: instance.value.name,
				gameVersion: instance.value.game_version,
				loader: instance.value.loader,
				link: instance.value.link,
			},
			modalStateBeforeFetch: {
				updatingModpack: updatingModpack.value,
				updatingProjectId: updatingProject.value?.id,
				updatingProjectVersions: updatingProjectVersions.value.map((version) => ({
					id: version.id,
					versionNumber: version.version_number,
					gameVersions: version.game_versions,
					loaders: version.loaders,
					datePublished: version.date_published,
				})),
			},
		})
		contentUpdaterModal.value?.show(initialVersionId)

		const versions = await getUpdaterProjectVersions(
			instance.value.link.project_id,
			initialVersionId,
		)

		if (!isActiveUpdateRequest(requestId) || !updatingModpack.value) return

		loadingVersions.value = false

		if (versions.length === 0) return

		const preselectedVersion =
			versions.find((version) => version.id === initialVersionId) ?? versions[0] ?? null
		debug('handleModpackUpdate: resolved modpack updater preselection', {
			type: 'modpack',
			initialVersionId,
			foundInitialVersion: versions.some((version) => version.id === initialVersionId),
			preselectedVersion: preselectedVersion
				? {
						id: preselectedVersion.id,
						versionNumber: preselectedVersion.version_number,
						gameVersions: preselectedVersion.game_versions,
						loaders: preselectedVersion.loaders,
						datePublished: preselectedVersion.date_published,
					}
				: null,
			versionCount: versions.length,
			linkedModpackUpdateVersionId: linkedModpackUpdateVersionId.value,
			currentLinkedVersionId: instance.value.link.version_id,
		})

		updatingProjectVersions.value = versions
	}

	function fetchVersion(versionId: string, cacheBehaviour?: Parameters<typeof get_version>[1]) {
		return queryClient
			.fetchQuery(contentVersionQueryOptions.version(versionId, client))
			.catch(() => get_version(versionId, cacheBehaviour) as Promise<Labrinth.Versions.v2.Version>)
	}

	async function fetchAndSpliceVersion(
		versionId: string,
		cacheBehaviour?: Parameters<typeof get_version>[1],
		onError?: (err: unknown) => void,
		requestId = activeUpdateRequestId.value,
	) {
		const fullVersion = await fetchVersion(versionId, cacheBehaviour).catch((err) => {
			onError?.(err)
			return null
		})
		if (!isActiveUpdateRequest(requestId)) return
		if (!fullVersion) return
		updatingProjectVersions.value = mergeVersionIntoList(updatingProjectVersions.value, fullVersion)
	}

	async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
		if (version.changelog != null) return
		const requestId = activeUpdateRequestId.value
		loadingChangelog.value = true
		await fetchAndSpliceVersion(
			version.id,
			'must_revalidate',
			handleError as (err: unknown) => void,
			requestId,
		)
		if (isActiveUpdateRequest(requestId)) {
			loadingChangelog.value = false
		}
	}

	async function handleVersionHover(version: Labrinth.Versions.v2.Version) {
		if (version.changelog != null) return
		await fetchAndSpliceVersion(version.id, undefined, undefined, activeUpdateRequestId.value)
	}

	function resetUpdateState() {
		activeUpdateRequestId.value = 0
		updatingModpack.value = false
		updatingProject.value = null
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	}

	async function handleModpackUpdateRequest(selectedVersion: Labrinth.Versions.v2.Version) {
		pendingModpackUpdateVersion.value = selectedVersion

		const currentVersionId = instance.value?.link?.version_id
		const currentVersion = updatingProjectVersions.value.find((v) => v.id === currentVersionId)
		isModpackUpdateDowngrade.value = currentVersion
			? new Date(selectedVersion.date_published) < new Date(currentVersion.date_published)
			: false
		const shouldShowWarning =
			isModpackUpdateDowngrade.value ||
			versionChangesGameVersion(selectedVersion, instance.value.game_version)

		if (skipNonEssentialWarnings.value || !shouldShowWarning) {
			await handleModpackUpdateConfirm()
			return
		}

		modpackUpdateConfirmModal.value?.show()
	}

	async function handleModpackUpdateConfirm() {
		if (!pendingModpackUpdateVersion.value || !instance.value?.id) return

		const version = pendingModpackUpdateVersion.value
		pendingModpackUpdateVersion.value = null
		const requestId = activeUpdateRequestId.value

		contentUpdaterModal.value?.hide()
		isModpackUpdating.value = true
		try {
			await update_managed_modrinth_version(instance.value.id, version.id)
			await refetchContent()
		} finally {
			isModpackUpdating.value = false
			if (isActiveUpdateRequest(requestId)) {
				resetUpdateState()
			}
		}
	}

	function handleModpackUpdateCancel() {
		pendingModpackUpdateVersion.value = null
	}

	async function handleModalUpdate(
		selectedVersion: Labrinth.Versions.v2.Version,
		event?: MouseEvent,
	) {
		if (updatingModpack.value) {
			if (event?.shiftKey) {
				pendingModpackUpdateVersion.value = selectedVersion
				await handleModpackUpdateConfirm()
			} else {
				await handleModpackUpdateRequest(selectedVersion)
			}
		} else if (updatingProject.value) {
			const mod = updatingProject.value
			const requestId = activeUpdateRequestId.value

			try {
				if (mod.has_update && mod.update_version_id === selectedVersion.id) {
					await updateProject(mod)
				} else {
					await switchProjectVersion(mod, selectedVersion)
				}
			} finally {
				if (isActiveUpdateRequest(requestId)) {
					resetUpdateState()
				}
			}
		}
	}

	return {
		contentUpdaterModal,
		modpackUpdateConfirmModal,
		updatingProject,
		updatingProjectVersions,
		loadingVersions,
		loadingChangelog,
		updatingModpack,
		pendingModpackUpdateVersion,
		isModpackUpdateDowngrade,
		isModpackUpdating,
		handleUpdate,
		handleSwitchVersion,
		handleModpackUpdate,
		handleVersionSelect,
		handleVersionHover,
		handleModalUpdate,
		handleModpackUpdateConfirm,
		handleModpackUpdateCancel,
		resetUpdateState,
	}
}
