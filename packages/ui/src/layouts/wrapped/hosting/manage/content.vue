<script setup lang="ts">
import { type Archon, type Labrinth, ModrinthApiError } from '@modrinth/api-client'
import { ClipboardCopyIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import UnknownFileWarningModal from '#ui/components/modal/UnknownFileWarningModal.vue'
import { useUploadSessionUpload } from '#ui/composables/hosting/kyros-session-upload'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { waitForServerContextRuntimeReady } from '#ui/composables/server-context-runtime'
import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectServerSettingsModal,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'
import { versionChangesGameVersion } from '#ui/utils/version-compatibility'

import type { BrowseInstallPlan } from '../../../shared/browse-tab/composables/install-logic'
import {
	flushStoredServerAddonInstallQueue,
	getStoredServerAddonInstallQueue,
	getTargetInstallPreferences,
	resolveServerAddonInstallPlans,
} from '../../../shared/browse-tab/composables/install-logic'
import ManagedContentModal from '../../../shared/content-tab/components/managed-content-modal/index.vue'
import ConfirmModpackUpdateModal from '../../../shared/content-tab/components/modals/ConfirmModpackUpdateModal.vue'
import ConfirmUnlinkModal from '../../../shared/content-tab/components/modals/ConfirmUnlinkModal.vue'
import ContentUpdaterModal from '../../../shared/content-tab/components/modals/content-updater-modal/index.vue'
import ContentPageLayout from '../../../shared/content-tab/layout.vue'
import type { ManagedContentData } from '../../../shared/content-tab/providers/content-manager'
import { provideContentManager } from '../../../shared/content-tab/providers/content-manager'
import type { ContentItem } from '../../../shared/content-tab/types'
import { summarizeManagedContent } from '../../../shared/content-tab/utils/managed-content'

type AddonWithUiState = Archon.Content.v1.Addon & { installing?: boolean }
type ContentOwnerAvatarSource = {
	id: string
	name: string
	type: 'user' | 'organization'
}
const props = withDefaults(
	defineProps<{
		ownerAvatarUrlBase?: string
	}>(),
	{
		ownerAvatarUrlBase: 'https://modrinth.com',
	},
)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	modpackContent: {
		id: 'hosting.content.managed-content.modpack-header',
		defaultMessage: 'Modpack content',
	},
	failedToRemoveContent: {
		id: 'hosting.content.failed-to-remove',
		defaultMessage: 'Failed to remove content',
	},
	failedToToggle: {
		id: 'hosting.content.failed-to-toggle',
		defaultMessage: 'Failed to toggle {name}',
	},
	failedToUpload: {
		id: 'hosting.content.failed-to-upload',
		defaultMessage: 'Failed to upload file',
	},
	failedToUnlink: {
		id: 'hosting.content.failed-to-unlink',
		defaultMessage: 'Failed to unlink modpack',
	},
	failedToLoadModpackContent: {
		id: 'hosting.content.failed-to-load-modpack-content',
		defaultMessage: 'Failed to load modpack content',
	},
	failedToLoadVersions: {
		id: 'hosting.content.failed-to-load-versions',
		defaultMessage: 'Failed to load versions',
	},
	failedToUpdate: {
		id: 'hosting.content.failed-to-update',
		defaultMessage: 'Failed to update',
	},
	failedToBulkDelete: {
		id: 'hosting.content.failed-to-bulk-delete',
		defaultMessage: 'Failed to delete content',
	},
	failedToBulkEnable: {
		id: 'hosting.content.failed-to-bulk-enable',
		defaultMessage: 'Failed to enable content',
	},
	failedToBulkDisable: {
		id: 'hosting.content.failed-to-bulk-disable',
		defaultMessage: 'Failed to disable content',
	},
	failedToBulkUpdate: {
		id: 'hosting.content.failed-to-bulk-update',
		defaultMessage: 'Failed to update content',
	},
	failedToInstallContent: {
		id: 'hosting.content.failed-to-install',
		defaultMessage: 'Failed to install content',
	},
})

const client = injectModrinthClient()
const { server, worldId, busyReasons, installProgressItems, uploadState, cancelUpload } =
	injectModrinthServerContext()
const contentUploadSession = useUploadSessionUpload({
	client,
	scope: 'content',
	worldId,
	uploadState,
	cancelUpload,
})
const unknownFileWarningModal = ref<InstanceType<typeof UnknownFileWarningModal> | null>()
const unknownFileName = ref('')
let resolveUnknownFileConfirmation: ((confirmed: boolean) => void) | null = null
const skipUnknownFileWarningKey = 'hosting-skip-unknown-file-warning'
const { addNotification } = injectNotificationManager()
const { openServerSettings, browseServerContent } = injectServerSettingsModal()
const { canSetup, permissionDeniedMessage } = useServerPermissions()
const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()
const serverId = route.params.id as string

const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	if (loader === 'paper' || loader === 'purpur') return 'plugin'
	if (loader === 'vanilla') return 'datapack'
	return 'mod'
})

const queryKey = computed(() => ['content', 'list', 'v1', serverId])
const modpackContentQueryKey = computed(() => ['content', 'list', 'v1', serverId, 'modpack'])

function getContentOwnerAvatarUrl(owner: ContentOwnerAvatarSource) {
	const ownerId = owner.type === 'user' ? owner.name || owner.id : owner.id
	return `${props.ownerAvatarUrlBase}/${owner.type}/${encodeURIComponent(ownerId)}/avatar`
}

const contentQuery = useQuery({
	queryKey,
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
	enabled: computed(() => worldId.value !== null),
	staleTime: 30_000,
})

const isModpackContentModalOpen = ref(false)
const modpackContentQuery = useQuery({
	queryKey: modpackContentQueryKey,
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, {
			from_modpack: true,
		}),
	enabled: computed(() => worldId.value !== null && !!contentQuery.data.value?.modpack),
	staleTime: 30_000,
})

const setupActionDisabled = computed(() => !canSetup.value || busyReasons.value.length > 0)
const isInstallingContent = computed(
	() =>
		server.value?.status === 'installing' ||
		busyReasons.value.some(
			(r) =>
				r.reason.id === 'servers.busy.installing' || r.reason.id === 'servers.busy.syncing-content',
		),
)
const setupActionBusyMessage = computed(() => {
	if (!canSetup.value) return permissionDeniedMessage.value

	const filteredReasons = busyReasons.value.filter((r) => {
		if (
			isInstallingContent.value &&
			(r.reason.id === 'servers.busy.installing' || r.reason.id === 'servers.busy.syncing-content')
		)
			return false
		if (
			r.reason.id === 'servers.busy.backup-creating' ||
			r.reason.id === 'servers.busy.backup-restoring'
		)
			return false
		return true
	})
	return filteredReasons.length > 0 ? formatMessage(filteredReasons[0].reason) : null
})

const currentWorldInstallProgressItems = computed(() =>
	installProgressItems.value.filter((item) => item.world_id === worldId.value),
)
const contentActionDisabled = computed(() => !canSetup.value || busyReasons.value.length > 0)
const contentActionBusyMessage = computed(() => {
	if (!canSetup.value) return permissionDeniedMessage.value
	return busyReasons.value.length > 0 ? formatMessage(busyReasons.value[0].reason) : null
})

const modpackProjectId = computed(() => {
	const spec = contentQuery.data.value?.modpack?.spec
	return spec?.platform === 'modrinth' ? spec.project_id : null
})

const modpackVersionsQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'versions', 'v2', modpackProjectId.value]),
	queryFn: () =>
		client.labrinth.versions_v2.getProjectVersions(modpackProjectId.value!, {
			include_changelog: false,
		}),
	enabled: computed(() => !!modpackProjectId.value),
})

const projectQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'project', modpackProjectId.value]),
	queryFn: () => client.labrinth.projects_v2.get(modpackProjectId.value!),
	enabled: computed(() => !!modpackProjectId.value),
})

function getVersionTime(version: Labrinth.Versions.v2.Version) {
	return new Date(version.date_published).getTime()
}

function sortVersionsByPublishedDate(versions: Labrinth.Versions.v2.Version[]) {
	return [...versions].sort((a, b) => getVersionTime(b) - getVersionTime(a))
}

const currentModpackVersionId = computed(() => {
	const spec = contentQuery.data.value?.modpack?.spec
	return spec?.platform === 'modrinth' ? spec.version_id : null
})

const newestModpackUpdateVersion = computed(() => {
	const currentVersionId = currentModpackVersionId.value
	if (!currentVersionId) return null

	const versions = sortVersionsByPublishedDate(modpackVersionsQuery.data.value ?? [])
	const currentVersion = versions.find((version) => version.id === currentVersionId)
	const installedPublishedAt = contentQuery.data.value?.modpack?.date_published
	const storedCurrentTime = installedPublishedAt
		? new Date(installedPublishedAt).getTime()
		: Number.NaN
	const currentVersionTime = Number.isNaN(storedCurrentTime)
		? currentVersion
			? getVersionTime(currentVersion)
			: Number.NaN
		: storedCurrentTime

	return (
		versions.find((version) => {
			if (version.id === currentVersionId) return false
			if (Number.isNaN(currentVersionTime)) return true
			return getVersionTime(version) > currentVersionTime
		}) ?? null
	)
})

const managedContent = computed<ManagedContentData | null>(() => {
	const mp = contentQuery.data.value?.modpack
	if (!mp) return null
	const isLocal = mp.spec.platform === 'local_file'
	const project = projectQuery.data.value
	const projectId = isLocal ? null : mp.spec.project_id
	const addons = modpackContentQuery.data.value?.addons
	const summary = addons
		? summarizeManagedContent(addons.map(addonToContentItem))
		: modpackContentQuery.isLoading.value
			? undefined
			: []
	const title = isLocal
		? (mp.title ?? mp.spec.name)
		: (project?.title ?? mp.title ?? projectId ?? '')
	return {
		card: {
			kind: 'modpack',
			installing: isInstallingContent.value,
			manager: {
				name: title,
				iconUrl: (isLocal ? mp.icon_url : (project?.icon_url ?? mp.icon_url)) ?? undefined,
				link: projectId ? `/project/${project?.slug ?? projectId}` : undefined,
			},
			summary,
			versionNumber: isLocal ? undefined : (mp.version_number ?? undefined),
			versionLink:
				projectId && mp.spec.platform === 'modrinth'
					? `/project/${project?.slug ?? projectId}/version/${mp.spec.version_id}`
					: undefined,
			updatedAt: isLocal ? undefined : (mp.date_published ?? undefined),
		},
		disabled: setupActionDisabled.value,
		disabledText: setupActionBusyMessage.value ?? formatMessage(commonMessages.installingLabel),
	}
})

function friendlyAddonName(addon: Archon.Content.v1.Addon): string {
	if (addon.name) return addon.name
	let cleanName = addon.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

const modpackAddons = ref<Archon.Content.v1.Addon[]>([])

const addonLookup = computed(() => {
	const map = new Map<string, Archon.Content.v1.Addon>()
	for (const addon of contentQuery.data.value?.addons ?? []) {
		map.set(addon.filename, addon)
	}
	for (const addon of modpackAddons.value) {
		map.set(addon.filename, addon)
	}
	return map
})

const projectMetadataBatchSize = 800
const contentProjectIds = computed(() =>
	[...(contentQuery.data.value?.addons ?? []), ...modpackAddons.value]
		.map((addon) => addon.project_id)
		.filter((id): id is string => !!id)
		.filter((id, index, ids) => ids.indexOf(id) === index)
		.sort(),
)
const contentProjectsQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'projects', 'v2', contentProjectIds.value]),
	queryFn: async () => {
		const batches = []
		for (let index = 0; index < contentProjectIds.value.length; index += projectMetadataBatchSize) {
			batches.push(contentProjectIds.value.slice(index, index + projectMetadataBatchSize))
		}
		return (
			await Promise.all(batches.map((ids) => client.labrinth.projects_v2.getMultiple(ids)))
		).flat()
	},
	enabled: computed(() => contentProjectIds.value.length > 0),
})
const contentProjectsById = computed(
	() => new Map((contentProjectsQuery.data.value ?? []).map((project) => [project.id, project])),
)

function normalizeInstallFilename(filename: string) {
	const normalized = filename.endsWith('.disabled')
		? filename.slice(0, -'.disabled'.length)
		: filename
	return normalized.toLowerCase()
}

type FileInstallProgressItem = Archon.Websocket.v0.InstallProgressItem & {
	key: Archon.Websocket.v0.InstallProgressFileKey
}

const fileInstallProgressItems = computed<FileInstallProgressItem[]>(() =>
	currentWorldInstallProgressItems.value.filter(
		(item): item is FileInstallProgressItem => item.key.type === 'file',
	),
)

function getFileInstallFilenames(key: Archon.Websocket.v0.InstallProgressFileKey) {
	return [key.source_filename, key.target_filename]
		.filter((filename): filename is string => !!filename)
		.map(normalizeInstallFilename)
}

function isFileInstallActive(item: FileInstallProgressItem) {
	return item.error == null && item.progress !== 100
}

function getContentItemInstallFilename(item: ContentItem) {
	const filename = item.version?.file_name || item.file_name
	return normalizeInstallFilename(filename)
}

function getContentItemInstallProgress(item: ContentItem): FileInstallProgressItem | undefined {
	const projectId = item.project?.id
	const versionId = item.version?.id
	const filename = getContentItemInstallFilename(item)

	return fileInstallProgressItems.value.find((progressItem) => {
		const key = progressItem.key
		if (key.project_id === projectId) return true
		if (key.version_id === versionId) return true
		return getFileInstallFilenames(key).includes(filename)
	})
}

function decorateContentItemWithInstallProgress(
	contentItem: ContentItem,
	installProgress: FileInstallProgressItem,
): ContentItem {
	return {
		...contentItem,
		installProgress: isFileInstallActive(installProgress) ? installProgress.progress : undefined,
	}
}

const isFlushingStoredServerInstalls = ref(false)

function getInstalledProjectIds() {
	return new Set(
		(contentQuery.data.value?.addons ?? [])
			.map((addon) => addon.project_id)
			.filter((projectId): projectId is string => !!projectId),
	)
}

function toResolvePreferences(
	preferences?: BrowseInstallPlan['preferences'],
): Labrinth.Content.v3.ResolutionPreferences {
	return {
		game_versions: preferences?.gameVersions,
		loaders: preferences?.loaders,
	}
}

async function resolveStoredServerAddonPlans(plans: BrowseInstallPlan[]) {
	return await resolveServerAddonInstallPlans({
		plans,
		existingProjectIds: getInstalledProjectIds(),
		resolvePlan: async (plan, existingProjectIds) => {
			const target = getTargetInstallPreferences(
				{
					gameVersion: server.value?.mc_version,
					loader: server.value?.loader,
				},
				plan.contentType,
			)
			const resolved = await client.labrinth.content_v3.resolve({
				project_id: plan.projectId,
				version_id: plan.versionId,
				content_type: plan.contentType as Labrinth.Content.v3.ContentType,
				selected: toResolvePreferences(plan.preferences),
				target: toResolvePreferences(target),
				existing_project_ids: existingProjectIds,
			})

			return [resolved.primary, ...resolved.dependencies].map((item) => ({
				projectId: item.project_id,
				versionId: item.version_id,
			}))
		},
	})
}

async function flushStoredServerInstalls() {
	const wid = worldId.value
	if (!wid || isFlushingStoredServerInstalls.value) return

	const queuedPlans = getStoredServerAddonInstallQueue(serverId, wid)
	if (queuedPlans.size === 0) return

	try {
		await waitForServerContextRuntimeReady(client, serverId)
	} catch (error) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToInstallContent),
			text: error instanceof Error ? error.message : undefined,
		})
		return
	}

	isFlushingStoredServerInstalls.value = true
	try {
		const result = await flushStoredServerAddonInstallQueue({
			serverId,
			worldId: wid,
			install: async (plans) => {
				const addons = await resolveStoredServerAddonPlans(plans)
				if (addons.length > 0) {
					await client.archon.content_v1.addAddons(serverId, wid, addons)
				}
			},
		})

		if (!result.ok) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.failedToInstallContent),
				text: result.error instanceof Error ? result.error.message : undefined,
			})
			return
		}

		if (result.flushedPlans.length > 0) {
			await queryClient.invalidateQueries({ queryKey: queryKey.value })
		}
	} finally {
		isFlushingStoredServerInstalls.value = false
	}
}

const contentItems = computed<ContentItem[]>(() =>
	(contentQuery.data.value?.addons ?? []).map((addon) => {
		const contentItem = addonToContentItem(addon)
		if (!contentItem.installing) return contentItem

		const installProgress = getContentItemInstallProgress(contentItem)
		return installProgress
			? decorateContentItemWithInstallProgress(contentItem, installProgress)
			: contentItem
	}),
)
const contentReadyPending = computed(
	() =>
		contentQuery.isLoading.value &&
		contentQuery.data.value === undefined &&
		contentItems.value.length === 0,
)

function getContentItemId(item: ContentItem) {
	return item.file_name ?? item.id
}

watch(
	worldId,
	() => {
		void flushStoredServerInstalls()
	},
	{ immediate: true },
)

const deleteMutation = useMutation({
	mutationFn: ({ addon }: { addon: Archon.Content.v1.Addon }) =>
		client.archon.content_v1.deleteAddon(serverId, worldId.value!, {
			filename: addon.filename,
			kind: addon.kind,
		}),
	onMutate: async ({ addon }) => {
		await queryClient.cancelQueries({ queryKey: queryKey.value })
		const previousData = queryClient.getQueryData<Archon.Content.v1.Addons>(queryKey.value)
		queryClient.setQueryData(queryKey.value, (oldData: Archon.Content.v1.Addons | undefined) => {
			if (!oldData) return oldData
			return {
				...oldData,
				addons: (oldData.addons ?? []).filter((a) => a.filename !== addon.filename),
			}
		})
		return { previousData }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: queryKey.value })
	},
	onError: (err, _vars, context) => {
		if (context?.previousData) {
			queryClient.setQueryData(queryKey.value, context.previousData)
		}
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToRemoveContent),
			text: err instanceof Error ? err.message : undefined,
		})
	},
})

const toggleMutation = useMutation({
	mutationFn: async ({ addon }: { addon: Archon.Content.v1.Addon }) => {
		const request: Archon.Content.v1.RemoveAddonRequest = {
			filename: addon.filename,
			kind: addon.kind,
		}
		if (addon.disabled) {
			await client.archon.content_v1.enableAddon(serverId, worldId.value!, request)
		} else {
			await client.archon.content_v1.disableAddon(serverId, worldId.value!, request)
		}
		return { filename: addon.filename, newDisabled: !addon.disabled }
	},
	onSuccess: ({ filename, newDisabled }) => {
		queryClient.setQueryData(queryKey.value, (oldData: Archon.Content.v1.Addons | undefined) => {
			if (!oldData) return oldData
			return {
				...oldData,
				addons: (oldData.addons ?? []).map((a) =>
					a.filename === filename ? { ...a, disabled: newDisabled } : a,
				),
			}
		})
		queryClient.invalidateQueries({ queryKey: queryKey.value })
	},
	onError: (_err, { addon }) => {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToToggle, { name: friendlyAddonName(addon) }),
		})
	},
})

async function handleToggleEnabled(item: ContentItem) {
	if (contentActionDisabled.value) return
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	await toggleMutation.mutateAsync({ addon })
}

async function handleDeleteItem(item: ContentItem) {
	if (contentActionDisabled.value) return
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	await deleteMutation.mutateAsync({ addon })
}

function itemsToAddonRequests(items: ContentItem[]): Archon.Content.v1.RemoveAddonRequest[] {
	return items.flatMap((item) => {
		if (item.installing) return []
		const addon = addonLookup.value.get(item.file_name)
		if (!addon) return []
		return [{ filename: addon.filename, kind: addon.kind }]
	})
}

async function handleBulkDelete(items: ContentItem[]) {
	if (contentActionDisabled.value) return
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	try {
		await client.archon.content_v1.deleteAddons(serverId, worldId.value!, requests)
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	} catch (err) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToBulkDelete),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleBulkEnable(items: ContentItem[]) {
	if (contentActionDisabled.value) return
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	try {
		await client.archon.content_v1.enableAddons(serverId, worldId.value!, requests)
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	} catch (err) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToBulkEnable),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleBulkDisable(items: ContentItem[]) {
	if (contentActionDisabled.value) return
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	try {
		await client.archon.content_v1.disableAddons(serverId, worldId.value!, requests)
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	} catch (err) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToBulkDisable),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

const modpackUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const modpackContentModal = ref<InstanceType<typeof ManagedContentModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal>>()

const updatingProject = ref<ContentItem | null>(null)
const updatingModpack = ref(false)
const loadingChangelog = ref(false)

watch(
	() => modpackContentQuery.data.value?.addons,
	(addons) => {
		if (!isModpackContentModalOpen.value || !addons) return
		modpackAddons.value = addons
		modpackContentModal.value?.setItems(addons.map(addonToContentItem))
	},
)

const updatingProjectId = computed(() => updatingProject.value?.project?.id ?? null)

const projectVersionsQuery = useQuery({
	queryKey: computed(() => ['labrinth', 'versions', 'v2', updatingProjectId.value]),
	queryFn: () =>
		client.labrinth.versions_v2.getProjectVersions(updatingProjectId.value!, {
			include_changelog: false,
		}),
	enabled: computed(() => !!updatingProjectId.value && !updatingModpack.value),
})

const updatingProjectVersions = computed(() => {
	const source = updatingModpack.value
		? modpackVersionsQuery.data.value
		: projectVersionsQuery.data.value
	if (!source) return []
	return sortVersionsByPublishedDate(source)
})

const loadingVersions = computed(() =>
	updatingModpack.value
		? modpackVersionsQuery.isLoading.value
		: projectVersionsQuery.isLoading.value,
)

const modpackUpdateModal = ref<InstanceType<typeof ConfirmModpackUpdateModal>>()
const pendingModpackUpdateVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const isModpackUpdateDowngrade = ref(false)

const currentGameVersion = computed(
	() => contentQuery.data.value?.game_version ?? server.value?.mc_version ?? '',
)
const currentLoader = computed(
	() => contentQuery.data.value?.modloader ?? server.value?.loader ?? '',
)

function handleBrowseContent() {
	if (contentActionDisabled.value) return
	const contentType = type.value
	if (browseServerContent && ['mod', 'plugin', 'datapack'].includes(contentType)) {
		browseServerContent({
			serverId,
			worldId: worldId.value,
			type: contentType as 'mod' | 'plugin' | 'datapack',
		})
		return
	}

	router.push({
		path: `/discover/${type.value}s`,
		query: { sid: serverId, wid: worldId.value },
	})
}

function handleUploadFiles() {
	if (contentActionDisabled.value) return
	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.accept = type.value === 'datapack' ? '.zip' : '.jar'
	input.onchange = async () => {
		if (!input.files) return
		const files = Array.from(input.files)
		const wid = worldId.value
		if (!wid) return

		try {
			const fileRecognition = await Promise.all(files.map(isFileOnModrinth))
			const unrecognizedFileSet = new Set(files.filter((_, index) => !fileRecognition[index]))
			const confirmedFiles: File[] = []
			for (const file of files) {
				if (!unrecognizedFileSet.has(file) || (await confirmUnknownFileInstallation(file.name))) {
					confirmedFiles.push(file)
				}
			}
			if (confirmedFiles.length === 0) return

			const result = await contentUploadSession.uploadFiles(
				confirmedFiles.map((file) => ({ file, filename: file.name })),
			)
			if (result === 'completed') await contentQuery.refetch()
		} catch (err) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.failedToUpload),
				text: err instanceof Error ? err.message : undefined,
			})
		}
	}
	input.click()
}

async function isFileOnModrinth(file: File) {
	const buffer = await file.arrayBuffer()
	const digest = await crypto.subtle.digest('SHA-1', buffer)
	const hash = Array.from(new Uint8Array(digest), (byte) =>
		byte.toString(16).padStart(2, '0'),
	).join('')

	try {
		await client.labrinth.versions_v2.getVersionFromFileHash(hash, 'sha1')
		return true
	} catch (error) {
		return !(error instanceof ModrinthApiError && error.statusCode === 404)
	}
}

function confirmUnknownFileInstallation(fileName: string) {
	if (localStorage.getItem(skipUnknownFileWarningKey) === 'true') {
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

function handleUnknownFileContinue(dontShowAgain: boolean) {
	if (dontShowAgain) localStorage.setItem(skipUnknownFileWarningKey, 'true')
	resolveUnknownFileWarning(true)
}

function addonToContentItem(addon: AddonWithUiState): ContentItem {
	const projectMetadata = addon.project_id
		? contentProjectsById.value.get(addon.project_id)
		: undefined
	return {
		project: {
			...(projectMetadata ?? {}),
			id: addon.project_id ?? addon.filename,
			slug: projectMetadata?.slug ?? addon.project_id ?? addon.filename,
			title: projectMetadata?.title ?? friendlyAddonName(addon),
			icon_url: addon.icon_url ?? projectMetadata?.icon_url ?? undefined,
		},
		version: {
			id: addon.version?.id ?? addon.filename,
			version_number: addon.version?.name ?? formatMessage(commonMessages.unknownLabel),
			file_name: addon.filename,
		},
		owner: addon.owner
			? {
					id: addon.owner.id,
					name: addon.owner.name,
					type: addon.owner.type,
					avatar_url: getContentOwnerAvatarUrl(addon.owner),
					link: `/${addon.owner.type}/${addon.owner.id}`,
				}
			: undefined,
		id: addon.id ?? addon.filename,
		enabled: !addon.disabled,
		file_name: addon.filename,
		date_added: addon.btime,
		project_type: addon.kind,
		has_update: !!addon.has_update,
		update_version_id: addon.has_update,
		environment: addon.version?.environment ?? undefined,
		pack_client_retained: addon.pack_client_retained,
		pack_client_depends: addon.pack_client_depends,
		installing: addon.installing ?? addon.status === 'pending',
	}
}

async function handleViewModpackContent() {
	isModpackContentModalOpen.value = true

	if (modpackContentQuery.data.value) {
		modpackAddons.value = modpackContentQuery.data.value.addons ?? []
		modpackContentModal.value?.show(modpackAddons.value.map(addonToContentItem))
		void modpackContentQuery.refetch()
		return
	}

	modpackContentModal.value?.showLoading()
	try {
		const { data } = await modpackContentQuery.refetch()
		if (!data) throw new Error('Failed to load modpack content')
		modpackAddons.value = data.addons ?? []
		const items = (data.addons ?? []).map(addonToContentItem)
		modpackContentModal.value?.show(items)
	} catch (err) {
		isModpackContentModalOpen.value = false
		modpackContentModal.value?.hide()
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToLoadModpackContent),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleModpackContentToggle(item: ContentItem) {
	if (contentActionDisabled.value) return
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	modpackContentModal.value?.updateItem(item.file_name, { disabled: true })
	try {
		await toggleMutation.mutateAsync({ addon })
		modpackAddons.value = modpackAddons.value.map((a) =>
			a.filename === addon.filename ? { ...a, disabled: !addon.disabled } : a,
		)
		queryClient.setQueryData(
			modpackContentQueryKey.value,
			(oldData: Archon.Content.v1.Addons | undefined) =>
				oldData
					? {
							...oldData,
							addons: (oldData.addons ?? []).map((a) =>
								a.filename === addon.filename ? { ...a, disabled: !addon.disabled } : a,
							),
						}
					: oldData,
		)
		modpackContentModal.value?.updateItem(item.file_name, {
			enabled: !item.enabled,
			disabled: false,
		})
	} catch {
		modpackContentModal.value?.updateItem(item.file_name, { disabled: false })
	}
}

async function handleModpackBulkToggle(items: ContentItem[], enable: boolean) {
	if (contentActionDisabled.value) return
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return

	// Optimistic update
	for (const item of items) {
		modpackAddons.value = modpackAddons.value.map((a) =>
			a.filename === item.file_name ? { ...a, disabled: !enable } : a,
		)
		modpackContentModal.value?.updateItem(item.file_name, { enabled: enable })
	}

	try {
		if (enable) {
			await client.archon.content_v1.enableAddons(serverId, worldId.value!, requests)
		} else {
			await client.archon.content_v1.disableAddons(serverId, worldId.value!, requests)
		}
		queryClient.setQueryData(
			modpackContentQueryKey.value,
			(oldData: Archon.Content.v1.Addons | undefined) =>
				oldData
					? {
							...oldData,
							addons: (oldData.addons ?? []).map((addon) =>
								items.some((item) => item.file_name === addon.filename)
									? { ...addon, disabled: !enable }
									: addon,
							),
						}
					: oldData,
		)
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	} catch (err) {
		for (const item of items) {
			modpackAddons.value = modpackAddons.value.map((a) =>
				a.filename === item.file_name ? { ...a, disabled: enable } : a,
			)
			modpackContentModal.value?.updateItem(item.file_name, { enabled: !enable })
		}
		addNotification({
			type: 'error',
			title: formatMessage(enable ? messages.failedToBulkEnable : messages.failedToBulkDisable),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

function handleModpackUnlink() {
	modpackUnlinkModal.value?.show()
}

async function handleModpackUnlinkConfirm() {
	if (setupActionDisabled.value) return
	try {
		await client.archon.content_v1.unlinkModpack(serverId, worldId.value!)
		await contentQuery.refetch()
	} catch (err) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToUnlink),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleBulkUpdate(items: ContentItem[]) {
	if (contentActionDisabled.value) return
	const addons = items
		.filter((item) => item.has_update && !item.installing)
		.map((item) => ({
			filename: item.file_name,
			version_id: item.update_version_id ?? undefined,
		}))
	if (addons.length === 0) return
	try {
		await client.archon.content_v1.updateAddons(serverId, worldId.value!, addons)
		await queryClient.invalidateQueries({ queryKey: queryKey.value })
	} catch (err) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToBulkUpdate),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleUpdateItem(id: string) {
	const item = contentItems.value.find((i) => getContentItemId(i) === id)
	if (!item?.has_update || !item.project?.id || !item.version?.id) return

	updatingModpack.value = false
	updatingProject.value = item
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.update_version_id ?? undefined)
}

async function handleSwitchVersion(item: ContentItem) {
	if (!item.project?.id || !item.version?.id) return

	updatingModpack.value = false
	updatingProject.value = item
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.version.id, { switchMode: true })
}

async function handleModpackUpdate() {
	if (setupActionDisabled.value) return
	const mp = contentQuery.data.value?.modpack
	if (!mp || mp.spec.platform !== 'modrinth') return

	updatingModpack.value = true
	updatingProject.value = null
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(
		newestModpackUpdateVersion.value?.id ?? mp.has_update ?? undefined,
	)
}

function spliceVersionInCache(fullVersion: Labrinth.Versions.v2.Version) {
	const projectId = updatingModpack.value ? modpackProjectId.value : updatingProjectId.value
	if (!projectId) return
	const key = ['labrinth', 'versions', 'v2', projectId]
	queryClient.setQueryData(key, (old: Labrinth.Versions.v2.Version[] | undefined) => {
		if (!old) return old
		return old.map((v) => (v.id === fullVersion.id ? fullVersion : v))
	})
}

async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (version.changelog) return
	loadingChangelog.value = true
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		spliceVersionInCache(fullVersion)
	} catch {
		// Silently fail on changelog fetch
	} finally {
		loadingChangelog.value = false
	}
}

async function handleVersionHover(version: Labrinth.Versions.v2.Version) {
	if (version.changelog) return
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		spliceVersionInCache(fullVersion)
	} catch {
		// Silently fail on hover prefetch
	}
}

function resetUpdateState() {
	updatingModpack.value = false
	updatingProject.value = null
	loadingChangelog.value = false
}

function handleModalUpdate(selectedVersion: Labrinth.Versions.v2.Version, event?: MouseEvent) {
	if (updatingModpack.value) {
		if (setupActionDisabled.value) return
		pendingModpackUpdateVersion.value = selectedVersion

		const mpSpec = contentQuery.data.value?.modpack?.spec
		const currentVersionId = mpSpec?.platform === 'modrinth' ? mpSpec.version_id : undefined
		const currentVersion = updatingProjectVersions.value.find((v) => v.id === currentVersionId)
		isModpackUpdateDowngrade.value = currentVersion
			? new Date(selectedVersion.date_published) < new Date(currentVersion.date_published)
			: false
		const shouldShowWarning =
			isModpackUpdateDowngrade.value ||
			versionChangesGameVersion(selectedVersion, currentGameVersion.value)

		if (event?.shiftKey || !shouldShowWarning) {
			handleModpackUpdateConfirm()
		} else {
			modpackUpdateModal.value?.show()
		}
		return
	}

	if (contentActionDisabled.value) return
	performUpdate(selectedVersion)
}

function setAddonInstalling(filename: string, installing: boolean) {
	queryClient.setQueryData(queryKey.value, (oldData: Archon.Content.v1.Addons | undefined) => {
		if (!oldData) return oldData
		return {
			...oldData,
			addons: (oldData.addons ?? []).map((a) =>
				a.filename === filename ? { ...a, installing } : a,
			),
		}
	})
}

async function performUpdate(selectedVersion: Labrinth.Versions.v2.Version) {
	if (
		(updatingModpack.value && setupActionDisabled.value) ||
		(!updatingModpack.value && contentActionDisabled.value)
	)
		return
	const item = updatingProject.value
	if (item) {
		setAddonInstalling(item.file_name, true)
	}
	try {
		if (updatingModpack.value) {
			const mp = contentQuery.data.value?.modpack
			if (!mp || mp.spec.platform !== 'modrinth') return
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: mp.spec.project_id,
					version_id: selectedVersion.id,
				},
				soft_override: true,
			})
		} else if (item) {
			const addon = addonLookup.value.get(item.file_name)
			if (addon) {
				await client.archon.content_v1.updateAddon(serverId, worldId.value!, {
					filename: addon.filename,
					version_id: selectedVersion.id,
				})
			}
		}
		await contentQuery.refetch()
	} catch (err) {
		if (item) {
			setAddonInstalling(item.file_name, false)
		}
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToUpdate),
			text: err instanceof Error ? err.message : undefined,
		})
	} finally {
		resetUpdateState()
	}
}

function handleModpackUpdateConfirm() {
	if (setupActionDisabled.value) return
	if (pendingModpackUpdateVersion.value) {
		contentUpdaterModal.value?.hide()
		performUpdate(pendingModpackUpdateVersion.value)
		pendingModpackUpdateVersion.value = null
	}
}

function handleModpackUpdateCancel() {
	pendingModpackUpdateVersion.value = null
}

function getOverflowOptions(item: ContentItem) {
	const options: { id: string; icon?: typeof ClipboardCopyIcon; action: () => void }[] = []

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

provideContentManager({
	items: contentItems,
	loading: computed(() => contentQuery.isLoading.value),
	error: computed(() => contentQuery.error.value ?? null),
	managedContent,
	isPackLocked: ref(false),
	isBusy: contentActionDisabled,
	busyMessage: contentActionBusyMessage,
	disableAddContent: computed(() => !canSetup.value),
	disableAddContentTooltip: permissionDeniedMessage.value,
	contentTypeLabel: type,
	toggleEnabled: handleToggleEnabled,
	deleteItem: handleDeleteItem,
	bulkDeleteItems: handleBulkDelete,
	bulkEnableItems: handleBulkEnable,
	bulkDisableItems: handleBulkDisable,
	refresh: async () => {
		await contentQuery.refetch()
	},
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	deletionContext: 'server',
	showEnvironmentWarnings: true,
	hasUpdateSupport: true,
	updateItem: handleUpdateItem,
	bulkUpdateItems: handleBulkUpdate,
	runManagedContentPrimaryAction: handleModpackUpdate,
	viewManagedContent: handleViewModpackContent,
	unlinkModpack: handleModpackUnlink,
	openManagedContentSettings: () => openServerSettings({ tabId: 'installation' }),
	switchVersion: handleSwitchVersion,
	getOverflowOptions,
	getItemId: getContentItemId,
	mapToTableItem: (item) => {
		const projectType = item.project_type ?? type.value
		const addon = addonLookup.value.get(item.file_name)
		const hasModrinthProject = !!addon?.project_id || (!!item.installing && !!item.project?.id)
		const projectSlugOrId = item.project.slug ?? item.project.id
		return {
			id: getContentItemId(item),
			project: item.project,
			projectLink: hasModrinthProject ? `/${projectType}/${projectSlugOrId}` : undefined,
			version: item.version,
			versionLink:
				hasModrinthProject && item.version?.id
					? `/${projectType}/${projectSlugOrId}/version/${item.version.id}`
					: undefined,
			owner: item.owner
				? { ...item.owner, link: item.owner.link ?? `/${item.owner.type}/${item.owner.id}` }
				: undefined,
			external: item.external ?? !hasModrinthProject,
			enabled: item.enabled,
		}
	},
	filterPersistKey: `server:${serverId}:${worldId.value}`,
})
</script>

<template>
	<ReadyTransition :pending="contentReadyPending">
		<ContentPageLayout :bottom-padding="false">
			<template #modals>
				<UnknownFileWarningModal
					ref="unknownFileWarningModal"
					mode="mod"
					:file-name="unknownFileName"
					@cancel="resolveUnknownFileWarning(false)"
					@continue="handleUnknownFileContinue"
				/>
				<ConfirmUnlinkModal
					ref="modpackUnlinkModal"
					server
					:action-disabled="setupActionDisabled"
					:action-disabled-tooltip="setupActionBusyMessage ?? undefined"
					@unlink="handleModpackUnlinkConfirm"
				/>
				<ManagedContentModal
					ref="modpackContentModal"
					:source-name="managedContent?.card.manager.name"
					:source-icon-url="managedContent?.card.manager.iconUrl"
					:header="formatMessage(messages.modpackContent)"
					enable-toggle
					show-environment-warnings
					:action-disabled="contentActionDisabled"
					:action-disabled-tooltip="contentActionBusyMessage ?? undefined"
					@update:enabled="handleModpackContentToggle"
					@bulk:enable="handleModpackBulkToggle($event, true)"
					@bulk:disable="handleModpackBulkToggle($event, false)"
					@hide="isModpackContentModalOpen = false"
				/>
				<ContentUpdaterModal
					v-if="updatingProject || updatingModpack"
					ref="contentUpdaterModal"
					:versions="updatingProjectVersions"
					:current-game-version="currentGameVersion"
					:current-loader="currentLoader"
					:current-version-id="
						updatingModpack
							? contentQuery.data.value?.modpack?.spec.platform === 'modrinth'
								? contentQuery.data.value.modpack.spec.version_id
								: ''
							: (updatingProject?.version?.id ?? '')
					"
					:is-app="false"
					:project-type="updatingModpack ? 'modpack' : updatingProject?.project_type"
					:project-icon-url="
						updatingModpack
							? managedContent?.card.manager.iconUrl
							: updatingProject?.project?.icon_url
					"
					:project-name="
						updatingModpack
							? (managedContent?.card.manager.name ?? formatMessage(commonMessages.modpackLabel))
							: (updatingProject?.project?.title ?? updatingProject?.file_name)
					"
					:loading="loadingVersions"
					:loading-changelog="loadingChangelog"
					:action-disabled="updatingModpack ? setupActionDisabled : contentActionDisabled"
					:action-disabled-tooltip="
						(updatingModpack ? setupActionBusyMessage : contentActionBusyMessage) ?? undefined
					"
					@update="handleModalUpdate"
					@cancel="resetUpdateState"
					@version-select="handleVersionSelect"
					@version-hover="handleVersionHover"
				/>
			</template>
		</ContentPageLayout>
	</ReadyTransition>
	<ConfirmModpackUpdateModal
		ref="modpackUpdateModal"
		:downgrade="isModpackUpdateDowngrade"
		:backup-tip="
			[managedContent?.card.manager.name, pendingModpackUpdateVersion?.version_number]
				.filter(Boolean)
				.join(' ')
		"
		server
		:action-disabled="setupActionDisabled"
		:action-disabled-tooltip="setupActionBusyMessage ?? undefined"
		@confirm="handleModpackUpdateConfirm"
		@cancel="handleModpackUpdateCancel"
	/>
</template>
