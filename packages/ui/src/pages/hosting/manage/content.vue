<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router'

import ContentPageLayout from '../../../components/instances/ContentPageLayout.vue'
import ConfirmLeaveModal from '../../../components/instances/modals/ConfirmLeaveModal.vue'
import ConfirmUnlinkModal from '../../../components/instances/modals/ConfirmUnlinkModal.vue'
import ContentUpdaterModal from '../../../components/instances/modals/ContentUpdaterModal.vue'
import ModpackContentModal from '../../../components/instances/modals/ModpackContentModal.vue'
import type {
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
} from '../../../components/instances/types'
import { defineMessages, useVIntl } from '../../../composables/i18n'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	provideContentManager,
} from '../../../providers'
import type { ContentModpackData, UploadState } from '../../../providers/content-manager'

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
})

const props = withDefaults(
	defineProps<{
		showClientOnlyFilter?: boolean
	}>(),
	{
		showClientOnlyFilter: false,
	},
)

const client = injectModrinthClient()
const { server, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const route = useRoute()
const router = useRouter()
const queryClient = useQueryClient()
const serverId = route.params.id as string

const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	return loader === 'paper' || loader === 'purpur' ? 'plugin' : 'mod'
})

const queryKey = computed(() => ['content', 'list', 'v1', serverId])

const contentQuery = useQuery({
	queryKey,
	queryFn: () =>
		client.archon.content_v1.getAddons(serverId, worldId.value!, { from_modpack: false }),
	enabled: computed(() => worldId.value !== null),
})

const modpackProjectId = computed(() => contentQuery.data.value?.modpack?.spec.project_id ?? null)

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
	const project = projectQuery.data.value
	return {
		project: {
			id: mp.spec.project_id,
			slug: project?.slug ?? mp.spec.project_id,
			title: mp.title ?? mp.spec.project_id,
			icon_url: mp.icon_url ?? undefined,
			description: mp.description ?? '',
			downloads: mp.downloads ?? 0,
			followers: mp.followers ?? 0,
		} as ContentModpackCardProject,
		projectLink: `/project/${project?.slug ?? mp.spec.project_id}`,
		version: {
			id: mp.spec.version_id,
			version_number: mp.version_number ?? '',
			date_published: mp.date_published ?? '',
		} as ContentModpackCardVersion,
		versionLink: `/project/${project?.slug ?? mp.spec.project_id}/version/${mp.spec.version_id}`,
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
		categories: (project?.display_categories ?? []).map((name) => ({
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
			text: err instanceof Error ? err.message : formatMessage(messages.failedToRemoveContent),
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
			text: formatMessage(messages.failedToToggle, { name: friendlyAddonName(addon) }),
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
	await client.archon.content_v1.deleteAddons(serverId, worldId.value!, requests)
	await queryClient.invalidateQueries({ queryKey: queryKey.value })
}

async function handleBulkEnable(items: ContentItem[]) {
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	await client.archon.content_v1.enableAddons(serverId, worldId.value!, requests)
	await queryClient.invalidateQueries({ queryKey: queryKey.value })
}

async function handleBulkDisable(items: ContentItem[]) {
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	await client.archon.content_v1.disableAddons(serverId, worldId.value!, requests)
	await queryClient.invalidateQueries({ queryKey: queryKey.value })
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
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)

const currentGameVersion = computed(() => contentQuery.data.value?.game_version ?? '')
const currentLoader = computed(() => contentQuery.data.value?.modloader ?? '')

function handleBrowseContent() {
	router.push({
		path: `/discover/${type.value}s`,
		query: { sid: serverId, wid: worldId.value },
	})
}

function handleUploadFiles() {
	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.accept = '.jar'
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
				text: err instanceof Error ? err.message : formatMessage(messages.failedToUpload),
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

function addonToContentItem(addon: Archon.Content.v1.Addon): ContentItem {
	return {
		project: {
			id: addon.project_id ?? addon.filename,
			slug: addon.project_id ?? addon.filename,
			title: friendlyAddonName(addon),
			icon_url: addon.icon_url ?? undefined,
		},
		version: {
			id: addon.version?.id ?? addon.filename,
			version_number: addon.version?.name ?? 'Unknown',
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
		enabled: !addon.disabled,
		file_name: addon.filename,
		project_type: addon.kind,
		has_update: !!addon.has_update,
		update_version_id: addon.has_update,
		environment: addon.version?.environment ?? undefined,
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
			text: err instanceof Error ? err.message : formatMessage(messages.failedToLoadModpackContent),
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
			text: err instanceof Error ? err.message : formatMessage(messages.failedToUnlink),
		})
	}
}

async function handleUpdateItem(fileNameKey: string) {
	const item = contentItems.value.find((i) => i.file_name === fileNameKey)
	if (!item?.has_update || !item.project?.id || !item.version?.id) return

	updatingModpack.value = false
	updatingProject.value = item
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.update_version_id ?? undefined)

	try {
		const versions = await client.labrinth.versions_v2.getProjectVersions(item.project.id, {
			include_changelog: false,
		})
		versions.sort(
			(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
		)
		updatingProjectVersions.value = versions
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to load versions',
		})
	} finally {
		loadingVersions.value = false
	}
}

async function handleModpackUpdate() {
	const mp = contentQuery.data.value?.modpack
	if (!mp?.spec.project_id) return

	updatingModpack.value = true
	updatingProject.value = null
	loadingChangelog.value = false

	const cached = modpackVersionsQuery.data.value
	if (cached) {
		const sorted = [...cached].sort(
			(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
		)
		updatingProjectVersions.value = sorted
		loadingVersions.value = false
	} else {
		updatingProjectVersions.value = []
		loadingVersions.value = true
	}

	await nextTick()

	contentUpdaterModal.value?.show(mp.spec.version_id ?? undefined)

	if (!cached) {
		try {
			const versions = await client.labrinth.versions_v2.getProjectVersions(mp.spec.project_id, {
				include_changelog: false,
			})
			versions.sort(
				(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
			)
			updatingProjectVersions.value = versions
		} catch (err) {
			addNotification({
				type: 'error',
				text: err instanceof Error ? err.message : 'Failed to load versions',
			})
		} finally {
			loadingVersions.value = false
		}
	}
}

async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	loadingChangelog.value = true
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
		if (index !== -1) {
			const newVersions = [...updatingProjectVersions.value]
			newVersions[index] = fullVersion
			updatingProjectVersions.value = newVersions
		}
	} catch {
		// Silently fail on changelog fetch
	} finally {
		loadingChangelog.value = false
	}
}

async function handleVersionHover(version: Labrinth.Versions.v2.Version) {
	if (version.changelog !== undefined) return
	try {
		const fullVersion = await client.labrinth.versions_v2.getVersion(version.id)
		const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
		if (index !== -1) {
			const newVersions = [...updatingProjectVersions.value]
			newVersions[index] = fullVersion
			updatingProjectVersions.value = newVersions
		}
	} catch {
		// Silently fail on hover prefetch
	}
}

function resetUpdateState() {
	updatingModpack.value = false
	updatingProject.value = null
	updatingProjectVersions.value = []
	loadingVersions.value = false
	loadingChangelog.value = false
}

async function handleModalUpdate(selectedVersion: Labrinth.Versions.v2.Version) {
	try {
		if (updatingModpack.value) {
			const mp = contentQuery.data.value?.modpack
			if (!mp) return
			await client.archon.content_v1.installContent(serverId, worldId.value!, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: mp.spec.project_id,
					version_id: selectedVersion.id,
				},
				soft_override: true,
			})
		} else if (updatingProject.value) {
			const addon = addonLookup.value.get(updatingProject.value.file_name)
			if (addon) {
				await client.archon.content_v1.updateAddon(serverId, worldId.value!, {
					filename: addon.filename,
				})
			}
		}
		await contentQuery.refetch()
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to update',
		})
	} finally {
		resetUpdateState()
	}
}

provideContentManager({
	items: contentItems,
	loading: computed(() => contentQuery.isLoading.value),
	error: computed(() => contentQuery.error.value ?? null),
	modpack,
	isPackLocked: ref(false),
	isBusy: ref(false),
	getItemId: (item) => item.file_name,
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
	showClientOnlyFilter: props.showClientOnlyFilter,
	deletionContext: 'server',
	backupLink: `/hosting/manage/${serverId}/backups`,
	hasUpdateSupport: true,
	updateItem: handleUpdateItem,
	updateModpack: handleModpackUpdate,
	viewModpackContent: handleViewModpackContent,
	unlinkModpack: handleModpackUnlink,
	mapToTableItem: (item) => ({
		id: item.file_name,
		project: item.project,
		projectLink: item.project?.id ? `/mod/${item.project.id}` : undefined,
		version: item.version,
		versionLink:
			item.project?.id && item.version?.id
				? `/mod/${item.project.id}/version/${item.version.id}`
				: undefined,
		owner: item.owner ? { ...item.owner, link: `/${item.owner.type}/${item.owner.id}` } : undefined,
		enabled: item.enabled,
	}),
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
			/>
			<ContentUpdaterModal
				v-if="updatingProject || updatingModpack"
				ref="contentUpdaterModal"
				:versions="updatingProjectVersions"
				:current-game-version="currentGameVersion"
				:current-loader="currentLoader"
				:current-version-id="
					updatingModpack
						? (contentQuery.data.value?.modpack?.spec.version_id ?? '')
						: (updatingProject?.version?.id ?? '')
				"
				:is-app="false"
				:is-modpack="updatingModpack"
				:project-icon-url="
					updatingModpack ? modpack?.project.icon_url : updatingProject?.project?.icon_url
				"
				:project-name="
					updatingModpack
						? (modpack?.project.title ?? 'Modpack')
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
	<ConfirmLeaveModal ref="confirmLeaveModal" />
</template>
