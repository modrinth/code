import type { Labrinth } from '@modrinth/api-client'
import type {
	BrowseHit,
	BrowseInstallCapability,
	BrowseInstallContentType,
	BrowseInstallState,
	BrowseProjectTypeLink,
	BrowseTags,
	BrowseToggle,
} from '@modrinth/ui'
import {
	commonMessages,
	defineMessages,
	getLatestMatchingInstallVersion,
	getProjectVersionFilterQuery,
	getSelectedInstallPreferences,
	getTargetInstallPreferences,
	injectNotificationManager,
	isServerContentEnvironmentProjectType,
	preferencesDiffer,
	provideBrowseManager,
	resolveInstallPlan,
	useBrowseViewState,
	useVIntl,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useEventListener } from '@vueuse/core'
import type { Ref } from 'vue'
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_project, get_search_results_v3, get_version_many } from '@/helpers/cache.js'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { injectContentInstall } from '@/providers/content-install'
import { injectServerInstall } from '@/providers/server-install'

import { browseTagsQueryOptions, settleQuery } from './queries'
import type { AppBrowseTargets } from './use-app-browse-targets'
import { useAppServerBrowse } from './use-app-server-browse'

interface ProjectTypeTab {
	label: string
	type: string
	shown?: boolean
}

const SERVER_INSTALL_TYPES: BrowseInstallContentType[] = [
	'modpack',
	'mod',
	'plugin',
	'datapack',
	'resourcepack',
	'shader',
]

const messages = defineMessages({
	addToAnInstance: {
		id: 'app.browse.add-to-an-instance',
		defaultMessage: 'Add to an instance',
	},
	environmentProvidedByServer: {
		id: 'search.filter.locked.server-environment.title',
		defaultMessage: 'Only client-side mods can be added to the server instance',
	},
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
	syncWithServer: {
		id: 'search.filter.locked.server.sync',
		defaultMessage: 'Sync with server',
	},
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
	hideAddedServers: {
		id: 'app.browse.hide-added-servers',
		defaultMessage: 'Hide servers already added',
	},
	hideInstalledModpacks: {
		id: 'app.browse.hide-installed-modpacks',
		defaultMessage: 'Hide already installed',
	},
	modpacksProjectType: {
		id: 'app.browse.project-type.modpacks',
		defaultMessage: 'Modpacks',
	},
	pluginsProjectType: {
		id: 'app.browse.project-type.plugins',
		defaultMessage: 'Plugins',
	},
	modsProjectType: { id: 'app.browse.project-type.mods', defaultMessage: 'Mods' },
	resourcePacksProjectType: {
		id: 'app.browse.project-type.resource-packs',
		defaultMessage: 'Resource Packs',
	},
	dataPacksProjectType: {
		id: 'app.browse.project-type.data-packs',
		defaultMessage: 'Data Packs',
	},
	shadersProjectType: { id: 'app.browse.project-type.shaders', defaultMessage: 'Shaders' },
	serversProjectType: { id: 'app.browse.project-type.servers', defaultMessage: 'Servers' },
})

export function useBrowseTags() {
	const { handleError } = injectNotificationManager()
	const tagsQuery = useQuery(browseTagsQueryOptions())

	watch(tagsQuery.error, (error) => {
		if (error) handleError(error)
	})

	const tags = computed<BrowseTags>(() => ({
		categories: tagsQuery.data.value?.categories ?? [],
		loaders: tagsQuery.data.value?.loaders ?? [],
		gameVersions: tagsQuery.data.value?.gameVersions ?? [],
	}))

	return {
		tags,
		ready: () => settleQuery(tagsQuery),
	}
}

async function getInstallProjectVersions(projectId: string) {
	const project = await get_project(projectId, 'must_revalidate')
	return (await get_version_many(
		project.versions,
		'must_revalidate',
	)) as Labrinth.Versions.v2.Version[]
}

export function useAppBrowseManager({
	tags,
	targets,
}: {
	tags: Readonly<Ref<BrowseTags>>
	targets: AppBrowseTargets
}) {
	const { formatMessage } = useVIntl()
	const { handleError } = injectNotificationManager()
	const { installingServerProjects, playServerProject, showAddServerToInstanceModal } =
		injectServerInstall()
	const { install: installVersion } = injectContentInstall()
	const queryClient = useQueryClient()
	const route = useRoute()
	const router = useRouter()
	const appSettings = useAppSettings()

	const { active, projectType, server, instanceTarget, serverFilters, providedFilters, target } =
		targets
	const { serverInstall } = server
	const { instance, isServerInstance } = instanceTarget
	const isServerContext = server.isServerContext
	const isSetup = serverInstall.isSetup
	const isFromWorlds = server.isFromWorlds

	const appServerBrowse = useAppServerBrowse({
		instance,
		isFromWorlds,
		allInstalledIds: instanceTarget.installedIds,
		markInstalled: instanceTarget.markInstalled,
		installingServerProjects,
		playServerProject,
		showAddServerToInstanceModal,
		handleError,
		router,
	})

	const offline = ref(!navigator.onLine)
	useEventListener(window, 'offline', () => (offline.value = true))
	useEventListener(window, 'online', () => (offline.value = false))

	async function search(params: string) {
		const response = (await get_search_results_v3(params, 'must_revalidate')) as {
			result: Labrinth.Search.v3.SearchResults
		} | null
		const hits = response?.result.hits ?? []
		for (const hit of hits) {
			for (const identifier of [hit.project_id, hit.slug]) {
				if (identifier) queryClient.setQueryData(['projects', 'summary', identifier], hit)
			}
		}
		return { hits, total_hits: response?.result.total_hits ?? 0 }
	}

	const projectTypeTabs = computed<ProjectTypeTab[]>(() => {
		const tab = (label: string, type: string, shown?: boolean) => ({ label, type, shown })

		if (isSetup.value) {
			return [
				tab(formatMessage(messages.modpacksProjectType), 'modpack'),
				tab(formatMessage(messages.modsProjectType), 'mod'),
				tab(formatMessage(messages.pluginsProjectType), 'plugin'),
				tab(formatMessage(messages.dataPacksProjectType), 'datapack'),
			]
		}

		if (isServerContext.value) {
			const shownForEnvironment = (type: string) =>
				isServerContentEnvironmentProjectType(serverFilters.environment.value, type)
			const serverLoader = server.platform.value.loader?.toLowerCase()
			return [
				tab(formatMessage(messages.modsProjectType), 'mod'),
				tab(
					formatMessage(messages.pluginsProjectType),
					'plugin',
					shownForEnvironment('plugin') &&
						tags.value.loaders.some(
							(loader) =>
								loader.name === serverLoader && loader.supported_project_types.includes('plugin'),
						),
				),
				tab(
					formatMessage(messages.resourcePacksProjectType),
					'resourcepack',
					shownForEnvironment('resourcepack'),
				),
				tab(
					formatMessage(messages.dataPacksProjectType),
					'datapack',
					shownForEnvironment('datapack'),
				),
				tab(formatMessage(messages.shadersProjectType), 'shader', shownForEnvironment('shader')),
			]
		}

		if (isFromWorlds.value) {
			return [tab(formatMessage(messages.serversProjectType), 'server')]
		}

		const current = instance.value
		const gameVersions = tags.value.gameVersions
		const supportsDataPacks =
			!current ||
			(gameVersions.findIndex((version) => version.version === current.game_version) <=
				gameVersions.findIndex((version) => version.version === '1.13') &&
				!isServerInstance.value)

		return [
			tab(formatMessage(messages.modpacksProjectType), 'modpack', !current),
			tab(formatMessage(messages.modsProjectType), 'mod', !current || current.loader !== 'vanilla'),
			tab(formatMessage(messages.resourcePacksProjectType), 'resourcepack'),
			tab(formatMessage(messages.dataPacksProjectType), 'datapack', supportsDataPacks),
			tab(formatMessage(messages.shadersProjectType), 'shader'),
			tab(formatMessage(messages.serversProjectType), 'server', !current),
		]
	})

	const projectTypes = computed<BrowseProjectTypeLink[]>(() => {
		const params = new URLSearchParams()
		for (const key of ['i', 'ai', 'from', 'sid'] as const) {
			const value = route.query[key]
			if (typeof value === 'string' && value) params.set(key, value)
		}
		if (server.worldId.value) params.set('wid', server.worldId.value)
		if (serverFilters.showEnvironment.value && serverFilters.environment.value) {
			params.set('env', serverFilters.environment.value)
		}
		const queryString = params.toString()
		const suffix = queryString ? `?${queryString}` : ''
		return projectTypeTabs.value.map(({ label, type, shown }) => ({
			label,
			href: `/browse/${type}${suffix}`,
			shown,
		}))
	})

	const view = useBrowseViewState({
		projectType,
		tags,
		searchQuery: (params) => ({
			queryKey: ['search', 'v3', params],
			queryFn: () => search(params),
			staleTime: 30_000,
			networkMode: 'always',
		}),
		providedFilters,
		environmentOverride: serverFilters.environmentOverride,
		active,
		persistentQueryParams: ['i', 'ai', 'shi', 'env', 'sid', 'wid', 'from'],
		extraQueryParams: () => ({
			sid: server.serverId.value || undefined,
			wid: server.worldId.value || undefined,
			ai: instanceTarget.hideInstalled.value ? 'true' : undefined,
			shi: serverFilters.hideInstalled.value ? 'true' : undefined,
			env: (serverFilters.showEnvironment.value && serverFilters.environment.value) || undefined,
		}),
		initialServerFilters: instance.value?.game_version
			? [{ type: 'server_game_version', option: instance.value.game_version }]
			: [],
		prefetch: {
			projectTypes: computed(() =>
				projectTypeTabs.value.filter((tab) => tab.shown !== false).map((tab) => tab.type),
			),
			providedFilters: targets.providedFiltersFor,
			environmentOverride: serverFilters.environmentOverrideFor,
		},
	})

	watch(
		view.hits,
		(hits) => {
			if (view.isServerType.value) appServerBrowse.updateServerHits(hits)
		},
		{ immediate: true },
	)

	watch(
		[view.query, view.filters, projectType],
		() => {
			if (isServerContext.value && projectType.value !== 'modpack') {
				serverFilters.syncHiddenInstalledIds()
			} else if (instance.value || projectType.value === 'modpack') {
				instanceTarget.syncHiddenIds()
			}
		},
		{ deep: true },
	)

	const clientFilterPreselectedFor = ref<string | null>(null)
	watch(
		[instance, projectType],
		([current, type]) => {
			if (current?.shared_instance?.role !== 'member' || type !== 'mod') return
			if (clientFilterPreselectedFor.value === current.id) return
			clientFilterPreselectedFor.value = current.id
			if (view.filters.value.some((filter) => filter.type === 'environment')) return
			view.filters.value = [
				...view.filters.value,
				{ type: 'environment', option: 'client', negative: false },
			]
		},
		{ immediate: true },
	)

	const instanceInstallingIds = ref(new Set<string>())

	function setInstanceInstalling(projectId: string, installing: boolean) {
		const next = new Set(instanceInstallingIds.value)
		if (installing) next.add(projectId)
		else next.delete(projectId)
		instanceInstallingIds.value = next
	}

	function selectedPreferences(type: string) {
		return getSelectedInstallPreferences({
			contentType: type,
			selectedFilters: view.filters.value,
			providedFilters: providedFilters.value,
			overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
		})
	}

	async function chooseInstanceVersion(hit: BrowseHit, type: string) {
		const current = instance.value
		if (!current) return null

		const selected = selectedPreferences(type)
		const targetPreferences = getTargetInstallPreferences(
			{ gameVersion: current.game_version, loader: current.loader },
			type,
		)
		if (!preferencesDiffer(selected, targetPreferences)) return null

		const version = getLatestMatchingInstallVersion(
			await getInstallProjectVersions(hit.project_id),
			selected,
			current.update_channel,
		)
		return version?.id ?? null
	}

	async function chooseModpackVersion(hit: BrowseHit, type: string) {
		const plan = await resolveInstallPlan({
			project: { project_id: hit.project_id, name: hit.name, icon_url: hit.icon_url },
			contentType: type as BrowseInstallContentType,
			selectedFilters: view.filters.value,
			providedFilters: providedFilters.value,
			overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
			targetPreferences: {},
			getProjectVersions: getInstallProjectVersions,
		})
		return plan.versionId
	}

	async function installToInstance(hit: BrowseHit, type: string) {
		const isModpack = hit.project_types?.includes('modpack')
		setInstanceInstalling(hit.project_id, true)
		try {
			const versionId = instance.value
				? await chooseInstanceVersion(hit, type)
				: isModpack
					? await chooseModpackVersion(hit, type)
					: null
			const selected = selectedPreferences(type)
			await installVersion(
				hit.project_id,
				versionId,
				instance.value ? instance.value.id : null,
				'SearchCard',
				(installedVersionId, installedProjectIds) => {
					setInstanceInstalling(hit.project_id, false)
					if (installedVersionId) {
						instanceTarget.markInstalled(installedProjectIds ?? [hit.project_id])
					}
				},
				(profile) => {
					router.push(`/instance/${profile}`)
				},
				{
					preferredLoader: instance.value?.loader ?? selected.loaders?.[0],
					preferredGameVersion: instance.value?.game_version ?? selected.gameVersions?.[0],
				},
			)
		} catch (error) {
			setInstanceInstalling(hit.project_id, false)
			handleError(error)
		}
	}

	function isInstalled(hit: BrowseHit) {
		return (
			instanceTarget.installedIds.value.has(hit.project_id) ||
			server.installedProjectIds.value.has(hit.project_id) ||
			server.serverData.value?.upstream?.project_id === hit.project_id
		)
	}

	function installsToServer(type: string) {
		return isServerContext.value && SERVER_INSTALL_TYPES.includes(type as BrowseInstallContentType)
	}

	const install: BrowseInstallCapability = {
		state: (hit): BrowseInstallState | null => {
			if (view.isServerType.value) return null
			const type = projectType.value
			const installed = isInstalled(hit) && type !== 'modpack' && !isSetup.value

			if (installsToServer(type)) {
				return serverInstall.stateOf(hit.project_id, { installed, contentType: type })
			}
			if (instanceInstallingIds.value.has(hit.project_id)) return 'installing'
			return installed ? 'installed' : 'idle'
		},
		run: (hit) => {
			const type = projectType.value
			if (installsToServer(type)) {
				return serverInstall.install(hit, {
					contentType: type as BrowseInstallContentType,
					selectedFilters: view.filters.value,
					providedFilters: providedFilters.value,
					overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
					getProjectVersions: getInstallProjectVersions,
				})
			}
			return installToInstance(hit, type)
		},
		idleLabel: (hit) =>
			isServerContext.value || instance.value || hit.project_types?.includes('modpack')
				? formatMessage(commonMessages.installButton)
				: formatMessage(messages.addToAnInstance),
	}

	const toggles = computed<BrowseToggle[]>(() => {
		const result: BrowseToggle[] = []
		if (projectType.value === 'modpack') {
			result.push({
				id: 'hide-installed',
				label: formatMessage(messages.hideInstalledModpacks),
				model: instanceTarget.hideInstalledModpacks,
			})
		} else if (isServerContext.value) {
			result.push({
				id: 'hide-installed',
				label: formatMessage(commonMessages.hideInstalledContentLabel),
				model: serverFilters.hideInstalled,
			})
			if (serverInstall.plans.value.size > 0) {
				result.push({
					id: 'hide-selected',
					label: formatMessage(commonMessages.hideSelectedContentLabel),
					model: serverFilters.hideSelected,
				})
			}
		} else if (instance.value) {
			result.push({
				id: 'hide-installed',
				label: formatMessage(
					isFromWorlds.value ? messages.hideAddedServers : commonMessages.hideInstalledContentLabel,
				),
				model: instanceTarget.hideInstalled,
			})
		}
		return result
	})

	const lockedFilterMessages = computed(() => {
		const fromServer = isServerContext.value
		return {
			gameVersion: formatMessage(
				fromServer ? messages.gameVersionProvidedByServer : messages.gameVersionProvidedByInstance,
			),
			modLoader: formatMessage(
				fromServer ? messages.modLoaderProvidedByServer : messages.modLoaderProvidedByInstance,
			),
			environment: formatMessage(messages.environmentProvidedByServer),
			syncButton: formatMessage(fromServer ? messages.syncWithServer : messages.syncFilterButton),
			providedBy: formatMessage(
				fromServer ? messages.providedByServer : messages.providedByInstance,
			),
		}
	})

	function savedSetting<
		K extends 'advancedFiltersCollapsed' | 'dismissedPhotosensitivityFilterWarning',
	>(
		key: K,
		settingsKey: 'advanced_filters_collapsed' | 'dismissed_photosensitivity_filter_warning',
	) {
		return computed({
			get: () => appSettings[key],
			set: (value: boolean) => {
				appSettings[key] = value
				getSettings()
					.then((settings) => {
						settings[settingsKey] = value
						return setSettings(settings)
					})
					.catch(handleError)
			},
		})
	}

	function projectBrowseQuery() {
		if (!active.value || !target.value) return undefined
		return { ...route.query, b: route.fullPath }
	}

	provideBrowseManager({
		view,
		tags,
		projectType,
		projectTypes,
		selection: serverInstall.selection,
		install,
		actions: (hit) => (view.isServerType.value ? appServerBrowse.getServerCardActions(hit) : []),
		providedFilters,
		hiddenFilterTypes: computed(() => (serverFilters.showEnvironment.value ? ['environment'] : [])),
		lockedFilterMessages,
		toggles,
		serverEnvironment: {
			selected: serverFilters.environment,
			shown: serverFilters.showEnvironment,
		},
		links: {
			project: (hit) => ({
				path: `/project/${hit.project_id ?? hit.slug}`,
				query: {
					...projectBrowseQuery(),
					...getProjectVersionFilterQuery({
						contentType: projectType.value,
						selectedFilters: view.filters.value,
						providedFilters: providedFilters.value,
						overriddenProvidedFilterTypes: view.overriddenProvidedFilterTypes.value,
					}),
				},
			}),
			server: (hit) => ({
				path: `/project/${hit.slug ?? hit.project_id}`,
				query: projectBrowseQuery(),
			}),
			organization: (organizationId) => `https://modrinth.com/organization/${organizationId}`,
		},
		serverPings: appServerBrowse.serverPings,
		serverModpackContent: appServerBrowse.getServerModpackContent,
		preferences: {
			advancedFiltersCollapsed: savedSetting(
				'advancedFiltersCollapsed',
				'advanced_filters_collapsed',
			),
			dismissedPhotosensitivityWarning: savedSetting(
				'dismissedPhotosensitivityFilterWarning',
				'dismissed_photosensitivity_filter_warning',
			),
		},
		offline,
		onContextMenu: appServerBrowse.handleRightClick,
	})

	return {
		contextMenuRef: appServerBrowse.contextMenuRef,
	}
}
