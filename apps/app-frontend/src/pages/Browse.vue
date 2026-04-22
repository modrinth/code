<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	CheckIcon,
	ClipboardCopyIcon,
	ExternalIcon,
	GlobeIcon,
	PlayIcon,
	PlusIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import type { CardAction, ProjectType, Tags } from '@modrinth/ui'
import {
	BrowsePageLayout,
	BrowseSidebar,
	commonMessages,
	CreationFlowModal,
	defineMessages,
	injectNotificationManager,
	provideBrowseManager,
	useBrowseSearch,
	useDebugLogger,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { convertFileSrc } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { Ref } from 'vue'
import { computed, onUnmounted, ref, shallowRef, watch } from 'vue'
import type { LocationQuery } from 'vue-router'
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router'

import ContextMenu from '@/components/ui/ContextMenu.vue'
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
import { add_server_to_profile, get_profile_worlds, getServerLatency } from '@/helpers/worlds'
import { injectContentInstall } from '@/providers/content-install'
import { injectServerInstall } from '@/providers/server-install'
import {
	createServerInstallContent,
	provideServerInstallContent,
} from '@/providers/setup/server-install-content'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { getServerAddress } from '@/store/install.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { installingServerProjects, playServerProject, showAddServerToInstanceModal } =
	injectServerInstall()
const { install: installVersion } = injectContentInstall()
const queryClient = useQueryClient()
const debugLog = useDebugLogger('Browse')

const router = useRouter()
const route = useRoute()
const serverSetupModalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
const serverInstallContent = createServerInstallContent({ serverSetupModalRef })
provideServerInstallContent(serverInstallContent)
const {
	serverIdQuery,
	serverFlowFrom,
	isFromWorlds,
	isServerContext,
	isSetupServerContext,
	effectiveServerWorldId,
	serverContextServerData,
	serverContentProjectIds,
	serverBackUrl,
	serverBackLabel,
	serverBrowseHeading,
	initServerContext,
	watchServerContextChanges,
	searchServerModpacks,
	getServerProjectVersions,
	enforceSetupModpackRoute,
	installProjectToServer,
	onServerFlowBack,
	handleServerModpackFlowCreate,
	markServerProjectInstalled,
} = serverInstallContent

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

if (isFromWorlds.value && route.params.projectType !== 'server') {
	router.replace({
		path: '/browse/server',
		query: route.query,
	})
}

enforceSetupModpackRoute(route.params.projectType as string | undefined)

const allInstalledIds = computed(
	() => new Set([...newlyInstalled.value, ...(installedProjectIds.value ?? [])]),
)

watchServerContextChanges()

await initInstanceContext()

async function initInstanceContext() {
	debugLog('initInstanceContext', {
		queryI: route.query.i,
		queryAi: route.query.ai,
		querySid: route.query.sid,
		queryWid: route.query.wid,
		queryFrom: route.query.from,
	})
	await initServerContext()

	if (route.query.i) {
		instance.value = (await getInstance(route.query.i as string).catch(handleError)) ?? null
		debugLog('instance loaded', {
			name: instance.value?.name,
			loader: instance.value?.loader,
			gameVersion: instance.value?.game_version,
		})

		if (route.query.from === 'worlds') {
			get_profile_worlds(route.query.i as string)
				.then((worlds) => {
					const serverProjectIds = worlds
						.filter((w) => w.type === 'server' && 'project_id' in w && w.project_id)
						.map((w) => (w as { project_id: string }).project_id)
					debugLog('installedServerProjectIds loaded', { count: serverProjectIds.length })
					installedProjectIds.value = serverProjectIds
				})
				.catch(handleError)
		} else {
			getInstalledProjectIds(route.query.i as string)
				.then((ids) => {
					debugLog('installedProjectIds loaded', { count: ids?.length })
					installedProjectIds.value = ids
				})
				.catch(handleError)
		}

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

	if (route.query.ai && !(route.params.projectType === 'modpack')) {
		debugLog('setting instanceHideInstalled from query', route.query.ai)
		instanceHideInstalled.value = route.query.ai === 'true'
	}
}

const instanceFilters = computed(() => {
	const filters = []

	if (instance.value) {
		const gameVersion = instance.value.game_version
		if (gameVersion) {
			filters.push({ type: 'game_version', option: gameVersion })
		}

		const platform = instance.value.loader
		const supportedModLoaders = ['fabric', 'forge', 'quilt', 'neoforge']

		if (platform && projectType.value === 'mod' && supportedModLoaders.includes(platform)) {
			filters.push({ type: 'mod_loader', option: platform })
		}

		if (isServerInstance.value) {
			filters.push({ type: 'environment', option: 'client' })
		}

		if (
			instanceHideInstalled.value &&
			(installedProjectIds.value || newlyInstalled.value.length > 0)
		) {
			const allInstalled = [...(installedProjectIds.value ?? []), ...newlyInstalled.value]
			allInstalled
				.map((x) => ({ type: 'project_id', option: `project_id:${x}`, negative: true }))
				.forEach((x) => filters.push(x))
		}
	}

	return filters
})

const serverHideInstalled = ref(false)
if (route.query.shi) {
	serverHideInstalled.value = route.query.shi === 'true'
}

const serverContextFilters = computed(() => {
	const filters: { type: string; option: string; negative?: boolean }[] = []
	if (!serverContextServerData.value) return filters
	const pt = projectType.value

	if (pt !== 'modpack') {
		const gameVersion = serverContextServerData.value.mc_version
		if (gameVersion) filters.push({ type: 'game_version', option: gameVersion })

		const platform = serverContextServerData.value.loader?.toLowerCase()
		if (platform && ['fabric', 'forge', 'quilt', 'neoforge'].includes(platform))
			filters.push({ type: 'mod_loader', option: platform })
		if (platform && ['paper', 'purpur'].includes(platform))
			filters.push({ type: 'plugin_loader', option: platform })

		if (pt === 'mod') filters.push({ type: 'environment', option: 'server' })
	}

	if (pt === 'modpack') {
		filters.push(
			{ type: 'environment', option: 'client' },
			{ type: 'environment', option: 'server' },
		)
	}

	if (serverHideInstalled.value && serverContentProjectIds.value.size > 0) {
		for (const id of serverContentProjectIds.value) {
			filters.push({ type: 'project_id', option: `project_id:${id}`, negative: true })
		}
	}

	return filters
})

const combinedProvidedFilters = computed(() =>
	isServerContext.value ? serverContextFilters.value : instanceFilters.value,
)

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
	checkServerRunningStates(lastServerHits.value)
}

const lastServerHits = shallowRef<Labrinth.Search.v3.ResultSearchProject[]>([])

async function handleAddServerToInstance(project: Labrinth.Search.v3.ResultSearchProject) {
	debugLog('handleAddServerToInstance', { projectId: project.project_id, name: project.name })
	const address = getServerAddress(project.minecraft_java_server)
	if (!address) return

	if (instance.value) {
		try {
			await add_server_to_profile(
				instance.value.path,
				project.name,
				address,
				'prompt',
				project.project_id,
				project.minecraft_java_server?.content?.kind,
			)
			newlyInstalled.value.push(project.project_id)
		} catch (err) {
			handleError(err as Error)
		}
	} else {
		showAddServerToInstanceModal(project.name, address)
	}
}

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

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	debugLog('went offline')
	offline.value = true
})
window.addEventListener('online', () => {
	debugLog('went online')
	offline.value = false
})

const messages = defineMessages({
	addServerToInstance: {
		id: 'app.browse.add-server-to-instance',
		defaultMessage: 'Add server to instance',
	},
	addServersToInstance: {
		id: 'app.browse.add-servers-to-instance',
		defaultMessage: 'Add servers to your instance',
	},
	addToInstance: {
		id: 'app.browse.add-to-instance',
		defaultMessage: 'Add to instance',
	},
	addToInstanceName: {
		id: 'app.browse.add-to-instance-name',
		defaultMessage: 'Add to {instanceName}',
	},
	added: {
		id: 'app.browse.added',
		defaultMessage: 'Added',
	},
	alreadyAdded: {
		id: 'app.browse.already-added',
		defaultMessage: 'Already added',
	},
	discoverContent: {
		id: 'app.browse.discover-content',
		defaultMessage: 'Discover content',
	},
	discoverServers: {
		id: 'app.browse.discover-servers',
		defaultMessage: 'Discover servers',
	},
	environmentProvidedByServer: {
		id: 'search.filter.locked.server-environment.title',
		defaultMessage: 'Only client-side mods can be added to the server instance',
	},
	gameVersionProvidedByInstance: {
		id: 'search.filter.locked.instance-game-version.title',
		defaultMessage: 'Game version is provided by the instance',
	},
	gameVersionProvidedByServer: {
		id: 'search.filter.locked.server-game-version.title',
		defaultMessage: 'Game version is provided by the server',
	},
	hideAddedServers: {
		id: 'app.browse.hide-added-servers',
		defaultMessage: 'Hide added servers',
	},
	hideInstalledContent: {
		id: 'app.browse.hide-installed-content',
		defaultMessage: 'Hide installed content',
	},
	installContentToInstance: {
		id: 'app.browse.install-content-to-instance',
		defaultMessage: 'Install content to instance',
	},
	modLoaderProvidedByInstance: {
		id: 'search.filter.locked.instance-loader.title',
		defaultMessage: 'Loader is provided by the instance',
	},
	modLoaderProvidedByServer: {
		id: 'search.filter.locked.server-loader.title',
		defaultMessage: 'Loader is provided by the server',
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

const breadcrumbs = useBreadcrumbs()
const browseTitle = computed(() =>
	formatMessage(isFromWorlds.value ? messages.discoverServers : messages.discoverContent),
)
breadcrumbs.setName('BrowseTitle', browseTitle.value)
if (instance.value) {
	const instanceLink = `/instance/${encodeURIComponent(instance.value.path)}`
	breadcrumbs.setContext({
		name: instance.value.name,
		link: isFromWorlds.value ? `${instanceLink}/worlds` : instanceLink,
	})
} else {
	breadcrumbs.setContext(null)
}

onBeforeRouteLeave(() => {
	breadcrumbs.setContext({
		name: browseTitle.value,
		link: `/browse/${projectType.value}`,
		query: route.query,
	})
})

const projectType = ref<ProjectType>(route.params.projectType as ProjectType)

watch(
	() => route.params.projectType as ProjectType,
	async (newType) => {
		if (isSetupServerContext.value) {
			enforceSetupModpackRoute(newType)
			if (newType !== 'modpack') return
		}

		if (!newType || newType === projectType.value) return

		debugLog('projectType route param changed', { from: projectType.value, to: newType })
		projectType.value = newType

		if (!route.query.i && instance.value) {
			debugLog('instance context removed, resetting')
			instance.value = null
			installedProjectIds.value = null
			instanceHideInstalled.value = false
			newlyInstalled.value = []
			isServerInstance.value = false
			breadcrumbs.setName('BrowseTitle', formatMessage(messages.discoverContent))
			breadcrumbs.setContext(null)
		}
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

	if (route.query.i) params.i = route.query.i
	if (route.query.ai) params.ai = route.query.ai
	if (route.query.from) params.from = route.query.from
	if (route.query.sid) params.sid = route.query.sid
	if (effectiveServerWorldId.value) params.wid = effectiveServerWorldId.value

	const queryString = new URLSearchParams(params as Record<string, string>).toString()
	const suffix = queryString ? `?${queryString}` : ''

	if (isSetupServerContext.value) {
		return [{ label: 'Modpacks', href: `/browse/modpack${suffix}` }]
	}

	if (isFromWorlds.value) {
		return [{ label: 'Servers', href: `/browse/server${suffix}` }]
	}

	return [
		{ label: 'Modpacks', href: `/browse/modpack${suffix}`, shown: modpacks },
		{ label: 'Mods', href: `/browse/mod${suffix}`, shown: mods },
		{ label: 'Resource Packs', href: `/browse/resourcepack${suffix}` },
		{ label: 'Data Packs', href: `/browse/datapack${suffix}`, shown: dataPacks },
		{ label: 'Shaders', href: `/browse/shader${suffix}` },
		{ label: 'Servers', href: `/browse/server${suffix}`, shown: !instance.value },
	]
})

const getServerModpackContent = (project: Labrinth.Search.v3.ResultSearchProject) => {
	const content = project.minecraft_java_server?.content
	if (content?.kind === 'modpack') {
		const { project_name, project_icon, project_id } = content
		if (!project_name) return undefined
		return {
			name: project_name,
			icon: project_icon ?? undefined,
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

const contextMenuRef = ref(null)
// @ts-expect-error - no event types
const handleRightClick = (event, result) => {
	// @ts-ignore
	contextMenuRef.value?.showMenu(event, result, [{ name: 'open_link' }, { name: 'copy_link' }])
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

const installContext = computed(() => {
	if (isServerContext.value && serverContextServerData.value) {
		return {
			name: serverContextServerData.value.name,
			loader: serverContextServerData.value.loader ?? '',
			gameVersion: serverContextServerData.value.mc_version ?? '',
			serverId: serverIdQuery.value,
			upstream: serverContextServerData.value.upstream,
			iconSrc: null as string | null,
			isMedal: serverContextServerData.value.is_medal,
			backUrl: serverBackUrl.value,
			backLabel: serverBackLabel.value,
			heading: serverBrowseHeading.value,
		}
	}
	if (instance.value) {
		return {
			name: instance.value.name,
			loader: instance.value.loader,
			gameVersion: instance.value.game_version,
			iconSrc: instance.value.icon_path ? convertFileSrc(instance.value.icon_path) : null,
			backUrl: `/instance/${encodeURIComponent(instance.value.path)}${isFromWorlds.value ? '/worlds' : ''}`,
			backLabel: 'Back to instance',
			heading: formatMessage(
				isFromWorlds.value ? messages.addServersToInstance : messages.installContentToInstance,
			),
			warning:
				isServerInstance.value && !isFromWorlds.value
					? 'Adding content can break compatibility when joining the server. Any added content will also be lost when you update the server instance content.'
					: undefined,
		}
	}
	return null
})

const installingProjectIds = ref<Set<string>>(new Set())

function getCardActions(
	result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
	currentProjectType: string,
): CardAction[] {
	if (currentProjectType === 'server') {
		const serverResult = result as Labrinth.Search.v3.ResultSearchProject
		const isInstalled = allInstalledIds.value.has(serverResult.project_id)

		if (isFromWorlds.value && instance.value) {
			return [
				{
					key: 'add-to-instance',
					label: formatMessage(isInstalled ? messages.added : messages.addToInstance),
					icon: isInstalled ? CheckIcon : PlusIcon,
					disabled: isInstalled,
					color: 'brand',
					type: 'outlined',
					onClick: () => handleAddServerToInstance(serverResult),
				},
			]
		}

		const actions: CardAction[] = []

		actions.push({
			key: 'add',
			label: '',
			icon: isInstalled ? CheckIcon : PlusIcon,
			disabled: isInstalled,
			circular: true,
			tooltip: isInstalled
				? formatMessage(messages.alreadyAdded)
				: instance.value
					? formatMessage(messages.addToInstanceName, { instanceName: instance.value.name })
					: formatMessage(messages.addServerToInstance),
			onClick: () => handleAddServerToInstance(serverResult),
		})

		if (runningServerProjects.value[serverResult.project_id]) {
			actions.push({
				key: 'stop',
				label: formatMessage(commonMessages.stopButton),
				icon: StopCircleIcon,
				color: 'red',
				type: 'outlined',
				onClick: () => handleStopServerProject(serverResult.project_id),
			})
		} else {
			const isInstalling = (installingServerProjects.value as string[]).includes(
				serverResult.project_id,
			)
			actions.push({
				key: 'play',
				label: formatMessage(
					isInstalling ? commonMessages.installingLabel : commonMessages.playButton,
				),
				icon: PlayIcon,
				disabled: isInstalling,
				color: 'brand',
				type: 'outlined',
				onClick: () => handlePlayServerProject(serverResult.project_id),
			})
		}

		return actions
	}

	// Non-server project actions
	const projectResult = result as (Labrinth.Search.v2.ResultSearchProject &
		Labrinth.Search.v3.ResultSearchProject) & {
		installed?: boolean
		installing?: boolean
	}
	const isInstalled =
		projectResult.installed ||
		allInstalledIds.value.has(projectResult.project_id || '') ||
		serverContentProjectIds.value.has(projectResult.project_id || '')
	const isInstalling = installingProjectIds.value.has(projectResult.project_id)

	if (
		isServerContext.value &&
		['modpack', 'mod', 'plugin', 'datapack'].includes(currentProjectType)
	) {
		return [
			{
				key: 'install',
				label: isInstalling ? 'Installing' : isInstalled ? 'Installed' : 'Install',
				icon: isInstalled ? CheckIcon : PlusIcon,
				disabled: isInstalled || isInstalling,
				color: 'brand',
				type: 'outlined',
				onClick: async () => {
					installingProjectIds.value.add(projectResult.project_id)
					try {
						const didInstall = await installProjectToServer(projectResult)
						if (didInstall !== false) {
							onSearchResultInstalled(projectResult.project_id)
						}
					} catch (err) {
						handleError(err as Error)
					} finally {
						installingProjectIds.value.delete(projectResult.project_id)
					}
				},
			},
		]
	}

	const isModpack = projectResult.project_types?.includes('modpack')
	const shouldUseInstallIcon = !!instance.value || isModpack

	return [
		{
			key: 'install',
			label: isInstalling
				? 'Installing'
				: isInstalled
					? 'Installed'
					: shouldUseInstallIcon
						? 'Install'
						: 'Add to an instance',
			icon: isInstalling ? SpinnerIcon : isInstalled ? CheckIcon : PlusIcon,
			disabled: isInstalled || isInstalling,
			color: 'brand',
			type: 'outlined',
			onClick: async () => {
				installingProjectIds.value.add(projectResult.project_id)
				await installVersion(
					projectResult.project_id,
					null,
					instance.value ? instance.value.path : null,
					'SearchCard',
					(versionId) => {
						installingProjectIds.value.delete(projectResult.project_id)
						if (versionId) {
							onSearchResultInstalled(projectResult.project_id)
						}
					},
					(profile) => {
						router.push(`/instance/${profile}`)
					},
					{
						preferredLoader: instance.value?.loader ?? undefined,
						preferredGameVersion: instance.value?.game_version ?? undefined,
					},
				).catch((err) => {
					installingProjectIds.value.delete(projectResult.project_id)
					handleError(err)
				})
			},
		},
	]
}

function onSearchResultInstalled(id: string) {
	if (isServerContext.value) {
		markServerProjectInstalled(id)
		return
	}
	newlyInstalled.value.push(id)
}

async function search(requestParams: string) {
	debugLog('searching v3', requestParams)
	const isServer = projectType.value === 'server'

	const rawResults = await queryClient.fetchQuery({
		queryKey: ['search', 'v3', requestParams],
		queryFn: () =>
			get_search_results_v3(requestParams) as Promise<{
				result: Labrinth.Search.v3.SearchResults & {
					hits: (Labrinth.Search.v3.ResultSearchProject & { installed?: boolean })[]
				}
			} | null>,
		staleTime: 30_000,
	})

	if (!rawResults) {
		return {
			projectHits: [],
			serverHits: [],
			total_hits: 0,
			per_page: 20,
		}
	}

	if (isServer) {
		const hits = rawResults.result.hits ?? []
		lastServerHits.value = hits
		serverPings.value = {}
		pingServerHits(hits)
		checkServerRunningStates(hits)
		return {
			projectHits: [],
			serverHits: hits,
			total_hits: rawResults.result.total_hits ?? 0,
			per_page: rawResults.result.hits_per_page,
		}
	}

	const hits = rawResults.result.hits.map((hit) => {
		const mapped = {
			...hit,
			title: hit.name,
			description: hit.summary,
		} as unknown as Labrinth.Search.v2.ResultSearchProject & { installed?: boolean }

		if (instance.value || isServerContext.value) {
			const installedIds = instance.value
				? new Set([...newlyInstalled.value, ...(installedProjectIds.value ?? [])])
				: serverContentProjectIds.value
			mapped.installed = installedIds.has(hit.project_id)
		}

		return mapped
	})

	return {
		projectHits: hits,
		serverHits: [],
		total_hits: rawResults.result.total_hits,
		per_page: rawResults.result.hits_per_page,
	}
}

const isServerFilterContext = computed(() => isServerContext.value || isServerInstance.value)

const lockedFilterMessages = computed(() => ({
	gameVersion: formatMessage(
		isServerFilterContext.value
			? messages.gameVersionProvidedByServer
			: messages.gameVersionProvidedByInstance,
	),
	modLoader: formatMessage(
		isServerFilterContext.value
			? messages.modLoaderProvidedByServer
			: messages.modLoaderProvidedByInstance,
	),
	environment: formatMessage(messages.environmentProvidedByServer),
	syncButton: formatMessage(messages.syncFilterButton),
	providedBy: formatMessage(
		isServerFilterContext.value ? messages.providedByServer : messages.providedByInstance,
	),
}))

const searchState = useBrowseSearch({
	projectType,
	tags,
	providedFilters: combinedProvidedFilters,
	search,
	persistentQueryParams: ['i', 'ai', 'shi', 'sid', 'wid', 'from'],
	getExtraQueryParams: () => ({
		sid: serverIdQuery.value || undefined,
		wid: effectiveServerWorldId.value || undefined,
		ai: instanceHideInstalled.value ? 'true' : undefined,
		shi: serverHideInstalled.value ? 'true' : undefined,
	}),
})

if (instance.value?.game_version) {
	const gv = instance.value.game_version
	const alreadyHasGv = searchState.serverCurrentFilters.value.some(
		(f) => f.type === 'server_game_version' && f.option === gv,
	)
	if (!alreadyHasGv) {
		searchState.serverCurrentFilters.value.push({ type: 'server_game_version', option: gv })
	}
}

await searchState.refreshSearch()

provideBrowseManager({
	tags,
	projectType,
	...searchState,
	getProjectLink: (result: Labrinth.Search.v2.ResultSearchProject) => ({
		path: `/project/${result.project_id ?? result.slug}`,
		query: instance.value ? { i: instance.value.path } : undefined,
	}),
	getServerProjectLink: (result: Labrinth.Search.v3.ResultSearchProject) =>
		`/project/${result.slug ?? result.project_id}`,
	selectableProjectTypes,
	showProjectTypeTabs: computed(() => !isServerContext.value),
	variant: 'app',
	getCardActions,
	installContext,
	providedFilters: combinedProvidedFilters,
	hideInstalled: computed({
		get: () => (isServerContext.value ? serverHideInstalled.value : instanceHideInstalled.value),
		set: (val: boolean) => {
			if (isServerContext.value) serverHideInstalled.value = val
			else instanceHideInstalled.value = val
		},
	}),
	showHideInstalled: computed(
		() => (isServerContext.value && projectType.value !== 'modpack') || !!instance.value,
	),
	hideInstalledLabel: computed(() =>
		formatMessage(isFromWorlds.value ? messages.hideAddedServers : messages.hideInstalledContent),
	),
	onInstalled: onSearchResultInstalled,
	serverPings,
	getServerModpackContent,
	onContextMenu: handleRightClick,
	offline,
	lockedFilterMessages,
})
</script>

<template>
	<div class="flex flex-col gap-3 p-6">
		<BrowsePageLayout>
			<template #after>
				<ContextMenu ref="contextMenuRef" @option-clicked="handleOptionsClick">
					<template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
					<template #copy_link> <ClipboardCopyIcon /> Copy link </template>
				</ContextMenu>
			</template>
		</BrowsePageLayout>
		<CreationFlowModal
			v-if="isServerContext && projectType === 'modpack'"
			ref="serverSetupModalRef"
			:type="serverFlowFrom === 'reset-server' ? 'reset-server' : 'server-onboarding'"
			:available-loaders="['vanilla', 'fabric', 'neoforge', 'forge', 'quilt', 'paper', 'purpur']"
			:show-snapshot-toggle="true"
			:on-back="onServerFlowBack"
			:search-modpacks="searchServerModpacks"
			:get-project-versions="getServerProjectVersions"
			@hide="() => {}"
			@browse-modpacks="() => {}"
			@create="handleServerModpackFlowCreate"
		/>
		<Teleport to="#sidebar-teleport-target">
			<BrowseSidebar />
		</Teleport>
	</div>
</template>
