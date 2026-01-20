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

			<!-- Filter + Sort + Pagination row -->
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

			<!-- Content cards -->
			<div class="flex flex-col gap-4">
				<div
					v-if="paginatedItems.length === 0"
					class="universal-card flex h-24 items-center justify-center text-secondary"
				>
					No content found.
				</div>
				<ContentCard
					v-for="item in paginatedItems"
					:key="item.file_name"
					v-model:selected="selectedStates[item.file_name]"
					:project="mapProject(item)"
					:version="mapVersion(item)"
					:owner="mapOwner(item)"
					:enabled="!item.disabled"
					:disabled="changingMods.has(item.file_name)"
					:overflow-options="getOverflowOptions(item)"
					@update:enabled="() => toggleDisableMod(item)"
					@delete="() => removeMod(item)"
					v-on="item.outdated ? { update: () => updateProject(item) } : {}"
				/>
			</div>

			<!-- Bottom pagination -->
			<div v-if="totalPages > 1" class="mt-4 flex justify-center">
				<Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
			</div>
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
	LinkIcon,
	ListFilterIcon,
	SearchIcon,
	ShareIcon,
	SlashIcon,
	SortAscIcon,
	SortDescIcon,
	SpinnerIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	type ComboboxOption,
	ContentCard,
	type ContentCardProject,
	type ContentCardVersion,
	type ContentOwner,
	FloatingActionBar,
	injectNotificationManager,
	OverflowMenu,
	type OverflowMenuOption,
	Pagination,
	ProgressBar,
	RadialHeader,
} from '@modrinth/ui'
import type { Organization, Project, TeamMember, Version } from '@modrinth/utils'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useDebounceFn } from '@vueuse/core'
import dayjs from 'dayjs'
import Fuse from 'fuse.js'
import { computed, onBeforeUnmount, onUnmounted, reactive, ref, watch, watchSyncEffect } from 'vue'
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
const filterType = ref('All')
const sortType = ref('Newest')
const currentPage = ref(1)
const itemsPerPage = 20

// Selection state
const selectedStates = reactive<Record<string, boolean>>({})
const changingMods = ref(new Set<string>())

// Bulk operations state
const isBulkOperating = ref(false)
const bulkProgress = ref(0)
const bulkTotal = ref(0)
const bulkOperation = ref<'enable' | 'disable' | 'delete' | 'update' | null>(null)

const shareModal = ref<InstanceType<typeof ShareModalWrapper> | null>()
const exportModal = ref(null)
const modpackVersionModal = ref<InstanceType<typeof ModpackVersionModal> | null>()

let refreshInterval: ReturnType<typeof setInterval> | null = null

const isPackLocked = computed(() => props.instance.linked_data?.locked ?? false)

watch(
	projects,
	(items) => {
		for (const item of items) {
			if (!(item.file_name in selectedStates)) {
				selectedStates[item.file_name] = false
			}
		}

		for (const key of Object.keys(selectedStates)) {
			if (!items.some((item) => item.file_name === key)) {
				selectedStates[key] = false
			}
		}
	},
	{ immediate: true },
)

const selectedItems = computed(() =>
	projects.value.filter((item) => selectedStates[item.file_name]),
)

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

const typeMap: Record<string, string> = {
	Mods: 'mod',
	'Resource Packs': 'resourcepack',
	Shaders: 'shader',
}

const fuse = new Fuse<ProjectListEntry>([], {
	keys: ['name', 'author.name', 'file_name'],
	threshold: 0.4,
	distance: 100,
})

const sortedProjects = computed(() => {
	const items = [...projects.value]
	switch (sortType.value) {
		case 'Oldest':
			return items.sort((a, b) => (a.updated.isAfter(b.updated) ? 1 : -1))
		case 'A-Z':
			return items.sort((a, b) => a.name.localeCompare(b.name))
		case 'Z-A':
			return items.sort((a, b) => b.name.localeCompare(a.name))
		default: // Newest
			return items.sort((a, b) => (a.updated.isAfter(b.updated) ? -1 : 1))
	}
})

watchSyncEffect(() => fuse.setCollection(sortedProjects.value))

const filteredProjects = computed(() => {
	const targetType = typeMap[filterType.value]
	const query = searchQuery.value.trim()

	let items: ProjectListEntry[]

	if (query) {
		items = fuse.search(query).map(({ item }) => item)
	} else {
		items = sortedProjects.value
	}

	if (targetType) {
		items = items.filter((item) => item.project_type === targetType)
	}

	return items
})

const totalPages = computed(() => Math.ceil(filteredProjects.value.length / itemsPerPage))

const paginatedItems = computed(() => {
	const start = (currentPage.value - 1) * itemsPerPage
	const end = start + itemsPerPage
	return filteredProjects.value.slice(start, end)
})

// Functions
function goToPage(page: number) {
	currentPage.value = page
}

const handleSearch = useDebounceFn(() => {
	currentPage.value = 1
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
	selectedStates[mod.file_name] = false

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
	for (const key of Object.keys(selectedStates)) {
		selectedStates[key] = false
	}
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
