<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { CompassIcon, FilterIcon, SearchIcon, SpinnerIcon, XIcon } from '@modrinth/assets'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import Fuse from 'fuse.js'
import { computed, onBeforeUnmount, ref, watch, watchSyncEffect } from 'vue'
import { onBeforeRouteLeave, useRoute } from 'vue-router'

import ButtonStyled from '../../../components/base/ButtonStyled.vue'
import FloatingActionBar from '../../../components/base/FloatingActionBar.vue'
import ProgressBar from '../../../components/base/ProgressBar.vue'
import ContentCardTable from '../../../components/instances/ContentCardTable.vue'
import ContentModpackCard from '../../../components/instances/ContentModpackCard.vue'
import ConfirmDeletionModal from '../../../components/instances/modals/ConfirmDeletionModal.vue'
import ModpackUnlinkModal from '../../../components/instances/modals/ModpackUnlinkModal.vue'
import type {
	ContentCardProject,
	ContentCardTableItem,
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
					link: `/user/${mod.owner}`,
				}
			: undefined,
		enabled: !mod.disabled,
		hasUpdate: false, // TODO: implement update checking
		_mod: mod,
	}))
})

const searchQuery = ref('')
const selectedFilters = ref<string[]>([])

// Fuse.js instance for fuzzy search
const fuse = new Fuse<ContentItem>([], {
	keys: ['project.title', 'version.file_name'],
	threshold: 0.4,
	distance: 100,
})

watchSyncEffect(() => fuse.setCollection(contentItems.value))

// Selection state
const selectedIds = ref<string[]>([])

// Bulk operations state
const isBulkOperating = ref(false)
const bulkProgress = ref(0)
const bulkTotal = ref(0)
const bulkOperation = ref<'enable' | 'disable' | 'delete' | null>(null)

const selectedItems = computed(() =>
	contentItems.value.filter((item) => selectedIds.value.includes(getStableModKey(item._mod))),
)

const _modpackUnlinkModal = ref<InstanceType<typeof ModpackUnlinkModal>>()
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()
const pendingDeletionItems = ref<ContentItem[]>([])

// Dynamic filter options
type FilterOption = {
	id: string
	label: string
}

const filterOptions = computed<FilterOption[]>(() => {
	const options: FilterOption[] = []

	// Add "Disabled" filter if there are disabled items
	if (contentItems.value.some((item) => !item.enabled)) {
		options.push({
			id: 'disabled',
			label: 'Disabled',
		})
	}

	return options
})

function toggleFilter(filterId: string) {
	const index = selectedFilters.value.indexOf(filterId)
	if (index === -1) {
		selectedFilters.value.push(filterId)
	} else {
		selectedFilters.value.splice(index, 1)
	}
}

const filteredItems = computed(() => {
	const query = searchQuery.value.trim()

	let items: ContentItem[]

	// Use Fuse.js for fuzzy search
	if (query) {
		items = fuse.search(query).map(({ item }) => item)
	} else {
		items = contentItems.value
	}

	// Apply filters if any are selected
	if (selectedFilters.value.length > 0) {
		items = items.filter((item) => {
			for (const filter of selectedFilters.value) {
				if (filter === 'disabled' && !item.enabled) return true
			}
			return false
		})
	}

	return items
})

const totalModsCount = computed(() => contentItems.value.length)

// Table items for ContentCardTable
const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredItems.value.map((item) => ({
		id: getStableModKey(item._mod),
		project: item.project,
		projectLink: item._mod.project_id ? `/mod/${item._mod.project_id}` : undefined,
		version: item.version,
		owner: item.owner,
		enabled: item.enabled,
		disabled: changingMods.value.has(getStableModKey(item._mod)),
	})),
)

function handleSearch() {
	// Debounce is handled externally if needed
}

const deleteMutation = useMutation({
	mutationFn: ({ path }: { path: string; modKey: string }) =>
		client.archon.content_v0.delete(serverId, { path }),
	onMutate: async ({ modKey }) => {
		// Cancel any outgoing refetches
		await queryClient.cancelQueries({ queryKey: contentQueryKey.value })

		// Snapshot previous value
		const previousData = queryClient.getQueryData<Archon.Content.v0.Mod[]>(contentQueryKey.value)

		// Optimistically remove the item
		queryClient.setQueryData(
			contentQueryKey.value,
			(oldData: Archon.Content.v0.Mod[] | undefined) => {
				if (!oldData) return oldData
				return oldData.filter((m) => getStableModKey(m) !== modKey)
			},
		)

		// Clear selection for deleted item
		selectedIds.value = selectedIds.value.filter((id) => id !== modKey)

		return { previousData }
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: contentQueryKey.value })
	},
	onError: (err, { modKey }, context) => {
		// Rollback to previous data
		if (context?.previousData) {
			queryClient.setQueryData(contentQueryKey.value, context.previousData)
		}
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

function performDelete(item: ContentItem) {
	const mod = item._mod
	const modKey = getStableModKey(mod)
	deleteMutation.mutate({ path: `/${type.value.toLowerCase()}s/${mod.filename}`, modKey })
}

function handleDelete(item: ContentItem) {
	pendingDeletionItems.value = [item]
	confirmDeletionModal.value?.show()
}

// ID-based event handlers for ContentCardTable
function handleToggleEnabledById(id: string, value: boolean) {
	const item = contentItems.value.find((i) => getStableModKey(i._mod) === id)
	if (item) handleToggleEnabled(item, value)
}

function handleDeleteById(id: string) {
	const item = contentItems.value.find((i) => getStableModKey(i._mod) === id)
	if (item) handleDelete(item)
}

function showBulkDeleteModal() {
	pendingDeletionItems.value = [...selectedItems.value]
	confirmDeletionModal.value?.show()
}

async function confirmDelete() {
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []

	if (itemsToDelete.length === 0) return

	if (itemsToDelete.length === 1) {
		performDelete(itemsToDelete[0])
		return
	}

	// Bulk delete with progress
	isBulkOperating.value = true
	bulkOperation.value = 'delete'
	bulkTotal.value = itemsToDelete.length
	bulkProgress.value = 0

	for (const item of itemsToDelete) {
		performDelete(item)
		bulkProgress.value++
		await new Promise((resolve) => setTimeout(resolve, 250))
	}

	// Clear selection after bulk operation
	selectedIds.value = []

	isBulkOperating.value = false
	bulkOperation.value = null
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

async function bulkToggleAll(enable: boolean) {
	const itemsToToggle = selectedItems.value.filter((item) => item.enabled !== enable)
	if (itemsToToggle.length === 0) {
		addNotification({
			type: 'info',
			text: `All selected ${type.value.toLowerCase()}s are already ${enable ? 'enabled' : 'disabled'}`,
		})
		return
	}

	isBulkOperating.value = true
	bulkOperation.value = enable ? 'enable' : 'disable'
	bulkTotal.value = itemsToToggle.length
	bulkProgress.value = 0

	for (const item of itemsToToggle) {
		handleToggleEnabled(item, enable)
		bulkProgress.value++
		await new Promise((resolve) => setTimeout(resolve, 250)) // 250ms gap between requests cos archon will shit itself lol
	}

	// Clear selection after bulk operation
	selectedIds.value = []

	isBulkOperating.value = false
	bulkOperation.value = null
}

function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (isBulkOperating.value) {
		e.preventDefault()
		return ''
	}
}

// @ts-expect-error client exists
if (import.meta.client) {
	watch(isBulkOperating, (operating) => {
		if (operating) {
			window.addEventListener('beforeunload', handleBeforeUnload)
		} else {
			window.removeEventListener('beforeunload', handleBeforeUnload)
		}
	})

	onBeforeUnmount(() => {
		window.removeEventListener('beforeunload', handleBeforeUnload)
	})
}

onBeforeRouteLeave(() => {
	// @ts-expect-error client exists
	if (isBulkOperating.value && import.meta.client) {
		return window.confirm('A bulk operation is in progress. Are you sure you want to leave?')
	}
	return true
})
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

			<div
				v-if="filterOptions.length > 0"
				class="flex flex-col justify-between gap-2 lg:flex-row lg:items-center"
			>
				<div class="flex flex-wrap items-center gap-1.5">
					<FilterIcon class="size-5 text-secondary" />
					<button
						class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-colors"
						:class="
							selectedFilters.length === 0
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
						@click="selectedFilters = []"
					>
						All
					</button>
					<button
						v-for="option in filterOptions"
						:key="option.id"
						class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-colors"
						:class="
							selectedFilters.includes(option.id)
								? 'border-green bg-brand-highlight text-brand'
								: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
						"
						@click="toggleFilter(option.id)"
					>
						{{ option.label }}
					</button>
				</div>
			</div>

			<!-- Content table -->
			<ContentCardTable
				v-model:selected-ids="selectedIds"
				:items="tableItems"
				:show-selection="true"
				@update:enabled="handleToggleEnabledById"
				@delete="handleDeleteById"
			>
				<template #empty>No content found.</template>
			</ContentCardTable>

			<ModpackUnlinkModal ref="modpackUnlinkModal" @unlink="handleModpackUnlinkConfirm" />
			<ConfirmDeletionModal
				ref="confirmDeletionModal"
				:count="pendingDeletionItems.length"
				:item-type="type.toLowerCase()"
				@delete="confirmDelete"
			/>
		</template>

		<FloatingActionBar :shown="selectedItems.length > 0 || isBulkOperating">
			<template v-if="!isBulkOperating">
				<span class="text-sm font-medium text-contrast"> {{ selectedItems.length }} selected </span>
				<div class="ml-auto flex items-center gap-2">
					<ButtonStyled>
						<button @click="bulkToggleAll(true)">Enable</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="bulkToggleAll(false)">Disable</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="showBulkDeleteModal">Delete</button>
					</ButtonStyled>
				</div>
			</template>
			<template v-else>
				<div class="flex flex-1 flex-col gap-2">
					<span class="text-sm font-medium text-contrast">
						{{
							bulkOperation === 'enable'
								? 'Enabling'
								: bulkOperation === 'disable'
									? 'Disabling'
									: 'Deleting'
						}}
						{{ type.toLowerCase() }}s... ({{ bulkProgress }}/{{ bulkTotal }})
					</span>
					<ProgressBar full-width :progress="bulkProgress" :max="bulkTotal" color="brand" />
				</div>
			</template>
		</FloatingActionBar>
	</div>
</template>
