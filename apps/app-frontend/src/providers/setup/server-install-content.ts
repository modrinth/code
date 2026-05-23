import type { AbstractModrinthClient, Archon, Labrinth } from '@modrinth/api-client'
import {
	addPendingServerContentInstalls,
	type BrowseInstallPlan,
	type BrowseSelectedProject,
	createContext,
	type CreationFlowContextValue,
	flushStoredServerAddonInstallQueue,
	getStoredServerAddonInstallQueue,
	injectModrinthClient,
	injectNotificationManager,
	type PendingServerContentInstall,
	type PendingServerContentInstallType,
	readPendingServerContentInstalls,
	readStoredServerInstallQueue,
	removePendingServerContentInstall,
	writePendingServerContentInstallBaseline,
	writeStoredServerInstallQueue,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import {
	computed,
	type ComputedRef,
	nextTick,
	onActivated,
	onDeactivated,
	type Ref,
	ref,
	shallowRef,
	watch,
} from 'vue'
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { useRoute, useRouter } from 'vue-router'

type ServerFlowFrom = 'onboarding' | 'reset-server'

type InstallableSearchResult = Labrinth.Search.v3.ResultSearchProject & {
	title?: string
	installing?: boolean
	installed?: boolean
}
type PendingServerContentInstallInput = Omit<PendingServerContentInstall, 'createdAt'>

export interface ServerModpackSelectionRequest {
	projectId: string
	versionId: string
	name: string
	iconUrl?: string
}

interface ServerSetupModalHandle {
	show: () => void | Promise<void>
	hide: () => void
	ctx?: CreationFlowContextValue | null
}

export interface ServerInstallContentContext {
	serverIdQuery: ComputedRef<string | null>
	worldIdQuery: ComputedRef<string | null>
	browseFrom: ComputedRef<string | null>
	serverFlowFrom: ComputedRef<ServerFlowFrom | null>
	isFromWorlds: ComputedRef<boolean>
	isServerContext: ComputedRef<boolean>
	isSetupServerContext: ComputedRef<boolean>
	effectiveServerWorldId: ComputedRef<string | null>
	serverContextServerData: Ref<Archon.Servers.v0.Server | null>
	serverContextWorldName: ComputedRef<string | null>
	serverContentProjectIds: Ref<Set<string>>
	queuedServerInstallProjectIds: ComputedRef<Set<string>>
	queuedServerInstallCount: ComputedRef<number>
	selectedServerInstallProjects: ComputedRef<BrowseSelectedProject[]>
	isInstallingQueuedServerInstalls: Ref<boolean>
	queuedInstallProgress: Ref<{ completed: number; total: number }>
	serverBackUrl: ComputedRef<string>
	serverBackLabel: ComputedRef<string>
	serverBrowseHeading: ComputedRef<string>
	clearQueuedServerInstalls: () => void
	removeQueuedServerInstall: (projectId: string) => void
	flushQueuedServerInstalls: () => Promise<boolean>
	discardQueuedServerInstallsAndBack: () => Promise<void>
	installQueuedServerInstallsAndBack: () => Promise<boolean>
	initServerContext: () => Promise<void>
	watchServerContextChanges: () => void
	searchServerModpacks: (
		query: string,
		limit?: number,
	) => Promise<Labrinth.Projects.v2.SearchResult>
	getServerProjectVersions: (projectId: string) => Promise<{ id: string }[]>
	enforceSetupModpackRoute: (currentProjectType: string | undefined) => void
	getQueuedServerInstallPlans: () => Map<string, BrowseInstallPlan<InstallableSearchResult>>
	setQueuedServerInstallPlans: (
		plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
	) => void
	openServerModpackInstallFlow: (request: ServerModpackSelectionRequest) => Promise<void>
	onServerFlowBack: () => void
	handleServerModpackFlowCreate: (config: CreationFlowContextValue) => Promise<void>
	markServerProjectInstalled: (id: string) => void
}

export const [injectServerInstallContent, provideServerInstallContent] =
	createContext<ServerInstallContentContext>('Browse', 'serverInstallContent')

function readQueryString(value: unknown): string | null {
	if (Array.isArray(value)) return value[0] ?? null
	return typeof value === 'string' && value.length > 0 ? value : null
}

function getQueuedInstallOwnerFallback(project: InstallableSearchResult) {
	if (project.organization) {
		const ownerId = project.organization_id ?? project.organization
		return {
			id: ownerId,
			name: project.organization,
			type: 'organization' as const,
			link: `https://modrinth.com/organization/${ownerId}`,
		}
	}

	if (!project.author) return null

	const ownerId = project.author_id ?? project.author
	return {
		id: ownerId,
		name: project.author,
		type: 'user' as const,
		link: `https://modrinth.com/user/${ownerId}`,
	}
}

async function getQueuedInstallOwner(
	client: AbstractModrinthClient,
	project: InstallableSearchResult,
) {
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
					link: `https://modrinth.com/organization/${organization.slug}`,
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
				link: `https://modrinth.com/user/${owner.username}`,
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
	const project = plan.project as InstallableSearchResult & { slug?: string | null }
	return {
		projectId: plan.projectId,
		versionId: plan.versionId,
		contentType: plan.contentType as PendingServerContentInstallType,
		title: project.title ?? project.name ?? 'Project',
		versionName: plan.versionName ?? null,
		versionNumber: plan.versionNumber ?? null,
		fileName: plan.fileName ?? null,
		owner,
		slug: project.slug ?? plan.projectId,
		iconUrl: project.icon_url ?? null,
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
	client: AbstractModrinthClient,
	plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
) {
	return Promise.all(
		getQueuedAddonInstallPlans(plans).map(async (plan) =>
			getQueuedInstallPlaceholder(plan, await getQueuedInstallOwner(client, plan.project)),
		),
	)
}

export function createServerInstallContent(opts: {
	serverSetupModalRef: Ref<ServerSetupModalHandle | null>
	isRouteInContext?: (route: RouteLocationNormalizedLoaded) => boolean
}) {
	const { serverSetupModalRef } = opts
	const route = useRoute()
	const router = useRouter()
	const client = injectModrinthClient()
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()

	const routeInContext = computed(() => opts.isRouteInContext?.(route) ?? true)
	const contextQuery = shallowRef(route.query)

	watch(
		[() => route.fullPath, routeInContext],
		() => {
			if (routeInContext.value) {
				contextQuery.value = route.query
			}
		},
		{ immediate: true },
	)

	const serverIdQuery = computed(() => readQueryString(contextQuery.value.sid))
	const worldIdQuery = computed(() => readQueryString(contextQuery.value.wid))
	const browseFrom = computed(() => readQueryString(contextQuery.value.from))
	const serverFlowFrom = computed<ServerFlowFrom | null>(() =>
		browseFrom.value === 'onboarding' || browseFrom.value === 'reset-server'
			? browseFrom.value
			: null,
	)

	const isFromWorlds = computed(() => browseFrom.value === 'worlds')
	const isServerContext = computed(() => !!serverIdQuery.value)
	const isSetupServerContext = computed(() => !!serverIdQuery.value && !!serverFlowFrom.value)

	const serverContextWorldId = ref<string | null>(worldIdQuery.value)
	const serverContextServerData = ref<Archon.Servers.v0.Server | null>(null)
	const serverContextServerFull = ref<Archon.Servers.v1.ServerFull | null>(null)
	const serverContentProjectIds = ref<Set<string>>(new Set())
	const serverContentInstallKeys = ref<Set<string>>(new Set())
	const queuedServerInstalls = ref<Map<string, BrowseInstallPlan<InstallableSearchResult>>>(
		new Map(),
	)
	const componentActive = ref(true)
	const queuedServerInstallProjectIds = computed(() => new Set(queuedServerInstalls.value.keys()))
	const queuedServerInstallCount = computed(() => queuedServerInstalls.value.size)
	const selectedServerInstallProjects = computed<BrowseSelectedProject[]>(() =>
		Array.from(queuedServerInstalls.value.values()).map((plan) => ({
			id: plan.projectId,
			name: plan.project.title ?? plan.project.name ?? 'Project',
			iconUrl: plan.project.icon_url ?? null,
		})),
	)
	const isInstallingQueuedServerInstalls = ref(false)
	const queuedInstallProgress = ref({ completed: 0, total: 0 })
	const effectiveServerWorldId = computed(() => worldIdQuery.value ?? serverContextWorldId.value)
	const serverContextWorld = computed(() => {
		const serverFull = serverContextServerFull.value
		if (!serverFull) return null

		const worldId = effectiveServerWorldId.value
		if (worldId) {
			return serverFull.worlds.find((world) => world.id === worldId) ?? null
		}

		return serverFull.worlds.find((world) => world.is_active) ?? serverFull.worlds[0] ?? null
	})
	const serverContextWorldName = computed(() => serverContextWorld.value?.name ?? null)
	const serverBackUrl = computed(() => {
		const sid = serverIdQuery.value
		if (!sid) return '/hosting/manage'
		if (serverFlowFrom.value === 'onboarding') {
			return `/hosting/manage/${sid}?resumeModal=setup-type`
		}
		if (serverFlowFrom.value === 'reset-server') {
			return `/hosting/manage/${sid}?openSettings=installation`
		}
		return getServerInstanceContentPath(sid, effectiveServerWorldId.value)
	})
	const serverBackLabel = computed(() => {
		if (serverFlowFrom.value === 'onboarding') return 'Back to setup'
		if (serverFlowFrom.value === 'reset-server') return 'Cancel reset'
		return 'Back to server'
	})
	const serverBrowseHeading = computed(() => {
		if (serverFlowFrom.value === 'reset-server') {
			return 'Selecting modpack to install after reset'
		}
		return 'Installing content'
	})

	onActivated(() => {
		componentActive.value = true
	})

	onDeactivated(() => {
		componentActive.value = false
	})

	async function getServerContextServerFull(serverId: string) {
		if (serverContextServerFull.value?.id === serverId) {
			return serverContextServerFull.value
		}

		const server = await client.archon.servers_v1.get(serverId)
		serverContextServerFull.value = server
		return server
	}

	async function resolveServerContextWorldId(serverId: string) {
		try {
			const server = await getServerContextServerFull(serverId)
			const activeWorld = server.worlds.find((world) => world.is_active)
			return activeWorld?.id ?? server.worlds[0]?.id ?? null
		} catch (err) {
			handleError(err as Error)
			return null
		}
	}

	async function refreshServerInstalledContent(serverId: string, worldId: string) {
		try {
			const content = await client.archon.content_v1.getAddons(serverId, worldId)
			const ids = new Set(
				(content.addons ?? [])
					.map((addon) => addon.project_id)
					.filter((projectId): projectId is string => !!projectId),
			)
			const keys = new Set(
				(content.addons ?? []).map((addon) => addon.project_id ?? addon.filename),
			)
			serverContentProjectIds.value = ids
			serverContentInstallKeys.value = keys
		} catch (err) {
			handleError(err as Error)
		}
	}

	async function initServerContext() {
		const sid = serverIdQuery.value
		if (!sid) return

		try {
			serverContextServerData.value = await client.archon.servers_v0.get(sid)
		} catch (err) {
			handleError(err as Error)
		}
		try {
			await getServerContextServerFull(sid)
		} catch (err) {
			handleError(err as Error)
		}

		let resolvedWorldId = effectiveServerWorldId.value
		if (!resolvedWorldId) {
			resolvedWorldId = await resolveServerContextWorldId(sid)
			if (resolvedWorldId) {
				serverContextWorldId.value = resolvedWorldId
			}
		}

		if (resolvedWorldId) {
			queuedServerInstalls.value = readStoredServerInstallQueue(sid, resolvedWorldId)
			await refreshServerInstalledContent(sid, resolvedWorldId)
		}
	}

	function watchServerContextChanges() {
		watch(
			[componentActive, routeInContext, serverIdQuery, effectiveServerWorldId],
			async ([active, inContext, sid, wid], [prevActive, prevInContext, prevSid, prevWid]) => {
				if (!active || !inContext) return

				if (!sid) {
					serverContextServerData.value = null
					serverContextServerFull.value = null
					serverContentProjectIds.value = new Set()
					serverContentInstallKeys.value = new Set()
					setQueuedServerInstallPlans(new Map())
					return
				}

				const hasServerDataForRoute = serverContextServerData.value?.server_id === sid
				const hasServerFullForRoute = serverContextServerFull.value?.id === sid
				const didEnterContext = !prevActive || !prevInContext
				const shouldReloadRouteContext =
					didEnterContext ||
					sid !== prevSid ||
					wid !== prevWid ||
					!hasServerDataForRoute ||
					!hasServerFullForRoute

				if (!hasServerDataForRoute || !hasServerFullForRoute) {
					serverContextWorldId.value = worldIdQuery.value
					if (!hasServerDataForRoute) {
						serverContextServerData.value = null
					}
					if (!hasServerFullForRoute) {
						serverContextServerFull.value = null
					}
					serverContentProjectIds.value = new Set()
					serverContentInstallKeys.value = new Set()
				}

				if (!hasServerDataForRoute) {
					try {
						serverContextServerData.value = await client.archon.servers_v0.get(sid)
					} catch (err) {
						handleError(err as Error)
					}
				}

				if (!hasServerFullForRoute) {
					try {
						const serverFull = await getServerContextServerFull(sid)
						if (!worldIdQuery.value) {
							const activeWorld = serverFull.worlds.find((world) => world.is_active)
							serverContextWorldId.value = activeWorld?.id ?? serverFull.worlds[0]?.id ?? null
						}
					} catch (err) {
						handleError(err as Error)
					}
				}

				if (shouldReloadRouteContext) {
					queuedServerInstalls.value = readStoredServerInstallQueue(sid, wid)
				}

				if (wid && shouldReloadRouteContext) {
					await refreshServerInstalledContent(sid, wid)
				}
			},
		)
	}

	function enforceSetupModpackRoute(currentProjectType: string | undefined) {
		if (!routeInContext.value || !isSetupServerContext.value || currentProjectType === 'modpack')
			return
		router.replace({
			path: '/browse/modpack',
			query: contextQuery.value,
		})
	}

	async function searchServerModpacks(query: string, limit: number = 10) {
		return client.labrinth.projects_v2.search({
			query: query || undefined,
			new_filters:
				'project_types = "modpack" AND (client_side = "optional" OR client_side = "required") AND server_side = "required"',
			limit,
		})
	}

	async function getServerProjectVersions(projectId: string) {
		const versions = await client.labrinth.versions_v3.getProjectVersions(projectId)
		return versions.map((version) => ({ id: version.id }))
	}

	async function openServerModpackInstallFlow(request: ServerModpackSelectionRequest) {
		if (!serverIdQuery.value || !effectiveServerWorldId.value) {
			throw new Error('Missing server context')
		}

		const modalInstance = serverSetupModalRef.value
		if (!modalInstance) return

		modalInstance.show()
		await nextTick()

		const ctx = modalInstance.ctx
		if (!ctx) return

		ctx.setupType.value = 'modpack'
		ctx.modpackSelection.value = {
			projectId: request.projectId,
			versionId: request.versionId,
			name: request.name,
			iconUrl: request.iconUrl,
		}
		ctx.modal.value?.setStage('final-config')
	}

	function clearQueuedServerInstalls() {
		setQueuedServerInstallPlans(new Map())
	}

	function removeQueuedServerInstall(projectId: string) {
		const nextPlans = new Map(queuedServerInstalls.value)
		nextPlans.delete(projectId)
		setQueuedServerInstallPlans(nextPlans)
	}

	function setStoredServerInstallPlans(
		serverId: string,
		worldId: string,
		plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
	) {
		if (serverId === serverIdQuery.value && worldId === effectiveServerWorldId.value) {
			queuedServerInstalls.value = plans
		}
		writeStoredServerInstallQueue(serverId, worldId, plans)
	}

	async function flushQueuedServerInstalls(
		serverId: string | null = serverIdQuery.value,
		worldId: string | null = effectiveServerWorldId.value,
	) {
		if (isInstallingQueuedServerInstalls.value) return false

		if (!serverId || !worldId) {
			handleError(new Error('No server instance is available for install.'))
			return false
		}

		const queuedPlans = getStoredServerAddonInstallQueue<InstallableSearchResult>(serverId, worldId)
		if (queuedPlans.size === 0) return true

		isInstallingQueuedServerInstalls.value = true
		queuedInstallProgress.value = {
			completed: 0,
			total: queuedPlans.size,
		}

		try {
			const result = await flushStoredServerAddonInstallQueue({
				serverId,
				worldId,
				install: (plans) =>
					client.archon.content_v1.addAddons(
						serverId,
						worldId,
						plans.map((plan) => ({
							project_id: plan.projectId,
							version_id: plan.versionId,
						})),
					),
				onQueueChange: (plans) => setStoredServerInstallPlans(serverId, worldId, plans),
			})

			if (!result.ok) {
				for (const plan of result.attemptedPlans) {
					removePendingServerContentInstall(serverId, worldId, plan.projectId)
				}
				handleError(result.error as Error)
				return false
			}

			queuedInstallProgress.value = {
				completed: result.flushedPlans.length,
				total: result.flushedPlans.length,
			}
			serverContentProjectIds.value = new Set([
				...serverContentProjectIds.value,
				...result.flushedPlans.map((plan) => plan.projectId),
			])
			serverContentInstallKeys.value = new Set([
				...serverContentInstallKeys.value,
				...result.flushedPlans.map((plan) => plan.projectId),
			])
			if (result.flushedPlans.length > 0) {
				await queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', serverId] })
			}

			return true
		} finally {
			isInstallingQueuedServerInstalls.value = false
			queuedInstallProgress.value = { completed: 0, total: 0 }
		}
	}

	async function discardQueuedServerInstallsAndBack() {
		clearQueuedServerInstalls()
		await router.push(serverBackUrl.value)
	}

	async function installQueuedServerInstallsAndBack() {
		const sid = serverIdQuery.value
		const wid = effectiveServerWorldId.value
		const backUrl = serverBackUrl.value
		const plans = new Map(queuedServerInstalls.value)

		if (sid && wid) {
			writeStoredServerInstallQueue(sid, wid, plans)
			writePendingServerContentInstallBaseline(sid, wid, serverContentInstallKeys.value)
			addPendingServerContentInstalls(sid, wid, getQueuedInstallPlaceholderFallbacks(plans))
			void getQueuedInstallPlaceholders(client, plans)
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
		await router.push(backUrl)
		void flushQueuedServerInstalls(sid, wid)

		return true
	}

	function getQueuedServerInstallPlans() {
		return queuedServerInstalls.value
	}

	function setQueuedServerInstallPlans(
		plans: Map<string, BrowseInstallPlan<InstallableSearchResult>>,
	) {
		queuedServerInstalls.value = plans
		writeStoredServerInstallQueue(serverIdQuery.value, effectiveServerWorldId.value, plans)
	}

	function onServerFlowBack() {
		serverSetupModalRef.value?.hide()
	}

	async function handleServerModpackFlowCreate(config: CreationFlowContextValue) {
		const sid = serverIdQuery.value
		const wid = effectiveServerWorldId.value
		if (!sid || !wid || !config.modpackSelection.value) {
			config.loading.value = false
			return
		}

		try {
			await client.archon.content_v1.installContent(sid, wid, {
				content_variant: 'modpack',
				spec: {
					platform: 'modrinth',
					project_id: config.modpackSelection.value.projectId,
					version_id: config.modpackSelection.value.versionId,
				},
				soft_override: false,
				properties: config.buildProperties(),
			} satisfies Archon.Content.v1.InstallWorldContent)
			serverSetupModalRef.value?.hide()

			if (serverFlowFrom.value === 'onboarding') {
				await client.archon.servers_v1.endIntro(sid)
				await router.push(getServerInstanceContentPath(sid, wid))
				return
			}

			await router.push(`/hosting/manage/${sid}?openSettings=installation`)
		} catch (err) {
			handleError(err as Error)
			config.loading.value = false
		}
	}

	function markServerProjectInstalled(id: string) {
		serverContentProjectIds.value = new Set([...serverContentProjectIds.value, id])
	}

	function getServerInstanceContentPath(serverId: string, worldId: string | null) {
		const base = `/hosting/manage/${encodeURIComponent(serverId)}/instances`
		return worldId ? `${base}/${encodeURIComponent(worldId)}` : base
	}

	return {
		serverIdQuery,
		worldIdQuery,
		browseFrom,
		serverFlowFrom,
		isFromWorlds,
		isServerContext,
		isSetupServerContext,
		effectiveServerWorldId,
		serverContextServerData,
		serverContextWorldName,
		serverContentProjectIds,
		queuedServerInstallProjectIds,
		queuedServerInstallCount,
		selectedServerInstallProjects,
		isInstallingQueuedServerInstalls,
		queuedInstallProgress,
		serverBackUrl,
		serverBackLabel,
		serverBrowseHeading,
		clearQueuedServerInstalls,
		removeQueuedServerInstall,
		flushQueuedServerInstalls,
		discardQueuedServerInstallsAndBack,
		installQueuedServerInstallsAndBack,
		initServerContext,
		watchServerContextChanges,
		searchServerModpacks,
		getServerProjectVersions,
		enforceSetupModpackRoute,
		getQueuedServerInstallPlans,
		setQueuedServerInstallPlans,
		openServerModpackInstallFlow,
		onServerFlowBack,
		handleServerModpackFlowCreate,
		markServerProjectInstalled,
	}
}
