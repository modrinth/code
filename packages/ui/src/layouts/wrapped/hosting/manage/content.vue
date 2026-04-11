<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router'

import ConfirmLeaveModal from '#ui/components/modal/ConfirmLeaveModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	injectServerSettingsModal,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

import ConfirmModpackUpdateModal from '../../../shared/content-tab/components/modals/ConfirmModpackUpdateModal.vue'
import ConfirmUnlinkModal from '../../../shared/content-tab/components/modals/ConfirmUnlinkModal.vue'
import ContentUpdaterModal from '../../../shared/content-tab/components/modals/ContentUpdaterModal.vue'
import ModpackContentModal from '../../../shared/content-tab/components/modals/ModpackContentModal.vue'
import ContentPageLayout from '../../../shared/content-tab/layout.vue'
import type {
	ContentModpackData,
	UploadState,
} from '../../../shared/content-tab/providers/content-manager'
import { provideContentManager } from '../../../shared/content-tab/providers/content-manager'
import type {
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
} from '../../../shared/content-tab/types'

type AddonWithUiState = Archon.Content.v1.Addon & { installing?: boolean }

const { formatMessage } = useVIntl()

const messages = defineMessages({
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
})

const leaveMessages = defineMessages({
	uploadInProgress: {
		id: 'instances.confirm-leave-modal.upload-in-progress',
		defaultMessage: 'Upload in progress',
	},
	leavePageBody: {
		id: 'instances.confirm-leave-modal.body',
		defaultMessage:
			'Files are still being uploaded. Leaving this page will cancel the upload and your changes may be lost.',
	},
})

const client = injectModrinthClient()
const { server, worldId, busyReasons, isSyncingContent } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const { openServerSettings, browseServerContent } = injectServerSettingsModal()
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

const contentQuery = useQuery({
	queryKey,
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
	enabled: computed(() => worldId.value !== null),
	staleTime: 0,
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

const modpack = computed<ContentModpackData | null>(() => {
	const mp = contentQuery.data.value?.modpack
	if (!mp) return null
	const isLocal = mp.spec.platform === 'local_file'
	const project = projectQuery.data.value
	const projectId = isLocal ? null : mp.spec.project_id
	return {
		project: {
			id: projectId ?? mp.title ?? '',
			slug: project?.slug ?? projectId ?? '',
			title: mp.title ?? (isLocal ? mp.spec.name : projectId) ?? '',
			icon_url: mp.icon_url ?? undefined,
			description: mp.description ?? '',
			downloads: mp.downloads,
			followers: mp.followers,
			filename: isLocal ? mp.spec.filename : undefined,
		} as ContentModpackCardProject,
		projectLink: projectId ? `/project/${project?.slug ?? projectId}` : undefined,
		version: isLocal
			? undefined
			: ({
					id: mp.spec.version_id,
					version_number: mp.version_number ?? '',
					date_published: mp.date_published ?? '',
				} as ContentModpackCardVersion),
		versionLink:
			projectId && !isLocal
				? `/project/${project?.slug ?? projectId}/version/${mp.spec.version_id}`
				: undefined,
		owner: mp.owner
			? {
					id: mp.owner.id,
					name: mp.owner.name,
					avatar_url: mp.owner.icon_url ?? undefined,
					type: mp.owner.type,
					link:
						mp.owner.type === 'organization'
							? `/organization/${mp.owner.id}`
							: `/user/${mp.owner.id}`,
				}
			: undefined,
		categories: (project?.categories ?? []).map((name) => ({
			name,
			icon: name,
			project_type: 'modpack',
			header: 'categories',
		})) as ContentModpackCardCategory[],
		hasUpdate: !!mp.has_update,
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

const contentItems = computed<ContentItem[]>(() => {
	return (contentQuery.data.value?.addons ?? []).map(addonToContentItem)
})

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
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	await toggleMutation.mutateAsync({ addon })
}

async function handleDeleteItem(item: ContentItem) {
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	await deleteMutation.mutateAsync({ addon })
}

function itemsToAddonRequests(items: ContentItem[]): Archon.Content.v1.RemoveAddonRequest[] {
	return items.flatMap((item) => {
		const addon = addonLookup.value.get(item.file_name)
		if (!addon) return []
		return [{ filename: addon.filename, kind: addon.kind }]
	})
}

async function handleBulkDelete(items: ContentItem[]) {
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

const uploadState = ref<UploadState>({
	isUploading: false,
	currentFileName: null,
	currentFileProgress: 0,
	uploadedBytes: 0,
	totalBytes: 0,
	completedFiles: 0,
	totalFiles: 0,
})

const confirmLeaveModal = ref<InstanceType<typeof ConfirmLeaveModal>>()
const modpackUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const modpackContentModal = ref<InstanceType<typeof ModpackContentModal>>()
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal>>()

let activeUploadCancel: (() => void) | null = null

const isUploading = computed(() => uploadState.value.isUploading)

function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (isUploading.value) {
		e.preventDefault()
		return ''
	}
}

if (typeof window !== 'undefined') {
	watch(isUploading, (uploading) => {
		if (uploading) {
			window.addEventListener('beforeunload', handleBeforeUnload)
		} else {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		}
	})

	onBeforeUnmount(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload)
	})

	onBeforeRouteLeave(async () => {
		if (isUploading.value) {
			const shouldLeave = (await confirmLeaveModal.value?.prompt()) ?? false
			if (shouldLeave) {
				activeUploadCancel?.()
			}
			return shouldLeave
		}
		return true
	})
}

const updatingProject = ref<ContentItem | null>(null)
const updatingModpack = ref(false)
const loadingChangelog = ref(false)

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
	return [...source].sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)
})

const loadingVersions = computed(() =>
	updatingModpack.value
		? modpackVersionsQuery.isLoading.value
		: projectVersionsQuery.isLoading.value,
)

const modpackUpdateModal = ref<InstanceType<typeof ConfirmModpackUpdateModal>>()
const pendingModpackUpdateVersion = ref<Labrinth.Versions.v2.Version | null>(null)
const isModpackUpdateDowngrade = ref(false)

const currentGameVersion = computed(() => contentQuery.data.value?.game_version ?? '')
const currentLoader = computed(() => contentQuery.data.value?.modloader ?? '')

function handleBrowseContent() {
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
	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.accept = type.value === 'datapack' ? '.zip' : '.jar'
	input.onchange = async () => {
		if (!input.files) return
		const files = Array.from(input.files)
		const wid = worldId.value
		if (!wid) return

		uploadState.value = {
			isUploading: true,
			currentFileName: null,
			currentFileProgress: 0,
			uploadedBytes: 0,
			totalBytes: files.reduce((sum, f) => sum + f.size, 0),
			completedFiles: 0,
			totalFiles: files.length,
		}

		const handle = client.kyros.content_v1.uploadAddonFile(wid, files, {
			onProgress: (p) => {
				uploadState.value.currentFileProgress = p.progress
				uploadState.value.uploadedBytes = p.loaded
				uploadState.value.totalBytes = p.total
			},
		})
		activeUploadCancel = () => handle.cancel()

		try {
			await handle.promise
			uploadState.value.completedFiles = files.length
			await contentQuery.refetch()
		} catch (err) {
			if (err instanceof Error && err.message === 'Upload cancelled') return
			addNotification({
				type: 'error',
				title: formatMessage(messages.failedToUpload),
				text: err instanceof Error ? err.message : undefined,
			})
		} finally {
			activeUploadCancel = null
			uploadState.value = {
				isUploading: false,
				currentFileName: null,
				currentFileProgress: 0,
				uploadedBytes: 0,
				totalBytes: 0,
				completedFiles: 0,
				totalFiles: 0,
			}
		}
	}
	input.click()
}

function addonToContentItem(addon: AddonWithUiState): ContentItem {
	return {
		project: {
			id: addon.project_id ?? addon.filename,
			slug: addon.project_id ?? addon.filename,
			title: friendlyAddonName(addon),
			icon_url: addon.icon_url ?? undefined,
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
					avatar_url: addon.owner.icon_url ?? undefined,
					link: `/${addon.owner.type}/${addon.owner.id}`,
				}
			: undefined,
		id: addon.id ?? addon.filename,
		enabled: !addon.disabled,
		file_name: addon.filename,
		project_type: addon.kind,
		has_update: !!addon.has_update,
		update_version_id: addon.has_update,
		environment: addon.version?.environment ?? undefined,
		pack_client_retained: addon.pack_client_retained,
		pack_client_depends: addon.pack_client_depends,
		installing: addon.installing,
	}
}

async function handleViewModpackContent() {
	modpackContentModal.value?.showLoading()
	try {
		const data = await client.archon.content_v1.getAddons(serverId, worldId.value!, {
			from_modpack: true,
		})
		modpackAddons.value = data.addons ?? []
		const items = (data.addons ?? []).map(addonToContentItem)
		modpackContentModal.value?.show(items)
	} catch (err) {
		modpackContentModal.value?.hide()
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToLoadModpackContent),
			text: err instanceof Error ? err.message : undefined,
		})
	}
}

async function handleModpackContentToggle(item: ContentItem) {
	const addon = addonLookup.value.get(item.file_name)
	if (!addon) return
	modpackContentModal.value?.updateItem(item.file_name, { disabled: true })
	try {
		await toggleMutation.mutateAsync({ addon })
		modpackAddons.value = modpackAddons.value.map((a) =>
			a.filename === addon.filename ? { ...a, disabled: !addon.disabled } : a,
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
	const addons = items
		.filter((item) => item.has_update)
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
	const item = contentItems.value.find((i) => i.id === id)
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
	const mp = contentQuery.data.value?.modpack
	if (!mp || mp.spec.platform !== 'modrinth') return

	updatingModpack.value = true
	updatingProject.value = null
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(mp.has_update ?? undefined)
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
		if (event?.shiftKey) {
			pendingModpackUpdateVersion.value = selectedVersion
			handleModpackUpdateConfirm()
		} else {
			const mpSpec = contentQuery.data.value?.modpack?.spec
			const currentVersionId = mpSpec?.platform === 'modrinth' ? mpSpec.version_id : undefined
			const currentVersion = updatingProjectVersions.value.find((v) => v.id === currentVersionId)
			isModpackUpdateDowngrade.value = currentVersion
				? new Date(selectedVersion.date_published) < new Date(currentVersion.date_published)
				: false
			pendingModpackUpdateVersion.value = selectedVersion
			modpackUpdateModal.value?.show()
		}
		return
	}

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
	if (pendingModpackUpdateVersion.value) {
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
	modpack,
	isPackLocked: ref(false),
	isBusy: computed(() => busyReasons.value.length > 0),
	busyMessage: computed(() => {
		const bannerCoversInstalling = server.value?.status === 'installing' || isSyncingContent.value
		const filteredReasons = busyReasons.value.filter((r) => {
			if (
				bannerCoversInstalling &&
				(r.reason.id === 'servers.busy.installing' ||
					r.reason.id === 'servers.busy.syncing-content')
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
	}),
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
	uploadState,
	deletionContext: 'server',
	hasUpdateSupport: true,
	updateItem: handleUpdateItem,
	bulkUpdateItems: handleBulkUpdate,
	updateModpack: handleModpackUpdate,
	viewModpackContent: handleViewModpackContent,
	unlinkModpack: handleModpackUnlink,
	openSettings: () => openServerSettings({ tabId: 'installation' }),
	switchVersion: handleSwitchVersion,
	getOverflowOptions,
	mapToTableItem: (item) => {
		const projectType = item.project_type ?? type.value
		const addon = addonLookup.value.get(item.file_name)
		const hasModrinthProject = !!addon?.project_id
		return {
			id: item.id,
			project: item.project,
			projectLink: hasModrinthProject ? `/${projectType}/${item.project.id}` : undefined,
			version: item.version,
			versionLink:
				hasModrinthProject && item.version?.id
					? `/${projectType}/${item.project.id}/version/${item.version.id}`
					: undefined,
			owner: item.owner
				? { ...item.owner, link: `/${item.owner.type}/${item.owner.id}` }
				: undefined,
			enabled: item.enabled,
		}
	},
	filterPersistKey: `server:${serverId}:${worldId.value}`,
})
</script>

<template>
	<ContentPageLayout>
		<template #modals>
			<ConfirmUnlinkModal ref="modpackUnlinkModal" server @unlink="handleModpackUnlinkConfirm" />
			<ModpackContentModal
				ref="modpackContentModal"
				:modpack-name="modpack?.project.title"
				:modpack-icon-url="modpack?.project.icon_url"
				enable-toggle
				@update:enabled="handleModpackContentToggle"
				@bulk:enable="handleModpackBulkToggle($event, true)"
				@bulk:disable="handleModpackBulkToggle($event, false)"
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
					updatingModpack ? modpack?.project.icon_url : updatingProject?.project?.icon_url
				"
				:project-name="
					updatingModpack
						? (modpack?.project.title ?? formatMessage(commonMessages.modpackLabel))
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
	<ConfirmModpackUpdateModal
		ref="modpackUpdateModal"
		:downgrade="isModpackUpdateDowngrade"
		:backup-tip="
			[modpack?.project.title, pendingModpackUpdateVersion?.version_number]
				.filter(Boolean)
				.join(' ')
		"
		server
		@confirm="handleModpackUpdateConfirm"
		@cancel="handleModpackUpdateCancel"
	/>
	<ConfirmLeaveModal
		ref="confirmLeaveModal"
		:header="formatMessage(leaveMessages.uploadInProgress)"
		:body="formatMessage(leaveMessages.leavePageBody)"
		admonition-type="critical"
	/>
</template>
