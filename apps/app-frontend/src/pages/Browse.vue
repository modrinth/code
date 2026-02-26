<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ClipboardCopyIcon, ExternalIcon, GlobeIcon, PlayIcon, SearchIcon } from '@modrinth/assets'
import type { GameVersion, ProjectType, SortType, Tags } from '@modrinth/ui'
import {
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
	useSearch,
	useServerSearch,
	useVIntl,
} from '@modrinth/ui'
import type { Category, Platform } from '@modrinth/utils'
import { openUrl } from '@tauri-apps/plugin-opener'
import { $fetch } from 'ofetch'
import type { Ref } from 'vue'
import { computed, nextTick, ref, shallowRef, watch } from 'vue'
import type { LocationQuery } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
import type Instance from '@/components/ui/Instance.vue'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import NavTabs from '@/components/ui/NavTabs.vue'
import SearchCard from '@/components/ui/SearchCard.vue'
import { get_search_results } from '@/helpers/cache.js'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile.js'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { get_server_status } from '@/helpers/worlds'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { playServerProject, useInstall } from '@/store/install.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const installStore = useInstall()

const router = useRouter()
const route = useRoute()

const projectTypes = computed(() => {
	return [route.params.projectType as ProjectType]
})

const [categories, loaders, availableGameVersions] = await Promise.all([
	get_categories().catch(handleError).then(ref),
	get_loaders().catch(handleError).then(ref),
	get_game_versions().catch(handleError).then(ref),
])

const tags: Ref<Tags> = computed(() => ({
	gameVersions: availableGameVersions.value as GameVersion[],
	loaders: loaders.value as Platform[],
	categories: categories.value as Category[],
}))

type Instance = {
	game_version: string
	loader: string
	path: string
	install_stage: string
	icon_path?: string
	name: string
}

type InstanceProject = {
	metadata: {
		project_id: string
	}
}

const instance: Ref<Instance | null> = ref(null)
const instanceProjects: Ref<InstanceProject[] | null> = ref(null)
const instanceHideInstalled = ref(false)
const newlyInstalled = ref([])

const PERSISTENT_QUERY_PARAMS = ['i', 'ai']

await updateInstanceContext()

watch(route, () => {
	updateInstanceContext()
})

async function updateInstanceContext() {
	if (route.query.i) {
		;[instance.value, instanceProjects.value] = await Promise.all([
			getInstance(route.query.i).catch(handleError),
			getInstanceProjects(route.query.i).catch(handleError),
		])
		newlyInstalled.value = []
	}

	if (route.query.ai && !(projectTypes.value.length === 1 && projectTypes.value[0] === 'modpack')) {
		instanceHideInstalled.value = route.query.ai === 'true'
	}

	if (instance.value && instance.value.path !== route.query.i && route.path.startsWith('/browse')) {
		instance.value = null
		instanceHideInstalled.value = false
	}
}

const instanceFilters = computed(() => {
	const filters = []

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

		if (instanceHideInstalled.value && instanceProjects.value) {
			const installedMods = Object.values(instanceProjects.value)
				.filter((x) => x.metadata)
				.map((x) => x.metadata.project_id)

			installedMods.push(...newlyInstalled.value)

			installedMods
				?.map((x) => ({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				}))
				.forEach((x) => filters.push(x))
		}
	}

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

const previousFilterState = ref('')

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const breadcrumbs = useBreadcrumbs()
breadcrumbs.setContext({ name: 'Discover content', link: route.path, query: route.query })

const loading = ref(true)

const projectType = ref(route.params.projectType)

watch(projectType, () => {
	loading.value = true
})

type SearchResult = {
	project_id: string
}

type SearchResults = {
	total_hits: number
	limit: number
	hits: SearchResult[]
}

const results: Ref<SearchResults | null> = shallowRef(null)
const pageCount = computed(() =>
	results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

watch(requestParams, () => {
	if (!route.params.projectType) return
	refreshSearch()
})

async function refreshSearch() {
	let rawResults = await get_search_results(requestParams.value)
	if (!rawResults) {
		rawResults = {
			result: {
				hits: [],
				total_hits: 0,
				limit: 1,
			},
		}
	}
	if (instance.value) {
		for (const val of rawResults.result.hits) {
			val.installed =
				newlyInstalled.value.includes(val.project_id) ||
				Object.values(instanceProjects.value).some(
					(x) => x.metadata && x.metadata.project_id === val.project_id,
				)
		}
	}
	results.value = rawResults.result

	const currentFilterState = JSON.stringify({
		query: query.value,
		filters: currentFilters.value,
		sort: currentSortType.value,
		maxResults: maxResults.value,
		projectTypes: projectTypes.value,
	})

	if (previousFilterState.value && previousFilterState.value !== currentFilterState) {
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
		...createPageParams(),
	}

	breadcrumbs.setContext({
		name: 'Discover content',
		link: `/browse/${projectType.value}`,
		query: params,
	})
	await router.replace({ path: route.path, query: params })
	loading.value = false
}

async function setPage(newPageNumber: number) {
	currentPage.value = newPageNumber

	await onSearchChangeToTop()
}

const searchWrapper: Ref<HTMLElement | null> = ref(null)

async function onSearchChangeToTop() {
	await nextTick()

	window.scrollTo({ top: 0, behavior: 'smooth' })
}

function clearSearch() {
	query.value = ''
	currentPage.value = 1
}

watch(
	() => route.params.projectType,
	async (newType) => {
		// Check if the newType is not the same as the current value
		if (!newType || newType === projectType.value) return

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
			availableGameVersions.value.findIndex((x) => x.version === instance.value.game_version) <=
			availableGameVersions.value.findIndex((x) => x.version === '1.13')
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
		{ label: 'Servers', href: `/browse/server` },
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
	modLoaderProvidedByInstance: {
		id: 'search.filter.locked.instance-loader.title',
		defaultMessage: 'Loader is provided by the instance',
	},
	providedByInstance: {
		id: 'search.filter.locked.instance',
		defaultMessage: 'Provided by the instance',
	},
	syncFilterButton: {
		id: 'search.filter.locked.instance.sync',
		defaultMessage: 'Sync with instance',
	},
})

const serverHits = ref<Labrinth.Search.v3.ResultSearchProject[]>([])
const serverPings = ref<Record<string, number | undefined>>({})

const {
	serverCurrentSortType,
	serverCurrentFilters,
	serverToggledGroups,
	serverSortTypes,
	serverFilterTypes,
	serverRequestParams,
} = useServerSearch({ tags, query, maxResults, currentPage })

async function pingServerHits(hits: Labrinth.Search.v3.ResultSearchProject[]) {
	for (const hit of hits) {
		const address = hit.minecraft_java_server?.address
		if (!address) continue
		get_server_status(address)
			.then((status) => {
				serverPings.value = { ...serverPings.value, [hit.project_id]: status.ping }
			})
			.catch((err) => {
				console.error(`Failed to ping server ${address}:`, err)
			})
	}
}

async function refreshServerSearch() {
	try {
		const raw = await $fetch<Labrinth.Search.v3.SearchResults>(
			`https://api.modrinth.com/v3/search${serverRequestParams.value}`,
		)
		const hits = raw?.hits ?? []
		console.log('[v3 search] hits:', hits)
		serverHits.value = hits
		serverPings.value = {}
		pingServerHits(hits)
	} catch (e: any) {
		handleError(e)
	}
}

watch(
	() => [projectType.value, serverRequestParams.value],
	() => {
		if (projectType.value === 'server') {
			refreshServerSearch()
		}
	},
	{ immediate: true },
)

const getServerModpackContent = (project: Labrinth.Search.v3.ResultSearchProject) => {
	const content = project.minecraft_java_server?.content
	if (content?.kind === 'modpack') {
		const { project_name, project_icon, project_id } = content
		if (!project_name) return undefined
		return {
			name: project_name,
			icon: project_icon,
			onclick: () => {
				router.push(`/project/${project_id}`)
			},
			showCustomModpackTooltip: project_id === project.project_id,
		}
	}
	return undefined
}

const options = ref(null)
const handleRightClick = (event: any, result: any) => {
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
const handleOptionsClick = (args: any) => {
	switch (args.option) {
		case 'open_link':
			openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
			break
		case 'copy_link':
			navigator.clipboard.writeText(
				`https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
			)
			break
	}
}

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
				:open-by-default="true"
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
					{{ formatMessage(messages.gameVersionProvidedByInstance) }}
				</template>
				<template #locked-mod_loader>
					{{ formatMessage(messages.modLoaderProvidedByInstance) }}
				</template>
				<template #sync-button> {{ formatMessage(messages.syncFilterButton) }} </template>
			</SearchSidebarFilter>
		</template>
	</Teleport>
	<div ref="searchWrapper" class="flex flex-col gap-3 p-6">
		<template v-if="instance">
			<InstanceIndicator :instance="instance" />
			<h1 class="m-0 mb-1 text-xl">Install content to instance</h1>
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
			:provided-message="messages.providedByInstance"
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
					results &&
					results.hits &&
					results.hits.length === 0 &&
					serverHits &&
					serverHits.length === 0
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
						:server-region-code="project.minecraft_server?.country"
						:server-recent-plays="project.minecraft_java_server?.verified_plays_4w ?? 0"
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
								handleRightClick(event, { project_type: 'server', slug: project.slug })
						"
					>
						<template #actions>
							<ButtonStyled color="brand" type="outlined">
								<button
									:disabled="
										(installStore.installingServerProjects as string[]).includes(project.project_id)
									"
									@click="() => playServerProject(project.project_id)"
								>
									<PlayIcon />
									{{
										(installStore.installingServerProjects as string[]).includes(project.project_id)
											? 'Installing...'
											: 'Play'
									}}
								</button>
							</ButtonStyled>
						</template>
					</ProjectCard>
				</template>
				<template v-else>
					<SearchCard
						v-for="result in results?.hits ?? []"
						:key="result?.project_id"
						:project-type="projectType"
						:project="result"
						:instance="instance"
						:categories="[
							...categories.filter(
								(cat) =>
									result?.display_categories.includes(cat.name) && cat.project_type === projectType,
							),
							...loaders.filter(
								(loader) =>
									result?.display_categories.includes(loader.name) &&
									loader.supported_project_types?.includes(projectType),
							),
						]"
						:installed="result.installed || newlyInstalled.includes(result.project_id)"
						@install="
							(id) => {
								newlyInstalled.push(id)
							}
						"
						@contextmenu.prevent.stop="(event) => handleRightClick(event, result)"
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
