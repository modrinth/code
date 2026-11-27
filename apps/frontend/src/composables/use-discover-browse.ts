import type { Labrinth } from '@modrinth/api-client'
import { BookmarkIcon, DownloadIcon, HeartIcon, MoreVerticalIcon } from '@modrinth/assets'
import type {
	BrowseHit,
	BrowseInstallCapability,
	BrowseInstallContentType,
	BrowseToggle,
	CardAction,
	EnvironmentSearchOverride,
	FilterValue,
	ServerContentEnvironment,
	ServerSetupModalHandle,
} from '@modrinth/ui'
import {
	commonMessages,
	defineMessages,
	getHostingModEnvironmentOverride,
	getProjectVersionFilterQuery,
	injectModrinthClient,
	injectNotificationManager,
	injectUserPreferences,
	isServerContentEnvironmentProjectType,
	parseServerContentEnvironment,
	PROJECT_DEP_MARKER_QUERY,
	provideBrowseManager,
	useBrowseViewState,
	useVIntl,
} from '@modrinth/ui'
import { cycleValue } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import { useTimeoutFn } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'
import type { LocationQueryRaw } from 'vue-router'

import { navigateTo, useRoute } from '#app'
import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import { projectQueryOptions, warmProjectCheckCaches } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { useDiscoverProjectTypeLinks } from '~/composables/use-discover-project-type-links'
import { useServerInstallContent } from '~/composables/use-server-install-content'
import { withLabrinthCanaryHeader } from '~/helpers/canary.ts'
import { getProjectTypeForUrl } from '~/helpers/projects'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const HOVER_DURATION_TO_PREFETCH_MS = 500
const HOSTING_CONTEXT_PARAMS = ['sid', 'wid', 'from', 'shi']
const SERVER_INSTALL_TYPES: BrowseInstallContentType[] = [
	'modpack',
	'mod',
	'plugin',
	'datapack',
	'resourcepack',
	'shader',
]
const MOD_LOADERS = ['fabric', 'forge', 'quilt', 'neoforge']
const PLUGIN_LOADERS = ['paper', 'purpur']

const layoutPreferenceKeys = {
	mod: 'mods',
	plugin: 'plugins',
	datapack: 'datapacks',
	shader: 'shaders',
	resourcepack: 'resourcepacks',
	modpack: 'modpacks',
	server: 'servers',
	user: 'users',
} as const satisfies Partial<Record<DisplayLocation, keyof Labrinth.Users.v3.LayoutPreferences>>

const maxResultsForView: Record<DisplayMode, number[]> = {
	list: [5, 10, 15, 20, 50, 100],
	grid: [6, 12, 18, 24, 48, 96],
	gallery: [6, 10, 16, 20, 50, 100],
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
	gameVersionShaderMessage: {
		id: 'search.filter.game-version-shader-message',
		defaultMessage:
			'Shader packs for older versions most likely work on newer versions with only minor issues.',
	},
	unsupportedContentType: {
		id: 'discover.install.error.unsupported-content-type',
		defaultMessage: 'This content type cannot be installed to a server from Discover.',
	},
})

function negativeProjectFilters(projectIds: Iterable<string>): FilterValue[] {
	return Array.from(projectIds, (id) => ({
		type: 'project_id',
		option: `project_id:${id}`,
		negative: true,
	}))
}

function parseSearchParams(requestParams: string): Labrinth.Search.SearchParams {
	const params = new URLSearchParams(requestParams.replace(/^\?/, ''))
	return {
		query: params.get('query') ?? undefined,
		offset: params.get('offset') ?? undefined,
		index: params.get('index') ?? undefined,
		limit: params.get('limit') ?? undefined,
		new_filters: params.get('new_filters') ?? undefined,
	}
}

function stripUnrenderedFields(hits: BrowseHit[]): BrowseHit[] {
	return hits.map((hit) => {
		const { dependencies, dependency_project_ids, compatible_dependency_project_ids, ...rendered } =
			hit as BrowseHit & Record<string, unknown>
		return rendered as BrowseHit
	})
}

export function useDiscoverBrowse({
	setupModal,
	filtersMenuOpen,
}: {
	setupModal: Ref<ServerSetupModalHandle | null>
	filtersMenuOpen: Ref<boolean>
}) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const { updatePreferences } = injectUserPreferences()
	const { updateDiscoverFilterContext } = useCdnDownloadContext()
	const route = useRoute()
	const cosmetics = useCosmetics()
	const tags = useGeneratedState()
	const flags = useFeatureFlags()

	const currentType = computed(() =>
		queryAsStringOrEmpty(route.params.type).replaceAll(/^\/|s\/?$/g, ''),
	)
	const projectType = computed(
		() => tags.value.projectTypes.find((type) => type.id === currentType.value)?.id ?? 'mod',
	)
	const projectTypeForUrl = computed(() =>
		['mod', 'plugin', 'datapack'].includes(projectType.value) ? 'mod' : projectType.value,
	)

	const displayLocation = computed(() => projectType.value as DisplayLocation)
	function displayModeFor(type: string): DisplayMode {
		return cosmetics.value.searchDisplayMode[type as DisplayLocation] ?? 'list'
	}
	function maxResultsOptionsFor(type: string) {
		return maxResultsForView[displayModeFor(type)] ?? [20]
	}
	const displayMode = computed(() => displayModeFor(displayLocation.value))

	function cycleDisplayMode() {
		const next = cycleValue(
			cosmetics.value.searchDisplayMode[displayLocation.value],
			tags.value.projectViewModes.filter((mode) => mode !== 'grid'),
		)
		cosmetics.value.searchDisplayMode[displayLocation.value] = next

		const preferenceKey =
			layoutPreferenceKeys[displayLocation.value as keyof typeof layoutPreferenceKeys]
		if (!preferenceKey) return

		void updatePreferences({
			layouts: {
				[preferenceKey]: next === 'list' ? 'rows' : 'grid',
			} as Partial<Labrinth.Users.v3.LayoutPreferences>,
		}).catch(() => undefined)
	}

	const { serverId, serverData, platform, installedProjectIds, serverInstall, target } =
		useServerInstallContent({ setupModal })
	const isSetup = serverInstall.isSetup

	const hideInstalled = ref(route.query.shi === 'true' && projectType.value !== 'modpack')
	const hideSelected = ref(false)
	const hiddenInstalledIds = ref(new Set<string>())
	const hiddenInstalledIdsInitialized = ref(false)

	function syncHiddenInstalledIds() {
		hiddenInstalledIds.value = new Set(installedProjectIds.value)
		hiddenInstalledIdsInitialized.value = true
	}

	watch(
		installedProjectIds,
		(ids) => {
			if (ids.size > 0 && !hiddenInstalledIdsInitialized.value) syncHiddenInstalledIds()
		},
		{ immediate: true },
	)
	watch(hideInstalled, (hide) => {
		if (hide) syncHiddenInstalledIds()
	})
	watch(
		() => serverInstall.plans.value.size,
		(count) => {
			if (count === 0) hideSelected.value = false
		},
	)

	const serverEnvironment = ref<ServerContentEnvironment | null>(
		parseServerContentEnvironment(route.query.env),
	)
	const showServerEnvironment = computed(() => !!serverId.value && !isSetup.value)

	watch(
		[serverEnvironment, currentType, showServerEnvironment],
		([environment, type, show]) => {
			if (!show) return
			const allowed = type !== 'modpack' && isServerContentEnvironmentProjectType(environment, type)
			if (allowed && (route.query.env ?? null) === environment) return
			void navigateTo(
				{
					path: allowed ? route.path : '/discover/mods',
					query: { ...route.query, env: environment ?? undefined },
				},
				{ replace: true },
			)
		},
		{ immediate: true },
	)

	function environmentOverrideFor(type: string): EnvironmentSearchOverride | undefined {
		if (showServerEnvironment.value && type === 'mod') {
			return getHostingModEnvironmentOverride(serverEnvironment.value)
		}
		return isSetup.value ? getHostingModEnvironmentOverride(null) : undefined
	}
	const environmentOverride = computed(() => environmentOverrideFor(projectType.value))

	function providedFiltersFor(type: string): FilterValue[] {
		const filters: FilterValue[] = []

		if (serverData.value && type !== 'modpack' && !isSetup.value) {
			const { gameVersion } = platform.value
			if (gameVersion) filters.push({ type: 'game_version', option: gameVersion })

			const loader = platform.value.loader?.toLowerCase()
			if (type === 'mod' && loader && MOD_LOADERS.includes(loader)) {
				filters.push({ type: 'mod_loader', option: loader })
			}
			if (type === 'plugin' && loader && PLUGIN_LOADERS.includes(loader)) {
				filters.push({ type: 'plugin_loader', option: loader })
			}
			if (hideInstalled.value) filters.push(...negativeProjectFilters(hiddenInstalledIds.value))
			if (hideSelected.value) {
				filters.push(...negativeProjectFilters(serverInstall.projectIds.value))
			}
		}

		if (serverId.value && type === 'modpack') {
			filters.push({ type: 'environment', option: 'server' })
		}
		return filters
	}
	const providedFilters = computed(() => providedFiltersFor(projectType.value))

	const { projectTypeLinks, isServerContext } = useDiscoverProjectTypeLinks(
		computed(() => target.value?.loader),
	)

	const hostingContextQuery = computed(() => {
		const query: LocationQueryRaw = {}
		for (const key of HOSTING_CONTEXT_PARAMS) {
			const value = route.query[key]
			if (value != null) query[key] = value
		}
		return Object.keys(query).length > 0 ? query : undefined
	})

	async function fetchSearch(requestParams: string) {
		const raw = await client.labrinth.projects_v3.search(parseSearchParams(requestParams), {
			headers: withLabrinthCanaryHeader(),
		})
		return {
			hits: stripUnrenderedFields(raw.hits ?? []),
			total_hits: raw.total_hits,
		}
	}

	const view = useBrowseViewState({
		projectType,
		tags,
		searchQuery: (params) => ({
			queryKey: ['discover', 'search', 'v3', params],
			queryFn: () => fetchSearch(params),
			staleTime: 30_000,
		}),
		providedFilters,
		environmentOverride,
		persistentQueryParams: ['sid', 'wid', 'shi', 'env', 'from'],
		extraQueryParams: () => ({
			shi: hideInstalled.value ? 'true' : undefined,
			env: (showServerEnvironment.value && serverEnvironment.value) || undefined,
		}),
		maxResultsOptions: computed(() => maxResultsOptionsFor(displayLocation.value)),
		displayMode,
		prefetch: {
			projectTypes: computed(() => projectTypeLinks.value.map((link) => link.type.slice(0, -1))),
			providedFilters: providedFiltersFor,
			environmentOverride: environmentOverrideFor,
			maxResultsOptions: maxResultsOptionsFor,
		},
	})

	watch(view.hits, (hits) => warmProjectCheckCaches(queryClient, hits), { immediate: true })
	watch(view.filters, (filters) => updateDiscoverFilterContext(filters), {
		deep: true,
		immediate: true,
	})
	watch([view.query, view.filters, projectType], syncHiddenInstalledIds, { deep: true })

	const toggles = computed<BrowseToggle[]>(() => {
		if (!serverData.value || projectType.value === 'modpack') return []
		const result: BrowseToggle[] = [
			{
				id: 'hide-installed',
				label: formatMessage(commonMessages.hideInstalledContentLabel),
				model: hideInstalled,
			},
		]
		if (serverInstall.plans.value.size > 0) {
			result.push({
				id: 'hide-selected',
				label: formatMessage(commonMessages.hideSelectedContentLabel),
				model: hideSelected,
			})
		}
		return result
	})

	function installContentType(): BrowseInstallContentType | null {
		const type = projectType.value as BrowseInstallContentType
		return SERVER_INSTALL_TYPES.includes(type) ? type : null
	}

	const install: BrowseInstallCapability = {
		state: (hit) => {
			if (!serverData.value || view.isServerType.value || flags.value.showDiscoverProjectButtons) {
				return null
			}
			return serverInstall.stateOf(hit.project_id, {
				installed:
					!isSetup.value &&
					(installedProjectIds.value.has(hit.project_id) ||
						serverData.value.upstream?.project_id === hit.project_id),
				contentType: projectType.value,
			})
		},
		run: (hit) => {
			const contentType = installContentType()
			if (!contentType) {
				handleError(new Error(formatMessage(messages.unsupportedContentType)))
				return
			}
			return serverInstall.install(hit, {
				contentType,
				selectedFilters: view.filters.value,
				providedFilters: providedFilters.value,
				overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
				getProjectVersions: (projectId) =>
					client.labrinth.versions_v2.getProjectVersions(projectId, { include_changelog: false }),
			})
		},
		idleIcon: DownloadIcon,
	}

	function previewActions(): CardAction[] {
		const noop = () => {}
		return [
			{ key: 'download', label: 'Download', icon: DownloadIcon, color: 'brand', onClick: noop },
			{ key: 'heart', label: '', icon: HeartIcon, circular: true, onClick: noop },
			{ key: 'bookmark', label: '', icon: BookmarkIcon, circular: true, onClick: noop },
			{
				key: 'more',
				label: '',
				icon: MoreVerticalIcon,
				circular: true,
				type: 'transparent',
				onClick: noop,
			},
		]
	}

	function serverModpackContent(hit: BrowseHit) {
		const content = hit.minecraft_java_server?.content
		if (content?.kind !== 'modpack' || !content.project_name) return undefined
		return {
			name: content.project_name,
			icon: content.project_icon ?? undefined,
			onclick:
				content.project_id !== hit.project_id
					? () =>
							navigateTo({
								path: `/project/${content.project_id}`,
								query: { ...PROJECT_DEP_MARKER_QUERY },
							})
					: undefined,
			showCustomModpackTooltip: content.project_id === hit.project_id,
		}
	}

	let prefetchTimeout: ReturnType<typeof useTimeoutFn> | null = null

	function prefetch(hit: BrowseHit) {
		warmProjectCheckCaches(queryClient, hit)
		void queryClient.prefetchQuery(projectQueryOptions.v2(hit.project_id, client))
		void queryClient.prefetchQuery(projectQueryOptions.v3(hit.project_id, client))

		if (view.isServerType.value) {
			const content = hit.minecraft_java_server?.content
			if (content?.kind === 'modpack' && content.version_id) {
				void queryClient.prefetchQuery(versionQueryOptions.v3(content.version_id, client))
			}
			return
		}

		void queryClient.prefetchQuery(projectQueryOptions.members(hit.project_id, client))
		void queryClient.prefetchQuery(projectQueryOptions.dependencies(hit.project_id, client))
		void queryClient.prefetchQuery(projectQueryOptions.versionsV3(hit.project_id, client))
	}

	const advancedFiltersCollapsed = computed({
		get: () => flags.value.advancedFiltersCollapsed,
		set: (value) => {
			flags.value.advancedFiltersCollapsed = value
			saveFeatureFlags()
		},
	})

	const dismissedPhotosensitivityWarning = computed({
		get: () => flags.value.dismissedPhotosensitivityFilterWarning,
		set: (value) => {
			flags.value.dismissedPhotosensitivityFilterWarning = value
			saveFeatureFlags()
		},
	})

	provideBrowseManager({
		view,
		tags,
		projectType,
		selection: serverInstall.selection,
		install,
		actions: () =>
			flags.value.showDiscoverProjectButtons && !view.isServerType.value ? previewActions() : [],
		providedFilters,
		hiddenFilterTypes: computed(() => (showServerEnvironment.value ? ['environment'] : [])),
		lockedFilterMessages: {
			gameVersion: formatMessage(messages.gameVersionProvidedByServer),
			modLoader: formatMessage(messages.modLoaderProvidedByServer),
			syncButton: formatMessage(messages.syncFilterButton),
			providedBy: formatMessage(messages.providedByServer),
			gameVersionShaderMessage: formatMessage(messages.gameVersionShaderMessage),
		},
		toggles,
		serverEnvironment: { selected: serverEnvironment, shown: showServerEnvironment },
		links: {
			project: (hit) => ({
				path: `/${getProjectTypeForUrl(projectTypeForUrl.value, hit.loaders)}/${hit.slug ?? hit.project_id}`,
				query: {
					...hostingContextQuery.value,
					...getProjectVersionFilterQuery({
						contentType: projectType.value,
						selectedFilters: view.filters.value,
						providedFilters: providedFilters.value,
						overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
					}),
				},
			}),
			server: (hit) => {
				const path = `/server/${hit.slug ?? hit.project_id}`
				return hostingContextQuery.value ? { path, query: hostingContextQuery.value } : path
			},
			organization: (organizationId) => `/organization/${organizationId}`,
		},
		serverModpackContent,
		display: { mode: displayMode, cycle: cycleDisplayMode },
		preferences: { advancedFiltersCollapsed, dismissedPhotosensitivityWarning },
		filtersMenuOpen,
		loadingComponent: LogoAnimated,
		onHover: (hit) => {
			prefetchTimeout = useTimeoutFn(() => prefetch(hit), HOVER_DURATION_TO_PREFETCH_MS, {
				immediate: false,
			})
			prefetchTimeout.start()
		},
		onHoverEnd: () => prefetchTimeout?.stop(),
	})

	return {
		view,
		projectType,
		projectTypeLinks,
		isServerContext,
		serverId,
		serverInstall,
		target,
	}
}
