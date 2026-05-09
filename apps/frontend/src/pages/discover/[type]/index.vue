<script setup lang="ts">
import type { Archon, Labrinth } from '@modrinth/api-client'
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
import type {
	BrowseInstallContentType,
	BrowseInstallPlan,
	CardAction,
	CreationFlowContextValue,
	PendingServerContentInstall,
	PendingServerContentInstallType,
} from '@modrinth/ui'
import {
	addPendingServerContentInstalls,
	BrowseInstallHeader,
	BrowsePageLayout,
	BrowseSidebar,
	commonMessages,
	CreationFlowModal,
	defineMessages,
	flushInstallQueue,
	getTargetInstallPreferences,
	injectModrinthClient,
	injectNotificationManager,
	PROJECT_DEP_MARKER_QUERY,
	provideBrowseManager,
	readPendingServerContentInstalls,
	removePendingServerContentInstall,
	requestInstall,
	SelectedProjectsFloatingBar,
	useBrowseSearch,
	useDebugLogger,
	useStickyObserver,
	useVIntl,
	writePendingServerContentInstallBaseline,
} from '@modrinth/ui'
import { cycleValue } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useTimeoutFn } from '@vueuse/core'
import { computed, nextTick, ref, watch } from 'vue'

import LogoAnimated from '~/components/brand/LogoAnimated.vue'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import { projectQueryOptions } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { withLabrinthCanaryHeader } from '~/helpers/canary.ts'
import type { DisplayLocation, DisplayMode } from '~/plugins/cosmetics.ts'

const { formatMessage } = useVIntl()
const debug = useDebugLogger('Discover')

const { updateDiscoverFilterContext } = useCdnDownloadContext()

const client = injectModrinthClient()
const queryClient = useQueryClient()

const filtersMenuOpen = ref(false)
const stickyInstallHeaderRef = ref<HTMLElement | null>(null)

useStickyObserver(stickyInstallHeaderRef, 'DiscoverInstallHeader')

const route = useRoute()

const cosmetics = useCosmetics()
const tags = useGeneratedState()
const flags = useFeatureFlags()
const auth = await useAuth()

const { addNotification, handleError } = injectNotificationManager()

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

const currentServerId = computed(() => queryAsString(route.query.sid) || null)
const fromContext = computed(() => queryAsString(route.query.from) || null)
const currentWorldId = computed(() => queryAsString(route.query.wid) || null)

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
const hideSelectedServerInstalls = ref(false)
const installingProjectIds = ref<Set<string>>(new Set())
const optimisticallyInstalledProjectIds = ref<Set<string>>(new Set())
const hiddenInstalledProjectIds = ref<Set<string>>(new Set())
const hiddenInstalledProjectIdsInitialized = ref(false)

interface InstallableSearchResult extends Labrinth.Search.v2.ResultSearchProject {
	installed?: boolean
}
type PendingServerContentInstallInput = Omit<PendingServerContentInstall, 'createdAt'>

const queuedServerInstalls = ref<Map<string, BrowseInstallPlan<InstallableSearchResult>>>(new Map())
const queuedServerInstallProjectIds = computed(() => new Set(queuedServerInstalls.value.keys()))
const queuedServerInstallCount = computed(() => queuedServerInstalls.value.size)
const selectedServerInstallProjects = computed(() =>
	Array.from(queuedServerInstalls.value.values()).map((plan) => ({
		id: plan.projectId,
		name: plan.project.title ?? formatMessage(commonMessages.projectLabel),
		iconUrl: plan.project.icon_url ?? null,
	})),
)
const isInstallingQueuedServerInstalls = ref(false)
const queuedInstallProgress = ref({ completed: 0, total: 0 })
const serverInstallQueue = {
	get: () => queuedServerInstalls.value,
	set: (plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>) => {
		queuedServerInstalls.value = plans
	},
}

function getQueuedInstallOwnerFallback(project: InstallableSearchResult) {
	if (project.organization) {
		const ownerId = project.organization_id ?? project.organization
		return {
			id: ownerId,
			name: project.organization,
			type: 'organization' as const,
			link: `/organization/${ownerId}`,
		}
	}

	if (!project.author) return null

	const ownerId = project.author_id ?? project.author
	return {
		id: ownerId,
		name: project.author,
		type: 'user' as const,
		link: `/user/${ownerId}`,
	}
}

async function getQueuedInstallOwner(project: InstallableSearchResult) {
	const fallback = getQueuedInstallOwnerFallback(project)

	try {
		if (project.organization) {
			const organization = await client.labrinth.projects_v3.getOrganization(project.project_id)
			if (organization) {
				return {
					id: organization.id,
					name: organization.name,
					type: 'organization' as const,
					avatar_url: organization.icon_url ?? undefined,
					link: `/organization/${organization.slug}`,
				}
			}
		}

		const members = await client.labrinth.projects_v3.getMembers(project.project_id)
		const owner =
			members.find((member) => member.user.id === project.author_id)?.user ??
			members.find((member) => member.is_owner || member.role === 'Owner')?.user ??
			members[0]?.user

		if (owner) {
			return {
				id: owner.id,
				name: owner.username,
				type: 'user' as const,
				avatar_url: owner.avatar_url,
				link: `/user/${owner.username}`,
			}
		}
	} catch {
		return fallback
	}

	return fallback
}

function getQueuedAddonInstallPlans(
	plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
) {
	return Array.from(plans.values()).filter((plan) => plan.contentType !== 'modpack')
}

function getQueuedInstallPlaceholder(
	plan: BrowseInstallPlan<InstallableSearchResult>,
	owner: PendingServerContentInstallInput['owner'],
): PendingServerContentInstallInput {
	return {
		projectId: plan.projectId,
		versionId: plan.versionId,
		contentType: plan.contentType as PendingServerContentInstallType,
		title: plan.project.title ?? formatMessage(commonMessages.projectLabel),
		versionName: plan.versionName ?? null,
		versionNumber: plan.versionNumber ?? null,
		fileName: plan.fileName ?? null,
		owner,
		slug: plan.project.slug ?? plan.projectId,
		iconUrl: plan.project.icon_url ?? null,
	}
}

function getQueuedInstallPlaceholderFallbacks(
	plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
) {
	return getQueuedAddonInstallPlans(plans).map((plan) =>
		getQueuedInstallPlaceholder(plan, getQueuedInstallOwnerFallback(plan.project)),
	)
}

async function getQueuedInstallPlaceholders(
	plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
) {
	return Promise.all(
		getQueuedAddonInstallPlans(plans).map(async (plan) =>
			getQueuedInstallPlaceholder(plan, await getQueuedInstallOwner(plan.project)),
		),
	)
}

function setProjectInstalling(projectId: string, installing: boolean) {
	const next = new Set(installingProjectIds.value)
	if (installing) {
		next.add(projectId)
	} else {
		next.delete(projectId)
	}
	installingProjectIds.value = next
}

function markProjectInstalled(projectId: string) {
	optimisticallyInstalledProjectIds.value = new Set([
		...optimisticallyInstalledProjectIds.value,
		projectId,
	])
}

function getServerInstalledProjectIds(data = serverContentData.value) {
	return new Set(
		(data?.addons ?? [])
			.map((addon) => addon.project_id)
			.filter((projectId): projectId is string => !!projectId),
	)
}

function getServerInstalledContentKeys(data = serverContentData.value) {
	return new Set((data?.addons ?? []).map((addon) => addon.project_id ?? addon.filename))
}

function syncHiddenInstalledProjectIds() {
	hiddenInstalledProjectIds.value = new Set([
		...getServerInstalledProjectIds(),
		...optimisticallyInstalledProjectIds.value,
	])
	hiddenInstalledProjectIdsInitialized.value = true
}

const contentQueryKey = computed(() => ['content', 'list', currentServerId.value ?? ''] as const)
const { data: serverContentData, error: serverContentError } = useQuery({
	queryKey: contentQueryKey,
	queryFn: () => client.archon.content_v1.getAddons(currentServerId.value!, currentWorldId.value!),
	enabled: computed(() => !!currentServerId.value && !!currentWorldId.value),
})

watch(serverContentError, (error) => {
	if (error) {
		console.error('Failed to load server content:', error)
		handleError(error)
	}
})

watch(
	serverContentData,
	(data) => {
		if (!data) return
		if (!hiddenInstalledProjectIdsInitialized.value) {
			syncHiddenInstalledProjectIds()
		}
	},
	{ immediate: true },
)

const installContentMutation = useMutation({
	mutationFn: ({
		serverId,
		worldId,
		projectId,
		versionId,
	}: {
		serverId: string
		worldId: string
		projectId: string
		versionId: string
	}) =>
		client.archon.content_v1.addAddon(serverId, worldId, {
			project_id: projectId,
			version_id: versionId,
		}),
	onSuccess: (_data, variables) => {
		queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', variables.serverId] })
		queryClient.invalidateQueries({ queryKey: ['content', 'list'] })
	},
})

if (route.query.shi && projectType.value?.id !== 'modpack') {
	serverHideInstalled.value = route.query.shi === 'true'
}

watch(serverHideInstalled, (hideInstalled) => {
	if (hideInstalled) {
		syncHiddenInstalledProjectIds()
	}
})

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
			filters.push({ type: 'game_version', option: gameVersion })
		}

		const platform = serverData.value.loader?.toLowerCase()

		const modLoaders = ['fabric', 'forge', 'quilt', 'neoforge']
		if (platform && modLoaders.includes(platform)) {
			filters.push({ type: 'mod_loader', option: platform })
		}

		const pluginLoaders = ['paper', 'purpur']
		if (platform && pluginLoaders.includes(platform)) {
			filters.push({ type: 'plugin_loader', option: platform })
		}

		if (projectType.value?.id === 'mod') {
			filters.push({ type: 'environment', option: 'server' })
		}

		if (serverHideInstalled.value && hiddenInstalledProjectIds.value.size > 0) {
			for (const x of hiddenInstalledProjectIds.value) {
				filters.push({
					type: 'project_id',
					option: `project_id:${x}`,
					negative: true,
				})
			}
		}

		if (hideSelectedServerInstalls.value && queuedServerInstallProjectIds.value.size > 0) {
			for (const id of queuedServerInstallProjectIds.value) {
				filters.push({
					type: 'project_id',
					option: `project_id:${id}`,
					negative: true,
				})
			}
		}
	}

	if (currentServerId.value && projectType.value?.id === 'modpack') {
		filters.push(
			{ type: 'environment', option: 'client' },
			{ type: 'environment', option: 'server' },
		)
	}
	debug('serverFilters result:', filters)
	return filters
})

function getCurrentServerInstallType(): BrowseInstallContentType {
	const type = projectType.value?.id
	if (type === 'modpack' || type === 'mod' || type === 'plugin' || type === 'datapack') {
		return type
	}
	throw new Error(formatMessage(messages.unsupportedContentType))
}

function getServerInstallTargetPreferences(contentType: BrowseInstallContentType) {
	return getTargetInstallPreferences(
		{
			gameVersion: serverData.value?.mc_version,
			loader: serverData.value?.loader,
		},
		contentType,
	)
}

function getInstallProjectVersions(projectId: string) {
	return client.labrinth.versions_v2.getProjectVersions(projectId, {
		include_changelog: false,
	})
}

function clearQueuedServerInstalls() {
	queuedServerInstalls.value = new Map()
}

function removeQueuedServerInstall(projectId: string) {
	const nextPlans = new Map(queuedServerInstalls.value)
	nextPlans.delete(projectId)
	queuedServerInstalls.value = nextPlans
}

watch([currentServerId, currentWorldId], ([serverId, worldId], [prevServerId, prevWorldId]) => {
	if (serverId !== prevServerId || worldId !== prevWorldId) {
		clearQueuedServerInstalls()
	}
})

async function flushQueuedServerInstalls(
	serverId: string | null = currentServerId.value,
	worldId: string | null = currentWorldId.value,
) {
	if (queuedServerInstalls.value.size === 0) return true
	if (isInstallingQueuedServerInstalls.value) return false

	if (!serverId || !worldId) {
		handleError(new Error(formatMessage(messages.noServerWorld)))
		return false
	}

	isInstallingQueuedServerInstalls.value = true
	queuedInstallProgress.value = {
		completed: 0,
		total: queuedServerInstalls.value.size,
	}

	try {
		const result = await flushInstallQueue({
			queue: serverInstallQueue,
			install: async (plan) => {
				await installContentMutation.mutateAsync({
					serverId,
					worldId,
					projectId: plan.projectId,
					versionId: plan.versionId,
				})
				markProjectInstalled(plan.projectId)
			},
			onError: (error, plan) => {
				removePendingServerContentInstall(serverId, worldId, plan.projectId)
				handleError(error as Error)
			},
			onProgress: (completed, total) => {
				queuedInstallProgress.value = { completed, total }
			},
		})

		return result.ok
	} finally {
		isInstallingQueuedServerInstalls.value = false
		queuedInstallProgress.value = { completed: 0, total: 0 }
	}
}

async function discardQueuedServerInstallsAndBack() {
	clearQueuedServerInstalls()
	await navigateTo(serverBackUrl.value)
}

async function installQueuedServerInstallsAndBack() {
	const sid = currentServerId.value
	const wid = currentWorldId.value
	const backUrl = serverBackUrl.value
	const plans = new Map(queuedServerInstalls.value)

	if (sid && wid) {
		writePendingServerContentInstallBaseline(sid, wid, [
			...getServerInstalledContentKeys(),
			...optimisticallyInstalledProjectIds.value,
		])
		addPendingServerContentInstalls(sid, wid, getQueuedInstallPlaceholderFallbacks(plans))
		void getQueuedInstallPlaceholders(plans)
			.then((items) => {
				const pendingProjectIds = new Set(
					readPendingServerContentInstalls(sid, wid).map((item) => item.projectId),
				)
				addPendingServerContentInstalls(
					sid,
					wid,
					items.filter((item) => pendingProjectIds.has(item.projectId)),
				)
			})
			.catch((err) => handleError(err as Error))
	}
	await navigateTo(backUrl)

	const ok = await flushQueuedServerInstalls(sid, wid)
	if (!ok) {
		queuedServerInstalls.value = new Map()
		addNotification({
			type: 'error',
			title: formatMessage(messages.someProjectsFailedTitle),
			text: formatMessage(messages.someProjectsFailedText),
		})
	}

	return true
}

async function serverInstall(project: InstallableSearchResult) {
	if (!serverData.value || !currentServerId.value || !currentWorldId.value) {
		handleError(new Error('No server to install to.'))
		return
	}
	const contentType = getCurrentServerInstallType()
	const isModpack = contentType === 'modpack'

	try {
		if (!isModpack && queuedServerInstallProjectIds.value.has(project.project_id)) {
			removeQueuedServerInstall(project.project_id)
			return
		}

		if (isModpack || !queuedServerInstallProjectIds.value.has(project.project_id)) {
			setProjectInstalling(project.project_id, true)
		}

		await requestInstall({
			project,
			contentType,
			mode: isModpack ? 'immediate' : 'queue',
			selectedFilters: isModpack ? [] : searchState.currentFilters.value,
			providedFilters: isModpack ? [] : serverFilters.value,
			overriddenProvidedFilterTypes: isModpack
				? []
				: searchState.overriddenProvidedFilterTypes.value,
			targetPreferences: getServerInstallTargetPreferences(contentType),
			getProjectVersions: getInstallProjectVersions,
			queue: serverInstallQueue,
			install: async (plan) => {
				const modalInstance = onboardingModalRef.value
				if (!modalInstance) {
					setProjectInstalling(plan.projectId, false)
					return
				}

				onboardingInstallingProject.value = plan.project
				modalInstance.show()
				await nextTick()
				const ctx = modalInstance.ctx
				ctx.setupType.value = 'modpack'
				ctx.modpackSelection.value = {
					projectId: plan.projectId,
					versionId: plan.versionId,
					name: plan.project.title,
					iconUrl: plan.project.icon_url ?? undefined,
				}
				ctx.modal.value?.setStage('final-config')
			},
		})
	} catch (e) {
		console.error(e)
		if (isModpack) {
			setProjectInstalling(project.project_id, false)
		}
		handleError(e instanceof Error ? e : new Error(`Error installing content ${e}`))
	} finally {
		if (!isModpack) {
			setProjectInstalling(project.project_id, false)
		}
	}
}

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

async function search(requestParams: string) {
	debug('search() called', {
		requestParams: requestParams.substring(0, 100),
		isServer: isServerType.value,
		projectTypeId: projectTypeId.value,
	})
	const config = useRuntimeConfig()
	let base = import.meta.server ? config.apiBaseUrl : config.public.apiBaseUrl

	if (isServerType.value) {
		base = base.replace(/\/v\d\//, '/v3/').replace(/\/v\d$/, '/v3')
	}

	const url = `${base}search${requestParams}`
	debug('search() fetching:', url.substring(0, 120))

	const raw = await $fetch<Labrinth.Search.v2.SearchResults | Labrinth.Search.v3.SearchResults>(
		url,
		{
			headers: withLabrinthCanaryHeader(),
		},
	)

	debug('search() response', { total_hits: raw.total_hits, hitCount: raw.hits?.length })

	if ('hits_per_page' in raw) {
		// v3 response (servers)
		return {
			projectHits: [],
			serverHits: raw.hits as Labrinth.Search.v3.ResultSearchProject[],
			total_hits: raw.total_hits,
			per_page: raw.hits_per_page,
		}
	}

	return {
		projectHits: raw.hits as Labrinth.Search.v2.ResultSearchProject[],
		serverHits: [],
		total_hits: raw.total_hits,
		per_page: raw.limit,
	}
}

function getCardActions(
	result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
	currentProjectType: string,
): CardAction[] {
	if (currentProjectType === 'server') return []

	const projectResult = result as InstallableSearchResult

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

const onboardingModalRef = ref<InstanceType<typeof CreationFlowModal> | null>(null)
const onboardingInstallingProject = ref<InstallableSearchResult | null>(null)

function onOnboardingHide() {
	if (onboardingInstallingProject.value) {
		setProjectInstalling(onboardingInstallingProject.value.project_id, false)
		onboardingInstallingProject.value = null
	}
}

function onOnboardingBack() {
	onboardingModalRef.value?.hide()
}

async function onModpackFlowCreate(config: CreationFlowContextValue) {
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
			properties: config.buildProperties(),
		} satisfies Archon.Content.v1.InstallWorldContent)

		if (fromContext.value === 'onboarding') {
			await client.archon.servers_v1.endIntro(currentServerId.value)
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', currentServerId.value] })
			navigateTo(`/hosting/manage/${currentServerId.value}/content`)
		} else {
			navigateTo(`/hosting/manage/${currentServerId.value}?openSettings=installation`)
		}
	} catch (e) {
		handleError(new Error(`Error installing modpack: ${e}`))
		config.loading.value = false
	}
}

const serverBackUrl = computed(() => {
	if (!serverData.value) return ''
	const id = serverData.value.server_id
	if (fromContext.value === 'onboarding') return `/hosting/manage/${id}?resumeModal=setup-type`
	if (fromContext.value === 'reset-server') return `/hosting/manage/${id}?openSettings=installation`
	return `/hosting/manage/${id}/content`
})

const serverBackLabel = computed(() => {
	if (fromContext.value === 'onboarding') return formatMessage(messages.backToSetup)
	if (fromContext.value === 'reset-server') return formatMessage(messages.cancelReset)
	return formatMessage(messages.backToServer)
})

const serverBrowseHeading = computed(() =>
	fromContext.value === 'reset-server'
		? formatMessage(messages.resetModpackHeading)
		: formatMessage(commonMessages.installingContentLabel),
)

const installContext = computed(() => {
	if (!serverData.value) return null
	return {
		name: serverData.value.name,
		loader: serverData.value.loader ?? '',
		gameVersion: serverData.value.mc_version ?? '',
		serverId: currentServerId.value,
		upstream: serverData.value.upstream,
		iconSrc: serverIcon.value,
		isMedal: serverData.value.is_medal,
		backUrl: serverBackUrl.value,
		backLabel: serverBackLabel.value,
		heading: serverBrowseHeading.value,
		queuedCount: queuedServerInstallCount.value,
		selectedProjects: selectedServerInstallProjects.value,
		isInstallingSelected: isInstallingQueuedServerInstalls.value,
		installProgress: queuedInstallProgress.value,
		clearQueued: clearQueuedServerInstalls,
		clearSelected: clearQueuedServerInstalls,
		onBack: flushQueuedServerInstalls,
		discardSelectedAndBack: discardQueuedServerInstallsAndBack,
		installSelected: installQueuedServerInstallsAndBack,
	}
})

const messages = defineMessages({
	unsupportedContentType: {
		id: 'discover.install.error.unsupported-content-type',
		defaultMessage: 'This content type cannot be installed to a server from browse.',
	},
	noServerWorld: {
		id: 'discover.install.error.no-server-world',
		defaultMessage: 'No server world is available for install.',
	},
	someProjectsFailedTitle: {
		id: 'discover.install.error.some-projects-failed.title',
		defaultMessage: 'Some projects failed to install',
	},
	someProjectsFailedText: {
		id: 'discover.install.error.some-projects-failed.description',
		defaultMessage: 'Failed projects were not added. You can try installing them again.',
	},
	backToSetup: {
		id: 'discover.install.back-to-setup',
		defaultMessage: 'Back to setup',
	},
	cancelReset: {
		id: 'discover.install.cancel-reset',
		defaultMessage: 'Cancel reset',
	},
	backToServer: {
		id: 'discover.install.back-to-server',
		defaultMessage: 'Back to server',
	},
	resetModpackHeading: {
		id: 'discover.install.heading.reset-modpack',
		defaultMessage: 'Selecting modpack to install after reset',
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
	syncFilterButton: {
		id: 'search.filter.locked.server.sync',
		defaultMessage: 'Sync with server',
	},
	seoTitle: {
		id: 'discover.seo.title',
		defaultMessage:
			'Search {projectType, select, mod {mods} modpack {modpacks} resourcepack {resource packs} shader {shaders} plugin {plugins} datapack {datapacks} other {projects}}',
	},
	seoTitleWithQuery: {
		id: 'discover.seo.title-with-query',
		defaultMessage:
			'Search {projectType, select, mod {mods} modpack {modpacks} resourcepack {resource packs} shader {shaders} plugin {plugins} datapack {datapacks} other {projects}} | {query}',
	},
	seoDescription: {
		id: 'discover.seo.description',
		defaultMessage:
			'Search and browse thousands of Minecraft {projectType, select, mod {mods} modpack {modpacks} resourcepack {resource packs} shader {shaders} plugin {plugins} datapack {datapacks} other {projects}} on Modrinth with instant, accurate search results. Our filters help you quickly find the best Minecraft {projectType, select, mod {mods} modpack {modpacks} resourcepack {resource packs} shader {shaders} plugin {plugins} datapack {datapacks} other {projects}}.',
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

watch(queuedServerInstallCount, (count) => {
	if (count === 0) {
		hideSelectedServerInstalls.value = false
	}
})

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
searchState.refreshSearch()

const ogTitle = computed(() =>
	searchState.query.value
		? formatMessage(messages.seoTitleWithQuery, {
				projectType: projectType.value?.id ?? 'project',
				query: searchState.query.value,
			})
		: formatMessage(messages.seoTitle, { projectType: projectType.value?.id ?? 'project' }),
)
const description = computed(() =>
	formatMessage(messages.seoDescription, { projectType: projectType.value?.id ?? 'project' }),
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
		`/${projectType.value?.id ?? 'project'}/${result.slug ? result.slug : result.project_id}`,
	getServerProjectLink: (result: Labrinth.Search.v3.ResultSearchProject) =>
		`/server/${result.slug ?? result.project_id}`,
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
		class="normal-page__header browse-install-header-bleed sticky top-0 z-20 mb-4 flex flex-col gap-2 border-0 bg-surface-1 py-3"
	>
		<BrowseInstallHeader />
	</div>
	<SelectedProjectsFloatingBar v-if="installContext" :install-context="installContext" />
	<aside class="normal-page__sidebar" :aria-label="formatMessage(commonMessages.filtersLabel)">
		<AdPlaceholder v-if="!auth.user && !serverData" />
		<BrowseSidebar />
	</aside>
	<section class="normal-page__content">
		<div class="flex flex-col gap-3">
			<BrowsePageLayout>
				<template #display-mode-icon>
					<GridIcon v-if="resultsDisplayMode === 'grid'" />
					<ImageIcon v-else-if="resultsDisplayMode === 'gallery'" />
					<ListIcon v-else />
				</template>
			</BrowsePageLayout>
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
		</div>
	</section>
</template>
<style lang="scss" scoped>
.browse-install-header-bleed {
	grid-column: 1 / -1;
	margin-inline: -1.5rem;
	padding-inline: 0.75rem !important;

	&::after {
		content: '';
		position: absolute;
		right: 50%;
		bottom: 0;
		width: 100vw;
		border-bottom: 1px solid var(--surface-5);
		transform: translateX(50%);
	}
}

.normal-page__content {
	display: contents;

	@media screen and (min-width: 1024px) {
		display: block;
	}
}

.normal-page__sidebar {
	grid-row: 3;

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

.content-wrapper {
	grid-row: 1;
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
