import type { Archon, Labrinth } from '@modrinth/api-client'
import {
	createContext,
	type CreationFlowContextValue,
	injectModrinthClient,
	injectNotificationManager,
} from '@modrinth/ui'
import { computed, type ComputedRef, nextTick, type Ref, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

type ServerFlowFrom = 'onboarding' | 'reset-server'
type ServerInstallableType = 'modpack' | 'mod' | 'plugin' | 'datapack'

type InstallableSearchResult = Labrinth.Search.v3.ResultSearchProject & {
	installing?: boolean
	installed?: boolean
}

interface ServerModpackSelectionRequest {
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
	serverContentProjectIds: Ref<Set<string>>
	serverBackUrl: ComputedRef<string>
	serverBackLabel: ComputedRef<string>
	serverBrowseHeading: ComputedRef<string>
	initServerContext: () => Promise<void>
	watchServerContextChanges: () => void
	searchServerModpacks: (
		query: string,
		limit?: number,
	) => Promise<Labrinth.Projects.v2.SearchResult>
	getServerProjectVersions: (projectId: string) => Promise<{ id: string }[]>
	enforceSetupModpackRoute: (currentProjectType: string | undefined) => void
	installProjectToServer: (project: InstallableSearchResult) => Promise<boolean>
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

export function createServerInstallContent(opts: {
	serverSetupModalRef: Ref<ServerSetupModalHandle | null>
}) {
	const { serverSetupModalRef } = opts
	const route = useRoute()
	const router = useRouter()
	const client = injectModrinthClient()
	const { handleError } = injectNotificationManager()

	const serverIdQuery = computed(() => readQueryString(route.query.sid))
	const worldIdQuery = computed(() => readQueryString(route.query.wid))
	const browseFrom = computed(() => readQueryString(route.query.from))
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
	const serverContentProjectIds = ref<Set<string>>(new Set())
	const effectiveServerWorldId = computed(() => worldIdQuery.value ?? serverContextWorldId.value)

	const serverBackUrl = computed(() => {
		const sid = serverIdQuery.value
		if (!sid) return '/hosting/manage'
		if (serverFlowFrom.value === 'onboarding') {
			return `/hosting/manage/${sid}?resumeModal=setup-type`
		}
		if (serverFlowFrom.value === 'reset-server') {
			return `/hosting/manage/${sid}?openSettings=installation`
		}
		return `/hosting/manage/${sid}/content`
	})
	const serverBackLabel = computed(() => {
		if (serverFlowFrom.value === 'onboarding') return 'Back to setup'
		if (serverFlowFrom.value === 'reset-server') return 'Cancel reset'
		return 'Back to server'
	})
	const serverBrowseHeading = computed(() => {
		if (serverFlowFrom.value === 'reset-server') {
			return 'Select modpack to install after reset'
		}
		return 'Install content to server'
	})

	async function resolveServerContextWorldId(serverId: string) {
		try {
			const server = await client.archon.servers_v1.get(serverId)
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
			serverContentProjectIds.value = ids
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

		let resolvedWorldId = effectiveServerWorldId.value
		if (!resolvedWorldId) {
			resolvedWorldId = await resolveServerContextWorldId(sid)
			if (resolvedWorldId) {
				serverContextWorldId.value = resolvedWorldId
			}
		}

		if (resolvedWorldId) {
			await refreshServerInstalledContent(sid, resolvedWorldId)
		}
	}

	function watchServerContextChanges() {
		watch([serverIdQuery, effectiveServerWorldId], async ([sid, wid], [prevSid, prevWid]) => {
			if (!sid) {
				serverContextServerData.value = null
				serverContentProjectIds.value = new Set()
				return
			}

			if (sid !== prevSid) {
				serverContentProjectIds.value = new Set()
				try {
					serverContextServerData.value = await client.archon.servers_v0.get(sid)
				} catch (err) {
					handleError(err as Error)
				}
			}

			if (wid && (sid !== prevSid || wid !== prevWid)) {
				await refreshServerInstalledContent(sid, wid)
			}
		})
	}

	function normalizeLoader(loader: string) {
		return loader.toLowerCase().replaceAll('_', '').replaceAll('-', '').replaceAll(' ', '')
	}

	function getCompatibleLoaders(loader: string) {
		const normalized = normalizeLoader(loader)
		if (!normalized) return new Set<string>()
		if (normalized === 'paper' || normalized === 'purpur' || normalized === 'spigot') {
			return new Set(['paper', 'purpur', 'spigot', 'bukkit'])
		}
		if (normalized === 'neoforge' || normalized === 'neo') {
			return new Set(['neoforge', 'neo'])
		}
		return new Set([normalized])
	}

	function enforceSetupModpackRoute(currentProjectType: string | undefined) {
		if (!isSetupServerContext.value || currentProjectType === 'modpack') return
		router.replace({
			path: '/browse/modpack',
			query: route.query,
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

	function getCurrentServerInstallType(): ServerInstallableType {
		const raw = Array.isArray(route.params.projectType)
			? route.params.projectType[0]
			: route.params.projectType
		if (raw === 'modpack' || raw === 'mod' || raw === 'plugin' || raw === 'datapack') {
			return raw
		}
		throw new Error('This content type cannot be installed to a server from browse.')
	}

	async function installProjectToServer(project: InstallableSearchResult) {
		const contentType = getCurrentServerInstallType()
		const sid = serverIdQuery.value
		const wid = effectiveServerWorldId.value
		if (!sid || !wid) {
			throw new Error('No server world is available for install.')
		}

		if (contentType === 'modpack') {
			const versions = await client.labrinth.versions_v2.getProjectVersions(project.project_id, {
				include_changelog: false,
			})
			const versionId = versions[0]?.id ?? project.version_id
			if (!versionId) {
				throw new Error('No version found for this modpack')
			}

			await openServerModpackInstallFlow({
				projectId: project.project_id,
				versionId,
				name: project.name,
				iconUrl: project.icon_url ?? undefined,
			})
			return false
		}

		const versions = await client.labrinth.versions_v2.getProjectVersions(project.project_id, {
			include_changelog: false,
		})
		const serverLoader = (serverContextServerData.value?.loader ?? '').toLowerCase()
		const serverGameVersion = (serverContextServerData.value?.mc_version ?? '').trim()
		const compatibleLoaders = getCompatibleLoaders(serverLoader)

		const hasGameVersionMatch = (version: Labrinth.Versions.v2.Version) =>
			!serverGameVersion || version.game_versions.includes(serverGameVersion)
		const hasLoaderMatch = (version: Labrinth.Versions.v2.Version) => {
			if (contentType === 'datapack') return true
			if (compatibleLoaders.size === 0) return true
			return version.loaders.some((loader) => compatibleLoaders.has(normalizeLoader(loader)))
		}

		let matchingVersion = versions.find(
			(version) => hasGameVersionMatch(version) && hasLoaderMatch(version),
		)
		if (!matchingVersion) {
			matchingVersion = versions.find((version) => hasLoaderMatch(version))
		}
		if (!matchingVersion) {
			matchingVersion = versions.find((version) => hasGameVersionMatch(version))
		}
		if (!matchingVersion) {
			matchingVersion = versions[0]
		}
		if (!matchingVersion) {
			throw new Error('No installable version was found for this project.')
		}

		await client.archon.content_v1.addAddon(sid, wid, {
			project_id: matchingVersion.project_id,
			version_id: matchingVersion.id,
		})

		serverContentProjectIds.value = new Set([...serverContentProjectIds.value, project.project_id])
		return true
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
				await router.push(`/hosting/manage/${sid}/content`)
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
	}
}
