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
const { server } = injectModrinthServerContext()
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
	queryFn: () => client.archon.content_v0.list(serverId),
})

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

const modLookup = computed(() => {
	const map = new Map<string, Archon.Content.v0.Mod>()
	if (contentData.value) {
		for (const mod of contentData.value) {
			map.set(getStableModKey(mod), mod)
		}
	}
	return map
})

const contentItems = computed<ContentItem[]>(() => {
	if (!contentData.value) return []
	return contentData.value.map((mod) => ({
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

const modpackUnlinkModal = ref<InstanceType<typeof ModpackUnlinkModal>>()

const deleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; modKey: string }) =>
		client.archon.content_v0.delete(serverId, { path }),
	onMutate: async ({ modKey }) => {
		await queryClient.cancelQueries({ queryKey: contentQueryKey.value })
		const previousData = queryClient.getQueryData<Archon.Content.v0.Mod[]>(contentQueryKey.value)
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v0.Mod[] | undefined) => {
				if (!oldData) return oldData
				return oldData.filter((m) => getStableModKey(m) !== modKey)
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
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v0.Mod[] | undefined) => {
				if (!oldData) return oldData
				return oldData.map((m) =>
					getStableModKey(m) === modKey
						? { ...m, disabled: newDisabled, filename: newFilename }
						: m,
				)
			},
		)
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (_err, { mod }) => {
		addNotification({
			type: 'error',
			text: `Failed to toggle ${friendlyModName(mod)}`,
		})
	},
})

async function handleToggleEnabled(item: ContentItem) {
	const mod = modLookup.value.get(item.file_name)
	if (!mod) return
	await toggleMutation.mutateAsync({ mod, modKey: item.file_name })
}

async function handleDeleteItem(item: ContentItem) {
	const mod = modLookup.value.get(item.file_name)
	if (!mod) return
	await deleteMutation.mutateAsync({
		path: `/${type.value}s/${mod.filename}`,
		modKey: item.file_name,
	})
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
		owner: item.owner,
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
