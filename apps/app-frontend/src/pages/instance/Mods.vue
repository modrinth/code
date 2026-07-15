<template>
	<ReadyTransition :pending="loading">
		<ContentPageLayout>
			<template #modals>
				<UnknownFileWarningModal
					ref="unknownFileWarningModal"
					mode="mod"
					:file-name="unknownFileName"
					@cancel="resolveUnknownFileWarning(false)"
					@continue="handleUnknownFileContinue"
				/>
				<ShareModalWrapper
					ref="shareModal"
					:share-title="formatMessage(messages.shareTitle)"
					:share-text="formatMessage(messages.shareText)"
					:open-in-new-tab="false"
				/>
				<ModpackContentModal
					ref="modpackContentModal"
					:modpack-name="displayedModpackProject?.title"
					:modpack-icon-url="displayedModpackProject?.icon_url ?? undefined"
					:enable-toggle="!props.isServerInstance"
					:busy="isBulkOperating"
					:get-overflow-options="getOverflowOptions"
					:switch-version="handleSwitchVersion"
					@update:enabled="handleModpackContentToggle"
					@bulk:enable="(items) => handleModpackContentBulkToggle(items, true)"
					@bulk:disable="(items) => handleModpackContentBulkToggle(items, false)"
				/>
				<ConfirmModpackUpdateModal
					ref="modpackUpdateConfirmModal"
					:downgrade="isModpackUpdateDowngrade"
					:backup-tip="
						[displayedModpackProject?.title, pendingModpackUpdateVersion?.version_number]
							.filter(Boolean)
							.join(' ')
					"
					@confirm="handleModpackUpdateConfirm"
					@cancel="handleModpackUpdateCancel"
				/>
				<ExportModal v-if="projects.length > 0" ref="exportModal" :instance="instance" />
				<ContentUpdaterModal
					v-if="updatingProject || updatingModpack"
					ref="contentUpdaterModal"
					:versions="updatingProjectVersions"
					:current-game-version="instance.game_version"
					:current-loader="instance.loader"
					:current-version-id="
						updatingModpack
							? (instance.link?.version_id ?? '')
							: (updatingProject?.version?.id ?? '')
					"
					:is-app="true"
					:project-type="updatingModpack ? 'modpack' : updatingProject?.project_type"
					:project-icon-url="
						updatingModpack ? displayedModpackProject?.icon_url : updatingProject?.project?.icon_url
					"
					:project-name="
						updatingModpack
							? (displayedModpackProject?.title ?? formatMessage(commonMessages.modpackLabel))
							: (updatingProject?.project?.title ?? updatingProject?.file_name)
					"
					:loading="loadingVersions"
					:loading-changelog="loadingChangelog"
					@update="handleModalUpdate"
					@cancel="resetUpdateState"
					@version-select="handleVersionSelect"
					@version-hover="handleVersionHover"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, FolderOpenIcon } from '@modrinth/assets'
import {
	type BulkOperationStatus,
	commonMessages,
	ConfirmModpackUpdateModal,
	ContentCardLayout as ContentPageLayout,
	type ContentItem,
	type ContentModpackCardCategory,
	type ContentModpackCardProject,
	type ContentModpackCardVersion,
	type ContentOwner,
	ContentUpdaterModal,
	defineMessages,
	injectNotificationManager,
	ModpackContentModal,
	type ModpackContentModalState,
	type OverflowMenuOption,
	provideAppBackup,
	provideContentManager,
	ReadyTransition,
	UnknownFileWarningModal,
	useDebugLogger,
	useVIntl,
	versionChangesGameVersion,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version, get_version_many } from '@/helpers/cache.js'
import {
	instance_bulk_update_progress_listener,
	instance_listener,
	type InstanceBulkUpdateProgress,
} from '@/helpers/events.js'
import { install_duplicate_instance, installJobInstanceId } from '@/helpers/install'
import {
	add_project_from_path,
	edit,
	get_linked_modpack_content,
	is_file_on_modrinth,
	list,
	remove_project,
	switch_project_version_with_dependencies,
	toggle_disable_project,
	update_all,
	update_managed_modrinth_version,
} from '@/helpers/instance'
import { type InstanceContentData, loadInstanceContentData } from '@/helpers/instance-content'
import { get as getSettings, set as setSettings } from '@/helpers/settings'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import { highlightModInInstance } from '@/helpers/utils.js'
import { injectContentInstall } from '@/providers/content-install'
import { useTheming } from '@/store/state'
import type { FeatureFlag } from '@/store/theme'

const messages = defineMessages({
	shareTitle: {
		id: 'app.instance.mods.share-title',
		defaultMessage: 'Sharing modpack content',
	},
	shareText: {
		id: 'app.instance.mods.share-text',
		defaultMessage: "Check out the projects I'm using in my modpack!",
	},
	successfullyUploaded: {
		id: 'app.instance.mods.successfully-uploaded',
		defaultMessage: 'Successfully uploaded',
	},
	projectWasAdded: {
		id: 'app.instance.mods.project-was-added',
		defaultMessage: '"{name}" was added',
	},
	projectsWereAdded: {
		id: 'app.instance.mods.projects-were-added',
		defaultMessage: '{count} projects were added',
	},
	contentTypeProject: {
		id: 'app.instance.mods.content-type-project',
		defaultMessage: 'project',
	},
	bulkUpdateResolvingVersions: {
		id: 'app.instance.mods.bulk-update.resolving-versions',
		defaultMessage: 'Resolving versions...',
	},
	bulkUpdateDownloadingProjects: {
		id: 'app.instance.mods.bulk-update.downloading-projects',
		defaultMessage: 'Downloading {current, number}/{total, number} projects...',
	},
	bulkUpdateFinishing: {
		id: 'app.instance.mods.bulk-update.finishing',
		defaultMessage: 'Finishing update...',
	},
})

let savedModalState: ModpackContentModalState | null = null

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const { installingItems, installRevisionByInstance, installFailureRevisionByInstance } =
	injectContentInstall()
const router = useRouter()
const queryClient = useQueryClient()
const debug = useDebugLogger('Mods:ContentUpdate')
const themeStore = useTheming()
const skipUnknownFileWarningFeatureFlag = 'skip_unknown_pack_warning' as FeatureFlag
const skipNonEssentialWarnings = computed(() =>
	themeStore.getFeatureFlag('skip_non_essential_warnings'),
)

const props = defineProps<{
	instance: GameInstance
	isServerInstance?: boolean
	openSettings?: () => void
	preloadedContent?: InstanceContentData | null
}>()

function hasPreloadedContent(contentData: InstanceContentData | null | undefined) {
	return contentData?.path === props.instance.id
}

const loading = ref(!hasPreloadedContent(props.preloadedContent))
const projects = ref<ContentItem[]>([])

const installingBuffer = ref<ContentItem[]>([])
const handledInstallRevision = ref(0)

watch(
	() => installingItems.value.get(props.instance.id),
	(items) => {
		if (items && items.length > 0) {
			installingBuffer.value = [...items]
		}
	},
	{ immediate: true, deep: true },
)

watch(projects, (newProjects) => {
	if (installingBuffer.value.length === 0) return
	const realProjectIds = new Set(newProjects.map((p) => p.project?.id).filter(Boolean))
	if (installingBuffer.value.every((item) => realProjectIds.has(item.project?.id))) {
		installingBuffer.value = []
	}
})

const mergedProjects = computed<ContentItem[]>(() => {
	const active = installingItems.value.get(props.instance.id)
	const pending = active ?? installingBuffer.value
	if (pending.length === 0) return projects.value
	const pendingProjectIds = new Set(pending.map((p) => p.project?.id).filter(Boolean))
	const displayProjects = projects.value.map((project) =>
		project.project?.id && pendingProjectIds.has(project.project.id)
			? { ...project, installing: true }
			: project,
	)
	const realProjectIds = new Set(displayProjects.map((p) => p.project?.id).filter(Boolean))
	const placeholders = pending.filter((item) => !realProjectIds.has(item.project?.id))
	return placeholders.length > 0 ? [...displayProjects, ...placeholders] : displayProjects
})

watch(
	() => installFailureRevisionByInstance.value.get(props.instance.id) ?? 0,
	(revision, previousRevision) => {
		if (revision === previousRevision) return
		installingBuffer.value = []
	},
)

const linkedModpackProject = ref<ContentModpackCardProject | null>(null)
const linkedModpackVersion = ref<ContentModpackCardVersion | null>(null)
const linkedModpackOwner = ref<ContentOwner | null>(null)
const linkedModpackCategories = ref<ContentModpackCardCategory[]>([])
const linkedModpackHasUpdate = ref(false)
const linkedModpackUpdateVersionId = ref<string | null>(null)
const localImportedModpackUnlinked = ref(false)

const localImportedModpackProject = computed<ContentModpackCardProject | null>(() => {
	const link = props.instance.link
	if (localImportedModpackUnlinked.value || link?.type !== 'imported_modpack') return null

	return {
		id: link.filename ?? props.instance.id,
		slug: link.filename ?? props.instance.id,
		title: link.name ?? props.instance.name,
		icon_url: props.instance.icon_path ? convertFileSrc(props.instance.icon_path) : undefined,
		description: '',
		filename: link.filename ?? undefined,
	}
})

const displayedModpackProject = computed(
	() => linkedModpackProject.value ?? localImportedModpackProject.value,
)

watch(
	() => props.instance.link,
	() => {
		localImportedModpackUnlinked.value = false
	},
)

const isModpackUpdating = ref(false)
const isBulkOperating = ref(false)
const isInstanceBusy = computed(() => props.instance?.install_stage !== 'installed')
const isPackLocked = computed(
	() =>
		props.instance?.link?.type === 'modrinth_modpack' ||
		props.instance?.link?.type === 'server_project_modpack',
)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
const modpackContentModal = ref<InstanceType<typeof ModpackContentModal> | null>()
const modpackUpdateConfirmModal = ref<InstanceType<typeof ConfirmModpackUpdateModal> | null>()
const unknownFileWarningModal = ref<InstanceType<typeof UnknownFileWarningModal> | null>()
const unknownFileName = ref('')
let resolveUnknownFileConfirmation: ((confirmed: boolean) => void) | null = null

const modpackContentQueryKey = computed(() => ['linkedModpackContent', props.instance.id])
const modpackContentQuery = useQuery({
	queryKey: modpackContentQueryKey,
	queryFn: () => get_linked_modpack_content(props.instance.id),
	enabled: computed(
		() =>
			!!props.instance?.id &&
			!!props.instance?.link &&
			props.instance.install_stage === 'installed',
	),
})

// TODO: Extract content operation and updater modal state into composables; this page currently owns file mutations, dependency installs, busy flags, and version selection flow.
const updatingProject = ref<ContentItem | null>(null)
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)
const updatingModpack = ref(false)
const pendingModpackUpdateVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const isModpackUpdateDowngrade = ref(false)
const activeContentOperationKeys = ref(new Set<string>())

let activeContentOperationCount = 0
let updateRequestId = 0
const activeUpdateRequestId = ref(0)

function fileNameFromPath(path: string) {
	return path.split('/').pop() ?? path
}

function matchesContentItem(
	item: ContentItem,
	target: ContentItem,
	originalFileName: string,
	originalFilePath?: string,
) {
	if (item.file_name === originalFileName || item.file_path === originalFilePath) return true

	const projectId = target.project?.id
	if (!projectId || item.project?.id !== projectId) return false

	const versionId = target.version?.id
	return !versionId || item.version?.id === versionId
}

function updateLinkedModpackContentCache(
	target: ContentItem,
	originalFileName: string,
	originalFilePath: string | undefined,
	updates: Partial<ContentItem>,
) {
	queryClient.setQueryData<ContentItem[]>(modpackContentQueryKey.value, (items) => {
		if (!items) return items

		return items.map((item) =>
			matchesContentItem(item, target, originalFileName, originalFilePath)
				? { ...item, ...updates }
				: item,
		)
	})
}

function getContentItemId(item: ContentItem | null | undefined) {
	return item?.file_path ?? item?.file_name ?? item?.id ?? ''
}

function getContentOperationKeys(item: ContentItem) {
	return [getContentItemId(item), item.file_path, item.file_name].filter(
		(key): key is string => !!key,
	)
}

function hasContentOperation(item: ContentItem) {
	const keys = getContentOperationKeys(item)
	return keys.some((key) => activeContentOperationKeys.value.has(key))
}

function canUpdateProject(item: ContentItem) {
	return !!item.file_path && !!item.has_update && !!item.update_version_id
}

function setContentItemBusy(item: ContentItem, busy: boolean, originalFileName = item.file_name) {
	item.installing = busy
	modpackContentModal.value?.updateItem(originalFileName, {
		installing: busy,
		disabled: busy,
	})
	if (item.file_name !== originalFileName) {
		modpackContentModal.value?.updateItem(item.file_name, {
			installing: busy,
			disabled: busy,
		})
	}
}

function beginContentOperation(item: ContentItem) {
	if (hasContentOperation(item)) return null

	const keys = getContentOperationKeys(item)
	activeContentOperationKeys.value = new Set([...activeContentOperationKeys.value, ...keys])
	activeContentOperationCount++
	isBulkOperating.value = true
	setContentItemBusy(item, true)

	return { keys, originalFileName: item.file_name }
}

function finishContentOperation(
	item: ContentItem,
	operation: { keys: string[]; originalFileName: string },
) {
	const nextKeys = new Set(activeContentOperationKeys.value)
	for (const key of operation.keys) {
		nextKeys.delete(key)
	}
	activeContentOperationKeys.value = nextKeys
	activeContentOperationCount = Math.max(0, activeContentOperationCount - 1)
	setContentItemBusy(item, false, operation.originalFileName)
	if (activeContentOperationCount === 0) {
		isBulkOperating.value = false
	}
}

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
	let versions = (await get_project_versions(projectId, 'bypass').catch((err) => {
		fetchError = err
		return null
	})) as Labrinth.Versions.v2.Version[] | null

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
		const pinnedVersion = (await get_version(pinnedVersionId, 'bypass').catch(
			() => null,
		)) as Labrinth.Versions.v2.Version | null

		if (pinnedVersion) {
			mergedVersions = mergeVersionIntoList(mergedVersions, pinnedVersion)
		}
	}

	return mergedVersions
}

async function handleBrowseContent() {
	if (!props.instance) return
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
		query: { i: props.instance.id },
	})
}

async function handleUploadFiles() {
	if (!props.instance) return
	const files = await open({ multiple: true })
	if (!files) return
	const selectedFiles: Array<{ path: string; filename: string }> = []
	for (const file of files) {
		const path = (file as { path?: string }).path ?? file
		if (typeof path !== 'string') continue
		selectedFiles.push({
			path,
			filename: path.split(/[\\/]/).pop() ?? path,
		})
	}

	const fileRecognition = await Promise.all(
		selectedFiles.map(async ({ path }) => {
			try {
				return await is_file_on_modrinth(path)
			} catch {
				return true
			}
		}),
	)

	const addedFiles: string[] = []
	for (const [index, { path, filename }] of selectedFiles.entries()) {
		if (!fileRecognition[index] && !(await confirmUnknownFileInstallation(filename))) {
			continue
		}
		try {
			await add_project_from_path(props.instance.id, path)
			addedFiles.push(filename)
		} catch (e) {
			handleError(e as Error)
		}
	}
	await initProjects()

	if (addedFiles.length > 0) {
		const names = addedFiles.map((f) => {
			const item = projects.value.find(
				(p) => p.file_name === f || p.file_name === f.replace('.zip', '.jar'),
			)
			return item?.project?.title ?? f
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.successfullyUploaded),
			text:
				names.length === 1
					? formatMessage(messages.projectWasAdded, { name: names[0] })
					: formatMessage(messages.projectsWereAdded, { count: names.length }),
		})
	}
}

function confirmUnknownFileInstallation(fileName: string) {
	if (themeStore.getFeatureFlag(skipUnknownFileWarningFeatureFlag)) {
		return Promise.resolve(true)
	}

	unknownFileName.value = fileName
	return new Promise<boolean>((resolve) => {
		resolveUnknownFileConfirmation = resolve
		void nextTick(() => unknownFileWarningModal.value?.show())
	})
}

function resolveUnknownFileWarning(confirmed: boolean) {
	const resolve = resolveUnknownFileConfirmation
	resolveUnknownFileConfirmation = null
	unknownFileName.value = ''
	resolve?.(confirmed)
}

async function handleUnknownFileContinue(dontShowAgain: boolean) {
	if (dontShowAgain) {
		themeStore.featureFlags[skipUnknownFileWarningFeatureFlag] = true
		try {
			const settings = await getSettings()
			settings.feature_flags[skipUnknownFileWarningFeatureFlag] = true
			await setSettings(settings)
		} catch (error) {
			handleError(error as Error)
		}
	}
	resolveUnknownFileWarning(true)
}

async function toggleDisableMod(mod: ContentItem, desiredEnabled?: boolean) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return
	const originalFilePath = mod.file_path

	try {
		const newPath = await toggle_disable_project(props.instance.id, mod.file_path, desiredEnabled)
		const newFileName = fileNameFromPath(newPath)
		const enabled = !newPath.endsWith('.disabled')
		mod.file_path = newPath
		mod.file_name = newFileName
		mod.enabled = enabled
		modpackContentModal.value?.updateItem(operation.originalFileName, {
			file_path: newPath,
			file_name: newFileName,
			enabled,
		})
		updateLinkedModpackContentCache(mod, operation.originalFileName, originalFilePath, {
			file_path: newPath,
			file_name: newFileName,
			enabled,
		})

		trackEvent('InstanceProjectDisable', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
			disabled: !enabled,
		})
	} catch (err) {
		handleError(err as Error)
	} finally {
		finishContentOperation(mod, operation)
	}
}

const toggleDisableDebounced = toggleDisableMod

async function removeMod(mod: ContentItem) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	try {
		const removedPath = mod.file_path
		await remove_project(props.instance.id, removedPath)
		projects.value = projects.value.filter((x) => removedPath !== x.file_path)

		trackEvent('InstanceProjectRemove', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
	} finally {
		finishContentOperation(mod, operation)
	}
}

function isBreakingDependency(dependency: Labrinth.Versions.v2.Dependency) {
	return dependency.dependency_type === 'required' || dependency.dependency_type === 'embedded'
}

function dependencyTargetsItem(dependency: Labrinth.Versions.v2.Dependency, item: ContentItem) {
	return (
		(!!dependency.project_id && dependency.project_id === item.project?.id) ||
		('version_id' in dependency &&
			!!dependency.version_id &&
			dependency.version_id === item.version?.id)
	)
}

async function getDeleteDependencyWarning(items: ContentItem[]) {
	if (props.isServerInstance) return null

	const deletingIds = new Set(items.map(getContentItemId))
	const remainingItems = projects.value.filter((item) => !deletingIds.has(getContentItemId(item)))
	const versionIds = [
		...new Set(remainingItems.map((item) => item.version?.id).filter((id): id is string => !!id)),
	]

	if (versionIds.length === 0) return null

	const versions = (await get_version_many(versionIds).catch((err) => {
		handleError(err as Error)
		return null
	})) as Labrinth.Versions.v2.Version[] | null

	if (!versions) return null

	const versionsById = new Map(versions.map((version) => [version.id, version]))

	const dependents = remainingItems
		.map((candidate) => {
			const version = candidate.version?.id ? versionsById.get(candidate.version.id) : null
			if (!version) return null

			const dependencies = items.filter((item) => {
				if (!item.project?.id && !item.version?.id) return false

				return version.dependencies?.some(
					(dependency) =>
						isBreakingDependency(dependency) && dependencyTargetsItem(dependency, item),
				)
			})

			return dependencies.length > 0 ? { item: candidate, dependencies } : null
		})
		.filter(
			(dependent): dependent is { item: ContentItem; dependencies: ContentItem[] } =>
				dependent !== null,
		)

	return dependents.length > 0 ? { items, dependents } : null
}

function formatBulkUpdateProgress(progress: InstanceBulkUpdateProgress): BulkOperationStatus {
	if (progress.stage === 'resolving_versions') {
		return {
			message: formatMessage(messages.bulkUpdateResolvingVersions),
			waiting: true,
		}
	}

	if (progress.stage === 'finishing') {
		return {
			message: formatMessage(messages.bulkUpdateFinishing),
			progress: progress.current,
			total: progress.total,
		}
	}

	return {
		message: formatMessage(messages.bulkUpdateDownloadingProjects, {
			current: progress.current,
			total: progress.total,
		}),
		progress: progress.current,
		total: progress.total,
	}
}

async function bulkUpdateAllProjects(onProgress?: (status: BulkOperationStatus) => void) {
	let unlisten: (() => void) | null = null
	try {
		if (onProgress) {
			onProgress({
				message: formatMessage(messages.bulkUpdateResolvingVersions),
				waiting: true,
			})
			unlisten = await instance_bulk_update_progress_listener((progress) => {
				if (progress.instanceId !== props.instance.id) return
				onProgress(formatBulkUpdateProgress(progress))
			})
		}

		await update_all(props.instance.id)
		await refreshContentState('must_revalidate')
	} catch (err) {
		handleError(err as Error)
		throw err
	} finally {
		unlisten?.()
	}
}

async function updateProject(mod: ContentItem) {
	if (!canUpdateProject(mod)) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	try {
		const updateVersionId = mod.update_version_id!
		await switch_project_version_with_dependencies(
			props.instance.id,
			mod.file_path,
			updateVersionId,
		)

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
		throw err
	} finally {
		await refreshContentState('must_revalidate')
		finishContentOperation(mod, operation)
	}
}

async function switchProjectVersion(mod: ContentItem, version: Labrinth.Versions.v2.Version) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	const oldPath = mod.file_path

	try {
		await switch_project_version_with_dependencies(props.instance.id, oldPath, version.id)

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
	} finally {
		await refreshContentState('must_revalidate')
		finishContentOperation(mod, operation)
	}
}

async function handleUpdate(id: string) {
	const item = projects.value.find((p) => getContentItemId(p) === id)
	if (!item || !canUpdateProject(item) || !item.project?.id || !item.version?.id) return

	const requestId = beginUpdateRequest()
	const itemId = getContentItemId(item)

	debug('handleUpdate triggered', {
		fileName: item.file_name,
		projectType: item.project_type,
		projectId: item.project.id,
		projectTitle: item.project.title,
		currentVersionId: item.version.id,
		currentVersionNumber: item.version.version_number,
		updateVersionId: item.update_version_id,
		instanceGameVersion: props.instance.game_version,
		instanceLoader: props.instance.loader,
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
			path: props.instance.id,
			name: props.instance.name,
			gameVersion: props.instance.game_version,
			loader: props.instance.loader,
			link: props.instance.link,
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

	if (!isActiveUpdateRequest(requestId) || getContentItemId(updatingProject.value) !== itemId)
		return

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
	if (!item.project?.id || !item.version?.id) return

	const requestId = beginUpdateRequest()
	const itemId = getContentItemId(item)

	updatingModpack.value = false
	updatingProject.value = item
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	const initialVersionId = item.version.id
	contentUpdaterModal.value?.show(initialVersionId, { switchMode: true })

	const versions = await getUpdaterProjectVersions(item.project.id, initialVersionId)

	if (!isActiveUpdateRequest(requestId) || getContentItemId(updatingProject.value) !== itemId)
		return

	loadingVersions.value = false

	updatingProjectVersions.value = versions
}

async function handleModpackContentToggle(item: ContentItem, enabled: boolean) {
	await toggleDisableDebounced(item, enabled)
}

async function handleModpackContentBulkToggle(items: ContentItem[], enabled: boolean) {
	await Promise.all(items.map((item) => toggleDisableMod(item, enabled)))
}

async function handleModpackContent() {
	if (!props.instance?.id) return

	if (modpackContentQuery.data.value?.length) {
		modpackContentModal.value?.show(modpackContentQuery.data.value)
		return
	}

	modpackContentModal.value?.showLoading()

	const { data, error } = await modpackContentQuery.refetch()

	if (data !== undefined) {
		modpackContentModal.value?.show(data)
	} else {
		if (error) handleError(error)
		modpackContentModal.value?.hide()
	}
}

async function refreshModpackContentItems(cacheBehaviour?: CacheBehaviour) {
	if (!props.instance?.id) return

	const contentItems = await queryClient
		.fetchQuery({
			queryKey: modpackContentQueryKey.value,
			queryFn: () => get_linked_modpack_content(props.instance.id, cacheBehaviour),
		})
		.catch(handleError)

	if (contentItems) {
		modpackContentModal.value?.setItems(contentItems)
	}
}

async function refreshContentState(cacheBehaviour?: CacheBehaviour) {
	await initProjects(cacheBehaviour)
	await refreshModpackContentItems(cacheBehaviour)
}

watch(
	() => installRevisionByInstance.value.get(props.instance.id) ?? 0,
	async (revision) => {
		if (revision <= handledInstallRevision.value) return
		handledInstallRevision.value = revision
		await refreshContentState('must_revalidate')
	},
)

async function handleModpackUpdate() {
	if (!props.instance?.link?.project_id) return

	const requestId = beginUpdateRequest()

	updatingModpack.value = true
	updatingProject.value = null
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	const initialVersionId =
		linkedModpackUpdateVersionId.value ?? props.instance?.link?.version_id ?? undefined
	debug('handleModpackUpdate: opening modpack updater modal', {
		type: 'modpack',
		initialVersionId,
		linkedModpackUpdateVersionId: linkedModpackUpdateVersionId.value,
		linkedModpackProject: linkedModpackProject.value,
		linkedModpackVersion: linkedModpackVersion.value,
		linkedModpackHasUpdate: linkedModpackHasUpdate.value,
		instance: {
			path: props.instance.id,
			name: props.instance.name,
			gameVersion: props.instance.game_version,
			loader: props.instance.loader,
			link: props.instance.link,
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

	const versions = await getUpdaterProjectVersions(props.instance.link.project_id, initialVersionId)

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
		currentLinkedVersionId: props.instance.link.version_id,
	})

	updatingProjectVersions.value = versions
}

async function fetchAndSpliceVersion(
	versionId: string,
	cacheBehaviour?: Parameters<typeof get_version>[1],
	onError?: (err: unknown) => void,
	requestId = activeUpdateRequestId.value,
) {
	const fullVersion = (await get_version(versionId, cacheBehaviour).catch(
		onError ?? (() => null),
	)) as Labrinth.Versions.v2.Version | null
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

	const currentVersionId = props.instance?.link?.version_id
	const currentVersion = updatingProjectVersions.value.find((v) => v.id === currentVersionId)
	isModpackUpdateDowngrade.value = currentVersion
		? new Date(selectedVersion.date_published) < new Date(currentVersion.date_published)
		: false
	const shouldShowWarning =
		isModpackUpdateDowngrade.value ||
		versionChangesGameVersion(selectedVersion, props.instance.game_version)

	if (skipNonEssentialWarnings.value || !shouldShowWarning) {
		await handleModpackUpdateConfirm()
		return
	}

	modpackUpdateConfirmModal.value?.show()
}

async function handleModpackUpdateConfirm() {
	if (!pendingModpackUpdateVersion.value || !props.instance?.id) return

	const version = pendingModpackUpdateVersion.value
	pendingModpackUpdateVersion.value = null

	contentUpdaterModal.value?.hide()
	isModpackUpdating.value = true
	try {
		await update_managed_modrinth_version(props.instance.id, version.id)
		await initProjects()
	} finally {
		isModpackUpdating.value = false
		resetUpdateState()
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

		try {
			if (mod.has_update && mod.update_version_id === selectedVersion.id) {
				await updateProject(mod)
			} else {
				await switchProjectVersion(mod, selectedVersion)
			}
		} finally {
			resetUpdateState()
		}
	}
}

async function unpairInstance() {
	await edit(props.instance.id, {
		link: null as unknown as undefined,
	})
	linkedModpackProject.value = null
	linkedModpackVersion.value = null
	linkedModpackOwner.value = null
	linkedModpackHasUpdate.value = false
	linkedModpackUpdateVersionId.value = null
	localImportedModpackUnlinked.value = true
	await initProjects()
}

async function handleShareItems(
	items: ContentItem[],
	format: 'names' | 'file-names' | 'urls' | 'markdown',
) {
	const source = items.length > 0 ? items : projects.value
	let text: string
	switch (format) {
		case 'names':
			text = source.map((x) => x.project?.title ?? x.file_name).join('\n')
			break
		case 'file-names':
			text = source.map((x) => x.file_name).join('\n')
			break
		case 'urls':
			text = source
				.filter((x) => x.project?.slug)
				.map((x) => `https://modrinth.com/${x.project_type}/${x.project?.slug}`)
				.join('\n')
			break
		case 'markdown':
			text = source
				.map((x) => {
					const name = x.project?.title ?? x.file_name
					if (x.project?.slug) {
						return `[${name}](https://modrinth.com/${x.project_type}/${x.project.slug})`
					}
					return name
				})
				.join('\n')
			break
	}
	await shareModal.value?.show(text)
}

function getOverflowOptions(item: ContentItem): OverflowMenuOption[] {
	const options: OverflowMenuOption[] = []

	options.push({
		id: formatMessage(commonMessages.showFileButton),
		icon: FolderOpenIcon,
		action: () => highlightModInInstance(props.instance.id, item.file_path),
	})

	if (item.project?.slug) {
		options.push({
			id: formatMessage(commonMessages.copyLinkButton),
			icon: ClipboardCopyIcon,
			action: async () => {
				await navigator.clipboard.writeText(
					`https://modrinth.com/${item.project_type}/${item.project?.slug}`,
				)
			},
		})
	}

	return options
}

async function initProjects(cacheBehaviour?: CacheBehaviour) {
	if (!props.instance) return

	const contentData = await loadInstanceContentData(props.instance.id, cacheBehaviour, handleError)
	applyContentData(contentData)
}

function applyContentData(contentData: InstanceContentData) {
	if (contentData.path !== props.instance.id) {
		return false
	}

	if (!contentData.contentItems) {
		loading.value = false
		return true
	}

	projects.value = contentData.contentItems.map((item) => ({
		...item,
		has_update: canUpdateProject(item),
	}))

	if (contentData.modpack) {
		linkedModpackProject.value = contentData.modpack.project
		linkedModpackVersion.value = contentData.modpack.version
		linkedModpackOwner.value = contentData.modpack.owner
		linkedModpackCategories.value = contentData.modpack.categories
		linkedModpackHasUpdate.value = contentData.modpack.hasUpdate
		linkedModpackUpdateVersionId.value = contentData.modpack.updateVersionId
	} else {
		linkedModpackProject.value = null
		linkedModpackVersion.value = null
		linkedModpackOwner.value = null
		linkedModpackCategories.value = []
		linkedModpackHasUpdate.value = false
		linkedModpackUpdateVersionId.value = null
	}

	loading.value = false
	return true
}

provideAppBackup({
	async createBackup() {
		const allInstances = await list()
		const prefix = `${props.instance.name} - Backup #`
		const existingNums = allInstances
			.filter((p) => p.name.startsWith(prefix))
			.map((p) => parseInt(p.name.slice(prefix.length), 10))
			.filter((n) => !isNaN(n))
		const nextNum = existingNums.length > 0 ? Math.max(...existingNums) + 1 : 1
		const job = await install_duplicate_instance(props.instance.id)
		const newInstanceId = installJobInstanceId(job)
		if (newInstanceId) {
			await edit(newInstanceId, { name: `${prefix}${nextNum}` })
		}
	},
})

const CONTENT_HINT_KEY = 'content-tab-modpack-hint-dismissed'
const showContentHint = ref(localStorage.getItem(CONTENT_HINT_KEY) === null)
function dismissContentHint() {
	showContentHint.value = false
	localStorage.setItem(CONTENT_HINT_KEY, 'true')
}

provideContentManager({
	items: mergedProjects,
	loading,
	error: ref(null),
	modpack: computed(() => {
		if (linkedModpackProject.value) {
			return {
				project: linkedModpackProject.value,
				projectLink: {
					path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}`,
					query: { i: props.instance.id },
				},
				version: linkedModpackVersion.value ?? undefined,
				versionLink:
					linkedModpackProject.value && linkedModpackVersion.value
						? {
								path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}/version/${linkedModpackVersion.value.id}`,
								query: { i: props.instance.id },
							}
						: undefined,
				owner: linkedModpackOwner.value
					? {
							...linkedModpackOwner.value,
							link: () =>
								openUrl(
									`https://modrinth.com/${linkedModpackOwner.value!.type}/${linkedModpackOwner.value!.id}`,
								),
						}
					: undefined,
				categories: linkedModpackCategories.value,
				hasUpdate: linkedModpackHasUpdate.value,
				disabled: isModpackUpdating.value,
				disabledText: isModpackUpdating.value
					? formatMessage(commonMessages.updatingLabel)
					: formatMessage(commonMessages.installingLabel),
			}
		}

		if (localImportedModpackProject.value) {
			return {
				project: localImportedModpackProject.value,
				categories: [],
				hasUpdate: false,
				disabled: isModpackUpdating.value,
				disabledText: isModpackUpdating.value
					? formatMessage(commonMessages.updatingLabel)
					: formatMessage(commonMessages.installingLabel),
			}
		}

		return null
	}),
	isPackLocked,
	isBusy: isInstanceBusy,
	isBulkOperating,
	skipNonEssentialWarnings,
	contentTypeLabel: ref(formatMessage(messages.contentTypeProject)),
	toggleEnabled: toggleDisableDebounced,
	bulkEnableItems: (items: ContentItem[]) =>
		Promise.all(
			items.filter((item) => !item.enabled).map((item) => toggleDisableMod(item, true)),
		).then(() => {}),
	bulkDisableItems: (items: ContentItem[]) =>
		Promise.all(
			items.filter((item) => item.enabled).map((item) => toggleDisableMod(item, false)),
		).then(() => {}),
	deleteItem: removeMod,
	bulkDeleteItems: (items: ContentItem[]) =>
		Promise.all(items.map((item) => removeMod(item))).then(() => {}),
	getDeleteDependencyWarning,
	refresh: () => initProjects('must_revalidate'),
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	hasUpdateSupport: true,
	updateItem: handleUpdate,
	bulkUpdateAll: bulkUpdateAllProjects,
	bulkUpdateItem: updateProject,
	updateModpack: props.isServerInstance ? undefined : handleModpackUpdate,
	viewModpackContent: handleModpackContent,
	unlinkModpack: unpairInstance,
	openSettings: props.openSettings,
	switchVersion: handleSwitchVersion,
	getOverflowOptions,
	showContentHint,
	dismissContentHint,
	shareItems: handleShareItems,
	getItemId: getContentItemId,
	mapToTableItem: (item: ContentItem) => ({
		id: getContentItemId(item),
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.file_name.replace('.disabled', ''),
			icon_url: null,
		},
		projectLink: item.project?.id
			? { path: `/project/${item.project.id}`, query: { i: props.instance.id } }
			: undefined,
		version: item.version ?? {
			id: item.file_name,
			version_number: formatMessage(commonMessages.unknownLabel),
			file_name: item.file_name,
		},
		versionLink:
			item.project?.id && item.version?.id
				? {
						path: `/project/${item.project.id}/version/${item.version.id}`,
						query: { i: props.instance.id },
					}
				: undefined,
		owner: item.owner
			? {
					...item.owner,
					link: () => openUrl(`https://modrinth.com/${item.owner!.type}/${item.owner!.id}`),
				}
			: undefined,
		enabled: item.enabled,
		installing: item.installing,
	}),
	filterPersistKey: props.instance.id,
})

type UnlistenFn = () => void

const initialContentReady = loadInitialContent()
void initialContentReady.then(restoreModpackContentModalState).catch(handleError)

function getInstallRevision() {
	return installRevisionByInstance.value.get(props.instance.id) ?? 0
}

function loadInitialContent() {
	const installRevision = getInstallRevision()
	if (installRevision > handledInstallRevision.value) {
		handledInstallRevision.value = installRevision
		return initProjects('must_revalidate')
	}

	if (props.preloadedContent && applyContentData(props.preloadedContent)) {
		return Promise.resolve()
	}

	return initProjects()
}

async function restoreModpackContentModalState() {
	if (!savedModalState) return

	const stateToRestore = savedModalState
	savedModalState = null
	await nextTick()
	modpackContentModal.value?.restore(stateToRestore)
}

// Save modal state when navigating away so it can be restored on back
const removeBeforeEach = router.beforeEach(() => {
	const state = modpackContentModal.value?.getState()
	savedModalState = state ?? null
})

let isUnmounted = false
let unlistenDragDrop: UnlistenFn | null = null
let unlistenInstances: UnlistenFn | null = null

onMounted(() => {
	void getCurrentWebview()
		.onDragDropEvent(async (event) => {
			if (event.payload.type !== 'drop' || !props.instance) return

			for (const file of event.payload.paths) {
				if (file.endsWith('.mrpack')) continue
				await add_project_from_path(props.instance.id, file).catch(handleError)
			}
			await initProjects()
		})
		.then((unlisten) => {
			if (isUnmounted) {
				unlisten()
				return
			}

			unlistenDragDrop = unlisten
		})
		.catch(handleError)

	void instance_listener(async (event: { event: string; instance_id: string }) => {
		if (
			props.instance &&
			event.instance_id === props.instance.id &&
			event.event === 'synced' &&
			props.instance.install_stage === 'installed' &&
			!isBulkOperating.value
		) {
			await initProjects()
		}
	})
		.then((unlisten) => {
			if (isUnmounted) {
				unlisten()
				return
			}

			unlistenInstances = unlisten
		})
		.catch(handleError)
})

watch(
	() => props.instance?.install_stage,
	async (newStage, oldStage) => {
		if (oldStage !== 'installed' && newStage === 'installed') {
			await refreshContentState('must_revalidate')
		} else if (oldStage === 'not_installed' && newStage === 'pack_installing') {
			await initProjects()
		}
	},
)

watch(
	() => props.instance?.link,
	async (newInstanceLink, oldInstanceLink) => {
		if (oldInstanceLink && !newInstanceLink) {
			await initProjects('must_revalidate')
		}
	},
)

watch(
	() => props.instance?.update_channel,
	async (newValue, oldValue) => {
		if (newValue !== oldValue) {
			await initProjects('must_revalidate')
		}
	},
)

onUnmounted(() => {
	isUnmounted = true
	removeBeforeEach()
	unlistenDragDrop?.()
	unlistenInstances?.()
})
</script>
