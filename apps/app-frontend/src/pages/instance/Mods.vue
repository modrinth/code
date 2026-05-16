<template>
	<ReadyTransition :pending="loading">
		<ContentPageLayout>
			<template #modals>
				<ShareModalWrapper
					ref="shareModal"
					:share-title="formatMessage(messages.shareTitle)"
					:share-text="formatMessage(messages.shareText)"
					:open-in-new-tab="false"
				/>
				<ModpackContentModal
					ref="modpackContentModal"
					:modpack-name="linkedModpackProject?.title"
					:modpack-icon-url="linkedModpackProject?.icon_url ?? undefined"
					:enable-toggle="!props.isServerInstance"
					:busy="isBulkOperating"
					:get-overflow-options="getOverflowOptions"
					:switch-version="handleSwitchVersion"
					@update:enabled="handleModpackContentToggle"
					@bulk:enable="handleModpackContentBulkToggle"
					@bulk:disable="handleModpackContentBulkToggle"
				/>
				<ConfirmModpackUpdateModal
					ref="modpackUpdateConfirmModal"
					:downgrade="isModpackUpdateDowngrade"
					:backup-tip="
						[linkedModpackProject?.title, pendingModpackUpdateVersion?.version_number]
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
							? (instance.linked_data?.version_id ?? '')
							: (updatingProject?.version?.id ?? '')
					"
					:is-app="true"
					:project-type="updatingModpack ? 'modpack' : updatingProject?.project_type"
					:project-icon-url="
						updatingModpack ? linkedModpackProject?.icon_url : updatingProject?.project?.icon_url
					"
					:project-name="
						updatingModpack
							? (linkedModpackProject?.title ?? formatMessage(commonMessages.modpackLabel))
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
	useDebugLogger,
	useVIntl,
	versionChangesGameVersion,
} from '@modrinth/ui'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import { type InstanceContentData, loadInstanceContentData } from '@/helpers/instance-content'
import {
	add_project_from_path,
	add_project_from_version,
	duplicate,
	edit,
	get,
	get_linked_modpack_content,
	list,
	remove_project,
	toggle_disable_project,
	update_managed_modrinth_version,
	update_project,
} from '@/helpers/profile'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import { highlightModInProfile } from '@/helpers/utils.js'
import { injectContentInstall } from '@/providers/content-install'
import { installVersionDependencies } from '@/store/install'

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
})

let savedModalState: ModpackContentModalState | null = null

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const { installingItems } = injectContentInstall()
const router = useRouter()
const debug = useDebugLogger('Mods:ContentUpdate')

const props = defineProps<{
	instance: GameInstance
	isServerInstance?: boolean
	openSettings?: () => void
	preloadedContent?: InstanceContentData | null
}>()

const loading = ref(true)
const projects = ref<ContentItem[]>([])

const installingBuffer = ref<ContentItem[]>([])

watch(
	() => installingItems.value.get(props.instance.path),
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
	const active = installingItems.value.get(props.instance.path)
	const pending = active ?? installingBuffer.value
	if (pending.length === 0) return projects.value
	const realProjectIds = new Set(projects.value.map((p) => p.project?.id).filter(Boolean))
	const placeholders = pending.filter((item) => !realProjectIds.has(item.project?.id))
	return placeholders.length > 0 ? [...projects.value, ...placeholders] : projects.value
})

const linkedModpackProject = ref<ContentModpackCardProject | null>(null)
const linkedModpackVersion = ref<ContentModpackCardVersion | null>(null)
const linkedModpackOwner = ref<ContentOwner | null>(null)
const linkedModpackCategories = ref<ContentModpackCardCategory[]>([])
const linkedModpackHasUpdate = ref(false)
const linkedModpackUpdateVersionId = ref<string | null>(null)

const isModpackUpdating = ref(false)
const isBulkOperating = ref(false)
const isInstanceBusy = computed(() => props.instance?.install_stage !== 'installed')
const isPackLocked = computed(() => props.instance?.linked_data?.locked ?? false)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
const modpackContentModal = ref<InstanceType<typeof ModpackContentModal> | null>()
const modpackUpdateConfirmModal = ref<InstanceType<typeof ConfirmModpackUpdateModal> | null>()

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

function getContentOperationKeys(item: ContentItem) {
	return [item.id, item.file_path, item.file_name, item.project?.id, item.version?.id].filter(
		(key): key is string => !!key,
	)
}

function hasContentOperation(item: ContentItem) {
	const keys = getContentOperationKeys(item)
	return keys.some((key) => activeContentOperationKeys.value.has(key))
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

async function handleBrowseContent() {
	if (!props.instance) return
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
		query: { i: props.instance.path },
	})
}

async function handleUploadFiles() {
	if (!props.instance) return
	const files = await open({ multiple: true })
	if (!files) return

	const addedFiles: string[] = []
	for (const file of files) {
		const path = (file as { path?: string }).path ?? file
		const fileName = typeof path === 'string' ? (path.split('/').pop() ?? path) : String(path)
		try {
			await add_project_from_path(props.instance.path, path)
			addedFiles.push(fileName)
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

async function toggleDisableMod(mod: ContentItem) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	try {
		const newPath = await toggle_disable_project(props.instance.path, mod.file_path)
		const newFileName = fileNameFromPath(newPath)
		mod.file_path = newPath
		mod.file_name = newFileName
		mod.enabled = !mod.enabled
		modpackContentModal.value?.updateItem(operation.originalFileName, {
			file_path: newPath,
			file_name: newFileName,
			enabled: mod.enabled,
		})

		trackEvent('InstanceProjectDisable', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
			disabled: !mod.enabled,
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
		await remove_project(props.instance.path, removedPath)
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

async function updateProject(mod: ContentItem) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	try {
		const updateVersionId = mod.update_version_id
		await update_project(props.instance.path, mod.file_path)

		if (updateVersionId) {
			const versionData = await get_version(updateVersionId, 'must_revalidate').catch(handleError)

			if (versionData) {
				const profile = await get(props.instance.path).catch(handleError)

				if (profile) {
					await installVersionDependencies(profile, versionData, 'update').catch(handleError)
				}
			}
		}

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

async function switchProjectVersion(mod: ContentItem, version: Labrinth.Versions.v2.Version) {
	if (!mod.file_path) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	const oldPath = mod.file_path
	const wasDisabled = mod.enabled === false || oldPath.endsWith('.disabled')
	let newPath: string | null = null
	let shouldRemoveNewOnError = false

	try {
		newPath = await add_project_from_version(props.instance.path, version.id, 'update')
		shouldRemoveNewOnError = newPath !== oldPath

		if (wasDisabled) {
			newPath = await toggle_disable_project(props.instance.path, newPath)
		}

		const profile = await get(props.instance.path).catch(handleError)
		if (profile) {
			await installVersionDependencies(profile, version, 'update').catch(handleError)
		}

		shouldRemoveNewOnError = false
		if (newPath !== oldPath) {
			await remove_project(props.instance.path, oldPath)
		}

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		if (shouldRemoveNewOnError && newPath && newPath !== oldPath) {
			await remove_project(props.instance.path, newPath).catch(() => {})
		}
		handleError(err as Error)
	} finally {
		await refreshContentState('must_revalidate')
		finishContentOperation(mod, operation)
	}
}

async function handleUpdate(id: string) {
	const item = projects.value.find((p) => p.id === id)
	if (!item?.has_update || !item.project?.id || !item.version?.id) return

	const requestId = beginUpdateRequest()

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
			path: props.instance.path,
			name: props.instance.name,
			gameVersion: props.instance.game_version,
			loader: props.instance.loader,
			linkedData: props.instance.linked_data,
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

	const versions = (await get_project_versions(item.project.id).catch((e) => {
		return handleError(e)
	})) as Labrinth.Versions.v2.Version[] | null

	if (!isActiveUpdateRequest(requestId) || updatingProject.value?.id !== item.id) return

	loadingVersions.value = false

	if (!versions) {
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

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)
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

	updatingModpack.value = false
	updatingProject.value = item
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.version.id, { switchMode: true })

	const versions = (await get_project_versions(item.project.id).catch((e) => {
		return handleError(e)
	})) as Labrinth.Versions.v2.Version[] | null

	if (!isActiveUpdateRequest(requestId) || updatingProject.value?.id !== item.id) return

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	updatingProjectVersions.value = versions
}

async function handleModpackContentToggle(item: ContentItem) {
	await toggleDisableDebounced(item)
}

async function handleModpackContentBulkToggle(items: ContentItem[]) {
	await Promise.all(items.map((item) => toggleDisableMod(item)))
}

async function handleModpackContent() {
	if (!props.instance?.path) return

	modpackContentModal.value?.showLoading()

	const contentItems = await get_linked_modpack_content(props.instance.path).catch(handleError)

	if (contentItems) {
		modpackContentModal.value?.show(contentItems)
	} else {
		modpackContentModal.value?.hide()
	}
}

async function refreshModpackContentItems(cacheBehaviour?: CacheBehaviour) {
	if (!props.instance?.path) return

	const contentItems = await get_linked_modpack_content(props.instance.path, cacheBehaviour).catch(
		handleError,
	)

	if (contentItems) {
		modpackContentModal.value?.setItems(contentItems)
	}
}

async function refreshContentState(cacheBehaviour?: CacheBehaviour) {
	await initProjects(cacheBehaviour)
	await refreshModpackContentItems(cacheBehaviour)
}

async function handleModpackUpdate() {
	if (!props.instance?.linked_data?.project_id) return

	const requestId = beginUpdateRequest()

	updatingModpack.value = true
	updatingProject.value = null
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	const initialVersionId =
		linkedModpackUpdateVersionId.value ?? props.instance?.linked_data?.version_id ?? undefined
	debug('handleModpackUpdate: opening modpack updater modal', {
		type: 'modpack',
		initialVersionId,
		linkedModpackUpdateVersionId: linkedModpackUpdateVersionId.value,
		linkedModpackProject: linkedModpackProject.value,
		linkedModpackVersion: linkedModpackVersion.value,
		linkedModpackHasUpdate: linkedModpackHasUpdate.value,
		instance: {
			path: props.instance.path,
			name: props.instance.name,
			gameVersion: props.instance.game_version,
			loader: props.instance.loader,
			linkedData: props.instance.linked_data,
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

	const versions = (await get_project_versions(props.instance.linked_data.project_id).catch(
		handleError,
	)) as Labrinth.Versions.v2.Version[] | null

	if (!isActiveUpdateRequest(requestId) || !updatingModpack.value) return

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)
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
		currentLinkedVersionId: props.instance.linked_data.version_id,
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
	const index = updatingProjectVersions.value.findIndex((v) => v.id === versionId)
	if (index !== -1) {
		const newVersions = [...updatingProjectVersions.value]
		newVersions[index] = fullVersion
		updatingProjectVersions.value = newVersions
	}
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

	const currentVersionId = props.instance?.linked_data?.version_id
	const currentVersion = updatingProjectVersions.value.find((v) => v.id === currentVersionId)
	isModpackUpdateDowngrade.value = currentVersion
		? new Date(selectedVersion.date_published) < new Date(currentVersion.date_published)
		: false
	const shouldShowWarning =
		isModpackUpdateDowngrade.value ||
		versionChangesGameVersion(selectedVersion, props.instance.game_version)

	if (!shouldShowWarning) {
		await handleModpackUpdateConfirm()
		return
	}

	modpackUpdateConfirmModal.value?.show()
}

async function handleModpackUpdateConfirm() {
	if (!pendingModpackUpdateVersion.value || !props.instance?.path) return

	const version = pendingModpackUpdateVersion.value
	pendingModpackUpdateVersion.value = null

	contentUpdaterModal.value?.hide()
	isModpackUpdating.value = true
	try {
		await update_managed_modrinth_version(props.instance.path, version.id)
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

		if (mod.has_update && mod.update_version_id === selectedVersion.id) {
			await updateProject(mod)
		} else {
			await switchProjectVersion(mod, selectedVersion)
		}

		resetUpdateState()
	}
}

async function unpairProfile() {
	await edit(props.instance.path, {
		linked_data: null as unknown as undefined,
	})
	linkedModpackProject.value = null
	linkedModpackVersion.value = null
	linkedModpackOwner.value = null
	linkedModpackHasUpdate.value = false
	linkedModpackUpdateVersionId.value = null
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
		action: () => highlightModInProfile(props.instance.path, item.file_path),
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

	const contentData = await loadInstanceContentData(
		props.instance.path,
		cacheBehaviour,
		handleError,
	)
	applyContentData(contentData)
}

function applyContentData(contentData: InstanceContentData) {
	if (contentData.path !== props.instance.path) return false

	if (!contentData.contentItems) {
		loading.value = false
		return true
	}

	projects.value = contentData.contentItems

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
		const allProfiles = await list()
		const prefix = `${props.instance.name} - Backup #`
		const existingNums = allProfiles
			.filter((p) => p.name.startsWith(prefix))
			.map((p) => parseInt(p.name.slice(prefix.length), 10))
			.filter((n) => !isNaN(n))
		const nextNum = existingNums.length > 0 ? Math.max(...existingNums) + 1 : 1
		const newPath = await duplicate(props.instance.path)
		await edit(newPath, { name: `${prefix}${nextNum}` })
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
	modpack: computed(() =>
		linkedModpackProject.value
			? {
					project: linkedModpackProject.value,
					projectLink: {
						path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}`,
						query: { i: props.instance.path },
					},
					version: linkedModpackVersion.value ?? undefined,
					versionLink:
						linkedModpackProject.value && linkedModpackVersion.value
							? {
									path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}/version/${linkedModpackVersion.value.id}`,
									query: { i: props.instance.path },
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
			: null,
	),
	isPackLocked,
	isBusy: isInstanceBusy,
	isBulkOperating,
	contentTypeLabel: ref(formatMessage(messages.contentTypeProject)),
	toggleEnabled: toggleDisableDebounced,
	bulkEnableItems: (items: ContentItem[]) =>
		Promise.all(items.filter((item) => !item.enabled).map((item) => toggleDisableMod(item))).then(
			() => {},
		),
	bulkDisableItems: (items: ContentItem[]) =>
		Promise.all(items.filter((item) => item.enabled).map((item) => toggleDisableMod(item))).then(
			() => {},
		),
	deleteItem: removeMod,
	bulkDeleteItems: (items: ContentItem[]) =>
		Promise.all(items.map((item) => removeMod(item))).then(() => {}),
	refresh: () => initProjects('must_revalidate'),
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	hasUpdateSupport: true,
	updateItem: handleUpdate,
	bulkUpdateItem: updateProject,
	updateModpack: props.isServerInstance ? undefined : handleModpackUpdate,
	viewModpackContent: handleModpackContent,
	unlinkModpack: unpairProfile,
	openSettings: props.openSettings,
	switchVersion: handleSwitchVersion,
	getOverflowOptions,
	showContentHint,
	dismissContentHint,
	shareItems: handleShareItems,
	mapToTableItem: (item: ContentItem) => ({
		id: item.id,
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.file_name.replace('.disabled', ''),
			icon_url: null,
		},
		projectLink: item.project?.id
			? { path: `/project/${item.project.id}`, query: { i: props.instance.path } }
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
						query: { i: props.instance.path },
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
	filterPersistKey: props.instance.path,
})

type UnlistenFn = () => void

const initialContentReady = loadInitialContent()
void initialContentReady.then(restoreModpackContentModalState).catch(handleError)

function loadInitialContent() {
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
let unlistenProfiles: UnlistenFn | null = null

onMounted(() => {
	void getCurrentWebview()
		.onDragDropEvent(async (event) => {
			if (event.payload.type !== 'drop' || !props.instance) return

			for (const file of event.payload.paths) {
				if (file.endsWith('.mrpack')) continue
				await add_project_from_path(props.instance.path, file).catch(handleError)
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

	void profile_listener(async (event: { event: string; profile_path_id: string }) => {
		if (
			props.instance &&
			event.profile_path_id === props.instance.path &&
			event.event === 'synced' &&
			props.instance.install_stage !== 'pack_installing' &&
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

			unlistenProfiles = unlisten
		})
		.catch(handleError)
})

watch(
	() => props.instance?.install_stage,
	async (newStage, oldStage) => {
		if (oldStage !== 'installed' && newStage === 'installed') {
			await initProjects('must_revalidate')
		} else if (oldStage === 'not_installed' && newStage === 'pack_installing') {
			await initProjects()
		}
	},
)

watch(
	() => props.instance?.linked_data,
	async (newLinkedData, oldLinkedData) => {
		if (oldLinkedData && !newLinkedData) {
			await initProjects('must_revalidate')
		}
	},
)

onUnmounted(() => {
	isUnmounted = true
	removeBeforeEach()
	unlistenDragDrop?.()
	unlistenProfiles?.()
})
</script>
