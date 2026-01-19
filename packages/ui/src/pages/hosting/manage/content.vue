<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import {
	CompassIcon,
	ListFilterIcon,
	SearchIcon,
	SortAscIcon,
	SortDescIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import Combobox, { type ComboboxOption } from '../../../components/base/Combobox.vue'
import Pagination from '../../../components/base/Pagination.vue'
import ContentCard from '../../../components/instances/ContentCard.vue'
import ContentModpackCard from '../../../components/instances/ContentModpackCard.vue'
import ModpackUnlinkModal from '../../../components/instances/modals/ModpackUnlinkModal.vue'
import type {
	ContentCardProject,
	ContentCardVersion,
	ContentModpackCardCategory,
	ContentModpackCardProject,
	ContentModpackCardVersion,
	ContentOwner,
} from '../../../components/instances/types'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '../../../providers'

const client = injectModrinthClient()
const { server } = injectModrinthServerContext()
const { addNotification } = injectNotificationManager()
const route = useRoute()
const queryClient = useQueryClient()
const serverId = route.params.id as string

const type = computed(() => {
	const loader = server.value?.loader?.toLowerCase()
	return loader === 'paper' || loader === 'purpur' ? 'Plugin' : 'Mod'
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

const modpack = computed(() => {
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
		owner: undefined as ContentOwner | undefined, // TODO: fetch team/user for owner
		categories: (modpackProject.value.categories?.map((cat) => ({
			name: cat,
			header: 'Categories',
		})) ?? []) as ContentModpackCardCategory[],
	}
})

// Fetch content list from Archon
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

// Track mods currently being changed
const changingMods = ref(new Set<string>())

// Helper to get stable key for a mod
function getStableModKey(mod: Archon.Content.v0.Mod): string {
	if (mod.project_id) {
		return `project-${mod.project_id}`
	}
	const baseFilename = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename
	return `file-${baseFilename}`
}

// Helper to get friendly mod name
function friendlyModName(mod: Archon.Content.v0.Mod): string {
	if (mod.name) return mod.name
	let cleanName = mod.filename.endsWith('.disabled') ? mod.filename.slice(0, -9) : mod.filename
	const lastDotIndex = cleanName.lastIndexOf('.')
	if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex)
	return cleanName
}

// Map Archon Mod to ContentItem for display
interface ContentItem {
	project: ContentCardProject
	version?: ContentCardVersion
	owner?: ContentOwner
	enabled: boolean
	hasUpdate?: boolean
	_mod: Archon.Content.v0.Mod // Keep reference to original mod
}

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
			? {
					id: mod.owner,
					name: mod.owner,
					type: 'user' as const,
				}
			: undefined,
		enabled: !mod.disabled,
		hasUpdate: false, // TODO: implement update checking
		_mod: mod,
	}))
})

const searchQuery = ref('')
const filterType = ref('All')
const sortType = ref('Newest')
const currentPage = ref(1)
const itemsPerPage = 10

const _modpackUnlinkModal = ref<InstanceType<typeof ModpackUnlinkModal>>()

const filterOptions: ComboboxOption<string>[] = [
	{ value: 'All', label: 'All' },
	{ value: 'Mods', label: 'Mods' },
	{ value: 'Resource Packs', label: 'Resource Packs' },
	{ value: 'Shaders', label: 'Shaders' },
]

const sortOptions: ComboboxOption<string>[] = [
	{ value: 'Newest', label: 'Newest' },
	{ value: 'Oldest', label: 'Oldest' },
	{ value: 'A-Z', label: 'A-Z' },
	{ value: 'Z-A', label: 'Z-A' },
]

const filteredItems = computed(() => {
	let items = contentItems.value

	// Filter by search query
	if (searchQuery.value) {
		const query = searchQuery.value.toLowerCase()
		items = items.filter(
			(item) =>
				item.project.title.toLowerCase().includes(query) ||
				item.owner?.name.toLowerCase().includes(query),
		)
	}

	// Sort items
	if (sortType.value === 'Oldest') {
		items = [...items].reverse()
	} else if (sortType.value === 'A-Z') {
		items = [...items].sort((a, b) => a.project.title.localeCompare(b.project.title))
	} else if (sortType.value === 'Z-A') {
		items = [...items].sort((a, b) => b.project.title.localeCompare(a.project.title))
	}

	return items
})

const totalPages = computed(() => Math.ceil(filteredItems.value.length / itemsPerPage))

const paginatedItems = computed(() => {
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredItems.value.slice(start, end)
})

const totalModsCount = computed(() => contentItems.value.length)

function goToPage(page: number) {
	currentPage.value = page
}

function handleSearch() {
	currentPage.value = 1
}

const deleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; modKey: string }) =>
		client.archon.content_v0.delete(serverId, { path }),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (err, { modKey }) => {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to remove content',
		})
		changingMods.value.delete(modKey)
	},
})

const toggleMutation = useMutation({
	mutationFn: async ({ mod, modKey }: { mod: Archon.Content.v0.Mod; modKey: string }) => {
		const folder = `${type.value.toLowerCase()}s`
		const currentFilename = mod.disabled ? `${mod.filename}.disabled` : mod.filename
		const newFilename = mod.disabled ? mod.filename : `${mod.filename}.disabled`
		await client.kyros.files_v0.moveFileOrFolder(
			`/${folder}/${currentFilename}`,
			`/${folder}/${newFilename}`,
		)
		return { newDisabled: !mod.disabled, modKey }
	},
	onSuccess: ({ newDisabled, modKey }) => {
		// Optimistically update the local cache immediately
		// Archon may take time to sync after Kyros renames the file
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v0.Mod[] | undefined) => {
				if (!oldData) return oldData
				return oldData.map((m) =>
					getStableModKey(m) === modKey ? { ...m, disabled: newDisabled } : m,
				)
			},
		)
		changingMods.value.delete(modKey)
		// Also invalidate to eventually get the real server state
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (_err, { mod, modKey }) => {
		addNotification({
			type: 'error',
			text: `Failed to toggle ${friendlyModName(mod)}`,
		})
		changingMods.value.delete(modKey)
	},
})

// Update mutation
const _updateMutation = useMutation({
	mutationFn: ({
		replace,
		project_id,
		version_id,
	}: {
		replace: string
		project_id: string
		version_id: string
		modKey: string
	}) => client.archon.content_v0.update(serverId, { replace, project_id, version_id }),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
		addNotification({
			type: 'success',
			text: 'Content updated successfully',
		})
	},
	onError: (err, { modKey }) => {
		addNotification({
			type: 'error',
			text: err instanceof Error ? err.message : 'Failed to update content',
		})
		changingMods.value.delete(modKey)
	},
})

function handleToggleEnabled(item: ContentItem, _value: boolean) {
	const mod = item._mod
	const modKey = getStableModKey(mod)
	changingMods.value.add(modKey)

	toggleMutation.mutate({ mod, modKey })
}

function handleDelete(item: ContentItem) {
	const mod = item._mod
	const modKey = getStableModKey(mod)
	changingMods.value.add(modKey)

	deleteMutation.mutate(
		{ path: `/${type.value.toLowerCase()}s/${mod.filename}`, modKey },
		{
			onSettled: () => {
				changingMods.value.delete(modKey)
			},
		},
	)
}

// TODO: implement update checking
// function handleUpdate(item: ContentItem) {
// 	const mod = item._mod
// 	if (!mod.project_id || !mod.version_id) {
// 		addNotification({
// 			type: 'error',
// 			text: 'Cannot update content without project information',
// 		})
// 		return
// 	}
//
// 	// TODO: Implement version selection modal or auto-update to latest
// 	console.log('Update:', item.project.title)
// }

// TODO: implement modpack update
// function handleModpackUpdate() {
// 	// TODO: Implement modpack update (needs version selection)
// 	console.log('Modpack update')
// }

function handleModpackContent() {
	// Navigate to modpack project page
	if (modpackProject.value?.slug) {
		window.location.href = `/project/${modpackProject.value.slug}`
	}
}

function handleModpackUnlink() {
	// Backend not implemented - show notification instead of modal
	addNotification({
		type: 'warning',
		text: 'Modpack unlinking is not yet available',
	})
	// modpackUnlinkModal.value?.show()
}

function handleModpackUnlinkConfirm() {
	// Backend not implemented
	addNotification({
		type: 'warning',
		text: 'Modpack unlinking is not yet available',
	})
}

function handleBrowseContent() {
	window.location.href = `/discover/${type.value.toLowerCase()}s?sid=${serverId}`
}

function handleUploadFiles() {
	// TODO: Implement file upload (integrate FileUploadDropdown)
	console.log('Upload files')
}
</script>

<template>
	<div class="flex flex-col gap-4">
		<!-- Loading state -->
		<div
			v-if="isLoadingContent"
			class="flex min-h-[50vh] w-full flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading {{ type.toLowerCase() }}s...
		</div>

		<!-- Error state -->
		<div
			v-else-if="contentError"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="universal-card flex flex-col items-center gap-4 p-6">
				<h2 class="m-0 text-xl font-bold">Failed to load content</h2>
				<p class="text-secondary">{{ contentError.message }}</p>
				<ButtonStyled color="brand">
					<button @click="() => refetch()">Retry</button>
				</ButtonStyled>
			</div>
		</div>

		<!-- Content loaded -->
		<template v-else>
			<ContentModpackCard
				v-if="modpack"
				:project="modpack.project"
				:version="modpack.version"
				:owner="modpack.owner"
				:categories="modpack.categories"
				@content="handleModpackContent"
				@unlink="handleModpackUnlink"
			/>
			<!-- @update="handleModpackUpdate" -->

			<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
				<div class="iconified-input flex-1 lg:max-w-lg">
					<SearchIcon aria-hidden="true" class="text-lg" />
					<input
						v-model="searchQuery"
						class="!h-10"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="`Search ${totalModsCount} ${type.toLowerCase()}s...`"
						@input="handleSearch"
					/>
					<ButtonStyled v-if="searchQuery" circular type="transparent" class="r-btn">
						<button @click="searchQuery = ''">
							<XIcon />
						</button>
					</ButtonStyled>
				</div>

				<div class="flex gap-2">
					<ButtonStyled color="brand">
						<button class="!h-10 flex items-center gap-2" @click="handleBrowseContent">
							<CompassIcon class="size-5" />
							<span>Browse content</span>
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button class="!h-10 !border-surface-4 !border-[1px]" @click="handleUploadFiles">
							Upload files
						</button>
					</ButtonStyled>
				</div>
			</div>

			<div class="flex flex-col justify-between gap-2 lg:flex-row lg:items-center">
				<div class="flex gap-2">
					<Combobox
						v-model="filterType"
						class="!w-[215px]"
						:options="filterOptions"
						@select="() => goToPage(1)"
					>
						<template #selected>
							<span class="flex items-center gap-2 font-semibold">
								<ListFilterIcon class="size-5 shrink-0 text-secondary" />
								<span class="text-contrast">{{ filterType }}</span>
							</span>
						</template>
					</Combobox>

					<Combobox
						v-model="sortType"
						class="!w-[192px]"
						:options="sortOptions"
						@select="() => goToPage(1)"
					>
						<template #selected>
							<span class="flex items-center gap-2 font-semibold">
								<SortAscIcon v-if="sortType === 'Oldest'" class="size-5 shrink-0 text-secondary" />
								<SortDescIcon v-else class="size-5 shrink-0 text-secondary" />
								<span class="text-contrast">{{ sortType }}</span>
							</span>
						</template>
					</Combobox>
				</div>

				<Pagination
					v-if="totalPages > 1"
					:page="currentPage"
					:count="totalPages"
					@switch-page="goToPage"
				/>
			</div>

			<div class="flex flex-col gap-4">
				<div
					v-if="paginatedItems.length === 0"
					class="universal-card flex h-24 items-center justify-center text-secondary"
				>
					No content found.
				</div>
				<ContentCard
					v-for="item in paginatedItems"
					:key="item.project.id"
					:project="item.project"
					:version="item.version"
					:owner="item.owner"
					:enabled="item.enabled"
					:disabled="changingMods.has(getStableModKey(item._mod))"
					@update:enabled="(val) => handleToggleEnabled(item, val)"
					@delete="() => handleDelete(item)"
				/>
				<!-- @update="item.hasUpdate ? () => handleUpdate(item) : undefined" -->
			</div>

			<div v-if="totalPages > 1" class="mt-4 flex justify-center">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>

			<ModpackUnlinkModal ref="modpackUnlinkModal" @unlink="handleModpackUnlinkConfirm" />
		</template>
	</div>
</template>
