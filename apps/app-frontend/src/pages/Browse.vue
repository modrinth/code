<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ClipboardCopyIcon,
	ExternalIcon,
	GlobeIcon,
	PlayIcon,
	PlusIcon,
	SearchIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import type { ProjectType, SortType, Tags } from '@modrinth/ui'
import {
	Admonition,
	ButtonStyled,
	Checkbox,
	defineMessages,
	DropdownSelect,
	injectNotificationManager,
	LoadingIndicator,
	Pagination,
	ProjectCard,
	ProjectCardList,
	SearchFilterControl,
	SearchSidebarFilter,
	StyledInput,
	useDebugLogger,
	useSearch,
	useServerSearch,
	useVIntl,
} from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Ref } from 'vue'
import { computed, nextTick, onUnmounted, ref, shallowRef, toRaw, watch } from 'vue'
import type { LocationQuery } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import type Instance from '@/components/ui/Instance.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get_project_v3, get_search_results_v3 } from '@/helpers/cache.js'
import { process_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process'
import {
	get as getInstance,
	get_installed_project_ids as getInstalledProjectIds,
	kill,
	list as listInstances,
} from '@/helpers/profile.js'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import type { GameInstance } from '@/helpers/types'
import { getServerLatency } from '@/helpers/worlds'
import { injectServerInstall } from '@/providers/server-install'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { getServerAddress } from '@/store/install.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { installingServerProjects, playServerProject, showAddServerToInstanceModal } =
	injectServerInstall()
const debugLog = useDebugLogger('Browse')

const router = useRouter()
const route = useRoute()

const projectTypes = computed(() => {
	debugLog('projectTypes computed', route.params.projectType)
	return [route.params.projectType as ProjectType]
})

debugLog('fetching tags (categories, loaders, gameVersions)')
const [categories, loaders, availableGameVersions] = await Promise.all([
	get_categories()
		.catch(handleError)
		.then(ref<Labrinth.Tags.v2.Category[]>),
	get_loaders()
		.catch(handleError)
		.then(ref<Labrinth.Tags.v2.Loader[]>),
	get_game_versions()
		.catch(handleError)
		.then(ref<Labrinth.Tags.v2.GameVersion[]>),
])

const tags: Ref<Tags> = computed(() => ({
	gameVersions: availableGameVersions.value ?? [],
	loaders: loaders.value ?? [],
	categories: categories.value ?? [],
}))

type Instance = {
	game_version: string
	loader: string
	path: string
	install_stage: string
	icon_path?: string
	name: string
	linked_data?: {
		project_id: string
		version_id: string
		locked: boolean
	}
}

const instance: Ref<Instance | null> = ref(null)
const installedProjectIds: Ref<string[] | null> = ref(null)
const instanceHideInstalled = ref(false)
const newlyInstalled = ref<string[]>([])
const isServerInstance = ref(false)

const allInstalledIds = computed(
	() => new Set([...newlyInstalled.value, ...(installedProjectIds.value ?? [])]),
)

const PERSISTENT_QUERY_PARAMS = ['i', 'ai']

await initInstanceContext()

async function initInstanceContext() {
	debugLog('initInstanceContext', { queryI: route.query.i, queryAi: route.query.ai })
	if (route.query.i) {
		instance.value = await getInstance(route.query.i).catch(handleError)
		debugLog('instance loaded', {
			name: instance.value?.name,
			loader: instance.value?.loader,
			gameVersion: instance.value?.game_version,
		})

		// Load installed project IDs in background — the page and initial search render immediately.
		// When this resolves, instanceFilters recomputes and triggers a search refresh
		// that applies the "hide installed" negative filters and marks installed badges.
		getInstalledProjectIds(route.query.i)
			.then((ids) => {
				debugLog('installedProjectIds loaded', { count: ids?.length })
				installedProjectIds.value = ids
			})
			.catch(handleError)

		if (instance.value?.linked_data?.project_id) {
			debugLog('checking linked project for server status', instance.value.linked_data.project_id)
			const projectV3 = await get_project_v3(
				instance.value.linked_data.project_id,
				'must_revalidate',
			).catch(handleError)
			if (projectV3?.minecraft_server != null) {
				debugLog('instance is a server instance')
				isServerInstance.value = true
			}
		}
	}

	if (route.query.ai && !(projectTypes.value.length === 1 && projectTypes.value[0] === 'modpack')) {
		debugLog('setting instanceHideInstalled from query', route.query.ai)
		instanceHideInstalled.value = route.query.ai === 'true'
	}
}

const instanceFilters = computed(() => {
	const filters = []
	debugLog('instanceFilters recomputing', {
		hasInstance: !!instance.value,
		isServer: isServerInstance.value,
		hideInstalled: instanceHideInstalled.value,
	})

	if (instance.value) {
		const gameVersion = instance.value.game_version
		if (gameVersion) {
			filters.push({
				type: 'game_version',
				option: gameVersion,
			})
		}

		const platform = instance.value.loader

		const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

		if (platform && projectTypes.value.includes('mod') && supportedModLoaders.includes(platform)) {
			filters.push({
				type: 'mod_loader',
				option: platform,
			})
		}

		if (isServerInstance.value) {
			filters.push({
				type: 'environment',
				option: 'client',
			})
		}

		if (
			instanceHideInstalled.value &&
			(installedProjectIds.value || newlyInstalled.value.length > 0)
		) {
			const allInstalled = [...(installedProjectIds.value ?? []), ...newlyInstalled.value]

			allInstalled
				.map((x) => ({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				}))
				.forEach((x) => filters.push(x))
		}
	}

	debugLog('instanceFilters result', filters)
	return filters
})

const {
	// Selections
	query,
	currentSortType,
	currentFilters,
	toggledGroups,
	maxResults,
	currentPage,
	overriddenProvidedFilterTypes,

	// Lists
	filters,
	sortTypes,

	// Computed
	requestParams,

	// Functions
	createPageParams,
} = useSearch(projectTypes, tags, instanceFilters)

const activeLoader = computed(() => {
	const filter = currentFilters.value.find((f) => f.type === 'mod_loader')
	if (filter) return filter.option
	if (projectType.value === 'datapack' || projectType.value === 'resourcepack') return 'vanilla'
	return instance.value?.loader ?? null
})

const activeGameVersion = computed(() => {
	const filter = currentFilters.value.find((f) => f.type === 'game_version')
	if (filter) return filter.option
	return instance.value?.game_version ?? null
})

const serverHits = shallowRef<Labrinth.Search.v3.ResultSearchProject[]>([])
const serverPings = shallowRef<Record<string, number | undefined>>({})
const runningServerProjects = ref<Record<string, string>>({})

async function checkServerRunningStates(hits: Labrinth.Search.v3.ResultSearchProject[]) {
	debugLog('checkServerRunningStates', { hitCount: hits.length })
	const packs = await listInstances()
	const newRunning: Record<string, string> = {}
	for (const hit of hits) {
		const inst = packs.find((p: GameInstance) => p.linked_data?.project_id === hit.project_id)
		if (inst) {
			const processes = await get_by_profile_path(inst.path).catch(() => [])
			if (Array.isArray(processes) && processes.length > 0) {
				newRunning[hit.project_id] = inst.path
			}
		}
	}
	debugLog('runningServerProjects updated', newRunning)
	runningServerProjects.value = newRunning
}

async function handleStopServerProject(projectId: string) {
	debugLog('handleStopServerProject', projectId)
	const instancePath = runningServerProjects.value[projectId]
	if (!instancePath) return
	await kill(instancePath).catch(() => {})
	const { [projectId]: _, ...rest } = runningServerProjects.value
	runningServerProjects.value = rest
}

async function handlePlayServerProject(projectId: string) {
	debugLog('handlePlayServerProject', projectId)
	await playServerProject(projectId)
	checkServerRunningStates(serverHits.value)
}

function handleAddServerToInstance(project: Labrinth.Search.v3.ResultSearchProject) {
	debugLog('handleAddServerToInstance', { projectId: project.project_id, name: project.name })
	const address = getServerAddress(project.minecraft_java_server)
	if (!address) return
	showAddServerToInstanceModal(project.name, address)
}

const unlistenProcesses = await process_listener(
	(e: { event: string; profile_path_id: string }) => {
		debugLog('process event', e)
		if (e.event === 'finished') {
			const projectId = Object.entries(runningServerProjects.value).find(
				([, path]) => path === e.profile_path_id,
			)?.[0]
			if (projectId) {
				const { [projectId]: _, ...rest } = runningServerProjects.value
				runningServerProjects.value = rest
			}
		}
	},
)

onUnmounted(() => {
	unlistenProcesses()
})

const {
	serverCurrentSortType,
	serverCurrentFilters,
	serverToggledGroups,
	serverSortTypes,
	serverFilterTypes,
	serverRequestParams,
	createServerPageParams,
} = useServerSearch({ tags, query, maxResults, currentPage })

async function pingServerHits(hits: Labrinth.Search.v3.ResultSearchProject[]) {
	debugLog('pingServerHits', { hitCount: hits.length })
	const pingsToFetch = hits.filter((hit) => hit.minecraft_java_server?.address)
	await Promise.all(
		pingsToFetch.map(async (hit) => {
			const address = hit.minecraft_java_server!.address!
			try {
				const latency = await getServerLatency(address)
				serverPings.value = { ...serverPings.value, [hit.project_id]: latency }
			} catch (err) {
				console.error(`Failed to ping server ${address}:`, err)
			}
		}),
	)
}

const previousFilterState = ref('')
let searchVersion = 0

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	debugLog('went offline')
	offline.value = true
})
window.addEventListener('online', () => {
	debugLog('went online')
	offline.value = false
})

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: route.query })

const loading = ref(true)

const projectType = ref<ProjectType>(route.params.projectType as ProjectType)

watch(projectType, () => {
	loading.value = true
})

interface SearchResults extends Labrinth.Search.v3.SearchResults {
	hits: (Labrinth.Search.v3.ResultSearchProject & { installed?: boolean })[]
}

const results: Ref<SearchResults | null> = shallowRef(null)
const pageCount = computed(() =>
	results.value ? Math.ceil(results.value.total_hits / results.value.hits_per_page) : 1,
)

const effectiveRequestParams = computed(() => {
	const isServer = projectType.value === 'server'
	debugLog('effectiveRequestParams computed', { isServer })
	return isServer ? serverRequestParams.value : requestParams.value
})

let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

watch(effectiveRequestParams, () => {
	if (!route.params.projectType) return
	debugLog('effectiveRequestParams changed, debouncing search')
	if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
	searchDebounceTimer = setTimeout(() => {
		refreshSearch()
	}, 200)
})

async function refreshSearch() {
	const version = ++searchVersion
	debugLog('refreshSearch start', { version, projectType: projectType.value })

	try {
		const isServer = projectType.value === 'server'
		const searchParams = isServer ? serverRequestParams.value : requestParams.value

		debugLog('searching v3', searchParams)
		let rawResults = (await get_search_results_v3(searchParams)) as {
			result: SearchResults
		} | null

		if (version !== searchVersion) {
			debugLog('search version stale, discarding', { version, current: searchVersion })
			return
		}

		if (!rawResults) {
			rawResults = {
				result: {
					hits: [],
					total_hits: 0,
					hits_per_page: maxResults.value,
					page: 1,
				},
			}
		}

		if (isServer) {
			const hits = rawResults.result.hits ?? []
			debugLog('server search results', {
				hitCount: hits.length,
				totalHits: rawResults.result.total_hits,
			})
			serverHits.value = hits
			serverPings.value = {}
			pingServerHits(hits)
			checkServerRunningStates(hits)
			results.value = {
				hits: [],
				total_hits: rawResults.result.total_hits ?? 0,
				hits_per_page: maxResults.value,
				page: 1,
			}
		} else {
			if (instance.value) {
				const allInstalledIds = new Set([
					...newlyInstalled.value,
					...(installedProjectIds.value ?? []),
				])

				rawResults.result.hits = rawResults.result.hits.map((val) => ({
					...val,
					installed: allInstalledIds.has(val.project_id),
				}))
			}
			debugLog('v3 search results', {
				hitCount: rawResults.result.hits.length,
				totalHits: rawResults.result.total_hits,
			})
			results.value = {
				...rawResults.result,
				hits_per_page: maxResults.value,
			}
		}

		const currentFilterState = JSON.stringify({
			query: query.value,
			filters: toRaw(currentFilters.value),
			sort: toRaw(currentSortType.value),
			maxResults: maxResults.value,
			projectTypes: toRaw(projectTypes.value),
		})

		if (previousFilterState.value && previousFilterState.value !== currentFilterState) {
			debugLog('filters changed, resetting to page 1')
			currentPage.value = 1
		}

		previousFilterState.value = currentFilterState

		const persistentParams: LocationQuery = {}

		for (const [key, value] of Object.entries(route.query)) {
			if (PERSISTENT_QUERY_PARAMS.includes(key)) {
				persistentParams[key] = value
			}
		}

		if (instanceHideInstalled.value) {
			persistentParams.ai = 'true'
		} else {
			delete persistentParams.ai
		}

		const params = {
			...persistentParams,
			...(isServer ? createServerPageParams() : createPageParams()),
		}

		breadcrumbs.setContext({
			name: 'Discover content',
			link: `/browse/${projectType.value}`,
			query: params,
		})
		debugLog('updating URL', params)
		router.replace({ path: route.path, query: params })

		loading.value = false
		debugLog('refreshSearch complete', { version })
	} catch (err) {
		debugLog('refreshSearch error', err)
		if (version === searchVersion) {
			loading.value = false
		}
	}
}

async function setPage(newPageNumber: number) {
	debugLog('setPage', newPageNumber)
	currentPage.value = newPageNumber

	await onSearchChangeToTop()
}

const searchWrapper: Ref<HTMLElement | null> = ref(null)

async function onSearchChangeToTop() {
	await nextTick()

	window.scrollTo({ top: 0, behavior: 'smooth' })
}

function clearSearch() {
	debugLog('clearSearch')
	query.value = ''
	currentPage.value = 1
}

watch(
	() => route.params.projectType as ProjectType,
	async (newType) => {
		// Check if the newType is not the same as the current value
		if (!newType || newType === projectType.value) return

		debugLog('projectType route param changed', { from: projectType.value, to: newType })
		projectType.value = newType

		currentSortType.value = { display: 'Relevance', name: 'relevance' }
		query.value = ''
	},
)

const selectableProjectTypes = computed(() => {
	let dataPacks = false,
		mods = false,
		modpacks = false

	if (instance.value) {
		if (
			availableGameVersions.value &&
			availableGameVersions.value.findIndex((x) => x.version === instance.value?.game_version) <=
				availableGameVersions.value.findIndex((x) => x.version === '1.13') &&
			!isServerInstance.value
		) {
			dataPacks = true
		}

		if (instance.value.loader !== 'vanilla') {
			mods = true
		}
	} else {
		dataPacks = true
		mods = true
		modpacks = true
	}

	const params: LocationQuery = {}

	if (route.query.i) {
		params.i = route.query.i
	}
	if (route.query.ai) {
		params.ai = route.query.ai
	}

	const links = [
		{ label: 'Modpacks', href: `/browse/modpack`, shown: modpacks },
		{ label: 'Mods', href: `/browse/mod`, shown: mods },
		{ label: 'Resource Packs', href: `/browse/resourcepack` },
		{ label: 'Data Packs', href: `/browse/datapack`, shown: dataPacks },
		{ label: 'Shaders', href: `/browse/shader` },
		{ label: 'Servers', href: `/browse/server`, shown: !instance.value },
	]

	if (params) {
		return links.map((link) => {
			return {
				...link,
				href: {
					path: link.href,
					query: params,
				},
			}
		})
	}

	return links
})

const messages = defineMessages({
	gameVersionProvidedByInstance: {
		id: 'search.filter.locked.instance-game-version.title',
		defaultMessage: 'Game version is provided by the instance',
	},
	gameVersionProvidedByServer: {
		id: 'search.filter.locked.server-game-version.title',
		defaultMessage: 'Game version is provided by the server',
	},
	modLoaderProvidedByInstance: {
		id: 'search.filter.locked.instance-loader.title',
		defaultMessage: 'Loader is provided by the instance',
	},
	modLoaderProvidedByServer: {
		id: 'search.filter.locked.server-loader.title',
		defaultMessage: 'Loader is provided by the server',
	},
	environmentProvidedByServer: {
		id: 'search.filter.locked.server-environment.title',
		defaultMessage: 'Only client-side mods can be added to the server instance',
	},
	providedByInstance: {
		id: 'search.filter.locked.instance',
		defaultMessage: 'Provided by the instance',
	},
	providedByServer: {
		id: 'search.filter.locked.server',
		defaultMessage: 'Provided by the server',
	},
	syncFilterButton: {
		id: 'search.filter.locked.instance.sync',
		defaultMessage: 'Sync with instance',
	},
})

const getServerModpackContent = (project: Labrinth.Search.v3.ResultSearchProject) => {
	const content = project.minecraft_java_server?.content
	if (content?.kind === 'modpack') {
		const { project_name, project_icon, project_id } = content
		if (!project_name) return undefined
		return {
			name: project_name,
			icon: project_icon,
			onclick:
				project_id !== project.project_id
					? () => {
							router.push(`/project/${project_id}`)
						}
					: undefined,
			showCustomModpackTooltip: project_id === project.project_id,
		}
	}
	return undefined
}

const options = ref(null)
// @ts-expect-error - no event types
const handleRightClick = (event, result) => {
	// @ts-ignore
	options.value?.showMenu(event, result, [
		{
			name: 'open_link',
		},
		{
			name: 'copy_link',
		},
	])
}
// @ts-expect-error - no event types
const handleOptionsClick = (args) => {
	switch (args.option) {
		case 'open_link':
			openUrl(`https://modrinth.com/${args.item.project_types?.[0] ?? 'project'}/${args.item.slug}`)
			break
		case 'copy_link':
			navigator.clipboard.writeText(
				`https://modrinth.com/${args.item.project_types?.[0] ?? 'project'}/${args.item.slug}`,
			)
			break
	}
}

debugLog('performing initial search')
await refreshSearch()

// Initialize previousFilterState after first search
previousFilterState.value = JSON.stringify({
	query: query.value,
	filters: currentFilters.value,
	sort: currentSortType.value,
	maxResults: maxResults.value,
	projectTypes: projectTypes.value,
})
</script>

<template>
	<Teleport v-if="filters" to="#sidebar-teleport-target">
		<div
			v-if="instance"
			class="border-0 border-b-[1px] p-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
		>
			<Checkbox
				v-model="instanceHideInstalled"
				label="Hide installed content"
				class="filter-checkbox"
				@update:model-value="onSearchChangeToTop()"
				@click.prevent.stop
			/>
		</div>
		<template v-if="projectType === 'server'">
			<SearchSidebarFilter
				v-for="filterType in serverFilterTypes.filter((f) => f.options.length > 0)"
				:key="`server-filter-${filterType.id}`"
				v-model:selected-filters="serverCurrentFilters"
				v-model:toggled-groups="serverToggledGroups"
				:provided-filters="[]"
				:filter-type="filterType"
				class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
				button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
				content-class="mb-3"
				inner-panel-class="ml-2 mr-3"
				:open-by-default="
					![
						'server_category_minecraft_server_meta',
						'server_category_minecraft_server_community',
						'server_game_version',
						'server_status',
					].includes(filterType.id)
				"
			>
				<template #header>
					<h3 class="text-base m-0">{{ filterType.formatted_name }}</h3>
				</template>
			</SearchSidebarFilter>
		</template>
		<template v-else>
			<SearchSidebarFilter
				v-for="filter in filters.filter((f) => f.display !== 'none')"
				:key="`filter-${filter.id}`"
				v-model:selected-filters="currentFilters"
				v-model:toggled-groups="toggledGroups"
				v-model:overridden-provided-filter-types="overriddenProvidedFilterTypes"
				:provided-filters="instanceFilters"
				:filter-type="filter"
				class="border-0 border-b-[1px] [&:first-child>button]:pt-4 last:border-b-0 border-[--brand-gradient-border] border-solid"
				button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none hover:bg-button-bg"
				content-class="mb-3"
				inner-panel-class="ml-2 mr-3"
				:open-by-default="
					filter.id.startsWith('category') || filter.id === 'environment' || filter.id === 'license'
				"
			>
				<template #header>
					<h3 class="text-base m-0">{{ filter.formatted_name }}</h3>
				</template>
				<template #locked-game_version>
					{{
						formatMessage(
							isServerInstance
								? messages.gameVersionProvidedByServer
								: messages.gameVersionProvidedByInstance,
						)
					}}
				</template>
				<template #locked-mod_loader>
					{{
						formatMessage(
							isServerInstance
								? messages.modLoaderProvidedByServer
								: messages.modLoaderProvidedByInstance,
						)
					}}
				</template>
				<template #locked-environment>
					{{ formatMessage(messages.environmentProvidedByServer) }}
				</template>
				<template #sync-button> {{ formatMessage(messages.syncFilterButton) }} </template>
			</SearchSidebarFilter>
		</template>
	</Teleport>
	<div ref="searchWrapper" class="flex flex-col gap-3 p-6">
		<template v-if="instance">
			<InstanceIndicator :instance="instance" />
			<h1 class="m-0 mb-1 text-xl">Install content to instance</h1>
			<Admonition v-if="isServerInstance" type="warning" class="mb-1">
				Adding content can break compatibility when joining the server. Any added content will also
				be lost when you update the server instance content.
			</Admonition>
		</template>
		<NavTabs :links="selectableProjectTypes" />
		<StyledInput
			v-model="query"
			:icon="SearchIcon"
			type="text"
			autocomplete="off"
			:placeholder="`Search ${projectType}s...`"
			clearable
			wrapper-class="w-full"
			input-class="h-12"
			@clear="clearSearch()"
		/>
		<div class="flex gap-2">
			<DropdownSelect
				v-slot="{ selected }"
				:model-value="projectType === 'server' ? serverCurrentSortType : currentSortType"
				class="max-w-[16rem]"
				name="Sort by"
				:options="(projectType === 'server' ? serverSortTypes : sortTypes) as any"
				:display-name="(option: SortType | undefined) => option?.display"
				@update:model-value="
					(v: SortType) => {
						if (projectType === 'server') serverCurrentSortType = v
						else currentSortType = v
					}
				"
			>
				<span class="font-semibold text-primary">Sort by: </span>
				<span class="font-semibold text-secondary">{{ selected }}</span>
			</DropdownSelect>
			<DropdownSelect
				v-slot="{ selected }"
				v-model="maxResults"
				name="Max results"
				:options="[5, 10, 15, 20, 50, 100]"
				class="max-w-[9rem]"
			>
				<span class="font-semibold text-primary">View: </span>
				<span class="font-semibold text-secondary">{{ selected }}</span>
			</DropdownSelect>
			<Pagination :page="currentPage" :count="pageCount" class="ml-auto" @switch-page="setPage" />
		</div>
		<SearchFilterControl
			v-if="projectType === 'server'"
			v-model:selected-filters="serverCurrentFilters"
			:filters="serverFilterTypes"
			:provided-filters="[]"
			:overridden-provided-filter-types="[]"
		/>
		<SearchFilterControl
			v-else
			v-model:selected-filters="currentFilters"
			:filters="filters.filter((f) => f.display !== 'none')"
			:provided-filters="instanceFilters"
			:overridden-provided-filter-types="overriddenProvidedFilterTypes"
			:provided-message="isServerInstance ? messages.providedByServer : messages.providedByInstance"
		/>
		<div class="search">
			<section v-if="loading" class="offline">
				<LoadingIndicator />
			</section>
			<section v-else-if="offline && results?.total_hits === 0" class="offline">
				You are currently offline. Connect to the internet to browse Modrinth!
			</section>
			<section
				v-else-if="
					projectType === 'server'
						? serverHits.length === 0
						: results && results.hits && results.hits.length === 0
				"
				class="offline"
			>
				No results found for your query!
			</section>

			<ProjectCardList v-else :layout="'list'">
				<template v-if="projectType === 'server'">
					<ProjectCard
						v-for="project in serverHits"
						:key="`server-card-${project.project_id}`"
						:title="project.name"
						:icon-url="project.icon_url || undefined"
						:summary="project.summary"
						:tags="project.categories"
						:link="`/project/${project.slug ?? project.project_id}`"
						:server-online-players="project.minecraft_java_server?.ping?.data?.players_online ?? 0"
						:server-region="project.minecraft_server?.region"
						:server-recent-plays="project.minecraft_java_server?.verified_plays_2w ?? 0"
						:server-modpack-content="getServerModpackContent(project)"
						:server-ping="serverPings[project.project_id]"
						:server-status-online="!!project.minecraft_java_server?.ping?.data"
						:hide-online-players-label="true"
						:hide-recent-plays-label="true"
						layout="list"
						:max-tags="2"
						is-server-project
						exclude-loaders
						@contextmenu.prevent.stop="
							(event: any) =>
								handleRightClick(event, { project_types: ['server'], slug: project.slug })
						"
					>
						<template #actions>
							<div class="flex gap-2">
								<ButtonStyled circular>
									<button
										v-tooltip="'Add server to instance'"
										@click.stop="() => handleAddServerToInstance(project)"
									>
										<PlusIcon />
									</button>
								</ButtonStyled>
								<ButtonStyled
									v-if="runningServerProjects[project.project_id]"
									color="red"
									type="outlined"
								>
									<button @click="() => handleStopServerProject(project.project_id)">
										<StopCircleIcon />
										Stop
									</button>
								</ButtonStyled>
								<ButtonStyled v-else color="brand" type="outlined">
									<button
										:disabled="(installingServerProjects as string[]).includes(project.project_id)"
										@click="() => handlePlayServerProject(project.project_id)"
									>
										<PlayIcon />
										{{
											(installingServerProjects as string[]).includes(project.project_id)
												? 'Installing...'
												: 'Play'
										}}
									</button>
								</ButtonStyled>
							</div>
						</template>
					</ProjectCard>
				</template>
				<template v-else>
					<SearchCard
						v-for="result in results?.hits ?? []"
						:key="result?.project_id"
						:project-type="projectType"
						:project="result"
						:instance="instance ?? undefined"
						:active-loader="activeLoader ?? undefined"
						:active-game-version="activeGameVersion ?? undefined"
						:categories="[
							...(categories ?? []).filter(
								(cat) =>
									result?.display_categories.includes(cat.name) && cat.project_type === projectType,
							),
							...(loaders ?? []).filter(
								(loader) =>
									result?.display_categories.includes(loader.name) &&
									loader.supported_project_types?.includes(projectType),
							),
						]"
						:installed="result.installed || allInstalledIds.has(result.project_id || '')"
						@install="
							(id) => {
								newlyInstalled.push(id)
							}
						"
						@contextmenu.prevent.stop="(event: any) => handleRightClick(event, result)"
					/>
				</template>

				<ContextMenu ref="options" @option-clicked="handleOptionsClick">
					<template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
					<template #copy_link> <ClipboardCopyIcon /> Copy link </template>
				</ContextMenu>
			</ProjectCardList>
			<div class="flex justify-end">
				<pagination
					:page="currentPage"
					:count="pageCount"
					class="pagination-after"
					@switch-page="setPage"
				/>
			</div>
		</div>
	</div>
</template>
