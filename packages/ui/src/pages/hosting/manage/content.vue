<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import ContentPageLayout from '../../../components/instances/ContentPageLayout.vue'
import ModpackUnlinkModal from '../../../components/instances/modals/ModpackUnlinkModal.vue'
import type {
	ContentItem,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '../../../components/instances/types'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	provideContentManager,
} from '../../../providers'
import type { ContentModpackData } from '../../../providers/content-manager'

const client = injectModrinthClient()
const { server, worldId } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const route = useRoute()
const queryClient = useQueryClient()
const serverId = route.params.id as string

const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	return loader === 'paper' || loader === 'purpur' ? 'plugin' : 'mod'
})

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

const contentQueryKey = computed(() => ['content', 'list', serverId])
const {
	data: contentData,
	error: contentError,
	refetch,
	isLoading: isLoadingContent,
} = useQuery({
	queryKey: contentQueryKey,
	queryFn: () => client.archon.content_v1.getAddons(serverId, worldId.value ?? undefined),
	enabled: computed(() => worldId.value !== null),
})

function friendlyAddonName(addon: Archon.Content.v1.Addon): string {
	if (addon.name) return addon.name
	let cleanName = addon.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

const addonLookup = computed(() => {
	const map = new Map<string, Archon.Content.v1.Addon>()
	if (contentData.value) {
		for (const addon of contentData.value.addons) {
			map.set(addon.filename, addon)
		}
	}
	return map
})

const contentItems = computed<ContentItem[]>(() => {
	if (!contentData.value) return []
	return contentData.value.addons.map((addon) => ({
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
})

const modpackUnlinkModal = ref<InstanceType<typeof ModpackUnlinkModal>>()

const deleteMutation = useMutation({
	mutationFn: ({ addon }: { addon: Archon.Content.v1.Addon }) =>
		client.archon.content_v1.deleteAddon(
			serverId,
			{ filename: addon.filename, kind: addon.kind },
			worldId.value ?? undefined,
		),
	onMutate: async ({ addon }) => {
		await queryClient.cancelQueries({ queryKey: contentQueryKey.value })
		const previousData = queryClient.getQueryData<Archon.Content.v1.Addons>(contentQueryKey.value)
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v1.Addons | undefined) => {
				if (!oldData) return oldData
				return {
					...oldData,
					addons: oldData.addons.filter((a) => a.filename !== addon.filename),
				}
			},
		)
		return { previousData }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (err, _vars, context) => {
		if (context?.previousData) {
			queryClient.setQueryData(contentQueryKey.value, context.previousData)
		}
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to remove content',
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
			await client.archon.content_v1.enableAddon(serverId, request, worldId.value ?? undefined)
		} else {
			await client.archon.content_v1.disableAddon(serverId, request, worldId.value ?? undefined)
		}
		return { filename: addon.filename, newDisabled: !addon.disabled }
	},
	onSuccess: ({ filename, newDisabled }) => {
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v1.Addons | undefined) => {
				if (!oldData) return oldData
				return {
					...oldData,
					addons: oldData.addons.map((a) =>
						a.filename === filename ? { ...a, disabled: newDisabled } : a,
					),
				}
			},
		)
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (_err, { addon }) => {
		addNotification({
			type: 'error',
			text: `Failed to toggle ${friendlyAddonName(addon)}`,
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

function handleBrowseContent() {
	window.location.href = `/discover/${type.value}s?sid=${serverId}`
}

function handleUploadFiles() {
	console.log('Upload files')
}

function handleModpackContent() {
	if (modpackProject.value?.slug) {
		window.location.href = `/project/${modpackProject.value.slug}`
	}
}

function handleModpackUnlink() {
	addNotification({
		type: 'warning',
		text: 'Modpack unlinking is not yet available',
	})
}

function handleModpackUnlinkConfirm() {
	addNotification({
		type: 'warning',
		text: 'Modpack unlinking is not yet available',
	})
}

provideContentManager({
	items: contentItems,
	loading: isLoadingContent,
	error: computed(() => contentError.value ?? null),
	modpack,
	isPackLocked: ref(false),
	isBusy: ref(false),
	getItemId: (item) => item.file_name,
	contentTypeLabel: type,
	toggleEnabled: handleToggleEnabled,
	deleteItem: handleDeleteItem,
	refresh: async () => {
		await refetch()
	},
	browse: handleBrowseContent,
	uploadFiles: handleUploadFiles,
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
			<ModpackUnlinkModal ref="modpackUnlinkModal" @unlink="handleModpackUnlinkConfirm" />
		</template>
	</ContentPageLayout>
</template>
