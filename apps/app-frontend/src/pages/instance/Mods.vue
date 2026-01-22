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

		<!-- Content loaded -->
		<template v-else-if="projects.length > 0">
			<!-- Search + Add Content row -->
			<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
				<div class="iconified-input flex-1 lg:max-w-lg">
					<SearchIcon aria-hidden="true" class="text-lg" />
					<input
						v-model="searchQuery"
						class="!h-10"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="`Search ${projects.length} project${projects.length === 1 ? '' : 's'}...`"
						@input="handleSearch"
					/>
					<ButtonStyled v-if="searchQuery" circular type="transparent" class="r-btn">
						<button @click="searchQuery = ''">
							<XIcon />
						</button>
					</ButtonStyled>
				</div>

				<AddContentButton :instance="instance" />
			</div>

			<!-- Filter + Sort row -->
			<div class="flex flex-col justify-between gap-2 lg:flex-row lg:items-center">
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
						class="rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-colors duration-200"
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
						color="brand"
						type="transparent"
						hover-color-fill="none"
					>
						<button :disabled="isBulkOperating" @click="updateAll">
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

			<!-- Content table -->
			<ContentCardTable
				v-model:selected-ids="selectedIds"
				:items="tableItems"
				:show-selection="true"
				@update:enabled="handleToggleEnabled"
				@delete="handleDelete"
				@update="handleUpdate"
			>
				<template #empty>No content found.</template>
			</ContentCardTable>
		</template>

		<!-- Empty state -->
		<div v-else class="w-full max-w-[48rem] mx-auto flex flex-col mt-6">
			<RadialHeader>
				<div class="flex items-center gap-6 w-[32rem] mx-auto">
					<img src="@/assets/sad-modrinth-bot.webp" class="h-24" />
					<span class="text-contrast font-bold text-xl">
						You haven't added any content to this instance yet.
					</span>
				</div>
			</RadialHeader>
			<div class="flex mt-4 mx-auto">
				<AddContentButton :instance="instance" />
			</div>
		</div>

		<!-- Floating action bar for batch operations -->
		<FloatingActionBar :shown="selectedItems.length > 0 || isBulkOperating">
			<template v-if="!isBulkOperating">
				<span class="text-sm font-medium text-contrast">{{ selectedItems.length }} selected</span>
				<div class="ml-auto flex items-center gap-2">
					<ButtonStyled
						v-if="!isPackLocked && selectedItems.some((m) => m.outdated)"
						color="brand"
						color-fill="text"
						hover-color-fill="text"
					>
						<button @click="updateSelected">
							<DownloadIcon />
							Update
						</button>
					</ButtonStyled>
					<ButtonStyled>
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
					<ButtonStyled v-if="selectedItems.some((m) => m.disabled)">
						<button @click="bulkEnable">
							<CheckCircleIcon />
							Enable
						</button>
					</ButtonStyled>
					<ButtonStyled v-if="selectedItems.some((m) => !m.disabled)">
						<button @click="bulkDisable">
							<SlashIcon />
							Disable
						</button>
					</ButtonStyled>
					<ButtonStyled color="red">
						<button @click="bulkDelete">
							<TrashIcon />
							Remove
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
										: 'Removing'
						}}
						content... ({{ bulkProgress }}/{{ bulkTotal }})
					</span>
					<ProgressBar full-width :progress="bulkProgress" :max="bulkTotal" color="brand" />
				</div>
			</template>
		</FloatingActionBar>

		<!-- Modals -->
		<ShareModalWrapper
			ref="shareModal"
			share-title="Sharing modpack content"
			share-text="Check out the projects I'm using in my modpack!"
			:open-in-new-tab="false"
		/>
		<ExportModal v-if="projects.length > 0" ref="exportModal" :instance="instance" />
		<ModpackVersionModal
			v-if="instance.linked_data"
			ref="modpackVersionModal"
			:instance="instance"
			:versions="props.versions"
		/>
	</div>
</template>

<script setup lang="ts">
import {
	CheckCircleIcon,
	CodeIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	FilterIcon,
	LinkIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	SlashIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	type ContentCardProject,
	ContentCardTable,
	type ContentCardTableItem,
	type ContentCardVersion,
	type ContentOwner,
	FloatingActionBar,
	injectNotificationManager,
	OverflowMenu,
	type OverflowMenuOption,
	ProgressBar,
	RadialHeader,
} from '@modrinth/ui'
import {
	formatProjectType,
	type Organization,
	type Project,
	type TeamMember,
	type Version,
} from '@modrinth/utils'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useDebounceFn } from '@vueuse/core'
import dayjs from 'dayjs'
import Fuse from 'fuse.js'
import { computed, onBeforeUnmount, onUnmounted, ref, watch, watchSyncEffect } from 'vue'
import { onBeforeRouteLeave } from 'vue-router'

import { TextInputIcon } from '@/assets/icons'
import AddContentButton from '@/components/ui/AddContentButton.vue'
import ExportModal from '@/components/ui/ExportModal.vue'
import ShareModalWrapper from '@/components/ui/modal/ShareModalWrapper.vue'
import ModpackVersionModal from '@/components/ui/ModpackVersionModal.vue'
import { trackEvent } from '@/helpers/analytics'
import {
	get_organization_many,
	get_project_many,
	get_team_many,
	get_version,
	get_version_many,
} from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events.js'
import {
	add_project_from_path,
	get,
	get_projects,
	remove_project,
	toggle_disable_project,
	update_project,
} from '@/helpers/profile.js'
import type { CacheBehaviour, ContentFile, GameInstance } from '@/helpers/types'
import { highlightModInProfile } from '@/helpers/utils.js'
import { installVersionDependencies } from '@/store/install'

const { handleError } = injectNotificationManager()

const props = defineProps<{
	instance: GameInstance
	versions: Version[]
}>()

type ProjectListEntryAuthor = {
	name: string
	slug: string
	type: 'user' | 'organization'
}

type ProjectListEntry = {
	path: string
	name: string
	slug?: string
	author: ProjectListEntryAuthor | null
	version: string | null
	file_name: string
	icon: string | undefined
	disabled: boolean
	updateVersion?: string
	outdated: boolean
	updated: dayjs.Dayjs
	project_type: string
	id?: string
	updating?: boolean
}

const loading = ref(true)
const projects = ref<ProjectListEntry[]>([])
const searchQuery = ref('')
const selectedFilters = ref<string[]>([])
const refreshingProjects = ref(false)

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
const modpackVersionModal = ref<InstanceType<typeof ModpackVersionModal> | null>()

const refreshInterval: ReturnType<typeof setInterval> | null = null

const isPackLocked = computed(() => props.instance.linked_data?.locked ?? false)

const hasOutdatedProjects = computed(() => projects.value.some((p) => p.outdated))

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
	if (!isPackLocked.value && projects.value.some((m) => m.outdated)) {
		options.push({
			id: 'updates',
			label: 'Updates',
		})
	}

	// Add "Disabled" filter if there are disabled mods
	if (projects.value.some((m) => m.disabled)) {
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

async function updateAll() {
	const itemsToUpdate = projects.value.filter((item) => item.outdated)
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

const fuse = new Fuse<ProjectListEntry>([], {
	keys: ['name', 'author.name', 'file_name'],
	threshold: 0.4,
	distance: 100,
})

const sortedProjects = computed(() => {
	const items = [...projects.value]
	// Sort by newest first
	return items.sort((a, b) => (a.updated.isAfter(b.updated) ? -1 : 1))
})

watchSyncEffect(() => fuse.setCollection(sortedProjects.value))

const filteredProjects = computed(() => {
	const query = searchQuery.value.trim()

	let items: ProjectListEntry[]

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
				if (filter === 'updates' && item.outdated) return true
				if (filter === 'disabled' && item.disabled) return true
				// Type filters (mod, shader, resourcepack, etc.)
				if (item.project_type === filter) return true
			}
			return false
		})
	}

	return items
})

const handleSearch = useDebounceFn(() => {
	// Debounce search to avoid too many updates
}, 150)

// Map functions for ContentCard props
function mapProject(item: ProjectListEntry): ContentCardProject {
	return {
		id: item.id ?? item.file_name,
		slug: item.slug ?? item.file_name,
		title: item.name,
		icon_url: item.icon,
	}
}

function mapVersion(item: ProjectListEntry): ContentCardVersion {
	return {
		id: item.file_name,
		version_number: item.version ?? 'Unknown',
		file_name: item.file_name,
	}
}

function mapOwner(item: ProjectListEntry): ContentOwner | undefined {
	if (!item.author) return undefined
	return {
		id: item.author.slug ?? item.author.name,
		name: item.author.name,
		type: item.author.type,
		link: `https://modrinth.com/${item.author.type}/${item.author.slug}`,
	}
}

function getOverflowOptions(item: ProjectListEntry): OverflowMenuOption[] {
	const options: OverflowMenuOption[] = [
		{
			id: 'Show file',
			action: () => highlightModInProfile(props.instance.path, item.path),
		},
	]

	if (item.slug) {
		options.push({
			id: 'Copy link',
			action: async () => {
				await navigator.clipboard.writeText(
					`https://modrinth.com/${item.project_type}/${item.slug}`,
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
		project: mapProject(item),
		version: mapVersion(item),
		owner: mapOwner(item),
		enabled: !item.disabled,
		disabled: changingMods.value.has(item.file_name),
		overflowOptions: getOverflowOptions(item),
	})),
)

// ID-based event handlers for ContentCardTable
function handleToggleEnabled(id: string, _value: boolean) {
	const item = projects.value.find((p) => p.file_name === id)
	if (item) toggleDisableMod(item)
}

function handleDelete(id: string) {
	const item = projects.value.find((p) => p.file_name === id)
	if (item) removeMod(item)
}

function handleUpdate(id: string) {
	const item = projects.value.find((p) => p.file_name === id)
	if (item?.outdated) updateProject(item)
}

// Project operations
async function toggleDisableMod(mod: ProjectListEntry) {
	// Skip if already processing this mod
	if (changingMods.value.has(mod.file_name)) return

	changingMods.value.add(mod.file_name)

	try {
		mod.path = await toggle_disable_project(props.instance.path, mod.path)
		mod.disabled = !mod.disabled

		trackEvent('InstanceProjectDisable', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.id,
			name: mod.name,
			project_type: mod.project_type,
			disabled: mod.disabled,
		})
	} catch (err) {
		handleError(err as Error)
	}

	changingMods.value.delete(mod.file_name)
}

async function removeMod(mod: ProjectListEntry) {
	await remove_project(props.instance.path, mod.path).catch(handleError)
	projects.value = projects.value.filter((x) => mod.path !== x.path)
	selectedIds.value = selectedIds.value.filter((id) => id !== mod.file_name)

	trackEvent('InstanceProjectRemove', {
		loader: props.instance.loader,
		game_version: props.instance.game_version,
		id: mod.id,
		name: mod.name,
		project_type: mod.project_type,
	})
}

async function updateProject(mod: ProjectListEntry) {
	mod.updating = true
	changingMods.value.add(mod.file_name)

	try {
		mod.path = await update_project(props.instance.path, mod.path).catch(handleError)

		if (mod.updateVersion) {
			const versionData = await get_version(mod.updateVersion, 'must_revalidate').catch(handleError)

			if (versionData) {
				const profile = await get(props.instance.path).catch(handleError)

				if (profile) {
					await installVersionDependencies(profile, versionData).catch(handleError)
				}
			}
		}

		mod.outdated = false
		mod.version = mod.updateVersion ?? null
		mod.updateVersion = undefined

		trackEvent('InstanceProjectUpdate', {
			loader: props.instance.loader,
			game_version: props.instance.game_version,
			id: mod.id,
			name: mod.name,
			project_type: mod.project_type,
		})
	} catch (err) {
		handleError(err as Error)
	}

	mod.updating = false
	changingMods.value.delete(mod.file_name)
}

// Bulk operations
async function bulkEnable() {
	const itemsToToggle = selectedItems.value.filter((item) => item.disabled)
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
	const itemsToToggle = selectedItems.value.filter((item) => !item.disabled)
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

async function bulkDelete() {
	const itemsToDelete = [...selectedItems.value]
	if (itemsToDelete.length === 0) return

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

async function updateSelected() {
	const itemsToUpdate = selectedItems.value.filter((item) => item.outdated)
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
	await shareModal.value?.show(items.map((x) => x.name).join('\n'))
}

async function shareFileNames() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(items.map((x) => x.file_name).join('\n'))
}

async function shareUrls() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(
		items
			.filter((x) => x.slug)
			.map((x) => `https://modrinth.com/${x.project_type}/${x.slug}`)
			.join('\n'),
	)
}

async function shareMarkdown() {
	const items = selectedItems.value.length > 0 ? selectedItems.value : projects.value
	await shareModal.value?.show(
		items
			.map((x) => {
				if (x.slug) {
					return `[${x.name}](https://modrinth.com/${x.project_type}/${x.slug})`
				}
				return x.name
			})
			.join('\n'),
	)
}

// Initialize projects
async function initProjects(cacheBehaviour?: CacheBehaviour) {
	const newProjects: ProjectListEntry[] = []

	const profileProjects = (await get_projects(props.instance.path, cacheBehaviour)) as Record<
		string,
		ContentFile
	>
	const fetchProjects: string[] = []
	const fetchVersions: string[] = []

	for (const value of Object.values(profileProjects)) {
		if (value.metadata) {
			fetchProjects.push(value.metadata.project_id)
			fetchVersions.push(value.metadata.version_id)
		}
	}

	const [modrinthProjects, modrinthVersions] = await Promise.all([
		(await get_project_many(fetchProjects).catch(handleError)) as Project[],
		(await get_version_many(fetchVersions).catch(handleError)) as Version[],
	])

	const [modrinthTeams, modrinthOrganizations] = await Promise.all([
		(await get_team_many(modrinthProjects.map((x) => x.team)).catch(handleError)) as TeamMember[][],
		(await get_organization_many(
			modrinthProjects.map((x) => x.organization).filter((x) => !!x),
		).catch(handleError)) as Organization[],
	])

	for (const [path, file] of Object.entries(profileProjects)) {
		if (file.metadata) {
			const project = modrinthProjects.find((x) => file.metadata?.project_id === x.id)
			const version = modrinthVersions.find((x) => file.metadata?.version_id === x.id)

			if (project && version) {
				const org = project.organization
					? modrinthOrganizations.find((x) => x.id === project.organization)
					: null

				const team = modrinthTeams.find((x) => x[0]?.team_id === project.team)

				let author: ProjectListEntryAuthor | null = null
				if (org) {
					author = {
						name: org.name,
						slug: org.slug,
						type: 'organization',
					}
				} else if (team) {
					const teamMember = team.find((x) => x.is_owner)
					if (teamMember) {
						author = {
							name: teamMember.user.username,
							slug: teamMember.user.username,
							type: 'user',
						}
					}
				}

				newProjects.push({
					path,
					name: project.title,
					slug: project.slug,
					author,
					version: version.version_number,
					file_name: file.file_name,
					icon: project.icon_url,
					disabled: file.file_name.endsWith('.disabled'),
					updateVersion: file.update_version_id,
					updated: dayjs(version.date_published),
					outdated: !!file.update_version_id,
					project_type: project.project_type,
					id: project.id,
				})

				continue
			}
		}

		newProjects.push({
			path,
			name: file.file_name.replace('.disabled', ''),
			author: null,
			version: null,
			file_name: file.file_name,
			icon: undefined,
			disabled: file.file_name.endsWith('.disabled'),
			outdated: false,
			updated: dayjs(0),
			project_type: file.project_type === 'shaderpack' ? 'shader' : file.project_type,
		})
	}

	projects.value = newProjects
	loading.value = false
}

// Lifecycle
await initProjects()

// Drag & drop
const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
	if (event.payload.type !== 'drop') return

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
