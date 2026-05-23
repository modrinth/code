<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BookmarkIcon,
	CheckIcon,
	DownloadIcon,
	GridIcon,
	HeartIcon,
	ImageIcon,
	ListIcon,
	MoreVerticalIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import type { CardAction } from '@modrinth/ui'
import {
	BrowseInstallHeader,
	BrowsePageLayout,
	BrowseSidebar,
	commonMessages,
	CreationFlowModal,
	defineMessages,
	formatProjectTypeSentence,
	injectModrinthClient,
	PROJECT_DEP_MARKER_QUERY,
	provideBrowseManager,
	SelectedProjectsFloatingBar,
	useBrowseSearch,
	useDebugLogger,
	useStickyObserver,
	useVIntl,
} from '@modrinth/ui'
import { cycleValue } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useTimeoutFn } from '@vueuse/core'
import { computed, ref, watch } from 'vue'
import type { LocationQueryRaw } from 'vue-router'

import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import type {
	ServerInstallModalHandle,
	ServerInstallSearchResult,
} from '~/composables/use-server-install-content'
import { useServerInstallContent } from '~/composables/use-server-install-content'
import { withLabrinthCanaryHeader } from '~/helpers/canary.ts'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('Discover')

const { updateDiscoverFilterContext } = useCdnDownloadContext()

const client = injectModrinthClient()
const queryClient = useQueryClient()

const filtersMenuOpen = ref(false)

const route = useRoute()

const cosmetics = useCosmetics()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

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

const handleServerProjectMouseEnter = (result: Labrinth.Search.v3.ResultSearchProject) => {
	const slug = result.slug || result.project_id

	prefetchTimeout = useTimeoutFn(
		async () => {
			queryClient.prefetchQuery(projectQueryOptions.v2(slug, client))
			queryClient.prefetchQuery(projectQueryOptions.v3(slug, client))

			const content = result.minecraft_java_server?.content
			if (content?.kind === 'modpack' && content.version_id) {
				queryClient.prefetchQuery(versionQueryOptions.v3(content.version_id, client))
			}
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

debug('initial route.params.type:', route.params.type, '→ currentType:', currentType.value)

const isServerType = computed(() => currentType.value === 'server')

const projectType = computed(() => tags.value.projectTypes.find((x) => x.id === currentType.value))

watch(
	() => projectType.value?.id,
	(val) => debug('projectType.id changed:', val),
)

const resultsDisplayLocation = computed<DisplayLocation | undefined>(
	() => projectType.value?.id as DisplayLocation,
)
const resultsDisplayMode = computed<DisplayMode>(() =>
	resultsDisplayLocation.value
		? cosmetics.value.searchDisplayMode[resultsDisplayLocation.value]
		: 'list',
)

const maxResultsForView = ref<Record<DisplayMode, number[]>>({
	list: [5, 10, 15, 20, 50, 100],
	grid: [6, 12, 18, 24, 48, 96],
	gallery: [6, 10, 16, 20, 50, 100],
})

const currentMaxResultsOptions = computed(
	() => maxResultsForView.value[resultsDisplayMode.value] ?? [20],
)

function cycleSearchDisplayMode() {
	if (!resultsDisplayLocation.value) return
	cosmetics.value.searchDisplayMode[resultsDisplayLocation.value] = cycleValue(
		cosmetics.value.searchDisplayMode[resultsDisplayLocation.value],
		tags.value.projectViewModes.filter((x) => x !== 'grid'),
	)
}

const onboardingModalRef = ref<ServerInstallModalHandle | null>(null)
const {
	currentServerId,
	fromContext,
	serverData,
	serverContentData,
	serverFilters,
	serverHideInstalled,
	hideSelectedServerInstalls,
	installingProjectIds,
	optimisticallyInstalledProjectIds,
	queuedServerInstallProjectIds,
	queuedServerInstallCount,
	isInstallingQueuedServerInstalls,
	installContext,
	setBrowseSearchState,
	syncHiddenInstalledProjectIds,
	serverInstall,
	onOnboardingHide,
	onOnboardingBack,
	onModpackFlowCreate,
} = useServerInstallContent({
	projectType,
	onboardingModalRef,
	debug,
})
const stickyInstallHeaderRef = ref<HTMLElement | null>(null)
const { isStuck: isInstallHeaderStuck } = useStickyObserver(
	stickyInstallHeaderRef,
	'DiscoverInstallHeader',
)

function getServerModpackContent(project: Labrinth.Search.v3.ResultSearchProject) {
	const content = project.minecraft_java_server?.content
	if (content?.kind === 'modpack') {
		const { project_name, project_icon, project_id } = content
		if (!project_name) return undefined
		return {
			name: project_name,
			icon: project_icon ?? undefined,
			onclick:
				project_id !== project.project_id
					? () =>
							navigateTo({
								path: `/project/${project_id}`,
								query: { ...PROJECT_DEP_MARKER_QUERY },
							})
					: undefined,
			showCustomModpackTooltip: project_id === project.project_id,
		}
	}
	return undefined
}

type DiscoverProjectSearchHit = Labrinth.Search.v2.ResultSearchProject & {
	version_id?: string | null
}

function mapV3ProjectHit(hit: Labrinth.Search.v3.ResultSearchProject): DiscoverProjectSearchHit {
	return {
		...hit,
		project_type: hit.project_types[0] ?? projectTypeId.value,
		title: hit.name,
		description: hit.summary,
		versions: hit.version_id ? [hit.version_id] : [],
		latest_version: hit.version_id,
		icon_url: hit.icon_url ?? '',
		client_side: 'unknown',
		server_side: 'unknown',
	}
}

const hostingContextQuery = computed(() => {
	const query: LocationQueryRaw = {}
	const hasHostingContext = route.query.sid != null

	for (const key of ['sid', 'wid', 'from', 'shi']) {
		const value = route.query[key]
		if (value != null) {
			query[key] = value
		}
	}

	if (hasHostingContext) {
		query.b = route.fullPath
	}

	return Object.keys(query).length > 0 ? query : undefined
})

function withHostingContext(path: string) {
	return hostingContextQuery.value ? { path, query: hostingContextQuery.value } : path
}

async function fetchSearch(requestParams: string) {
	debug('search() called', {
		requestParams: requestParams.substring(0, 100),
		isServer: isServerType.value,
		projectTypeId: projectTypeId.value,
	})

	const raw = await client.request<Labrinth.Search.v3.SearchResults>('/search', {
		api: 'labrinth',
		version: 3,
		method: 'GET',
		params: Object.fromEntries(new URLSearchParams(requestParams.replace(/^\?/, ''))),
		headers: withLabrinthCanaryHeader(),
	})

	debug('search() response', { total_hits: raw.total_hits, hitCount: raw.hits?.length })

	if (isServerType.value) {
		return {
			projectHits: [],
			serverHits: raw.hits,
			total_hits: raw.total_hits,
			per_page: raw.hits_per_page,
		}
	}

	return {
		projectHits: raw.hits.map(mapV3ProjectHit),
		serverHits: [],
		total_hits: raw.total_hits,
		per_page: raw.hits_per_page,
	}
}

async function search(requestParams: string) {
	return await queryClient.ensureQueryData({
		queryKey: ['discover', 'search', 'v3', requestParams],
		queryFn: () => fetchSearch(requestParams),
		staleTime: 30_000,
	})
}

function getCardActions(
	result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
	currentProjectType: string,
): CardAction[] {
	if (currentProjectType === 'server') return []

	const projectResult = result as ServerInstallSearchResult

	if (flags.value.showDiscoverProjectButtons) {
		return [
			{
				key: 'download',
				label: 'Download',
				icon: DownloadIcon,
				color: 'brand',
				onClick: () => {},
			},
			{
				key: 'heart',
				label: '',
				icon: HeartIcon,
				circular: true,
				onClick: () => {},
			},
			{
				key: 'bookmark',
				label: '',
				icon: BookmarkIcon,
				circular: true,
				onClick: () => {},
			},
			{
				key: 'more',
				label: '',
				icon: MoreVerticalIcon,
				circular: true,
				type: 'transparent',
				onClick: () => {},
			},
		]
	}

	if (serverData.value) {
		const isQueued = queuedServerInstallProjectIds.value.has(result.project_id)
		const isInstalled =
			projectResult.installed ||
			optimisticallyInstalledProjectIds.value.has(result.project_id) ||
			(serverContentData.value &&
				(serverContentData.value.addons ?? []).find((x) => x.project_id === result.project_id)) ||
			serverData.value.upstream?.project_id === result.project_id
		const isInstalling = installingProjectIds.value.has(result.project_id)
		const isInstallingSelection = isInstallingQueuedServerInstalls.value
		const validatingInstall =
			isInstalling && currentProjectType !== 'modpack' && !isInstallingSelection
		const installLabel = isInstalled
			? formatMessage(commonMessages.installedLabel)
			: isQueued
				? isInstalling || isInstallingSelection
					? validatingInstall
						? formatMessage(commonMessages.validatingLabel)
						: formatMessage(commonMessages.installingLabel)
					: formatMessage(commonMessages.selectedLabel)
				: isInstalling || isInstallingSelection
					? validatingInstall
						? formatMessage(commonMessages.validatingLabel)
						: formatMessage(commonMessages.installingLabel)
					: formatMessage(commonMessages.installButton)

		return [
			{
				key: 'install',
				label: installLabel,
				icon:
					isInstalling || isInstallingSelection
						? SpinnerIcon
						: isQueued || isInstalled
							? CheckIcon
							: DownloadIcon,
				iconClass: isInstalling || isInstallingSelection ? 'animate-spin' : undefined,
				disabled: !!isInstalled || isInstalling || isInstallingSelection,
				color: isQueued && !isInstalling && !isInstallingSelection ? 'green' : 'brand',
				type: 'outlined',
				onClick: () => serverInstall(projectResult),
			},
		]
	}

	return []
}

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
	seoTitle: {
		id: 'discover.seo.title',
		defaultMessage: 'Search {projectType}',
	},
	seoTitleWithQuery: {
		id: 'discover.seo.title-with-query',
		defaultMessage: 'Search {projectType} | {query}',
	},
	seoDescription: {
		id: 'discover.seo.description',
		defaultMessage:
			'Search and browse thousands of Minecraft {projectType} on Modrinth with instant, accurate search results. Our filters help you quickly find the best Minecraft {projectType}.',
	},
	gameVersionShaderMessage: {
		id: 'search.filter.game-version-shader-message',
		defaultMessage:
			'Shader packs for older versions most likely work on newer versions with only minor issues.',
	},
})

const projectTypeId = computed(() => projectType.value?.id ?? 'mod')

debug('projectTypeId:', projectTypeId.value)
watch(projectTypeId, (val) => debug('projectTypeId changed:', val))

const searchState = useBrowseSearch({
	projectType: projectTypeId,
	tags,
	providedFilters: serverFilters,
	search,
	persistentQueryParams: ['sid', 'wid', 'shi', 'from'],
	getExtraQueryParams: () => ({
		shi: serverHideInstalled.value ? 'true' : undefined,
	}),
	maxResultsOptions: currentMaxResultsOptions,
	displayMode: resultsDisplayMode,
})
setBrowseSearchState(searchState)

watch(
	() =>
		searchState.isServerType.value
			? searchState.serverCurrentFilters.value
			: searchState.currentFilters.value,
	(filters) => updateDiscoverFilterContext(filters),
	{ deep: true, immediate: true },
)

watch(
	[
		() => searchState.query.value,
		() => searchState.currentFilters.value,
		() => searchState.serverCurrentFilters.value,
		() => projectTypeId.value,
	],
	() => {
		syncHiddenInstalledProjectIds()
	},
	{ deep: true },
)

debug('calling initial refreshSearch')
await searchState.refreshSearch()

const ogTitle = computed(() =>
	searchState.query.value
		? formatMessage(messages.seoTitleWithQuery, {
				projectType: formatProjectTypeSentence(
					formatMessage,
					projectType.value?.id ?? 'project',
					2,
				),
				query: searchState.query.value,
			})
		: formatMessage(messages.seoTitle, {
				projectType: formatProjectTypeSentence(
					formatMessage,
					projectType.value?.id ?? 'project',
					2,
				),
			}),
)
const description = computed(() =>
	formatMessage(messages.seoDescription, {
		projectType: formatProjectTypeSentence(formatMessage, projectType.value?.id ?? 'project', 2),
	}),
)

useSeoMeta({
	description,
	ogTitle,
	ogDescription: description,
})

debug('calling provideBrowseManager')
provideBrowseManager({
	tags,
	projectType: projectTypeId,
	...searchState,
	getProjectLink: (result: Labrinth.Search.v2.ResultSearchProject) =>
		withHostingContext(
			`/${projectType.value?.id ?? 'project'}/${result.slug ? result.slug : result.project_id}`,
		),
	getServerProjectLink: (result: Labrinth.Search.v3.ResultSearchProject) =>
		withHostingContext(`/server/${result.slug ?? result.project_id}`),
	selectableProjectTypes: computed(() => []),
	showProjectTypeTabs: computed(() => false),
	variant: 'web',
	getCardActions,
	installContext,
	providedFilters: serverFilters,
	hideInstalled: serverHideInstalled,
	showHideInstalled: computed(() => !!serverData.value && projectType.value?.id !== 'modpack'),
	hideInstalledLabel: computed(() => formatMessage(commonMessages.hideInstalledContentLabel)),
	hideSelected: hideSelectedServerInstalls,
	showHideSelected: computed(
		() =>
			!!serverData.value &&
			projectType.value?.id !== 'modpack' &&
			queuedServerInstallCount.value > 0,
	),
	hideSelectedLabel: computed(() => formatMessage(commonMessages.hideSelectedContentLabel)),
	displayMode: resultsDisplayMode,
	cycleDisplayMode: cycleSearchDisplayMode,
	maxResultsOptions: currentMaxResultsOptions,
	getServerModpackContent,
	onProjectHover: handleProjectMouseEnter,
	onServerProjectHover: handleServerProjectMouseEnter,
	onProjectHoverEnd: handleProjectHoverEnd,
	filtersMenuOpen,
	lockedFilterMessages: {
		gameVersion: formatMessage(messages.gameVersionProvidedByServer),
		modLoader: formatMessage(messages.modLoaderProvidedByServer),
		syncButton: formatMessage(messages.syncFilterButton),
		providedBy: formatMessage(messages.providedByServer),
		gameVersionShaderMessage: formatMessage(messages.gameVersionShaderMessage),
	},
	loadingComponent: LogoAnimated,
})
</script>
<template>
	<Teleport v-if="flags.searchBackground" to="#absolute-background-teleport">
		<div class="search-background"></div>
	</Teleport>

	<div
		v-if="installContext"
		ref="stickyInstallHeaderRef"
		class="sticky top-0 z-20 -mx-6 border-0 border-solid border-divider bg-surface-1 px-6 pt-4"
		:class="[isInstallHeaderStuck ? 'border-t' : '']"
	>
		<BrowseInstallHeader divider bottom-padding />
	</div>

	<SelectedProjectsFloatingBar v-if="installContext" :install-context="installContext" />

	<div
		class="grid min-w-0 gap-3"
		:class="
			cosmetics.rightSearchLayout
				? 'lg:grid-cols-[minmax(0,1fr)_18.75rem]'
				: 'lg:grid-cols-[18.75rem_minmax(0,1fr)]'
		"
	>
		<section
			class="flex min-w-0 flex-col gap-3"
			:class="cosmetics.rightSearchLayout ? 'lg:order-1' : 'lg:order-2'"
		>
			<BrowsePageLayout>
				<template #display-mode-icon>
					<GridIcon v-if="resultsDisplayMode === 'grid'" />
					<ImageIcon v-else-if="resultsDisplayMode === 'gallery'" />
					<ListIcon v-else />
				</template>
			</BrowsePageLayout>
		</section>

		<aside
			class="min-w-0"
			:class="cosmetics.rightSearchLayout ? 'lg:order-2' : 'lg:order-1'"
			:aria-label="formatMessage(commonMessages.filtersLabel)"
		>
			<BrowseSidebar>
				<template #prepend>
					<AdPlaceholder v-if="!auth.user && !serverData" />
				</template>
			</BrowseSidebar>
		</aside>
	</div>

	<CreationFlowModal
		v-if="currentServerId && projectType?.id === 'modpack'"
		ref="onboardingModalRef"
		:type="fromContext === 'reset-server' ? 'reset-server' : 'server-onboarding'"
		:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
		:show-snapshot-toggle="true"
		:on-back="onOnboardingBack"
		@hide="onOnboardingHide"
		@browse-modpacks="() => {}"
		@create="onModpackFlowCreate"
	/>
</template>
<style lang="scss" scoped>
.search-background {
	width: 100%;
	height: 20rem;
	background-image: url('https://minecraft.wiki/images/Tiny_Takeover_Key_Art.png?025dc');
	background-size: cover;
	background-position: 50% 25%;
	pointer-events: none;
	mask-image: linear-gradient(to bottom, black, transparent);
	opacity: 0.25;
}
</style>
