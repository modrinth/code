<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, toRef } from 'vue'
import { useRoute } from 'vue-router'

import ContentPageLayout from '../../../components/instances/ContentPageLayout.vue'
import ConfirmUnlinkModal from '../../../components/instances/modals/ConfirmUnlinkModal.vue'
import type {
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
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
})

const props = withDefaults(defineProps<{ useV1Api?: boolean }>(), {
	useV1Api: false,
})

const useV1 = toRef(props, 'useV1Api')

const client = injectModrinthClient()
const { server, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const route = useRoute()
const queryClient = useQueryClient()
const serverId = route.params.id as string

// ── Shared: loader type ──
const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	return loader === 'paper' || loader === 'purpur' ? 'plugin' : 'mod'
})

// ── Shared: modpack (from Labrinth, same for both v0/v1) ──
const hasModpack = computed(() => server.value?.upstream?.kind === 'modpack')

const { data: modpackProject } = useQuery({
	queryKey: computed(() => ['project', server.value?.upstream?.project_id]),
	queryFn: () => client.labrinth.projects_v3.get(server.value!.upstream!.project_id),
	enabled: hasModpack,
})

const { data: modpackVersion } = useQuery({
	queryKey: computed(() => ['version', server.value?.upstream?.version_id]),
	queryFn: () => client.labrinth.versions_v3.getVersion(server.value!.upstream!.version_id),
	enabled: hasModpack,
})

const modpack = computed<ContentModpackData | null>(() => {
	if (!hasModpack.value || !modpackProject.value) return null

	return {
		project: {
			id: modpackProject.value.id,
			slug: modpackProject.value.slug,
			title: modpackProject.value.title,
			icon_url: modpackProject.value.icon_url ?? undefined,
			description: modpackProject.value.description,
			downloads: modpackProject.value.downloads,
			followers: modpackProject.value.followers,
		} as ContentModpackCardProject,
		version: modpackVersion.value
			? ({
					id: modpackVersion.value.id,
					version_number: modpackVersion.value.version_number,
					date_published: modpackVersion.value.date_published,
				} as ContentModpackCardVersion)
			: undefined,
		owner: undefined as ContentOwner | undefined,
		categories: (modpackProject.value.categories?.map((cat) => ({
			name: cat,
			header: 'Categories',
		})) ?? []) as ContentModpackCardCategory[],
		hasUpdate: false,
	}
})

// ── Content queries (both declared, only one enabled at a time) ──
const v0QueryKey = computed(() => ['content', 'list', 'v0', serverId])
const v1QueryKey = computed(() => ['content', 'list', 'v1', serverId])

const v0Query = useQuery({
	queryKey: v0QueryKey,
	queryFn: () => client.archon.content_v0.list(serverId),
	enabled: computed(() => !useV1.value),
})

const v1Query = useQuery({
	queryKey: v1QueryKey,
	queryFn: () => client.archon.content_v1.getAddons(serverId, worldId.value ?? undefined),
	enabled: computed(() => useV1.value && worldId.value !== null),
})

const contentError = computed(() => {
	if (useV1.value) return v1Query.error.value ?? null
	return v0Query.error.value ?? null
})

const isLoadingContent = computed(() => {
	if (useV1.value) return v1Query.isLoading.value
	return v0Query.isLoading.value
})

async function refetchContent() {
	if (useV1.value) await v1Query.refetch()
	else await v0Query.refetch()
}

// ── V0 helpers ──
function getStableModKey(mod: Archon.Content.v0.Mod): string {
	if (mod.project_id) {
		return `project-${mod.project_id}`
	}
	const baseFilename = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename
	return `file-${baseFilename}`
}

function friendlyModName(mod: Archon.Content.v0.Mod): string {
	if (mod.name) return mod.name
	let cleanName = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

// ── V1 helpers ──
function friendlyAddonName(addon: Archon.Content.v1.Addon): string {
	if (addon.name) return addon.name
	let cleanName = addon.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

// ── Lookups (guarded by flag) ──
const modLookup = computed(() => {
	const map = new Map<string, Archon.Content.v0.Mod>()
	if (!useV1.value && v0Query.data.value) {
		for (const mod of v0Query.data.value) {
			map.set(getStableModKey(mod), mod)
		}
	}
	return map
})

const addonLookup = computed(() => {
	const map = new Map<string, Archon.Content.v1.Addon>()
	if (useV1.value && v1Query.data.value) {
		for (const addon of v1Query.data.value.addons) {
			map.set(addon.filename, addon)
		}
	}
	return map
})

// ── ContentItems (branched) ──
const contentItems = computed<ContentItem[]>(() => {
	if (useV1.value) {
		if (!v1Query.data.value) return []
		return v1Query.data.value.addons.map((addon) => ({
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
						link: `/${addon.owner.type}/${addon.owner.id}`,
					}
				: undefined,
			enabled: !addon.disabled,
			file_name: addon.filename,
			project_type: addon.kind,
			has_update: addon.has_update,
			update_version_id: null,
		}))
	}

	if (!v0Query.data.value) return []
	return v0Query.data.value.map((mod) => ({
		project: {
			id: mod.project_id ?? mod.filename,
			slug: mod.project_id ?? mod.filename,
			title: friendlyModName(mod),
			icon_url: mod.icon_url,
		},
		version: {
			id: mod.version_id ?? mod.filename,
			version_number: mod.version_number ?? 'Unknown',
			file_name: mod.filename,
		},
		owner: mod.owner
			? { id: mod.owner, name: mod.owner, type: 'user' as const, link: `/user/${mod.owner}` }
			: undefined,
		enabled: !mod.disabled,
		file_name: getStableModKey(mod),
		project_type: type.value,
		has_update: false,
		update_version_id: null,
	}))
})

// ── V0 mutations ──
const v0DeleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; modKey: string }) =>
		client.archon.content_v0.delete(serverId, { path }),
	onMutate: async ({ modKey }) => {
		await queryClient.cancelQueries({ queryKey: v0QueryKey.value })
		const previousData = queryClient.getQueryData<Archon.Content.v0.Mod[]>(v0QueryKey.value)
		queryClient.setQueryData(v0QueryKey.value, (oldData: Archon.Content.v0.Mod[] | undefined) => {
			if (!oldData) return oldData
			return oldData.filter((m) => getStableModKey(m) !== modKey)
		})
		return { previousData }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: v0QueryKey.value })
	},
	onError: (err, _vars, context) => {
		if (context?.previousData) {
			queryClient.setQueryData(v0QueryKey.value, context.previousData)
		}
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToRemoveContent),
		})
	},
})

const v0ToggleMutation = useMutation({
	mutationFn: async ({ mod }: { mod: Archon.Content.v0.Mod; modKey: string }) => {
		const folder = `${type.value}s`
		const newFilename = mod.filename.endsWith('.disabled')
			? mod.filename.slice(0, -9)
			: `${mod.filename}.disabled`
		await client.kyros.files_v0.moveFileOrFolder(
			`/${folder}/${mod.filename}`,
			`/${folder}/${newFilename}`,
		)
		return { newDisabled: !mod.disabled, modKey: getStableModKey(mod), newFilename }
	},
	onSuccess: ({ newDisabled, modKey, newFilename }) => {
		queryClient.setQueryData(v0QueryKey.value, (oldData: Archon.Content.v0.Mod[] | undefined) => {
			if (!oldData) return oldData
			return oldData.map((m) =>
				getStableModKey(m) === modKey ? { ...m, disabled: newDisabled, filename: newFilename } : m,
			)
		})
		queryClient.invalidateQueries({ queryKey: v0QueryKey.value })
	},
	onError: (_err, { mod }) => {
		addNotification({
			type: 'error',
			text: formatMessage(messages.failedToToggle, { name: friendlyModName(mod) }),
		})
	},
})

// ── V1 mutations ──
const v1DeleteMutation = useMutation({
	mutationFn: ({ addon }: { addon: Archon.Content.v1.Addon }) =>
		client.archon.content_v1.deleteAddon(
			serverId,
			{ filename: addon.filename, kind: addon.kind },
			worldId.value ?? undefined,
		),
	onMutate: async ({ addon }) => {
		await queryClient.cancelQueries({ queryKey: v1QueryKey.value })
		const previousData = queryClient.getQueryData<Archon.Content.v1.Addons>(v1QueryKey.value)
		queryClient.setQueryData(v1QueryKey.value, (oldData: Archon.Content.v1.Addons | undefined) => {
			if (!oldData) return oldData
			return {
				...oldData,
				addons: oldData.addons.filter((a) => a.filename !== addon.filename),
			}
		})
		return { previousData }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: v1QueryKey.value })
	},
	onError: (err, _vars, context) => {
		if (context?.previousData) {
			queryClient.setQueryData(v1QueryKey.value, context.previousData)
		}
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToRemoveContent),
		})
	},
})

const v1ToggleMutation = useMutation({
	mutationFn: async ({ addon }: { addon: Archon.Content.v1.Addon }) => {
		const request: Archon.Content.v1.RemoveAddonRequest = {
			filename: addon.filename,
			kind: addon.kind,
		}
		if (addon.disabled) {
			await client.archon.content_v1.enableAddon(serverId, request, worldId.value ?? undefined)
		} else {
			await client.archon.content_v1.disableAddon(serverId, request, worldId.value ?? undefined)
		}
		return { filename: addon.filename, newDisabled: !addon.disabled }
	},
	onSuccess: ({ filename, newDisabled }) => {
		queryClient.setQueryData(v1QueryKey.value, (oldData: Archon.Content.v1.Addons | undefined) => {
			if (!oldData) return oldData
			return {
				...oldData,
				addons: oldData.addons.map((a) =>
					a.filename === filename ? { ...a, disabled: newDisabled } : a,
				),
			}
		})
		queryClient.invalidateQueries({ queryKey: v1QueryKey.value })
	},
	onError: (_err, { addon }) => {
		addNotification({
			type: 'error',
			text: formatMessage(messages.failedToToggle, { name: friendlyAddonName(addon) }),
		})
	},
})

// ── Unified handlers (branch internally) ──
async function handleToggleEnabled(item: ContentItem) {
	if (useV1.value) {
		const addon = addonLookup.value.get(item.file_name)
		if (!addon) return
		await v1ToggleMutation.mutateAsync({ addon })
	} else {
		const mod = modLookup.value.get(item.file_name)
		if (!mod) return
		await v0ToggleMutation.mutateAsync({ mod, modKey: item.file_name })
	}
}

async function handleDeleteItem(item: ContentItem) {
	if (useV1.value) {
		const addon = addonLookup.value.get(item.file_name)
		if (!addon) return
		await v1DeleteMutation.mutateAsync({ addon })
	} else {
		const mod = modLookup.value.get(item.file_name)
		if (!mod) return
		await v0DeleteMutation.mutateAsync({
			path: `/${type.value}s/${mod.filename}`,
			modKey: item.file_name,
		})
	}
}

// ── V1 bulk handlers ──
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
	await client.archon.content_v1.deleteAddons(serverId, requests, worldId.value ?? undefined)
	await queryClient.invalidateQueries({ queryKey: v1QueryKey.value })
}

async function handleBulkEnable(items: ContentItem[]) {
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	await client.archon.content_v1.enableAddons(serverId, requests, worldId.value ?? undefined)
	await queryClient.invalidateQueries({ queryKey: v1QueryKey.value })
}

async function handleBulkDisable(items: ContentItem[]) {
	const requests = itemsToAddonRequests(items)
	if (requests.length === 0) return
	await client.archon.content_v1.disableAddons(serverId, requests, worldId.value ?? undefined)
	await queryClient.invalidateQueries({ queryKey: v1QueryKey.value })
}

// ── Upload state ──
const uploadState = ref<UploadState>({
	isUploading: false,
	currentFileName: null,
	currentFileProgress: 0,
	completedFiles: 0,
	totalFiles: 0,
})

// ── Shared handlers ──
const modpackUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()

function handleBrowseContent() {
	window.location.href = `/discover/${type.value}s?sid=${serverId}`
}

function handleUploadFiles() {
	const input = document.createElement('input')
	input.type = 'file'
	input.multiple = true
	input.accept = '.jar'
	input.onchange = async () => {
		if (!input.files) return
		const files = Array.from(input.files)
		const folder = `/${type.value}s`

		uploadState.value = {
			isUploading: true,
			currentFileName: null,
			currentFileProgress: 0,
			completedFiles: 0,
			totalFiles: files.length,
		}

		try {
			for (const file of files) {
				uploadState.value.currentFileName = file.name
				uploadState.value.currentFileProgress = 0
				try {
					await client.kyros.files_v0
						.uploadFile(`${folder}/${file.name}`, file)
						.onProgress((p) => {
							uploadState.value.currentFileProgress = p.progress
						}).promise
				} catch (err) {
					addNotification({
						type: 'error',
						text: err instanceof Error ? err.message : formatMessage(messages.failedToUpload),
					})
				}
				uploadState.value.completedFiles++
			}
			await refetchContent()
		} finally {
			uploadState.value = {
				isUploading: false,
				currentFileName: null,
				currentFileProgress: 0,
				completedFiles: 0,
				totalFiles: 0,
			}
		}
	}
	input.click()
}

function handleModpackContent() {
	if (modpackProject.value?.slug) {
		window.location.href = `/project/${modpackProject.value.slug}`
	}
}

function handleModpackUnlink() {
	if (!useV1.value) return
	modpackUnlinkModal.value?.show()
}

async function handleModpackUnlinkConfirm() {
	if (!useV1.value) return
	try {
		await client.archon.content_v1.unlinkModpack(serverId, worldId.value ?? undefined)
		await queryClient.invalidateQueries({ queryKey: ['project', server.value?.upstream?.project_id] })
		await queryClient.invalidateQueries({ queryKey: ['version', server.value?.upstream?.version_id] })
		await refetchContent()
	} catch (err) {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : formatMessage(messages.failedToUnlink),
		})
	}
}

// ── Provider ──
provideContentManager({
	items: contentItems,
	loading: isLoadingContent,
	error: computed(() => contentError.value),
	modpack,
	isPackLocked: ref(false),
	isBusy: ref(false),
	getItemId: (item) => item.file_name,
	contentTypeLabel: type,
	toggleEnabled: handleToggleEnabled,
	deleteItem: handleDeleteItem,
	bulkDeleteItems: useV1.value ? handleBulkDelete : undefined,
	bulkEnableItems: useV1.value ? handleBulkEnable : undefined,
	bulkDisableItems: useV1.value ? handleBulkDisable : undefined,
	refresh: async () => {
		await refetchContent()
	},
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
	uploadState,
	hasUpdateSupport: false,
	viewModpackContent: handleModpackContent,
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
		</template>
	</ContentPageLayout>
</template>
