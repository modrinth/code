<template>
	<div class="flex flex-col gap-4">
		<!-- Loading state -->
		<div
			v-if="loading"
			class="flex min-h-[50vh] w-full flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading content...
		</div>

		<template v-else>
			<ContentModpackCard
				v-if="linkedModpackProject"
				:project="linkedModpackProject"
				:project-link="`/project/${linkedModpackProject.slug ?? linkedModpackProject.id}`"
				:version="linkedModpackVersion ?? undefined"
				:owner="linkedModpackOwner ?? undefined"
				:categories="linkedModpackCategories"
				:has-update="linkedModpackHasUpdate"
				@update="handleModpackUpdate"
				@content="handleModpackContent"
				@unlink="confirmUnlinkModal?.show()"
			/>

			<template v-if="projects.length > 0">
				<span v-if="linkedModpackProject" class="text-xl text-contrast font-semibold"
					>Additional content</span
				>
				<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
					<div class="iconified-input flex-1">
						<SearchIcon aria-hidden="true" class="text-lg" />
						<input
							v-model="searchQuery"
							class="!h-10"
							autocomplete="off"
							spellcheck="false"
							type="text"
							:placeholder="`Search ${projects.length} project${projects.length === 1 ? '' : 's'}...`"
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
							<button class="!h-10 !border-button-bg !border-[1px]" @click="handleUploadFiles">
								<FolderOpenIcon class="size-5" />
								Upload files
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col justify-between gap-2 lg:flex-row lg:items-center">
					<div class="flex flex-wrap items-center gap-1.5">
						<FilterIcon class="size-5 text-secondary" />
						<button
							class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
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
							class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
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

					<div class="flex items-center gap-2">
						<ButtonStyled
							v-if="!isPackLocked && hasOutdatedProjects"
							color="green"
							type="transparent"
							color-fill="text"
							hover-color-fill="background"
						>
							<button :disabled="isBulkOperating" @click="promptUpdateAll">
								<DownloadIcon />
								Update all
							</button>
						</ButtonStyled>

						<ButtonStyled type="transparent" hover-color-fill="none">
							<button :disabled="refreshingProjects" @click="refreshProjects">
								<RefreshCwIcon :class="refreshingProjects ? 'animate-spin' : ''" />
								Refresh
							</button>
						</ButtonStyled>
					</div>
				</div>

				<ContentCardTable
					v-model:selected-ids="selectedIds"
					:items="tableItems"
					:show-selection="true"
					@update:enabled="handleToggleEnabled"
					@delete="handleDelete"
					@update="handleUpdate"
				>
					<template #empty>
						<span>No content found.</span>
					</template>
				</ContentCardTable>
			</template>

			<div v-else class="mx-auto flex flex-col justify-center gap-8 p-6 text-center">
				<EmptyIllustration class="h-[80px] w-auto" />
				<div class="-mt-4 flex flex-col gap-4">
					<div class="flex flex-col items-center gap-1.5">
						<span class="text-2xl font-semibold text-contrast">No extra content added</span>
						<span class="text-primary">You can add content on top of a modpack!</span>
					</div>
					<div class="mx-auto flex gap-2">
						<ButtonStyled type="outlined">
							<button class="!h-10 !border-button-bg !border-[1px]" @click="handleUploadFiles">
								<FolderOpenIcon class="size-5" />
								Upload files
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button class="!h-10 flex items-center gap-2" @click="handleBrowseContent">
								<CompassIcon class="size-5" />
								<span>Browse content</span>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</template>

		<FloatingActionBar :shown="selectedItems.length > 0 || isBulkOperating">
			<template v-if="!isBulkOperating">
				<!-- Left section: count + clear -->
				<div class="flex items-center gap-0.5">
					<span class="px-4 py-2.5 text-base font-semibold text-contrast">
						{{ selectedItems.length }} project{{ selectedItems.length === 1 ? '' : 's' }} selected
					</span>
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled type="transparent">
						<button class="!text-primary" @click="clearSelection">Clear</button>
					</ButtonStyled>
				</div>

				<!-- Right section: actions -->
				<div class="ml-auto flex items-center gap-0.5">
					<ButtonStyled
						v-if="!isPackLocked && selectedItems.some((m) => m.has_update)"
						type="transparent"
						color="green"
						color-fill="text"
						hover-color-fill="background"
					>
						<button @click="promptUpdateSelected">
							<DownloadIcon />
							Update
						</button>
					</ButtonStyled>
					<ButtonStyled type="transparent">
						<OverflowMenu
							:options="[
								{ id: 'share-names', action: shareNames },
								{ id: 'share-file-names', action: shareFileNames },
								{ id: 'share-urls', action: shareUrls },
								{ id: 'share-markdown', action: shareMarkdown },
							]"
						>
							<ShareIcon />
							Share
							<DropdownIcon />
							<template #share-names>
								<TextInputIcon />
								Project names
							</template>
							<template #share-file-names>
								<FileIcon />
								File names
							</template>
							<template #share-urls>
								<LinkIcon />
								Project links
							</template>
							<template #share-markdown>
								<CodeIcon />
								Markdown links
							</template>
						</OverflowMenu>
					</ButtonStyled>
					<ButtonStyled v-if="selectedItems.every((m) => !m.enabled)" type="transparent">
						<button @click="bulkEnable">
							<PowerIcon />
							Enable
						</button>
					</ButtonStyled>
					<ButtonStyled v-else type="transparent">
						<button @click="bulkDisable">
							<PowerOffIcon />
							Disable
						</button>
					</ButtonStyled>
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled
						type="transparent"
						color="red"
						color-fill="text"
						hover-color-fill="background"
					>
						<button @click="bulkDelete">
							<TrashIcon />
							Delete
						</button>
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
									: bulkOperation === 'update'
										? 'Updating'
										: 'Deleting'
						}}
						content... ({{ bulkProgress }}/{{ bulkTotal }})
					</span>
					<ProgressBar full-width :progress="bulkProgress" :max="bulkTotal" color="brand" />
				</div>
			</template>
		</FloatingActionBar>

		<ShareModalWrapper
			ref="shareModal"
			share-title="Sharing modpack content"
			share-text="Check out the projects I'm using in my modpack!"
			:open-in-new-tab="false"
		/>
		<ModpackContentModal
			ref="modpackContentModal"
			:modpack-name="linkedModpackProject?.title"
			:modpack-icon-url="linkedModpackProject?.icon_url ?? undefined"
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
			:is-modpack="updatingModpack"
			:project-icon-url="
				updatingModpack ? linkedModpackProject?.icon_url : updatingProject?.project?.icon_url
			"
			:project-name="
				updatingModpack
					? (linkedModpackProject?.title ?? 'Modpack')
					: (updatingProject?.project?.title ?? updatingProject?.file_name)
			"
			:loading="loadingVersions"
			:loading-changelog="loadingChangelog"
			@update="handleModalUpdate"
			@version-select="handleVersionSelect"
		/>
		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			item-type="project"
			@delete="confirmDelete"
		/>
		<ConfirmUnlinkModal ref="confirmUnlinkModal" @unlink="unpairProfile" />
		<ConfirmBulkUpdateModal
			ref="confirmBulkUpdateModal"
			:count="pendingUpdateCount"
			@update="confirmBulkUpdate"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	EmptyIllustration,
	FileIcon,
	FilterIcon,
	FolderOpenIcon,
	LinkIcon,
	PowerIcon,
	PowerOffIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmBulkUpdateModal,
	ConfirmDeletionModal,
	ConfirmUnlinkModal,
	ContentCardTable,
	type ContentCardTableItem,
	type ContentItem,
	ContentModpackCard,
	type ContentModpackCardCategory,
	type ContentModpackCardProject,
	type ContentModpackCardVersion,
	type ContentOwner,
	ContentUpdaterModal,
	FloatingActionBar,
	injectNotificationManager,
	ModpackContentModal,
	OverflowMenu,
	type OverflowMenuOption,
	ProgressBar,
} from '@modrinth/ui'
import { formatProjectType } from '@modrinth/utils'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import Fuse from 'fuse.js'
import { computed, nextTick, onBeforeUnmount, onUnmounted, ref, watch, watchSyncEffect } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'

import { TextInputIcon } from '@/assets/icons'
import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { get_project_versions, get_version } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import {
	add_project_from_path,
	edit,
	get,
	get_content_items,
	get_linked_modpack_content,
	get_linked_modpack_info,
	remove_project,
	toggle_disable_project,
	update_managed_modrinth_version,
	update_project,
} from '@/helpers/profile'
import { get_categories } from '@/helpers/tags.js'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import { highlightModInProfile } from '@/helpers/utils.js'
import { installVersionDependencies } from '@/store/install'

const { handleError } = injectNotificationManager()
const router = useRouter()

const props = defineProps<{
	instance: GameInstance
	versions: Labrinth.Versions.v2.Version[]
}>()

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

	for (const file of files) {
		await add_project_from_path(
			props.instance.path,
			(file as { path?: string }).path ?? file,
		).catch(handleError)
	}
	await initProjects()
}

const loading = ref(true)
const projects = ref<ContentItem[]>([])
const searchQuery = ref('')
const selectedFilters = ref<string[]>([])
const refreshingProjects = ref(false)

// Linked modpack state
const linkedModpackProject = ref<ContentModpackCardProject | null>(null)
const linkedModpackVersion = ref<ContentModpackCardVersion | null>(null)
const linkedModpackOwner = ref<ContentOwner | null>(null)
const linkedModpackCategories = ref<ContentModpackCardCategory[]>([])
const linkedModpackHasUpdate = ref(false)
const linkedModpackUpdateVersionId = ref<string | null>(null)

// Selection state
const selectedIds = ref<string[]>([])
const changingMods = ref(new Set<string>())

// Bulk operations state
const isBulkOperating = ref(false)
const bulkProgress = ref(0)
const bulkTotal = ref(0)
const bulkOperation = ref<'enable' | 'disable' | 'delete' | 'update' | null>(null)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const contentUpdaterModal = ref<InstanceType<typeof ContentUpdaterModal> | null>()
const modpackContentModal = ref<InstanceType<typeof ModpackContentModal> | null>()
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal> | null>()
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal> | null>()
const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal> | null>()

// Pending deletion state
const pendingDeletionItems = ref<ContentItem[]>([])
const pendingBulkUpdateItems = ref<ContentItem[]>([])
const pendingUpdateCount = computed(() => pendingBulkUpdateItems.value.length)

// State for content updater modal
const updatingProject = ref<ContentItem | null>(null)
const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
const loadingVersions = ref(false)
const loadingChangelog = ref(false)
const updatingModpack = ref(false) // true when updating the linked modpack, false for content items

const refreshInterval: ReturnType<typeof setInterval> | null = null

const isPackLocked = computed(() => props.instance?.linked_data?.locked ?? false)

const hasOutdatedProjects = computed(() => projects.value.some((p) => p.has_update))

const selectedItems = computed(() =>
	projects.value.filter((item) => selectedIds.value.includes(item.file_name)),
)

// Dynamic filter options based on project types
type FilterOption = {
	id: string
	label: string
}

const filterOptions = computed<FilterOption[]>(() => {
	const options: FilterOption[] = []

	// Get frequency of each project type
	const frequency = projects.value.reduce((map: Record<string, number>, item) => {
		map[item.project_type] = (map[item.project_type] || 0) + 1
		return map
	}, {})

	// Sort types by frequency (most common first)
	const types = Object.keys(frequency).sort((a, b) => frequency[b] - frequency[a])

	// Add type filters
	for (const type of types) {
		options.push({
			id: type,
			label: formatProjectType(type) + 's',
		})
	}

	// Add "Updates" filter if there are outdated mods and pack is not locked
	if (!isPackLocked.value && projects.value.some((m) => m.has_update)) {
		options.push({
			id: 'updates',
			label: 'Updates',
		})
	}

	// Add "Disabled" filter if there are disabled mods
	if (projects.value.some((m) => !m.enabled)) {
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

async function refreshProjects() {
	if (refreshingProjects.value) return
	refreshingProjects.value = true
	try {
		await initProjects('must_revalidate')
	} finally {
		refreshingProjects.value = false
	}
}

function promptUpdateAll() {
	const itemsToUpdate = projects.value.filter((item) => item.has_update)
	if (itemsToUpdate.length === 0) return
	pendingBulkUpdateItems.value = itemsToUpdate
	confirmBulkUpdateModal.value?.show()
}

function promptUpdateSelected() {
	const itemsToUpdate = selectedItems.value.filter((item) => item.has_update)
	if (itemsToUpdate.length === 0) return
	pendingBulkUpdateItems.value = itemsToUpdate
	confirmBulkUpdateModal.value?.show()
}

async function confirmBulkUpdate() {
	const itemsToUpdate = pendingBulkUpdateItems.value
	if (itemsToUpdate.length === 0) return

	isBulkOperating.value = true
	bulkOperation.value = 'update'
	bulkTotal.value = itemsToUpdate.length
	bulkProgress.value = 0

	for (const item of itemsToUpdate) {
		await updateProject(item)
		bulkProgress.value++
	}

	clearSelection()
	isBulkOperating.value = false
	bulkOperation.value = null
	pendingBulkUpdateItems.value = []

	trackEvent('InstanceUpdateAll', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		count: itemsToUpdate.length,
		selected: false,
	})
}

async function updateAll() {
	const itemsToUpdate = projects.value.filter((item) => item.has_update)
	if (itemsToUpdate.length === 0) return

	isBulkOperating.value = true
	bulkOperation.value = 'update'
	bulkTotal.value = itemsToUpdate.length
	bulkProgress.value = 0

	for (const item of itemsToUpdate) {
		await updateProject(item)
		bulkProgress.value++
	}

	isBulkOperating.value = false
	bulkOperation.value = null

	trackEvent('InstanceUpdateAll', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		count: itemsToUpdate.length,
		selected: false,
	})
}

// Clean up invalid filters when options change
watch(filterOptions, () => {
	selectedFilters.value = selectedFilters.value.filter((f) =>
		filterOptions.value.some((opt) => opt.id === f),
	)
})

const fuse = new Fuse<ContentItem>([], {
	keys: ['project.title', 'owner.name', 'file_name'],
	threshold: 0.4,
	distance: 100,
})

const sortedProjects = computed(() => {
	const items = [...projects.value]
	// Sort alphabetically by project title (or file_name if no project)
	return items.sort((a, b) => {
		const nameA = a.project?.title ?? a.file_name
		const nameB = b.project?.title ?? b.file_name
		return nameA.toLowerCase().localeCompare(nameB.toLowerCase())
	})
})

watchSyncEffect(() => fuse.setCollection(sortedProjects.value))

const filteredProjects = computed(() => {
	const query = searchQuery.value.trim()

	let items: ContentItem[]

	if (query) {
		items = fuse.search(query).map(({ item }) => item)
	} else {
		items = sortedProjects.value
	}

	// Apply filters if any are selected
	if (selectedFilters.value.length > 0) {
		items = items.filter((item) => {
			// Check if item matches any of the selected filters
			for (const filter of selectedFilters.value) {
				// Special filters
				if (filter === 'updates' && item.has_update) return true
				if (filter === 'disabled' && !item.enabled) return true
				// Type filters (mod, shader, resourcepack, etc.)
				if (item.project_type === filter) return true
			}
			return false
		})
	}

	return items
})

function getOverflowOptions(item: ContentItem): OverflowMenuOption[] {
	const options: OverflowMenuOption[] = [
		{
			id: 'Show file',
			action: () => highlightModInProfile(props.instance.path, item.file_path),
		},
	]

	if (item.project?.slug) {
		options.push({
			id: 'Copy link',
			action: async () => {
				await navigator.clipboard.writeText(
					`https://modrinth.com/${item.project_type}/${item.project?.slug}`,
				)
			},
		})
	}

	return options
}

// Table items for ContentCardTable
const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredProjects.value.map((item) => ({
		id: item.file_name,
		project: item.project ?? {
			id: item.file_name,
			slug: null,
			title: item.file_name.replace('.disabled', ''),
			icon_url: null,
		},
		projectLink: item.project?.id ? `/project/${item.project.id}` : undefined,
		version: item.version ?? {
			id: item.file_name,
			version_number: 'Unknown',
			file_name: item.file_name,
		},
		owner: item.owner
			? {
					...item.owner,
					link: `https://modrinth.com/${item.owner.type}/${item.owner.id}`,
				}
			: undefined,
		enabled: item.enabled,
		disabled: changingMods.value.has(item.file_name),
		hasUpdate: !isPackLocked.value && item.has_update,
		overflowOptions: getOverflowOptions(item),
	})),
)

function handleToggleEnabled(id: string, _value: boolean) {
	const item = projects.value.find((p) => p.file_name === id)
	if (item) toggleDisableMod(item)
}

function handleDelete(id: string) {
	const item = projects.value.find((p) => p.file_name === id)
	if (item) {
		pendingDeletionItems.value = [item]
		confirmDeletionModal.value?.show()
	}
}

async function handleUpdate(id: string) {
	const item = projects.value.find((p) => p.file_name === id)
	if (!item?.has_update || !item.project?.id || !item.version?.id) return

	// Show modal immediately with loading state
	updatingProject.value = item
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(item.update_version_id ?? undefined)

	console.log('[handleUpdate] Fetching versions for project:', item.project.id)
	const versions = (await get_project_versions(item.project.id).catch((e) => {
		console.error('[handleUpdate] Error fetching versions:', e)
		return handleError(e)
	})) as Labrinth.Versions.v2.Version[] | null

	console.log('[handleUpdate] Got versions:', versions)
	loadingVersions.value = false

	if (!versions) {
		console.log('[handleUpdate] No versions returned, exiting')
		return
	}

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	console.log('[handleUpdate] Setting updatingProjectVersions:', versions.length, 'versions')
	updatingProjectVersions.value = versions
}

// Open content modal for linked modpack
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

// Open updater modal for linked modpack
async function handleModpackUpdate() {
	if (!props.instance?.linked_data?.project_id) return

	// Show modal immediately with loading state
	updatingModpack.value = true
	updatingProject.value = null
	updatingProjectVersions.value = []
	loadingVersions.value = true
	loadingChangelog.value = false

	await nextTick()

	contentUpdaterModal.value?.show(props.instance?.linked_data?.version_id ?? undefined)

	const versions = (await get_project_versions(props.instance.linked_data.project_id).catch(
		handleError,
	)) as Labrinth.Versions.v2.Version[] | null

	loadingVersions.value = false

	if (!versions) return

	versions.sort(
		(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
	)

	updatingProjectVersions.value = versions
}

// Handler for when user selects a version in the modal - fetch full version data with changelog
async function handleVersionSelect(version: Labrinth.Versions.v2.Version) {
	// If this version already has a changelog, no need to fetch
	if (version.changelog) return

	loadingChangelog.value = true

	// Fetch the full version data (includes changelog)
	const fullVersion = (await get_version(version.id, 'must_revalidate').catch(
		handleError,
	)) as Labrinth.Versions.v2.Version

	loadingChangelog.value = false

	if (!fullVersion) return

	// Update the version in our list with the full data
	// Create a new array to ensure Vue's reactivity detects the change
	const index = updatingProjectVersions.value.findIndex((v) => v.id === version.id)
	if (index !== -1) {
		const newVersions = [...updatingProjectVersions.value]
		newVersions[index] = fullVersion
		updatingProjectVersions.value = newVersions
	}
}

// Project operations
async function toggleDisableMod(mod: ContentItem) {
	// Skip if already processing this mod
	if (changingMods.value.has(mod.file_name)) return

	changingMods.value.add(mod.file_name)

	try {
		mod.file_path = await toggle_disable_project(props.instance.path, mod.file_path)
		mod.enabled = !mod.enabled

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
	}

	changingMods.value.delete(mod.file_name)
}

async function removeMod(mod: ContentItem) {
	await remove_project(props.instance.path, mod.file_path).catch(handleError)
	projects.value = projects.value.filter((x) => mod.file_path !== x.file_path)
	selectedIds.value = selectedIds.value.filter((id) => id !== mod.file_name)

	trackEvent('InstanceProjectRemove', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		id: mod.project?.id,
		name: mod.project?.title ?? mod.file_name,
		project_type: mod.project_type,
	})
}

async function updateProject(mod: ContentItem) {
	changingMods.value.add(mod.file_name)

	try {
		const newPath = await update_project(props.instance.path, mod.file_path)
		mod.file_path = newPath

		if (mod.update_version_id) {
			const versionData = await get_version(mod.update_version_id, 'must_revalidate').catch(
				handleError,
			)

			if (versionData) {
				const profile = await get(props.instance.path).catch(handleError)

				if (profile) {
					await installVersionDependencies(profile, versionData).catch(handleError)
				}
			}
		}

		mod.has_update = false
		if (mod.version && mod.update_version_id) {
			mod.version.id = mod.update_version_id
		}
		mod.update_version_id = null

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.project?.id,
			name: mod.project?.title ?? mod.file_name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
	}

	changingMods.value.delete(mod.file_name)
}

// Handler for ContentUpdaterModal update event
async function handleModalUpdate(selectedVersion: Labrinth.Versions.v2.Version) {
	if (updatingModpack.value) {
		// Handle modpack update
		if (!props.instance?.path) return

		await update_managed_modrinth_version(props.instance.path, selectedVersion.id)
		await initProjects()

		// Clear the modal state
		updatingModpack.value = false
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	} else if (updatingProject.value) {
		// Handle content item update
		const mod = updatingProject.value

		// Update the mod's update_version_id to the selected version
		mod.update_version_id = selectedVersion.id

		// Perform the update
		await updateProject(mod)

		// Clear the modal state
		updatingProject.value = null
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	}
}

// Bulk operations
async function bulkEnable() {
	const itemsToToggle = selectedItems.value.filter((item) => !item.enabled)
	if (itemsToToggle.length === 0) return

	isBulkOperating.value = true
	bulkOperation.value = 'enable'
	bulkTotal.value = itemsToToggle.length
	bulkProgress.value = 0

	for (const item of itemsToToggle) {
		await toggleDisableMod(item)
		bulkProgress.value++
	}

	clearSelection()
	isBulkOperating.value = false
	bulkOperation.value = null
}

async function bulkDisable() {
	const itemsToToggle = selectedItems.value.filter((item) => item.enabled)
	if (itemsToToggle.length === 0) return

	isBulkOperating.value = true
	bulkOperation.value = 'disable'
	bulkTotal.value = itemsToToggle.length
	bulkProgress.value = 0

	for (const item of itemsToToggle) {
		await toggleDisableMod(item)
		bulkProgress.value++
	}

	clearSelection()
	isBulkOperating.value = false
	bulkOperation.value = null
}

function bulkDelete() {
	const itemsToDelete = [...selectedItems.value]
	if (itemsToDelete.length === 0) return

	pendingDeletionItems.value = itemsToDelete
	confirmDeletionModal.value?.show()
}

async function confirmDelete() {
	const itemsToDelete = pendingDeletionItems.value
	if (itemsToDelete.length === 0) return

	if (itemsToDelete.length === 1) {
		// Single item deletion
		await removeMod(itemsToDelete[0])
	} else {
		// Bulk deletion
		isBulkOperating.value = true
		bulkOperation.value = 'delete'
		bulkTotal.value = itemsToDelete.length
		bulkProgress.value = 0

		for (const item of itemsToDelete) {
			await removeMod(item)
			bulkProgress.value++
		}

		isBulkOperating.value = false
		bulkOperation.value = null
	}

	pendingDeletionItems.value = []
}

async function updateSelected() {
	const itemsToUpdate = selectedItems.value.filter((item) => item.has_update)
	if (itemsToUpdate.length === 0) return

	isBulkOperating.value = true
	bulkOperation.value = 'update'
	bulkTotal.value = itemsToUpdate.length
	bulkProgress.value = 0

	for (const item of itemsToUpdate) {
		await updateProject(item)
		bulkProgress.value++
	}

	clearSelection()
	isBulkOperating.value = false
	bulkOperation.value = null

	trackEvent('InstanceUpdateAll', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		count: itemsToUpdate.length,
		selected: true,
	})
}

function clearSelection() {
	selectedIds.value = []
}

// Share functions
async function shareNames() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(items.map((x) => x.project?.title ?? x.file_name).join('\n'))
}

async function shareFileNames() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(items.map((x) => x.file_name).join('\n'))
}

async function shareUrls() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(
		items
			.filter((x) => x.project?.slug)
			.map((x) => `https://modrinth.com/${x.project_type}/${x.project?.slug}`)
			.join('\n'),
	)
}

async function shareMarkdown() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(
		items
			.map((x) => {
				const name = x.project?.title ?? x.file_name
				if (x.project?.slug) {
					return `[${name}](https://modrinth.com/${x.project_type}/${x.project.slug})`
				}
				return name
			})
			.join('\n'),
	)
}

// Unlink the modpack from this profile
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

// Initialize projects using get_content_items (handles enrichment on backend)
async function initProjects(cacheBehaviour?: CacheBehaviour) {
	if (!props.instance) return

	// Fetch content items and linked modpack info in parallel
	const [contentItems, modpackInfo, allCategories] = await Promise.all([
		get_content_items(props.instance.path, cacheBehaviour).catch(handleError),
		get_linked_modpack_info(props.instance.path, cacheBehaviour).catch(handleError),
		get_categories().catch(handleError),
	])

	if (!contentItems) {
		loading.value = false
		return
	}

	projects.value = contentItems

	// Set linked modpack data from backend response
	if (modpackInfo) {
		linkedModpackProject.value = {
			...modpackInfo.project,
			slug: modpackInfo.project.slug ?? modpackInfo.project.id,
			icon_url: modpackInfo.project.icon_url ?? undefined,
		}
		linkedModpackVersion.value = {
			...modpackInfo.version,
			date_published: modpackInfo.version.date_published.toString(),
		}
		linkedModpackOwner.value = modpackInfo.owner
			? {
					...modpackInfo.owner,
					avatar_url: modpackInfo.owner.avatar_url ?? undefined,
				}
			: null

		linkedModpackHasUpdate.value = modpackInfo.has_update
		linkedModpackUpdateVersionId.value = modpackInfo.update_version_id

		// Map categories to full category objects
		if (allCategories && modpackInfo.project.categories) {
			const seen = new Set<string>()
			linkedModpackCategories.value = allCategories
				.filter((cat: { name: string }) => {
					if (modpackInfo.project.categories.includes(cat.name) && !seen.has(cat.name)) {
						seen.add(cat.name)
						return true
					}
					return false
				})
				.map((cat: { name: string }) => ({
					...cat,
					name: cat.name.charAt(0).toUpperCase() + cat.name.slice(1),
				}))
		} else {
			linkedModpackCategories.value = []
		}
	} else {
		linkedModpackProject.value = null
		linkedModpackVersion.value = null
		linkedModpackOwner.value = null
		linkedModpackCategories.value = []
		linkedModpackHasUpdate.value = false
		linkedModpackUpdateVersionId.value = null
	}

	loading.value = false
}

// Lifecycle
await initProjects()

// Drag & drop
const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
	if (event.payload.type !== 'drop' || !props.instance) return

	for (const file of event.payload.paths) {
		if (file.endsWith('.mrpack')) continue
		await add_project_from_path(props.instance.path, file).catch(handleError)
	}
	await initProjects()
})

// Profile listener
const unlistenProfiles = await profile_listener(
	async (event: { event: string; profile_path_id: string }) => {
		if (
			props.instance &&
			event.profile_path_id === props.instance.path &&
			event.event === 'synced' &&
			props.instance.install_stage !== 'pack_installing'
		) {
			await initProjects()
		}
	},
)

// Navigation guard for bulk operations
function handleBeforeUnload(e: BeforeUnloadEvent) {
	if (isBulkOperating.value) {
		e.preventDefault()
		return ''
	}
}

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

onBeforeRouteLeave(() => {
	if (isBulkOperating.value) {
		return window.confirm('A bulk operation is in progress. Are you sure you want to leave?')
	}
	return true
})

onUnmounted(() => {
	unlisten()
	unlistenProfiles()
	if (refreshInterval) {
		clearInterval(refreshInterval)
	}
})
</script>
