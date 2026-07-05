<template>
	<div v-if="isLoading" class="flex min-h-[50vh] items-center justify-center">
		<SpinnerIcon class="h-12 w-12 animate-spin text-brand" />
	</div>
	<div v-else-if="collection" class="flex flex-col gap-4 p-6">
		<CollectionInstallModal ref="installModal" />
		<div class="grid grid-cols-[auto_1fr] gap-4 sm:grid-cols-[auto_1fr_auto]">
			<Avatar :src="collection.icon_url" size="64px" />
			<div class="flex flex-col gap-2">
				<h1 class="m-0 text-2xl font-extrabold text-contrast">
					{{ collection.name }}
				</h1>
				<div class="flex flex-wrap items-center gap-2 text-secondary">
					<div v-if="collection.status !== 'listed'" class="flex items-center gap-1">
						<LinkIcon v-if="collection.status === 'unlisted'" aria-hidden="true" />
						<LockIcon v-else aria-hidden="true" />
						{{
							collection.status === 'unlisted'
								? formatMessage(commonMessages.unlistedLabel)
								: formatMessage(commonMessages.privateLabel)
						}}
						<span class="ml-1">•</span>
					</div>
					<span>
						{{ formatMessage(messages.projectsCount, { count: projects?.length ?? 0 }) }}
					</span>
					<template v-if="creator">
						<span>•</span>
						<span class="flex items-center gap-1.5">
							{{ formatMessage(messages.curatedByLabel) }}
							<Avatar :src="creator.avatar_url" :alt="creator.username" size="20px" circle />
							{{ creator.username }}
						</span>
					</template>
				</div>
				<p v-if="collection.description" class="m-0 break-words text-secondary">
					{{ collection.description }}
				</p>
			</div>
			<div class="col-span-2 flex items-start sm:col-span-1">
				<ButtonStyled color="brand" size="large">
					<button :disabled="displayProjects.length === 0" @click="openInstallModal">
						<DownloadIcon aria-hidden="true" />
						{{
							hasActiveFilters
								? formatMessage(messages.installFilteredButton, {
										count: displayProjects.length,
									})
								: formatMessage(messages.installAllButton, { count: displayProjects.length })
						}}
					</button>
				</ButtonStyled>
			</div>
		</div>
		<HorizontalRule />

		<template v-if="projects && projects.length > 0">
			<div class="flex flex-col gap-3">
				<div class="flex flex-wrap items-center gap-2">
					<StyledInput
						v-model="searchQuery"
						:icon="SearchIcon"
						type="text"
						autocomplete="off"
						:placeholder="formatMessage(messages.searchPlaceholder)"
						clearable
						wrapper-class="w-full flex-grow sm:w-auto"
					/>
					<Combobox
						v-model="currentSort"
						:options="sortOptions"
						class="!w-[14rem] min-w-max max-w-full flex-grow sm:flex-grow-0"
					>
						<template #prefix>
							<span class="font-semibold text-primary">
								{{ formatMessage(commonMessages.sortByLabel) }}
							</span>
						</template>
					</Combobox>
					<ButtonStyled v-if="collectionFilterTypes.length > 0">
						<button @click="filtersOpen = !filtersOpen">
							<FilterIcon aria-hidden="true" />
							{{ formatMessage(commonMessages.filtersLabel) }}
							<span
								v-if="selectedFilters.length > 0"
								class="flex h-5 min-w-5 items-center justify-center rounded-full bg-brand px-1 text-xs font-bold text-brand-inverted"
							>
								{{ selectedFilters.length }}
							</span>
							<DropdownIcon
								aria-hidden="true"
								class="h-4 w-4 transition-transform"
								:class="{ 'rotate-180': filtersOpen }"
							/>
						</button>
					</ButtonStyled>
				</div>
				<div
					v-if="filtersOpen && collectionFilterTypes.length > 0"
					class="grid grid-cols-1 items-start gap-3 sm:grid-cols-2 lg:grid-cols-3"
				>
					<SearchSidebarFilter
						v-for="filterType in collectionFilterTypes"
						:key="filterType.id"
						v-model:selected-filters="selectedFilters"
						v-model:toggled-groups="toggledGroups"
						:filter-type="filterType"
						:provided-filters="[]"
						:open-by-default="false"
						class="card-shadow rounded-2xl border border-solid bg-surface-3 border-surface-4"
						button-class="button-animation flex flex-col gap-1 px-4 py-3 w-full bg-transparent cursor-pointer border-none"
						content-class="mb-4 mx-3"
						inner-panel-class="p-1"
					>
						<template #header>
							<h3 class="m-0 text-base font-semibold text-contrast">
								{{ filterType.formatted_name }}
							</h3>
						</template>
					</SearchSidebarFilter>
				</div>
				<SearchFilterControl
					v-model:selected-filters="selectedFilters"
					:filters="collectionFilterTypes"
					:provided-filters="[]"
					:overridden-provided-filter-types="[]"
				/>
			</div>
			<ProjectCardList v-if="displayProjects.length > 0" layout="list">
				<ProjectCard
					v-for="project in displayProjects"
					:key="project.id"
					:link="`/project/${project.slug ?? project.id}`"
					:title="project.title"
					:icon-url="project.icon_url"
					:banner="project.gallery?.find((element) => element.featured)?.url"
					:summary="project.description"
					:date-updated="project.updated"
					:downloads="project.downloads ?? 0"
					:followers="project.followers ?? 0"
					:tags="project.categories"
					:environment="{
						clientSide: project.client_side,
						serverSide: project.server_side,
					}"
					:color="project.color"
					layout="list"
				>
					<template #actions>
						<ButtonStyled color="brand">
							<button
								:disabled="installingProjectIds.has(project.id)"
								@click.stop.prevent="installProject(project)"
							>
								<DownloadIcon aria-hidden="true" />
								{{
									installingProjectIds.has(project.id)
										? formatMessage(commonMessages.installingLabel)
										: formatMessage(commonMessages.installButton)
								}}
							</button>
						</ButtonStyled>
					</template>
				</ProjectCard>
			</ProjectCardList>
			<EmptyState v-else type="no-search-result">
				<template #heading>{{ formatMessage(messages.noResultsLabel) }}</template>
				<template #actions>
					<ButtonStyled v-if="searchQuery || selectedFilters.length > 0">
						<button @click="clearSearchAndFilters">
							<XIcon aria-hidden="true" />
							{{ formatMessage(messages.clearFiltersButton) }}
						</button>
					</ButtonStyled>
				</template>
			</EmptyState>
		</template>
		<EmptyState v-else type="empty-inbox">
			<template #heading>{{ formatMessage(messages.noProjectsLabel) }}</template>
		</EmptyState>
	</div>
	<EmptyState v-else type="empty-inbox">
		<template #heading>{{ formatMessage(messages.notFoundLabel) }}</template>
	</EmptyState>
</template>

<script setup>
import {
	DownloadIcon,
	DropdownIcon,
	FilterIcon,
	getCategoryIcon,
	getLoaderIcon,
	LinkIcon,
	LockIcon,
	SearchIcon,
	SpinnerIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	Combobox,
	commonMessages,
	defineMessages,
	EmptyState,
	formatCategory,
	formatCategoryHeader,
	formatLoader,
	HorizontalRule,
	injectModrinthClient,
	injectNotificationManager,
	ProjectCard,
	ProjectCardList,
	SearchFilterControl,
	SearchSidebarFilter,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import CollectionInstallModal from '@/components/ui/CollectionInstallModal.vue'
import { get_project_many, get_user } from '@/helpers/cache.js'
import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags.js'
import { injectContentInstall } from '@/providers/content-install'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const client = injectModrinthClient()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const { install } = injectContentInstall()

const messages = defineMessages({
	curatedByLabel: {
		id: 'app.collection.label.curated-by',
		defaultMessage: 'Curated by',
	},
	projectsCount: {
		id: 'app.collection.label.projects-count',
		defaultMessage: '{count, plural, =0 {No projects} one {# project} other {# projects}}',
	},
	installAllButton: {
		id: 'app.collection.button.install-all',
		defaultMessage: 'Install all ({count})',
	},
	installFilteredButton: {
		id: 'app.collection.button.install-filtered',
		defaultMessage: 'Install filtered ({count})',
	},
	searchPlaceholder: {
		id: 'app.collection.search.placeholder',
		defaultMessage: 'Search collection...',
	},
	noProjectsLabel: {
		id: 'app.collection.label.no-projects',
		defaultMessage: 'No projects in collection yet',
	},
	noResultsLabel: {
		id: 'app.collection.label.no-results',
		defaultMessage: 'No projects match your search',
	},
	clearFiltersButton: {
		id: 'app.collection.button.clear-filters',
		defaultMessage: 'Clear filters',
	},
	notFoundLabel: {
		id: 'app.collection.label.not-found',
		defaultMessage: 'Collection not found',
	},
	gameVersionFilterLabel: {
		id: 'search.filter_type.game_version',
		defaultMessage: 'Game version',
	},
	loaderFilterLabel: {
		id: 'search.filter_type.mod_loader',
		defaultMessage: 'Loader',
	},
	showAllVersionsLabel: {
		id: 'search.filter_type.game_version.all_versions',
		defaultMessage: 'Show all versions',
	},
})

const collectionId = computed(() => route.params.id)

const {
	data: collection,
	isPending: collectionIsPending,
	error: collectionError,
} = useQuery({
	queryKey: computed(() => ['collection', collectionId.value]),
	queryFn: () => client.labrinth.collections.get(collectionId.value),
	enabled: computed(() => !!collectionId.value),
})

watch(collectionError, (error) => {
	if (error) handleError(error)
})

const { data: creator } = useQuery({
	queryKey: computed(() => ['user', collection.value?.user]),
	queryFn: () => get_user(collection.value.user),
	enabled: computed(() => !!collection.value?.user),
})

const { data: projects, isFetching: projectsIsFetching } = useQuery({
	queryKey: computed(() => ['collection-projects', collection.value?.projects]),
	queryFn: async () => {
		const fetched = await get_project_many(collection.value.projects)
		const result = (fetched ?? []).filter((project) => !!project)
		for (const project of result) {
			project.categories = (project.categories ?? []).concat(project.loaders ?? [])
		}
		return result
	},
	enabled: computed(() => !!collection.value?.projects?.length),
	placeholderData: [],
})

const { data: tags } = useQuery({
	queryKey: ['collection-tags'],
	queryFn: async () => {
		const [gameVersions, loaders, categories] = await Promise.all([
			get_game_versions(),
			get_loaders(),
			get_categories(),
		])
		return { gameVersions, loaders, categories }
	},
})

const isLoading = computed(() => collectionIsPending.value || projectsIsFetching.value)

watch(
	collection,
	(newCollection) => {
		if (newCollection) {
			breadcrumbs.setName('Collection', newCollection.name)
		}
	},
	{ immediate: true },
)

const searchQuery = ref('')
const filtersOpen = ref(false)
const selectedFilters = ref([])
const toggledGroups = ref([])

const sortOptions = [
	{ value: 'downloads', label: 'Downloads' },
	{ value: 'follows', label: 'Followers' },
	{ value: 'updated', label: 'Date updated' },
	{ value: 'newest', label: 'Date published' },
	{ value: 'name', label: 'Name' },
]
const currentSort = ref('downloads')

const collectionFilterTypes = computed(() => {
	if (!tags.value) return []

	const gameVersions = new Set()
	const loaders = new Set()
	const categories = new Set()
	for (const project of projects.value ?? []) {
		project.game_versions?.forEach((version) => gameVersions.add(version))
		project.loaders?.forEach((loader) => loaders.add(loader))
		project.categories?.forEach((category) => categories.add(category))
	}
	for (const loader of loaders) {
		categories.delete(loader)
	}

	const filterTypes = []

	const gameVersionOptions = tags.value.gameVersions
		.filter((gameVersion) => gameVersions.has(gameVersion.version))
		.map((gameVersion) => ({
			id: gameVersion.version,
			toggle_group: gameVersion.version_type !== 'release' ? 'all_versions' : undefined,
			method: 'or',
			value: gameVersion.version,
		}))
	if (gameVersionOptions.length > 0) {
		filterTypes.push({
			id: 'game_version',
			formatted_name: formatMessage(messages.gameVersionFilterLabel),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'v',
			supports_negative_filter: false,
			searchable: true,
			toggle_groups: gameVersionOptions.some((option) => option.toggle_group)
				? [{ id: 'all_versions', formatted_name: formatMessage(messages.showAllVersionsLabel) }]
				: [],
			options: gameVersionOptions,
		})
	}

	const loaderOptions = tags.value.loaders
		.filter((loader) => loaders.has(loader.name))
		.map((loader) => ({
			id: loader.name,
			formatted_name: formatLoader(formatMessage, loader.name),
			icon: getLoaderIcon(loader.name),
			method: 'or',
			value: loader.name,
		}))
	if (loaderOptions.length > 0) {
		filterTypes.push({
			id: 'mod_loader',
			formatted_name: formatMessage(messages.loaderFilterLabel),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'g',
			supports_negative_filter: true,
			searchable: false,
			options: loaderOptions,
		})
	}

	const seenCategories = new Set()
	const categoryOptions = []
	for (const category of tags.value.categories) {
		if (!categories.has(category.name) || seenCategories.has(category.name)) continue
		seenCategories.add(category.name)
		categoryOptions.push({
			id: category.name,
			formatted_name: formatCategory(formatMessage, category.name),
			icon: getCategoryIcon(category.name),
			method: 'or',
			value: category.name,
		})
	}
	if (categoryOptions.length > 0) {
		filterTypes.push({
			id: 'category',
			formatted_name: formatCategoryHeader(formatMessage, 'categories'),
			supported_project_types: [],
			display: 'scrollable',
			query_param: 'f',
			supports_negative_filter: true,
			searchable: false,
			options: categoryOptions,
		})
	}

	return filterTypes
})

function projectMatchesFilters(project) {
	for (const filterType of collectionFilterTypes.value) {
		const selected = selectedFilters.value.filter((filter) => filter.type === filterType.id)
		if (selected.length === 0) continue
		const values =
			filterType.id === 'game_version'
				? (project.game_versions ?? [])
				: filterType.id === 'mod_loader'
					? (project.loaders ?? [])
					: (project.categories ?? [])
		if (selected.some((filter) => filter.negative && values.includes(filter.option))) {
			return false
		}
		const included = selected.filter((filter) => !filter.negative)
		if (included.length > 0 && !included.some((filter) => values.includes(filter.option))) {
			return false
		}
	}
	return true
}

const hasActiveFilters = computed(
	() => searchQuery.value.trim().length > 0 || selectedFilters.value.length > 0,
)

const displayProjects = computed(() => {
	const query = searchQuery.value.trim().toLowerCase()
	const filtered = (projects.value ?? []).filter(
		(project) =>
			(!query ||
				project.title?.toLowerCase().includes(query) ||
				project.description?.toLowerCase().includes(query)) &&
			projectMatchesFilters(project),
	)
	return filtered.sort((a, b) => {
		switch (currentSort.value) {
			case 'follows':
				return (b.followers ?? 0) - (a.followers ?? 0)
			case 'updated':
				return dayjs(b.updated).diff(dayjs(a.updated))
			case 'newest':
				return dayjs(b.published).diff(dayjs(a.published))
			case 'name':
				return a.title.localeCompare(b.title)
			default:
				return (b.downloads ?? 0) - (a.downloads ?? 0)
		}
	})
})

function clearSearchAndFilters() {
	searchQuery.value = ''
	selectedFilters.value = []
}

const installModal = ref(null)

function openInstallModal() {
	installModal.value?.show(displayProjects.value, {
		collectionName: collection.value?.name,
	})
}

const installingProjectIds = ref(new Set())

async function installProject(project) {
	const next = new Set(installingProjectIds.value)
	next.add(project.id)
	installingProjectIds.value = next
	try {
		await install(project.id, undefined, undefined, 'CollectionPage')
	} catch (err) {
		handleError(err)
	} finally {
		const after = new Set(installingProjectIds.value)
		after.delete(project.id)
		installingProjectIds.value = after
	}
}
</script>
