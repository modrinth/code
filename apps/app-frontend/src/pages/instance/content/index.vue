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
				<ManagedContentModal
					ref="managedContentModal"
					:header="managedContentModalHeader"
					:source-name="managedContent?.card.manager.name"
					:source-icon-url="managedContent?.card.manager.iconUrl"
					:enable-toggle="!isServerInstance && !isSharedMember && !isQuarantined"
					:action-disabled="isBulkOperating || isInstanceBusy"
					:get-overflow-options="getOverflowOptions"
					:switch-version="
						isServerInstance || isSharedMember || isQuarantined ? undefined : handleSwitchVersion
					"
					@update:enabled="handleManagedContentToggle"
					@bulk:enable="(items) => handleManagedContentBulkToggle(items, true)"
					@bulk:disable="(items) => handleManagedContentBulkToggle(items, false)"
				/>
				<ConfirmDisableModal
					ref="sharedDisableConfirmModal"
					:count="pendingManagedContentDisableItems.length"
					:item-type="formatMessage(messages.contentTypeProject)"
					:warning="managedContentPolicy.disableWarning(pendingManagedContentDisableItems)"
					:action-disabled="isInstanceBusy"
					@disable="confirmPendingManagedContentDisable"
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
				<ExportModal
					v-if="projects.length > 0 && !instance.quarantined"
					ref="exportModal"
					:instance="instance"
				/>
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
import { ClipboardCopyIcon, FolderOpenIcon, LockIcon, LockOpenIcon } from '@modrinth/assets'
import {
	type BulkOperationStatus,
	type ButtonMenuOption,
	commonMessages,
	ConfirmDisableModal,
	ConfirmModpackUpdateModal,
	ContentCardLayout as ContentPageLayout,
	type ContentItem,
	type ContentOwner,
	ContentUpdaterModal,
	dedupeManagedContentItems,
	defineMessages,
	injectNotificationManager,
	type ManagedContentData,
	ManagedContentModal,
	type ManagedContentModalState,
	type ManagedContentProject,
	type ManagedContentVersion,
	provideContentManager,
	ReadyTransition,
	summarizeManagedContent,
	UnknownFileWarningModal,
	useDebugLogger,
	useVIntl,
	versionChangesGameVersion,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { useManagedContentPolicy } from '@/composables/instances/use-managed-content-policy'
import { useAppEvent } from '@/composables/use-app-event'
import { type FeatureFlag, useAppSettings } from '@/composables/use-app-settings.ts'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version, get_version_many } from '@/helpers/cache.js'
import {
	add_project_from_path,
	edit,
	get_linked_modpack_content,
	get_shared_instance_publish_preview,
	getInstanceIconUrl,
	is_file_on_modrinth,
	remove_project,
	set_project_locked,
	switch_project_version_with_dependencies,
	toggle_disable_project,
	update_all,
	update_managed_modrinth_version,
} from '@/helpers/instance'
import { type InstanceContentData, loadInstanceContentData } from '@/helpers/instance-content'
import { get as getSettings, set as setSettings } from '@/helpers/settings'
import type { CacheBehaviour } from '@/helpers/types'
import { highlightModInInstance } from '@/helpers/utils.js'
import { type AppEventPayload, injectAppEvents } from '@/providers/app-events'
import { injectContentInstall } from '@/providers/content-install'

import { injectInstancePage } from '../instance-context'
import { instanceContentQueryOptions, instanceKeys } from '../query-options'
import { injectSharedInstance } from '../shared-instance-context'

type InstanceBulkUpdateProgress = AppEventPayload<'instance_bulk_update_progress'>

const messages = defineMessages({
	modpackContentHeader: {
		id: 'app.instance.content.managed-content.modpack-header',
		defaultMessage: 'Modpack content',
	},
	sharedContentHeader: {
		id: 'app.instance.content.managed-content.shared-header',
		defaultMessage: 'Shared content',
	},
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
	lockedContent: {
		id: 'app.instance.mods.locked-content',
		defaultMessage: 'Content in locked instances cannot be changed.',
	},
	freezeContent: {
		id: 'app.instance.mods.freeze-content',
		defaultMessage: 'Freeze version',
	},
	unfreezeContent: {
		id: 'app.instance.mods.unfreeze-content',
		defaultMessage: 'Unfreeze version',
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

let savedModalState: ManagedContentModalState | null = null

function contentOwnerLink(owner: ContentOwner): NonNullable<ContentOwner['link']> {
	if (owner.type === 'user') return `/user/${encodeURIComponent(owner.id)}`
	return () => {
		void openUrl(`https://modrinth.com/organization/${owner.id}`)
	}
}

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const appEvents = injectAppEvents()
const { installingItems, installRevisionByInstance, installFailureRevisionByInstance } =
	injectContentInstall()
const router = useRouter()
const queryClient = useQueryClient()
const debug = useDebugLogger('Mods:ContentUpdate')
const appSettings = useAppSettings()
const skipUnknownFileWarningFeatureFlag = 'skip_unknown_pack_warning' as FeatureFlag
const skipNonEssentialWarnings = computed(() =>
	appSettings.getFeatureFlag('skip_non_essential_warnings'),
)

const instancePage = injectInstancePage()
const sharedInstanceState = injectSharedInstance()
const instance = instancePage.instance
const isServerInstance = instancePage.isServerInstance
const openSettings = () => instancePage.openSettings(1)
const managedContentPolicy = useManagedContentPolicy(computed(() => instance.value))
const {
	isManagedModpack: isSharedMember,
	isQuarantined,
	canMutateContent,
	canUpdateContent: canUpdateProject,
} = managedContentPolicy

const contentQuery = useQuery(
	computed(() => ({
		...instanceContentQueryOptions(instancePage.instanceId.value),
		enabled: !!instancePage.instanceId.value,
	})),
)
const loading = ref(contentQuery.data.value === undefined)
const projects = ref<ContentItem[]>([])

const installingBuffer = ref<ContentItem[]>([])
const handledInstallRevision = ref(0)

watch(
	() => installingItems.value.get(instance.value.id),
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
	const active = installingItems.value.get(instance.value.id)
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
	() => installFailureRevisionByInstance.value.get(instance.value.id) ?? 0,
	(revision, previousRevision) => {
		if (revision === previousRevision) return
		installingBuffer.value = []
	},
)

const linkedModpackProject = ref<ManagedContentProject | null>(null)
const linkedModpackVersion = ref<ManagedContentVersion | null>(null)
const linkedModpackUpdateVersionId = ref<string | null>(null)
const localImportedModpackUnlinked = ref(false)

const localImportedModpackProject = computed<ManagedContentProject | null>(() => {
	const link = instance.value.link
	if (localImportedModpackUnlinked.value || link?.type !== 'imported_modpack') return null

	return {
		id: link.filename ?? instance.value.id,
		slug: link.filename ?? instance.value.id,
		title: link.name ?? instance.value.name,
		icon_url: getInstanceIconUrl(instance.value.icon_path) ?? undefined,
		filename: link.filename ?? undefined,
	}
})

const displayedModpackProject = computed(
	() => linkedModpackProject.value ?? localImportedModpackProject.value,
)

watch(
	() => instance.value.link,
	() => {
		localImportedModpackUnlinked.value = false
	},
)

const isModpackUpdating = ref(false)
const isBulkOperating = ref(false)
const isInstanceBusy = computed(() => instance.value?.install_stage !== 'installed')
const showSharedContentFilter = computed(() => instance.value.shared_instance?.role === 'member')
const isPackLocked = computed(
	() =>
		instance.value.quarantined ||
		instance.value?.link?.type === 'modrinth_modpack' ||
		instance.value?.link?.type === 'server_project_modpack',
)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
const managedContentModal = ref<InstanceType<typeof ManagedContentModal> | null>()
const modpackUpdateConfirmModal = ref<InstanceType<typeof ConfirmModpackUpdateModal> | null>()
const sharedDisableConfirmModal = ref<InstanceType<typeof ConfirmDisableModal> | null>()
const pendingManagedContentDisableItems = ref<ContentItem[]>([])
const unknownFileWarningModal = ref<InstanceType<typeof UnknownFileWarningModal> | null>()
const unknownFileName = ref('')
let resolveUnknownFileConfirmation: ((confirmed: boolean) => void) | null = null

const modpackContentQueryKey = computed(() => instanceKeys.linkedContent(instance.value.id))
const modpackContentQuery = useQuery({
	queryKey: modpackContentQueryKey,
	queryFn: () => get_linked_modpack_content(instance.value.id),
	enabled: computed(
		() =>
			!!instance.value?.id &&
			!!instance.value?.link &&
			instance.value.install_stage === 'installed',
	),
})

const hasSharedManagedContent = computed(() => {
	if (instance.value.shared_instance?.role === 'owner') return false

	const linkType = instance.value.link?.type
	return (
		!!instance.value.shared_instance ||
		linkType === 'server_project' ||
		linkType === 'server_project_modpack'
	)
})

const managedContentItems = computed(() => {
	const linkedContent = modpackContentQuery.data.value ?? []
	const sourcedContent = hasSharedManagedContent.value
		? projects.value.filter((item) =>
				['server_project', 'shared_instance'].includes(item.source_kind ?? ''),
			)
		: []

	return dedupeManagedContentItems([...linkedContent, ...sourcedContent])
})

const managedContentSummary = computed(() =>
	modpackContentQuery.isLoading.value && modpackContentQuery.data.value === undefined
		? undefined
		: summarizeManagedContent(managedContentItems.value),
)

const managedContent = computed<ManagedContentData | null>(() => {
	const attachment = instance.value.shared_instance
	const sharedManager = sharedInstanceState.manager.value
	const linkedProject = instancePage.linkedProject.value
	const linkType = instance.value.link?.type
	const isSharedOwner = attachment?.role === 'owner'

	if (
		!isSharedOwner &&
		(attachment || linkType === 'server_project' || linkType === 'server_project_modpack')
	) {
		const serverManaged =
			sharedManager?.type === 'server' ||
			!!attachment?.server_manager_name ||
			linkType === 'server_project' ||
			linkType === 'server_project_modpack' ||
			(!attachment && isServerInstance.value)
		const managerName = serverManaged
			? (sharedManager?.name ??
				attachment?.server_manager_name ??
				linkedProject?.name ??
				instance.value.name)
			: (sharedManager?.name ?? instance.value.name)
		const managerIcon = serverManaged
			? (sharedManager?.avatarUrl ??
				attachment?.server_manager_icon_url ??
				linkedProject?.icon_url ??
				undefined)
			: (sharedManager?.avatarUrl ??
				(instance.value.icon_path ? convertFileSrc(instance.value.icon_path) : undefined))
		const managerLink = serverManaged
			? linkedProject
				? {
						path: `/project/${linkedProject.slug ?? linkedProject.id}`,
						query: { i: instancePage.instanceId.value },
					}
				: undefined
			: sharedManager?.type === 'user'
				? `/user/${encodeURIComponent(sharedManager.name)}`
				: undefined

		return {
			card: {
				kind: serverManaged ? 'server' : 'shared-instance',
				installing: isInstanceBusy.value,
				manager: {
					name: managerName,
					iconUrl: managerIcon,
					link: managerLink,
				},
				summary: managedContentSummary.value,
				syncedAt: sharedInstanceState.lastUpdateCheckAt.value,
				updateAvailable: instancePage.sharedInstanceUpdateAvailable.value,
			},
			disabled: attachment?.status === 'applying' || isInstanceBusy.value,
			disabledText: formatMessage(commonMessages.updatingLabel),
		}
	}

	const project = displayedModpackProject.value
	if (!project) return null

	return {
		card: {
			kind: 'modpack',
			installing: isInstanceBusy.value,
			manager: {
				name: project.title,
				iconUrl: project.icon_url ?? undefined,
				link: linkedModpackProject.value
					? {
							path: `/project/${project.slug ?? project.id}`,
							query: { i: instancePage.instanceId.value },
						}
					: undefined,
			},
			summary: managedContentSummary.value,
			versionNumber: linkedModpackVersion.value?.version_number,
			versionLink:
				linkedModpackProject.value && linkedModpackVersion.value
					? {
							path: `/project/${linkedModpackProject.value.slug ?? linkedModpackProject.value.id}/version/${linkedModpackVersion.value.id}`,
							query: { i: instancePage.instanceId.value },
						}
					: undefined,
			updatedAt: linkedModpackVersion.value?.date_published,
		},
		disabled: isModpackUpdating.value || isInstanceBusy.value,
		disabledText: formatMessage(commonMessages.updatingLabel),
	}
})

const managedContentModalHeader = computed(() =>
	formatMessage(
		managedContent.value?.card.kind === 'modpack'
			? messages.modpackContentHeader
			: messages.sharedContentHeader,
	),
)

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

function canDeleteContent(item: ContentItem) {
	return canMutateContent(item)
}

function canToggleContent(item: ContentItem) {
	return canMutateContent(item)
}

function canChangeContentVersion(item: ContentItem) {
	return canMutateContent(item) && !item.locked
}

async function reconcileSharedInstancePublishState() {
	if (instance.value.shared_instance?.role !== 'owner') return

	await get_shared_instance_publish_preview(instance.value.id).catch((error) => {
		debug('Failed to reconcile shared instance publish state', { error })
	})
}

function setContentItemBusy(item: ContentItem, busy: boolean, originalFileName = item.file_name) {
	item.installing = busy
	managedContentModal.value?.updateItem(originalFileName, {
		installing: busy,
		disabled: busy,
	})
	if (item.file_name !== originalFileName) {
		managedContentModal.value?.updateItem(item.file_name, {
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
	if (!instance.value || instance.value.quarantined) return
	await instancePage.browseContent(instance.value.loader === 'vanilla' ? 'resourcepack' : 'mod')
}

async function handleUploadFiles() {
	if (!instance.value || instance.value.quarantined) return
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

	const confirmedFiles: Array<{ path: string; filename: string }> = []
	for (const [index, { path, filename }] of selectedFiles.entries()) {
		if (!fileRecognition[index] && !(await confirmUnknownFileInstallation(filename))) {
			continue
		}
		confirmedFiles.push({ path, filename })
	}

	const addedFiles = (
		await Promise.all(
			confirmedFiles.map(async ({ path, filename }) => {
				try {
					const installedPath = await add_project_from_path(instance.value.id, path)
					return { filename, installedPath }
				} catch (error) {
					handleError(error as Error)
					return null
				}
			}),
		)
	).filter((result): result is { filename: string; installedPath: string } => result !== null)
	const uniqueAddedFiles = [
		...new Map(addedFiles.map((file) => [file.installedPath, file])).values(),
	]

	await initProjects('must_revalidate')

	if (uniqueAddedFiles.length > 0) {
		const names = uniqueAddedFiles.map(({ filename, installedPath }) => {
			const item = projects.value.find((project) => project.file_path === installedPath)
			return item?.project?.title ?? filename
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
	if (appSettings.getFeatureFlag(skipUnknownFileWarningFeatureFlag)) {
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
		appSettings.featureFlags[skipUnknownFileWarningFeatureFlag] = true
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

async function toggleDisableMod(
	mod: ContentItem,
	desiredEnabled?: boolean,
	reconcileSharedState = true,
) {
	if (!mod.file_path || !canToggleContent(mod)) return
	const operation = beginContentOperation(mod)
	if (!operation) return
	const originalFilePath = mod.file_path

	try {
		const newPath = await toggle_disable_project(instance.value.id, mod.file_path, desiredEnabled)
		const newFileName = fileNameFromPath(newPath)
		const enabled = !newPath.endsWith('.disabled')
		mod.file_path = newPath
		mod.file_name = newFileName
		mod.enabled = enabled
		managedContentModal.value?.updateItem(operation.originalFileName, {
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
			loader: instance.value.loader,
			game_version: instance.value.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
			disabled: !enabled,
		})

		if (reconcileSharedState) {
			await reconcileSharedInstancePublishState()
		}
	} catch (err) {
		handleError(err as Error)
	} finally {
		finishContentOperation(mod, operation)
	}
}

const toggleDisableDebounced = toggleDisableMod

async function removeMod(mod: ContentItem) {
	if (!mod.file_path || !canDeleteContent(mod)) return
	const operation = beginContentOperation(mod)
	if (!operation) return

	try {
		const removedPath = mod.file_path
		await remove_project(instance.value.id, removedPath)
		projects.value = projects.value.filter((x) => removedPath !== x.file_path)

		trackEvent('InstanceProjectRemove', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
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
	if (isServerInstance.value) return null

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
			unlisten = appEvents.on('instance_bulk_update_progress', (progress) => {
				if (progress.instanceId !== instance.value.id) return
				onProgress(formatBulkUpdateProgress(progress))
			})
		}

		await update_all(instance.value.id)
		await refreshContentState('must_revalidate')
	} catch (err) {
		handleError(err as Error)
		throw err
	} finally {
		unlisten?.()
	}
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
	if (!item || item.locked || !canUpdateProject(item) || !item.project?.id || !item.version?.id)
		return

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
	if (!canChangeContentVersion(item)) return
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

async function handleManagedContentToggle(item: ContentItem, enabled: boolean) {
	if (!enabled && managedContentPolicy.disableWarning([item])) {
		pendingManagedContentDisableItems.value = [item]
		sharedDisableConfirmModal.value?.show()
		return
	}

	await toggleDisableDebounced(item, enabled)
}

async function handleManagedContentBulkToggle(items: ContentItem[], enabled: boolean) {
	if (!enabled && managedContentPolicy.disableWarning(items)) {
		pendingManagedContentDisableItems.value = items
		sharedDisableConfirmModal.value?.show()
		return
	}

	await setManagedContentEnabled(items, enabled)
}

async function confirmPendingManagedContentDisable() {
	const items = [...pendingManagedContentDisableItems.value]
	pendingManagedContentDisableItems.value = []
	await setManagedContentEnabled(items, false)
}

async function setManagedContentEnabled(items: ContentItem[], enabled: boolean) {
	await Promise.all(items.map((item) => toggleDisableMod(item, enabled, false)))
	await reconcileSharedInstancePublishState()
}

async function handleManagedContent() {
	if (!instance.value?.id) return

	if (modpackContentQuery.data.value !== undefined) {
		managedContentModal.value?.show(managedContentItems.value)
		return
	}

	managedContentModal.value?.showLoading()

	const { data, error } = await modpackContentQuery.refetch()

	if (data !== undefined) {
		managedContentModal.value?.show(managedContentItems.value)
	} else {
		if (error) handleError(error)
		managedContentModal.value?.hide()
	}
}

async function refreshManagedContentItems(cacheBehaviour?: CacheBehaviour) {
	if (!instance.value?.id) return

	const contentItems = await queryClient
		.fetchQuery({
			queryKey: modpackContentQueryKey.value,
			queryFn: () => get_linked_modpack_content(instance.value.id, cacheBehaviour),
		})
		.catch(handleError)

	if (contentItems) {
		managedContentModal.value?.setItems(managedContentItems.value)
	}
}

async function refreshContentState(cacheBehaviour?: CacheBehaviour) {
	await initProjects(cacheBehaviour)
	await refreshManagedContentItems(cacheBehaviour)
}

watch(
	() => installRevisionByInstance.value.get(instance.value.id) ?? 0,
	async (revision) => {
		if (revision <= handledInstallRevision.value) return
		handledInstallRevision.value = revision
		await refreshContentState('must_revalidate')
	},
)

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

	const versions = await getUpdaterProjectVersions(instance.value.link.project_id, initialVersionId)

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

	contentUpdaterModal.value?.hide()
	isModpackUpdating.value = true
	try {
		await update_managed_modrinth_version(instance.value.id, version.id)
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
	await edit(instance.value.id, {
		link: null as unknown as undefined,
	})
	linkedModpackProject.value = null
	linkedModpackVersion.value = null
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

function getOverflowOptions(item: ContentItem): ButtonMenuOption[] {
	const options: ButtonMenuOption[] = []

	options.push({
		id: 'show-file',
		label: formatMessage(commonMessages.showFileButton),
		icon: FolderOpenIcon,
		action: () => highlightModInInstance(instance.value.id, item.file_path),
	})

	if (item.project?.slug) {
		options.push({
			id: 'copy-link',
			label: formatMessage(commonMessages.copyLinkButton),
			icon: ClipboardCopyIcon,
			action: async () => {
				await navigator.clipboard.writeText(
					`https://modrinth.com/${item.project_type}/${item.project?.slug}`,
				)
			},
		})
	}

	if (canMutateContent(item)) {
		options.push(
			{ type: 'divider' },
			{
				id: item.locked ? 'unfreeze-content' : 'freeze-content',
				label: formatMessage(item.locked ? messages.unfreezeContent : messages.freezeContent),
				icon: item.locked ? LockOpenIcon : LockIcon,
				action: () => handleContentFreeze(item, !item.locked),
			},
		)
	}

	return options
}

async function handleContentFreeze(item: ContentItem, frozen: boolean) {
	if (!item.file_path || !canMutateContent(item)) return
	const operation = beginContentOperation(item)
	if (!operation) return
	const originalFilePath = item.file_path

	try {
		await set_project_locked(instance.value.id, item.file_path, frozen)
		item.locked = frozen
		managedContentModal.value?.updateItem(operation.originalFileName, { locked: frozen })
		updateLinkedModpackContentCache(item, operation.originalFileName, originalFilePath, {
			locked: frozen,
		})
	} catch (err) {
		handleError(err as Error)
	} finally {
		finishContentOperation(item, operation)
	}
}

async function initProjects(cacheBehaviour?: CacheBehaviour, staleTime = 0) {
	if (!instance.value) return

	const contentData = await queryClient.fetchQuery({
		...instanceContentQueryOptions(instance.value.id),
		queryFn: () => loadInstanceContentData(instance.value.id, cacheBehaviour, handleError),
		staleTime,
	})
	applyContentData(contentData)
}

function applyContentData(contentData: InstanceContentData) {
	if (contentData.path !== instance.value.id) {
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
		linkedModpackUpdateVersionId.value = contentData.modpack.updateVersionId
	} else {
		linkedModpackProject.value = null
		linkedModpackVersion.value = null
		linkedModpackUpdateVersionId.value = null
	}

	loading.value = false
	return true
}

function contentVersionLabel(item: ContentItem): string {
	if (item.embedded_metadata?.version) return item.embedded_metadata.version
	return formatMessage(commonMessages.unknownLabel)
}

provideContentManager({
	items: mergedProjects,
	loading,
	error: ref(null),
	managedContent,
	isPackLocked,
	isBusy: isInstanceBusy,
	disableAddContent: isQuarantined,
	disableAddContentTooltip: formatMessage(messages.lockedContent),
	isBulkOperating,
	skipNonEssentialWarnings,
	contentTypeLabel: ref(formatMessage(messages.contentTypeProject)),
	toggleEnabled: toggleDisableDebounced,
	bulkEnableItems: async (items: ContentItem[]) => {
		await Promise.all(
			items
				.filter((item) => canToggleContent(item) && !item.enabled)
				.map((item) => toggleDisableMod(item, true, false)),
		)
		await reconcileSharedInstancePublishState()
	},
	bulkDisableItems: async (items: ContentItem[]) => {
		await Promise.all(
			items
				.filter((item) => canToggleContent(item) && item.enabled)
				.map((item) => toggleDisableMod(item, false, false)),
		)
		await reconcileSharedInstancePublishState()
	},
	deleteItem: removeMod,
	bulkDeleteItems: (items: ContentItem[]) =>
		Promise.all(items.filter(canDeleteContent).map((item) => removeMod(item))).then(() => {}),
	canDeleteItem: canDeleteContent,
	canToggleItem: canToggleContent,
	getDeleteWarning: managedContentPolicy.deleteWarning,
	getDisableWarning: managedContentPolicy.disableWarning,
	getDeleteDependencyWarning,
	refresh: () => initProjects('must_revalidate'),
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	hasUpdateSupport: true,
	updateItem: handleUpdate,
	bulkUpdateAll: bulkUpdateAllProjects,
	bulkUpdateItem: updateProject,
	runManagedContentPrimaryAction:
		instance.value.shared_instance?.role === 'member'
			? instancePage.reviewSharedInstanceUpdate
			: instance.value.link?.type === 'modrinth_modpack' && !isQuarantined.value
				? handleModpackUpdate
				: undefined,
	viewManagedContent: handleManagedContent,
	unlinkModpack: unpairInstance,
	openManagedContentSettings: openSettings,
	switchVersion: handleSwitchVersion,
	getOverflowOptions,
	shareItems: handleShareItems,
	getItemId: getContentItemId,
	mapToTableItem: (item: ContentItem) => ({
		id: getContentItemId(item),
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.embedded_metadata?.name ?? item.file_name.replace('.disabled', ''),
			icon_url: item.embedded_metadata?.icon_url ?? null,
		},
		projectLink: item.project?.id
			? { path: `/project/${item.project.id}`, query: { i: instancePage.instanceId.value } }
			: undefined,
		version: item.version ?? {
			id: item.file_name,
			version_number: contentVersionLabel(item),
			file_name: item.file_name,
		},
		versionLink:
			item.project?.id && item.version?.id
				? {
						path: `/project/${item.project.id}/version/${item.version.id}`,
						query: { i: instancePage.instanceId.value },
					}
				: undefined,
		owner: item.owner
			? {
					...item.owner,
					link: contentOwnerLink(item.owner),
				}
			: undefined,
		external: item.external ?? !item.project,
		enabled: canMutateContent(item) ? item.enabled : undefined,
		locked: item.locked,
		installing: item.installing,
		hideDelete: !canDeleteContent(item),
		hideSwitchVersion: !canChangeContentVersion(item) || !item.project?.id || !item.version?.id,
		hasUpdate: canUpdateProject(item) && !item.locked,
	}),
	showSharedContentFilter,
	filterPersistKey: instance.value.id,
})

type UnlistenFn = () => void

const initialContentReady = loadInitialContent()
void initialContentReady.then(restoreManagedContentModalState).catch(handleError)

function getInstallRevision() {
	return installRevisionByInstance.value.get(instance.value.id) ?? 0
}

function loadInitialContent() {
	const installRevision = getInstallRevision()
	if (installRevision > handledInstallRevision.value) {
		handledInstallRevision.value = installRevision
		return initProjects('must_revalidate')
	}

	return initProjects(undefined, 30_000)
}

watch(
	contentQuery.data,
	(data) => {
		if (data) applyContentData(data)
	},
	{ immediate: true },
)
watch(contentQuery.error, (error) => {
	if (error) {
		loading.value = false
		handleError(error)
	}
})

async function restoreManagedContentModalState() {
	if (!savedModalState) return

	const stateToRestore = savedModalState
	savedModalState = null
	await nextTick()
	managedContentModal.value?.restore(stateToRestore)
}

// Save modal state when navigating away so it can be restored on back
const removeBeforeEach = router.beforeEach(() => {
	const state = managedContentModal.value?.getState()
	savedModalState = state ?? null
})

let isUnmounted = false
let unlistenDragDrop: UnlistenFn | null = null

useAppEvent('instance', async (event) => {
	if (
		instance.value &&
		event.instance_id === instance.value.id &&
		event.event === 'synced' &&
		instance.value.install_stage === 'installed' &&
		!isBulkOperating.value
	) {
		await initProjects()
	}
})

onMounted(() => {
	void getCurrentWebview()
		.onDragDropEvent(async (event) => {
			if (event.payload.type !== 'drop' || !instance.value) return

			for (const file of event.payload.paths) {
				if (file.endsWith('.mrpack')) continue
				await add_project_from_path(instance.value.id, file).catch(handleError)
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
})

watch(
	() => instance.value?.install_stage,
	async (newStage, oldStage) => {
		if (oldStage !== 'installed' && newStage === 'installed') {
			await refreshContentState('must_revalidate')
		} else if (oldStage === 'not_installed' && newStage === 'pack_installing') {
			await initProjects()
		}
	},
)

watch(
	() => instance.value?.link,
	async (newInstanceLink, oldInstanceLink) => {
		if (oldInstanceLink && !newInstanceLink) {
			await initProjects('must_revalidate')
		}
	},
)

watch(
	() => instance.value?.update_channel,
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
})
</script>
