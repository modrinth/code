<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	BookmarkIcon,
	CheckIcon,
	DownloadIcon,
	FilterIcon,
	GameIcon,
	GridIcon,
	HeartIcon,
	ImageIcon,
	InfoIcon,
	LeftArrowIcon,
	ListIcon,
	MinecraftServerIcon,
	MoreVerticalIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Checkbox,
	type CreationFlowContextValue,
	CreationFlowModal,
	defineMessages,
	DropdownSelect,
	injectModrinthClient,
	injectNotificationManager,
	Pagination,
	ProjectCard,
	ProjectCardList,
	SearchFilterControl,
	SearchSidebarFilter,
	type SortType,
	StyledInput,
	Toggle,
	useDebugLogger,
	useSearch,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString, cycleValue } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useThrottleFn, useTimeoutFn } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'

import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('Discover')

const client = injectModrinthClient()
const queryClient = useQueryClient()

const filtersMenuOpen = ref(false)

const route = useRoute()
const router = useRouter()

const cosmetics = useCosmetics()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const { handleError } = injectNotificationManager()

let prefetchTimeout: ReturnType<typeof useTimeoutFn> | null = null
const HOVER_DURATION_TO_PREFETCH_MS = 500

const handleProjectMouseEnter = (result: Labrinth.Search.v2.ResultSearchProject) => {
	const slug = result.slug || result.project_id
	prefetchTimeout = useTimeoutFn(
		() => {
			queryClient.prefetchQuery(projectQueryOptions.v2(slug, client))
			queryClient.prefetchQuery(projectQueryOptions.v3(result.project_id, client))
			queryClient.prefetchQuery(projectQueryOptions.members(result.project_id, client))
			queryClient.prefetchQuery(projectQueryOptions.dependencies(result.project_id, client))
			queryClient.prefetchQuery(projectQueryOptions.versionsV3(result.project_id, client))
		},
		HOVER_DURATION_TO_PREFETCH_MS,
		{ immediate: false },
	)
	prefetchTimeout.start()
}

const handleProjectHoverEnd = () => {
	if (prefetchTimeout) prefetchTimeout.stop()
}

const currentType = computed(() =>
	queryAsStringOrEmpty(route.params.type).replaceAll(/^\/|s\/?$/g, ''),
)

const projectType = computed(() => tags.value.projectTypes.find((x) => x.id === currentType.value))
const projectTypes = computed(() => (projectType.value ? [projectType.value.id] : []))

const resultsDisplayLocation = computed<DisplayLocation | undefined>(
	() => projectType.value?.id as DisplayLocation,
)
const resultsDisplayMode = computed<DisplayMode>(() =>
	resultsDisplayLocation.value
		? cosmetics.value.searchDisplayMode[resultsDisplayLocation.value]
		: 'list',
)

const currentServerId = computed(() => queryAsString(route.query.sid) || null)
const fromContext = computed(() => queryAsString(route.query.from) || null)
const currentWorldId = computed(() => queryAsString(route.query.wid) || undefined)
debug('currentServerId:', currentServerId.value)

const {
	data: serverData,
	isLoading: serverDataLoading,
	error: serverDataError,
} = useQuery({
	queryKey: computed(() => ['servers', 'detail', currentServerId.value] as const),
	queryFn: () => {
		debug('serverData queryFn firing for:', currentServerId.value)
		return client.archon.servers_v0.get(currentServerId.value!)
	},
	enabled: computed(() => {
		const enabled = !!currentServerId.value
		debug('serverData enabled:', enabled)
		return enabled
	}),
})

watch(serverData, (val) =>
	debug('serverData changed:', val?.server_id, val?.name, val?.loader, val?.mc_version),
)
watch(serverDataLoading, (val) => debug('serverData loading:', val))
watch(serverDataError, (val) => {
	if (val) debug('serverData error:', val)
})

const serverIcon = computed(() => {
	if (!currentServerId.value || !import.meta.client) return null
	return localStorage.getItem(`server-icon-${currentServerId.value}`)
})

const serverHideInstalled = ref(false)
const eraseDataOnInstall = ref(false)

// TanStack Query for server content list
const contentQueryKey = computed(() => ['content', 'list', currentServerId.value ?? ''] as const)
const { data: serverContentData, error: serverContentError } = useQuery({
	queryKey: contentQueryKey,
	queryFn: () => client.archon.content_v1.getAddons(currentServerId.value!, currentWorldId.value!),
	enabled: computed(() => !!currentServerId.value && !!currentWorldId.value),
})

// Watch for errors and notify user
watch(serverContentError, (error) => {
	if (error) {
		console.error('Failed to load server content:', error)
		handleError(error)
	}
})

// Install content mutation
const installContentMutation = useMutation({
	mutationFn: ({
		serverId,
		projectId,
		versionId,
	}: {
		serverId: string
		projectId: string
		versionId: string
	}) =>
		client.archon.content_v1.addAddon(serverId, currentWorldId.value!, {
			project_id: projectId,
			version_id: versionId,
		}),
	onSuccess: () => {
		if (currentServerId.value) {
			queryClient.invalidateQueries({ queryKey: ['content', 'list', currentServerId.value] })
		}
	},
})

const PERSISTENT_QUERY_PARAMS = ['sid', 'wid', 'shi', 'from']

if (route.query.shi && projectType.value?.id !== 'modpack') {
	serverHideInstalled.value = route.query.shi === 'true'
}

const serverFilters = computed(() => {
	debug(
		'serverFilters recomputing, serverData:',
		!!serverData.value,
		'projectType:',
		projectType.value?.id,
	)
	const filters = []
	if (serverData.value && projectType.value?.id !== 'modpack') {
		const gameVersion = serverData.value.mc_version
		if (gameVersion) {
			filters.push({
				type: 'game_version',
				option: gameVersion,
			})
		}

		const platform = serverData.value.loader?.toLowerCase()

		const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

		if (platform && modLoaders.includes(platform)) {
			filters.push({
				type: 'mod_loader',
				option: platform,
			})
		}

		const pluginLoaders = ['paper', 'purpur']

		if (platform && pluginLoaders.includes(platform)) {
			filters.push({
				type: 'plugin_loader',
				option: platform,
			})
		}

		if (projectType.value?.id === 'mod') {
			filters.push({
				type: 'environment',
				option: 'server',
			})
		}

		if (serverHideInstalled.value && serverContentData.value) {
			const installedIds = (serverContentData.value.addons ?? [])
				.filter((x) => x.project_id)
				.map((x) => x.project_id)
				.filter((id): id is string => id !== null)

			installedIds
				.map((x: string) => ({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				}))
				.forEach((x) => filters.push(x))
		}
	}
	debug('serverFilters result:', filters)
	return filters
})

const maxResultsForView = ref<Record<DisplayMode, number[]>>({
	list: [5, 10, 15, 20, 50, 100],
	grid: [6, 12, 18, 24, 48, 96],
	gallery: [6, 10, 16, 20, 50, 100],
})

const currentMaxResultsOptions = computed(
	() => maxResultsForView.value[resultsDisplayMode.value] ?? [20],
)

const LOADER_FILTER_TYPES = [
	'mod_loader',
	'plugin_loader',
	'modpack_loader',
	'shader_loader',
	'plugin_platform',
] as const

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
} = useSearch(projectTypes, tags, serverFilters)
debug('useSearch initialized, requestParams:', requestParams.value)

const selectedFilterTags = computed(() =>
	currentFilters.value
		.filter(
			(f) =>
				f.type.startsWith('category_') ||
				LOADER_FILTER_TYPES.includes(f.type as (typeof LOADER_FILTER_TYPES)[number]),
		)
		.map((f) => f.option),
)
const excludeLoaders = computed(
	() =>
		currentFilters.value.some((f) =>
			LOADER_FILTER_TYPES.includes(f.type as (typeof LOADER_FILTER_TYPES)[number]),
		) || ['resourcepack', 'datapack'].includes(currentType.value),
)

const loadersNotForThisType = computed(() => {
	return (
		tags.value?.loaders
			?.filter((loader) => !loader.supported_project_types.includes(currentType.value))
			?.map((loader) => loader.name) ?? []
	)
})

const deprioritizedTags = computed(() => {
	return [...selectedFilterTags.value, ...loadersNotForThisType.value]
})

const messages = defineMessages({
	gameVersionProvidedByServer: {
		id: 'search.filter.locked.server-game-version.title',
		defaultMessage: 'Game version is provided by the server',
	},
	modLoaderProvidedByServer: {
		id: 'search.filter.locked.server-loader.title',
		defaultMessage: 'Loader is provided by the server',
	},
	providedByServer: {
		id: 'search.filter.locked.server',
		defaultMessage: 'Provided by the server',
	},
	syncFilterButton: {
		id: 'search.filter.locked.server.sync',
		defaultMessage: 'Sync with server',
	},
	gameVersionShaderMessage: {
		id: 'search.filter.game-version-shader-message',
		defaultMessage:
			'Shader packs for older versions most likely work on newer versions with only minor issues.',
	},
})

interface InstallableSearchResult extends Labrinth.Search.v2.ResultSearchProject {
	installing?: boolean
	installed?: boolean
}

async function serverInstall(project: InstallableSearchResult) {
	if (!serverData.value || !currentServerId.value) {
		handleError(new Error('No server to install to.'))
		return
	}
	project.installing = true
	try {
		const versions = await client.labrinth.versions_v2.getProjectVersions(project.project_id)

		const version = versions.find(
			(x) =>
				x.game_versions.includes(serverData.value!.mc_version!) &&
				x.loaders.includes(serverData.value!.loader!.toLowerCase()),
		)

		if (!version) {
			handleError(
				new Error(
					`No compatible version found for ${serverData.value!.mc_version} / ${serverData.value!.loader}`,
				),
			)
			project.installing = false
			return
		}

		if (projectType.value?.id === 'modpack') {
			if (fromContext.value === 'onboarding') {
				// Show creation flow modal overlay on discovery page
				const modalInstance = onboardingModalRef.value
				if (modalInstance) {
					onboardingInstallingProject.value = project
					modalInstance.show()
					await nextTick()
					const ctx = modalInstance.ctx
					ctx.setupType.value = 'modpack'
					ctx.modpackSelection.value = {
						projectId: project.project_id,
						versionId: version.id,
						name: project.title,
						iconUrl: project.icon_url ?? undefined,
					}
					ctx.modal.value?.setStage('final-config')
				}
				return
			}
			await client.archon.content_v1.installContent(currentServerId.value, currentWorldId.value!, {
				content_variant: 'modpack',
				spec: { platform: 'modrinth', project_id: project.project_id, version_id: version.id },
				soft_override: !eraseDataOnInstall.value,
			})
			project.installed = true
			navigateTo(`/hosting/manage/${currentServerId.value}/options/loader`)
		} else if (projectType.value?.id === 'mod' || projectType.value?.id === 'plugin') {
			await installContentMutation.mutateAsync({
				serverId: currentServerId.value,
				projectId: version.project_id,
				versionId: version.id,
			})
			project.installed = true
		}
	} catch (e) {
		console.error(e)
		handleError(new Error(`Error installing content ${e}`))
	}
	project.installing = false
}

const noLoad = ref(false)
const {
	data: rawResults,
	refresh: refreshSearch,
	pending: searchLoading,
} = useLazyFetch(
	() => {
		const config = useRuntimeConfig()
		const base = import.meta.server ? config.apiBaseUrl : config.public.apiBaseUrl

		const url = `${base}search${requestParams.value}`
		debug('useLazyFetch URL:', url)
		return url
	},
	{
		watch: false,
		transform: (hits) => {
			debug('useLazyFetch transform, hits:', (hits as any)?.total_hits)
			noLoad.value = false
			return hits as Labrinth.Search.v2.SearchResults
		},
	},
)

watch(searchLoading, (val) => debug('searchLoading:', val))
watch(rawResults, (val) => debug('rawResults changed, total_hits:', val?.total_hits))

const results = computed(() => rawResults.value)
const pageCount = computed(() =>
	results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

function scrollToTop(behavior: ScrollBehavior = 'smooth') {
	window.scrollTo({ top: 0, behavior })
}

function updateSearchResults(pageNumber: number = 1, resetScroll = true) {
	debug(
		'updateSearchResults called, page:',
		pageNumber,
		'query:',
		query.value,
		'requestParams:',
		requestParams.value,
	)
	currentPage.value = pageNumber
	if (resetScroll) {
		scrollToTop()
	}
	noLoad.value = true

	if (query.value === null) {
		debug('updateSearchResults: query is null, returning early')
		return
	}

	debug('updateSearchResults: calling refreshSearch')
	refreshSearch()

	if (import.meta.client) {
		const persistentParams: Record<string, any> = {}

		for (const [key, value] of Object.entries(route.query)) {
			if (PERSISTENT_QUERY_PARAMS.includes(key)) {
				persistentParams[key] = value
			}
		}

		if (serverHideInstalled.value) {
			persistentParams.shi = 'true'
		} else {
			delete persistentParams.shi
		}

		const params = {
			...persistentParams,
			...createPageParams(),
		}

		router.replace({ path: route.path, query: params })
	}
}

watch([currentFilters], () => {
	updateSearchResults(1, false)
})

const throttledSearch = useThrottleFn(() => updateSearchResults(), 500, true)

function cycleSearchDisplayMode() {
	if (!resultsDisplayLocation.value) {
		// if no display location, abort
		return
	}
	cosmetics.value.searchDisplayMode[resultsDisplayLocation.value] = cycleValue(
		cosmetics.value.searchDisplayMode[resultsDisplayLocation.value],
		tags.value.projectViewModes.filter((x) => x !== 'grid'),
	)
	setClosestMaxResults()
}

function setClosestMaxResults() {
	const maxResultsOptions = maxResultsForView.value[resultsDisplayMode.value] ?? [20]
	const currentMax = maxResults.value
	if (!maxResultsOptions.includes(currentMax)) {
		maxResults.value = maxResultsOptions.reduce((prev: number, curr: number) => {
			return Math.abs(curr - currentMax) <= Math.abs(prev - currentMax) ? curr : prev
		})
	}
}

const ogTitle = computed(
	() =>
		`Search ${projectType.value?.display ?? 'project'}s${query.value ? ' | ' + query.value : ''}`,
)
const description = computed(
	() =>
		`Search and browse thousands of Minecraft ${projectType.value?.display ?? 'project'}s on Modrinth with instant, accurate search results. Our filters help you quickly find the best Minecraft ${projectType.value?.display ?? 'project'}s.`,
)

const serverBackUrl = computed(() => {
	if (!serverData.value) return ''
	const id = serverData.value.server_id
	return fromContext.value === 'onboarding'
		? `/hosting/manage/${id}?resumeModal=setup-type`
		: `/hosting/manage/${id}/content`
})

// Onboarding modpack flow: show creation flow modal overlay on discovery page
const onboardingModalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
const onboardingInstallingProject = ref<InstallableSearchResult | null>(null)

function onOnboardingHide() {
	if (onboardingInstallingProject.value) {
		onboardingInstallingProject.value.installing = false
		onboardingInstallingProject.value = null
	}
}

function onOnboardingBack() {
	onboardingModalRef.value?.hide()
}

async function onOnboardingCreate(config: CreationFlowContextValue) {
	if (!currentServerId.value || !config.modpackSelection.value) return

	try {
		await client.archon.content_v1.installContent(currentServerId.value, currentWorldId.value!, {
			content_variant: 'modpack',
			spec: {
				platform: 'modrinth',
				project_id: config.modpackSelection.value.projectId,
				version_id: config.modpackSelection.value.versionId,
			},
			soft_override: false,
		} satisfies Archon.Content.v1.InstallWorldContent)
		await client.archon.servers_v1.endIntro(currentServerId.value)
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', currentServerId.value] })
		navigateTo(`/hosting/manage/${currentServerId.value}/content`)
	} catch (e) {
		handleError(new Error(`Error installing modpack: ${e}`))
		config.loading.value = false
	}
}

useSeoMeta({
	description,
	ogTitle,
	ogDescription: description,
})
</script>
<template>
	<Teleport v-if="flags.searchBackground" to="#absolute-background-teleport">
		<div class="search-background"></div>
	</Teleport>
	<Teleport v-if="serverData" to="#discover-header-prefix" defer>
		<div
			class="mb-4 flex flex-wrap items-center justify-between gap-3 border-0 border-b border-solid border-divider pb-4"
		>
			<button
				tabindex="-1"
				class="flex cursor-pointer flex-col gap-4 bg-transparent text-primary"
				@click="navigateTo(serverBackUrl)"
			>
				<span class="flex items-center gap-2">
					<Avatar
						:src="
							serverData.is_medal
								? 'https://cdn-raw.modrinth.com/medal_icon.webp'
								: (serverIcon ?? MinecraftServerIcon)
						"
						size="48px"
					/>
					<span class="flex flex-col gap-2">
						<span class="bold font-extrabold text-contrast">
							{{ serverData.name }}
						</span>
						<span class="flex items-center gap-2 font-semibold text-secondary">
							<GameIcon class="h-5 w-5 text-secondary" />
							{{ serverData.loader }} {{ serverData.mc_version }}
						</span>
					</span>
				</span>
			</button>
			<ButtonStyled>
				<button @click="navigateTo(serverBackUrl)">
					<LeftArrowIcon />
					{{ fromContext === 'onboarding' ? 'Back to setup' : 'Back to server' }}
				</button>
			</ButtonStyled>
		</div>
		<h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Install content to server</h1>
	</Teleport>

	<aside
		:class="{
			'normal-page__sidebar': true,
		}"
		aria-label="Filters"
	>
		<AdPlaceholder v-if="!auth.user && !serverData" />
		<div v-if="filtersMenuOpen" class="fixed inset-0 z-40 bg-bg"></div>
		<div
			class="flex flex-col gap-3"
			:class="{
				'fixed inset-0 z-50 m-4 mb-0 overflow-auto rounded-t-3xl bg-bg-raised': filtersMenuOpen,
			}"
		>
			<div
				v-if="filtersMenuOpen"
				class="sticky top-0 z-10 mx-1 flex items-center justify-between gap-3 border-0 border-b-[1px] border-solid border-divider bg-bg-raised px-6 py-4"
			>
				<h3 class="m-0 text-lg text-contrast">Filters</h3>
				<ButtonStyled circular>
					<button
						@click="
							() => {
								filtersMenuOpen = false
								scrollToTop('instant')
							}
						"
					>
						<XIcon />
					</button>
				</ButtonStyled>
			</div>
			<div
				v-if="serverData && projectType?.id === 'modpack'"
				class="card-shadow rounded-2xl bg-bg-raised"
			>
				<div class="flex flex-row items-center gap-2 px-6 py-4 text-contrast">
					<h3 class="m-0 text-lg">Options</h3>
				</div>
				<div class="flex flex-row items-center justify-between gap-2 px-6">
					<label for="erase-data-on-install"> Erase all data on install </label>
					<Toggle id="erase-data-on-install" v-model="eraseDataOnInstall" class="flex-none" />
				</div>
				<div class="px-6 py-4 text-sm">
					If enabled, existing mods, worlds, and configurations, will be deleted before installing
					the selected modpack.
				</div>
			</div>
			<div
				v-if="serverData && projectType?.id !== 'modpack'"
				class="card-shadow rounded-2xl bg-bg-raised p-4"
			>
				<Checkbox
					v-model="serverHideInstalled"
					label="Hide installed content"
					class="filter-checkbox"
					@update:model-value="updateSearchResults()"
				/>
			</div>
			<SearchSidebarFilter
				v-for="filter in filters.filter((f) => f.display !== 'none')"
				:key="`filter-${filter.id}`"
				v-model:selected-filters="currentFilters"
				v-model:toggled-groups="toggledGroups"
				v-model:overridden-provided-filter-types="overriddenProvidedFilterTypes"
				:provided-filters="serverFilters"
				:filter-type="filter"
				:class="
					filtersMenuOpen
						? 'border-0 border-b-[1px] border-solid border-divider last:border-b-0'
						: 'card-shadow rounded-2xl bg-bg-raised'
				"
				button-class="button-animation flex flex-col gap-1 px-6 py-4 w-full bg-transparent cursor-pointer border-none"
				content-class="mb-4 mx-3"
				inner-panel-class="p-1"
				:open-by-default="!(currentType === 'shader' && filter.id === 'game_version')"
			>
				<template #header>
					<h3 class="m-0 text-lg">{{ filter.formatted_name }}</h3>
				</template>
				<template v-if="currentType === 'shader' && filter.id === 'game_version'" #prefix>
					<div class="mb-4 grid grid-cols-[auto_1fr] gap-2 px-3 text-sm font-medium text-blue">
						<InfoIcon class="mt-1 size-4" />
						<span> {{ formatMessage(messages.gameVersionShaderMessage) }}</span>
					</div>
				</template>
				<template #locked-game_version>
					{{ formatMessage(messages.gameVersionProvidedByServer) }}
				</template>
				<template #locked-mod_loader>
					{{ formatMessage(messages.modLoaderProvidedByServer) }}
				</template>
				<template #sync-button> {{ formatMessage(messages.syncFilterButton) }}</template>
			</SearchSidebarFilter>
		</div>
	</aside>
	<section class="normal-page__content">
		<div class="flex flex-col gap-3">
			<StyledInput
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:placeholder="`Search ${projectType?.display ?? 'project'}s...`"
				clearable
				wrapper-class="w-full"
				input-class="!h-12"
				@input="throttledSearch()"
				@clear="updateSearchResults()"
			/>
			<div class="flex flex-wrap items-center gap-2">
				<DropdownSelect
					v-slot="{ selected }"
					v-model="currentSortType"
					class="!w-auto flex-grow md:flex-grow-0"
					name="Sort by"
					:options="[...sortTypes]"
					:display-name="(option?: SortType) => option?.display"
					@change="updateSearchResults()"
				>
					<span class="font-semibold text-primary">Sort by: </span>
					<span class="font-semibold text-secondary">{{ selected }}</span>
				</DropdownSelect>
				<DropdownSelect
					v-slot="{ selected }"
					v-model="maxResults"
					name="Max results"
					:options="currentMaxResultsOptions"
					:default-value="maxResults"
					class="!w-auto flex-grow md:flex-grow-0"
					@change="updateSearchResults()"
				>
					<span class="font-semibold text-primary">View: </span>
					<span class="font-semibold text-secondary">{{ selected }}</span>
				</DropdownSelect>
				<div class="lg:hidden">
					<ButtonStyled>
						<button @click="filtersMenuOpen = true">
							<FilterIcon />
							Filter results...
						</button>
					</ButtonStyled>
				</div>
				<ButtonStyled circular>
					<button
						:v-tooltip="capitalizeString(resultsDisplayMode + ' view')"
						:aria-label="capitalizeString(resultsDisplayMode + ' view')"
						@click="cycleSearchDisplayMode()"
					>
						<GridIcon v-if="resultsDisplayMode === 'grid'" />
						<ImageIcon v-else-if="resultsDisplayMode === 'gallery'" />
						<ListIcon v-else />
					</button>
				</ButtonStyled>
				<Pagination
					:page="currentPage"
					:count="pageCount"
					class="mx-auto sm:ml-auto sm:mr-0"
					@switch-page="updateSearchResults"
				/>
			</div>
			<SearchFilterControl
				v-model:selected-filters="currentFilters"
				:filters="filters.filter((f) => f.display !== 'none')"
				:provided-filters="serverFilters"
				:overridden-provided-filter-types="overriddenProvidedFilterTypes"
				:provided-message="messages.providedByServer"
			/>
			<LogoAnimated v-if="searchLoading && !noLoad" />
			<div v-else-if="results && results.hits && results.hits.length === 0" class="no-results">
				<p>No results found for your query!</p>
			</div>
			<div v-else class="search-results-container">
				<ProjectCardList
					aria-label="Search results"
					:layout="
						resultsDisplayMode === 'grid' || resultsDisplayMode === 'gallery' ? 'grid' : 'list'
					"
				>
					<template v-for="result in results?.hits" :key="result.project_id">
						<ProjectCard
							:link="`/${projectType?.id ?? 'project'}/${result.slug ? result.slug : result.project_id}`"
							:title="result.title"
							:icon-url="result.icon_url"
							:author="{ name: result.author, link: `/user/${result.author}` }"
							:date-updated="result.date_modified"
							:date-published="result.date_created"
							:displayed-date="currentSortType.name === 'newest' ? 'published' : 'updated'"
							:downloads="result.downloads"
							:summary="result.description"
							:tags="result.display_categories"
							:all-tags="result.categories"
							:deprioritized-tags="deprioritizedTags"
							:exclude-loaders="excludeLoaders"
							:followers="result.follows"
							:banner="result.featured_gallery ?? undefined"
							:color="result.color ?? undefined"
							:environment="
								['mod', 'modpack'].includes(currentType)
									? {
											clientSide: result.client_side,
											serverSide: result.server_side,
										}
									: undefined
							"
							:layout="
								resultsDisplayMode === 'grid' || resultsDisplayMode === 'gallery' ? 'grid' : 'list'
							"
							@mouseenter="handleProjectMouseEnter(result)"
							@mouseleave="handleProjectHoverEnd"
						>
							<template v-if="flags.showDiscoverProjectButtons || serverData" #actions>
								<template v-if="flags.showDiscoverProjectButtons">
									<ButtonStyled color="brand">
										<button>
											<DownloadIcon />
											Download
										</button>
									</ButtonStyled>
									<ButtonStyled circular>
										<button>
											<HeartIcon />
										</button>
									</ButtonStyled>
									<ButtonStyled circular>
										<button>
											<BookmarkIcon />
										</button>
									</ButtonStyled>
									<ButtonStyled circular type="transparent">
										<button>
											<MoreVerticalIcon />
										</button>
									</ButtonStyled>
								</template>
								<template v-else-if="serverData">
									<ButtonStyled color="brand" type="outlined">
										<button
											v-if="
												(result as InstallableSearchResult).installed ||
												(serverContentData &&
													(serverContentData.addons ?? []).find(
														(x) => x.project_id === result.project_id,
													)) ||
												serverData.upstream?.project_id === result.project_id
											"
											disabled
										>
											<CheckIcon />
											Installed
										</button>
										<button v-else-if="(result as InstallableSearchResult).installing" disabled>
											Installing...
										</button>
										<button v-else @click="serverInstall(result as InstallableSearchResult)">
											<DownloadIcon />
											Install
										</button>
									</ButtonStyled>
								</template>
							</template>
						</ProjectCard>
					</template>
				</ProjectCardList>
			</div>
			<div class="pagination-after">
				<pagination
					:page="currentPage"
					:count="pageCount"
					class="justify-end"
					@switch-page="updateSearchResults"
				/>
			</div>
		</div>
	</section>

	<CreationFlowModal
		v-if="currentServerId && fromContext === 'onboarding' && projectType?.id === 'modpack'"
		ref="onboardingModalRef"
		type="server-onboarding"
		:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
		:show-snapshot-toggle="true"
		:on-back="onOnboardingBack"
		@hide="onOnboardingHide"
		@browse-modpacks="() => {}"
		@create="onOnboardingCreate"
	/>
</template>
<style lang="scss" scoped>
.normal-page__content {
	// Passthrough children as grid items on mobile
	display: contents;

	@media screen and (min-width: 1024px) {
		display: block;
	}
}

// Move the filters "sidebar" on mobile underneath the search card
.normal-page__sidebar {
	grid-row: 3;

	// Always show on desktop
	@media screen and (min-width: 1024px) {
		display: block;
	}
}

.filters-card {
	padding: var(--spacing-card-md);

	@media screen and (min-width: 1024px) {
		padding: var(--spacing-card-lg);
	}
}

.sidebar-menu {
	display: none;
}

.sidebar-menu_open {
	display: block;
}

.sidebar-menu-heading {
	margin: 1.5rem 0 0.5rem 0;
}

// EthicalAds
.content-wrapper {
	grid-row: 1;
}

.search-controls {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-md);
	flex-wrap: wrap;
	padding: var(--spacing-card-md);
	grid-row: 2;

	.search-filter-container {
		display: flex;
		width: 100%;
		align-items: center;

		.sidebar-menu-close-button {
			max-height: none;
			// match height of the search field
			height: 40px;
			transition: box-shadow 0.1s ease-in-out;
			margin-right: var(--spacing-card-md);

			&.open {
				color: var(--color-button-text-active);
				background-color: var(--color-brand-highlight);
				box-shadow:
					inset 0 0 0 transparent,
					0 0 0 2px var(--color-brand);
			}
		}

		.iconified-input {
			flex: 1;

			input {
				width: 100%;
				margin: 0;
			}
		}
	}

	.sort-controls {
		width: 100%;
		display: flex;
		flex-direction: row;
		gap: var(--spacing-card-md);
		flex-wrap: wrap;
		align-items: center;

		.labeled-control {
			flex: 1;
			display: flex;
			flex-direction: column;
			align-items: center;
			flex-wrap: wrap;
			gap: 0.5rem;

			.labeled-control__label {
				white-space: nowrap;
			}
		}

		.square-button {
			margin-top: auto;
			// match height of search dropdowns
			height: 40px;
			width: 40px; // make it square!
		}
	}
}

.search-controls__sorting {
	min-width: 14rem;
}

.labeled-control__label,
.labeled-control__control {
	display: block;
}

.pagination-before {
	grid-row: 4;
}

.search-results-container {
	grid-row: 5;
}

.pagination-after {
	grid-row: 6;
}

.no-results {
	text-align: center;
	display: flow-root;
}

.loading-logo {
	margin: 2rem;
}

@media screen and (min-width: 750px) {
	.search-controls {
		flex-wrap: nowrap;
		flex-direction: row;
	}

	.sort-controls {
		min-width: fit-content;
		max-width: fit-content;
		flex-wrap: nowrap;
	}

	.labeled-control {
		align-items: center;
		display: flex;
		flex-direction: column !important;
		flex-wrap: wrap;
		gap: 0.5rem;
		max-width: fit-content;
	}

	.labeled-control__label {
		flex-shrink: 0;
		margin-bottom: 0 !important;
	}
}

@media screen and (min-width: 860px) {
	.labeled-control {
		flex-wrap: nowrap !important;
		flex-direction: row !important;
	}
}

@media screen and (min-width: 1024px) {
	.sidebar-menu {
		display: block;
		margin-top: 0;
	}

	.sidebar-menu-close-button {
		display: none;
	}

	.labeled-control {
		flex-wrap: wrap !important;
		flex-direction: column !important;
	}
}

@media screen and (min-width: 1100px) {
	.labeled-control {
		flex-wrap: nowrap !important;
		flex-direction: row !important;
	}
}

.search-background {
	width: 100%;
	height: 20rem;
	background-image: url('https://minecraft.wiki/images/The_Garden_Awakens_Key_Art_No_Creaking.jpg?9968c');
	background-size: cover;
	background-position: center;
	pointer-events: none;
	mask-image: linear-gradient(to bottom, black, transparent);
	opacity: 0.25;
}
</style>
