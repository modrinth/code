<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	DownloadIcon,
	FilterIcon,
	GameIcon,
	GridIcon,
	ImageIcon,
	LeftArrowIcon,
	ListIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonStyled,
	Checkbox,
	DropdownSelect,
	injectNotificationManager,
	NewProjectCard,
	Pagination,
	SearchFilterControl,
	SearchSidebarFilter,
	type SortType,
	useSearch,
} from '@modrinth/ui'
import { capitalizeString, cycleValue, type Mod as InstallableMod } from '@modrinth/utils'
import { useThrottleFn } from '@vueuse/core'
import { computed, type Reactive, watch } from 'vue'

import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import ProjectCard from '~/components/ui/ProjectCard.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'
import { useModrinthServers } from '~/composables/servers/modrinth-servers.ts'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const { formatMessage } = useVIntl()

const filtersMenuOpen = ref(false)

const route = useNativeRoute()
const router = useNativeRouter()

const cosmetics = useCosmetics()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const { handleError } = injectNotificationManager()

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

const server = ref<Reactive<ModrinthServer>>()
const serverHideInstalled = ref(false)
const eraseDataOnInstall = ref(false)

const PERSISTENT_QUERY_PARAMS = ['sid', 'shi']

async function updateServerContext() {
	const serverId = queryAsString(route.query.sid)

	if (!serverId) {
		server.value = undefined
		return
	}

	try {
		if (!auth.value.user) {
			router.push('/auth/sign-in?redirect=' + encodeURIComponent(route.fullPath))
			return
		}

		if (!server.value || server.value.serverId !== serverId) {
			server.value = await useModrinthServers(serverId, ['general', 'content'])
		}

		if (route.query.shi && projectType.value?.id !== 'modpack' && server.value) {
			serverHideInstalled.value = route.query.shi === 'true'
		}
	} catch (error) {
		console.error('Failed to load server context:', error)
		server.value = undefined
	}
}

if (import.meta.client && route.query.sid) {
	updateServerContext().catch((error) => {
		console.error('Failed to initialize server context:', error)
	})
}

watch(
	() => route.query.sid,
	() => {
		updateServerContext().catch((error) => {
			console.error('Failed to update server context:', error)
		})
	},
)

const serverFilters = computed(() => {
	const filters = []
	if (server.value && projectType.value?.id !== 'modpack') {
		const gameVersion = server.value.general?.mc_version
		if (gameVersion) {
			filters.push({
				type: 'game_version',
				option: gameVersion,
			})
		}

		const platform = server.value.general?.loader?.toLowerCase()

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

		if (serverHideInstalled.value) {
			const installedMods = server.value.content?.data
				.filter((x: InstallableMod) => x.project_id)
				.map((x: InstallableMod) => x.project_id)
				.filter((id): id is string => id !== undefined)

			installedMods
				.map((x: string) => ({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				}))
				.forEach((x) => filters.push(x))
		}
	}
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
})

interface InstallableSearchResult extends Labrinth.Search.v2.ResultSearchProject {
	installing?: boolean
	installed?: boolean
}

async function serverInstall(project: InstallableSearchResult) {
	if (!server.value) {
		handleError(new Error('No server to install to.'))
		return
	}
	project.installing = true
	try {
		const versions = (await useBaseFetch(
			`project/${project.project_id}/version`,
			{},
			true,
		)) as Labrinth.Versions.v2.Version[]

		const version =
			versions.find(
				(x) =>
					x.game_versions.includes(server.value!.general.mc_version) &&
					x.loaders.includes(server.value!.general.loader.toLowerCase()),
			) ?? versions[0]

		if (projectType.value?.id === 'modpack') {
			await server.value.general.reinstall(
				false,
				project.project_id,
				version.id,
				undefined,
				eraseDataOnInstall.value,
			)
			project.installed = true
			navigateTo(`/hosting/manage/${server.value.serverId}/options/loader`)
		} else if (projectType.value?.id === 'mod') {
			await server.value.content.install('mod', version.project_id, version.id)
			await server.value.refresh(['content'])
			project.installed = true
		} else if (projectType.value?.id === 'plugin') {
			await server.value.content.install('plugin', version.project_id, version.id)
			await server.value.refresh(['content'])
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

		return `${base}search${requestParams.value}`
	},
	{
		watch: false,
		transform: (hits) => {
			noLoad.value = false
			return hits as Labrinth.Search.v2.SearchResults
		},
	},
)

const results = shallowRef(toRaw(rawResults))
const pageCount = computed(() =>
	results.value ? Math.ceil(results.value.total_hits / results.value.limit) : 1,
)

function scrollToTop(behavior: ScrollBehavior = 'smooth') {
	window.scrollTo({ top: 0, behavior })
}

function updateSearchResults(pageNumber: number = 1, resetScroll = true) {
	currentPage.value = pageNumber
	if (resetScroll) {
		scrollToTop()
	}
	noLoad.value = true

	if (query.value === null) {
		return
	}

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
		tags.value.projectViewModes,
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
	<Teleport v-if="server" to="#discover-header-prefix">
		<div
			class="mb-4 flex flex-wrap items-center justify-between gap-3 border-0 border-b border-solid border-divider pb-4"
		>
			<nuxt-link
				:to="`/servers/manage/${server.serverId}/content`"
				tabindex="-1"
				class="flex flex-col gap-4 text-primary"
			>
				<span class="flex items-center gap-2">
					<Avatar
						:src="
							server.general.is_medal
								? 'https://cdn-raw.modrinth.com/medal_icon.webp'
								: server.general.image
						"
						size="48px"
					/>
					<span class="flex flex-col gap-2">
						<span class="bold font-extrabold text-contrast">
							{{ server.general.name }}
						</span>
						<span class="flex items-center gap-2 font-semibold text-secondary">
							<GameIcon class="h-5 w-5 text-secondary" />
							{{ server.general.loader }} {{ server.general.mc_version }}
						</span>
					</span>
				</span>
			</nuxt-link>
			<ButtonStyled>
				<nuxt-link :to="`/hosting/manage/${server.serverId}/content`">
					<LeftArrowIcon />
					Back to server
				</nuxt-link>
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
		<AdPlaceholder v-if="!auth.user && !server" />
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
				v-if="server && projectType?.id === 'modpack'"
				class="card-shadow rounded-2xl bg-bg-raised"
			>
				<div class="flex flex-row items-center gap-2 px-6 py-4 text-contrast">
					<h3 class="m-0 text-lg">Options</h3>
				</div>
				<div class="flex flex-row items-center justify-between gap-2 px-6">
					<label for="erase-data-on-install"> Erase all data on install </label>
					<input
						id="erase-data-on-install"
						v-model="eraseDataOnInstall"
						label="Erase all data on install"
						class="switch stylized-toggle flex-none"
						type="checkbox"
					/>
				</div>
				<div class="px-6 py-4 text-sm">
					If enabled, existing mods, worlds, and configurations, will be deleted before installing
					the selected modpack.
				</div>
			</div>
			<div
				v-if="server && projectType?.id !== 'modpack'"
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
				:open-by-default="true"
			>
				<template #header>
					<h3 class="m-0 text-lg">{{ filter.formatted_name }}</h3>
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
			<div class="iconified-input w-full">
				<SearchIcon aria-hidden="true" class="text-lg" />
				<input
					v-model="query"
					class="h-12"
					autocomplete="off"
					spellcheck="false"
					type="text"
					:placeholder="`Search ${projectType?.display ?? 'project'}s...`"
					@input="throttledSearch()"
				/>
				<Button
					v-if="query"
					class="r-btn"
					@click="
						() => {
							query = ''
							updateSearchResults()
						}
					"
				>
					<XIcon />
				</Button>
			</div>
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
				<div
					id="search-results"
					class="project-list"
					:class="'display-mode--' + resultsDisplayMode"
					role="list"
					aria-label="Search results"
				>
					<template v-for="result in results?.hits" :key="result.project_id">
						<ProjectCard
							v-if="flags.oldProjectCards"
							:id="result.slug ? result.slug : result.project_id"
							:display="resultsDisplayMode"
							:featured-image="
								result.featured_gallery ? result.featured_gallery : result.gallery[0]
							"
							:type="result.project_type"
							:author="result.author"
							:name="result.title"
							:description="result.description"
							:created-at="result.date_created"
							:updated-at="result.date_modified"
							:downloads="result.downloads.toString()"
							:follows="result.follows.toString()"
							:icon-url="result.icon_url"
							:client-side="result.client_side"
							:server-side="result.server_side"
							:categories="result.display_categories"
							:search="true"
							:show-updated-date="!server && currentSortType.name !== 'newest'"
							:show-created-date="!server"
							:hide-loaders="
								projectType ? ['resourcepack', 'datapack'].includes(projectType.id) : false
							"
							:color="result.color ?? undefined"
						>
							<template v-if="server">
								<button
									v-if="
										(result as InstallableSearchResult).installed ||
										(server?.content?.data &&
											server.content.data.find(
												(x: InstallableMod) => x.project_id === result.project_id,
											)) ||
										server.general?.project?.id === result.project_id
									"
									disabled
									class="btn btn-outline btn-primary"
								>
									<CheckIcon />
									Installed
								</button>
								<button
									v-else-if="(result as InstallableSearchResult).installing"
									disabled
									class="btn btn-outline btn-primary"
								>
									Installing...
								</button>
								<button
									v-else
									class="btn btn-outline btn-primary"
									@click="serverInstall(result as InstallableSearchResult)"
								>
									<DownloadIcon />
									Install
								</button>
							</template>
						</ProjectCard>
						<NuxtLink
							v-if="flags.newProjectCards"
							:to="`/${projectType?.id ?? 'project'}/${result.slug ? result.slug : result.project_id}`"
						>
							<NewProjectCard :project="result" :categories="result.display_categories">
								<template v-if="false" #actions></template>
							</NewProjectCard>
						</NuxtLink>
					</template>
				</div>
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

#search-results {
	min-height: 20vh;
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

.stylized-toggle:checked::after {
	background: var(--color-accent-contrast) !important;
}
</style>
